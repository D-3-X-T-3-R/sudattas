use crate::handlers::db_errors::map_db_error_to_status;
use crate::handlers::order_events::create_order_event;
use crate::order_state_machine::{self, OrderState};
use chrono::{DateTime, Utc};
use core_db_entities::entity::sea_orm_active_enums::{
    FulfillmentStatus, PaymentStatus, Status as PaymentIntentStatus,
};
use core_db_entities::entity::{order_status, orders, payment_intents};
use proto::proto::core::CreateOrderEventRequest;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, QueryFilter, QuerySelect, Statement, TransactionTrait,
};
use tonic::{Request, Status};
use tracing::{info, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpirySkipReason {
    IntentNotExpired,
    IntentStatusNotPending,
    MissingOrder,
    CodOrder,
    CapturedOrder,
    NonPendingOrderState,
    FulfillmentAlreadyStarted,
}

impl ExpirySkipReason {
    fn as_str(self) -> &'static str {
        match self {
            ExpirySkipReason::IntentNotExpired => "intent_not_expired",
            ExpirySkipReason::IntentStatusNotPending => "intent_status_not_pending",
            ExpirySkipReason::MissingOrder => "order_missing",
            ExpirySkipReason::CodOrder => "cod_order",
            ExpirySkipReason::CapturedOrder => "captured_order",
            ExpirySkipReason::NonPendingOrderState => "non_pending_order_state",
            ExpirySkipReason::FulfillmentAlreadyStarted => "fulfillment_already_started",
        }
    }
}

#[derive(Debug, Default)]
struct ExpiryRunStats {
    claimed: u64,
    expired: u64,
    skipped: u64,
    failed: u64,
}

fn is_prepaid_like(payment_method: Option<&str>) -> bool {
    !payment_method
        .unwrap_or("prepaid")
        .trim()
        .eq_ignore_ascii_case("cod")
}

fn is_system_expiry_eligible(
    order: &orders::Model,
    order_status_name: &str,
    intent: &payment_intents::Model,
    now: DateTime<Utc>,
) -> Result<(), ExpirySkipReason> {
    if !matches!(intent.status, PaymentIntentStatus::Pending) {
        return Err(ExpirySkipReason::IntentStatusNotPending);
    }
    if intent.expires_at >= now {
        return Err(ExpirySkipReason::IntentNotExpired);
    }
    if !is_prepaid_like(order.payment_method.as_deref()) {
        return Err(ExpirySkipReason::CodOrder);
    }
    if matches!(order.payment_status, Some(PaymentStatus::Captured)) {
        return Err(ExpirySkipReason::CapturedOrder);
    }
    if !matches!(order.fulfillment_status, FulfillmentStatus::NotCreated) {
        return Err(ExpirySkipReason::FulfillmentAlreadyStarted);
    }
    let pending_state = order_status_name.eq_ignore_ascii_case("active_sale")
        || order_status_name.eq_ignore_ascii_case("pending");
    if !pending_state {
        return Err(ExpirySkipReason::NonPendingOrderState);
    }
    Ok(())
}

async fn mark_intent_failed(
    txn: &sea_orm::DatabaseTransaction,
    intent_id: i64,
) -> Result<(), Status> {
    let Some(intent) = payment_intents::Entity::find_by_id(intent_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
    else {
        return Ok(());
    };
    if !matches!(
        intent.status,
        PaymentIntentStatus::Pending
            | PaymentIntentStatus::ClientVerified
            | PaymentIntentStatus::NeedsReview
    ) {
        return Ok(());
    }
    let mut active = intent.into_active_model();
    active.status = ActiveValue::Set(PaymentIntentStatus::Failed);
    active.update(txn).await.map_err(map_db_error_to_status)?;
    Ok(())
}

async fn fail_open_intents_for_order(
    txn: &sea_orm::DatabaseTransaction,
    order_id: i64,
) -> Result<(), Status> {
    txn.execute(Statement::from_sql_and_values(
        sea_orm::DbBackend::MySql,
        r#"UPDATE PaymentIntents
           SET status = 'failed'
           WHERE order_id = ?
             AND status IN ('pending', 'client_verified', 'needs_review')"#,
        [order_id.into()],
    ))
    .await
    .map_err(map_db_error_to_status)?;
    Ok(())
}

async fn load_order_status_name(
    txn: &sea_orm::DatabaseTransaction,
    status_id: i64,
) -> Result<String, Status> {
    order_status::Entity::find_by_id(status_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .map(|row| row.status_name)
        .ok_or_else(|| Status::internal(format!("OrderStatus {} not found", status_id)))
}

async fn claim_expired_pending_intent_ids(
    txn: &sea_orm::DatabaseTransaction,
    batch_limit: u64,
) -> Result<Vec<i64>, Status> {
    let rows = txn
        .query_all(Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            r#"SELECT intent_id
               FROM PaymentIntents pi
               JOIN Orders o ON o.OrderID = pi.order_id
               JOIN OrderStatus os ON os.StatusID = o.StatusID
               WHERE pi.status = 'pending'
                 AND pi.expires_at < ?
                 AND pi.order_id IS NOT NULL
                 AND LOWER(COALESCE(o.payment_method, 'prepaid')) <> 'cod'
                 AND COALESCE(o.payment_status, 'pending') <> 'captured'
                 AND o.fulfillment_status = 'not_created'
                 AND os.StatusName IN ('active_sale', 'pending')
               ORDER BY pi.expires_at ASC
               LIMIT ?
               FOR UPDATE SKIP LOCKED"#,
            [
                Utc::now().into(),
                i64::try_from(batch_limit).unwrap_or(i64::MAX).into(),
            ],
        ))
        .await
        .map_err(map_db_error_to_status)?;

    let mut ids = Vec::with_capacity(rows.len());
    for row in rows {
        if let Ok(intent_id) = row.try_get::<i64>("", "intent_id") {
            ids.push(intent_id);
        }
    }
    Ok(ids)
}

async fn system_expire_stale_unpaid_order(
    txn: &sea_orm::DatabaseTransaction,
    intent: &payment_intents::Model,
) -> Result<bool, Status> {
    let now = Utc::now();
    let Some(order_id) = intent.order_id else {
        mark_intent_failed(txn, intent.intent_id).await?;
        return Ok(false);
    };

    let Some(order) = orders::Entity::find_by_id(order_id)
        .lock(sea_orm::sea_query::LockType::Update)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
    else {
        warn!(
            intent_id = intent.intent_id,
            order_id,
            skip_reason = ExpirySkipReason::MissingOrder.as_str(),
            "stale_order_expiry: order missing for expired intent; marking intent failed"
        );
        mark_intent_failed(txn, intent.intent_id).await?;
        return Ok(false);
    };

    let from_status_name = load_order_status_name(txn, order.status_id).await?;
    if let Err(reason) = is_system_expiry_eligible(&order, &from_status_name, intent, now) {
        warn!(
            intent_id = intent.intent_id,
            order_id,
            skip_reason = reason.as_str(),
            "stale_order_expiry: skipped system expiry for expired intent"
        );
        mark_intent_failed(txn, intent.intent_id).await?;
        return Ok(false);
    }

    txn.execute(Statement::from_sql_and_values(
        sea_orm::DbBackend::MySql,
        r#"UPDATE OrderDetails
           SET item_status = 'cancelled',
               cancelled_at = COALESCE(cancelled_at, UTC_TIMESTAMP())
           WHERE OrderID = ?
             AND item_status = 'active'"#,
        [order_id.into()],
    ))
    .await
    .map_err(map_db_error_to_status)?;

    crate::cancellation_saga::restore_inventory_once(txn, order_id).await?;

    order_state_machine::transition_order_status(
        txn,
        order_id,
        OrderState::Cancelled,
        "system_unpaid_order_expired",
        "system",
        Some("System expired stale unpaid prepaid checkout and released inventory"),
        Some(PaymentStatus::Failed),
    )
    .await?;

    fail_open_intents_for_order(txn, order_id).await?;

    let _ = create_order_event(
        txn,
        Request::new(CreateOrderEventRequest {
            order_id,
            event_type: "stale_unpaid_expiry_applied".to_string(),
            from_status: Some(from_status_name),
            to_status: Some("cancelled".to_string()),
            actor_type: "system".to_string(),
            message: Some("Stale unpaid prepaid order expired by system".to_string()),
        }),
    )
    .await;

    Ok(true)
}

pub async fn expire_stale_pending_orders(
    db: &DatabaseConnection,
    batch_limit: u64,
) -> Result<u64, Status> {
    let txn = db.begin().await.map_err(map_db_error_to_status)?;
    let mut stats = ExpiryRunStats::default();
    let claimed_intent_ids = claim_expired_pending_intent_ids(&txn, batch_limit).await?;
    stats.claimed = claimed_intent_ids.len() as u64;
    info!(
        claimed = stats.claimed,
        "stale_order_expiry: claimed expired unpaid prepaid intents"
    );
    if claimed_intent_ids.is_empty() {
        txn.commit().await.map_err(map_db_error_to_status)?;
        return Ok(0);
    }

    let expired_intents = payment_intents::Entity::find()
        .filter(payment_intents::Column::IntentId.is_in(claimed_intent_ids))
        .all(&txn)
        .await
        .map_err(map_db_error_to_status)?;

    for intent in expired_intents {
        match system_expire_stale_unpaid_order(&txn, &intent).await {
            Ok(true) => stats.expired += 1,
            Ok(false) => stats.skipped += 1,
            Err(err) => {
                stats.failed += 1;
                warn!(
                    intent_id = intent.intent_id,
                    order_id = ?intent.order_id,
                    error = %err,
                    "stale_order_expiry: system expiry failed for claimed intent"
                );
            }
        }
    }

    txn.commit().await.map_err(map_db_error_to_status)?;
    info!(
        claimed = stats.claimed,
        expired = stats.expired,
        skipped = stats.skipped,
        failed = stats.failed,
        "stale_order_expiry: run completed"
    );
    Ok(stats.expired)
}

#[cfg(test)]
mod tests {
    use super::{is_prepaid_like, is_system_expiry_eligible, ExpirySkipReason};
    use chrono::{Duration, Utc};
    use core_db_entities::entity::sea_orm_active_enums::{
        FulfillmentStatus, PaymentStatus, Status as PaymentIntentStatus,
    };
    use core_db_entities::entity::{orders, payment_intents};

    fn sample_order(
        payment_method: Option<&str>,
        payment_status: Option<PaymentStatus>,
        fulfillment_status: FulfillmentStatus,
    ) -> orders::Model {
        let now = Utc::now();
        orders::Model {
            order_id: 1,
            order_number: None,
            public_order_ref: "SUD-TEST-REF".to_string(),
            user_id: 1,
            order_date: now,
            created_at: now,
            cancel_window_ends_at: Some(now - Duration::hours(2)),
            earliest_booking_at: None,
            pickup_target_at: None,
            pickup_target_reason: None,
            pickup_target_set_by: None,
            pickup_target_updated_at: None,
            shipping_address_id: 1,
            total_amount: None,
            status_id: 1,
            payment_status,
            payment_method: payment_method.map(|v| v.to_string()),
            currency: Some("INR".to_string()),
            updated_at: Some(now),
            subtotal_minor: 1_000,
            items_total_minor_before_discount: Some(1_000),
            shipping_minor: Some(0),
            shipping_charge_minor: Some(0),
            tax_total_minor: Some(0),
            discount_total_minor: Some(0),
            items_total_minor_after_discount: Some(1_000),
            grand_total_minor: 1_000,
            applied_coupon_id: None,
            applied_coupon_code: None,
            applied_discount_paise: None,
            refund_settlement_status: None,
            fulfillment_status,
            invoice_id: None,
            invoice_number: None,
            invoice_generated_at: None,
            invoice_storage_path: None,
        }
    }

    fn sample_intent(
        status: PaymentIntentStatus,
        expires_at: chrono::DateTime<Utc>,
    ) -> payment_intents::Model {
        payment_intents::Model {
            intent_id: 1,
            razorpay_order_id: "order_test_1".to_string(),
            order_id: Some(1),
            active_order_id: Some(1),
            user_id: Some(1),
            amount_paise: 1_000,
            currency: Some("INR".to_string()),
            status,
            razorpay_payment_id: None,
            metadata: None,
            created_at: Some(Utc::now()),
            expires_at,
            gateway_fee_paise: None,
            gateway_tax_paise: None,
        }
    }

    #[test]
    fn prepaid_like_treats_non_cod_as_prepaid() {
        assert!(is_prepaid_like(Some("prepaid")));
        assert!(is_prepaid_like(Some("razorpay")));
        assert!(is_prepaid_like(None));
        assert!(!is_prepaid_like(Some("cod")));
    }

    #[test]
    fn expiry_eligibility_ignores_cancel_window_and_allows_stale_prepaid() {
        let now = Utc::now();
        let order = sample_order(
            Some("prepaid"),
            Some(PaymentStatus::Pending),
            FulfillmentStatus::NotCreated,
        );
        let intent = sample_intent(PaymentIntentStatus::Pending, now - Duration::hours(1));
        assert!(
            is_system_expiry_eligible(&order, "active_sale", &intent, now).is_ok(),
            "stale unpaid prepaid order should be eligible regardless of cancel window timestamps"
        );
    }

    #[test]
    fn expiry_rejects_cod_order() {
        let now = Utc::now();
        let order = sample_order(
            Some("cod"),
            Some(PaymentStatus::Pending),
            FulfillmentStatus::NotCreated,
        );
        let intent = sample_intent(PaymentIntentStatus::Pending, now - Duration::hours(1));
        assert_eq!(
            is_system_expiry_eligible(&order, "active_sale", &intent, now),
            Err(ExpirySkipReason::CodOrder)
        );
    }

    #[test]
    fn expiry_rejects_captured_order() {
        let now = Utc::now();
        let order = sample_order(
            Some("prepaid"),
            Some(PaymentStatus::Captured),
            FulfillmentStatus::NotCreated,
        );
        let intent = sample_intent(PaymentIntentStatus::Pending, now - Duration::hours(1));
        assert_eq!(
            is_system_expiry_eligible(&order, "active_sale", &intent, now),
            Err(ExpirySkipReason::CapturedOrder)
        );
    }

    #[test]
    fn expiry_rejects_non_pending_order_status() {
        let now = Utc::now();
        let order = sample_order(
            Some("prepaid"),
            Some(PaymentStatus::Pending),
            FulfillmentStatus::NotCreated,
        );
        let intent = sample_intent(PaymentIntentStatus::Pending, now - Duration::hours(1));
        assert_eq!(
            is_system_expiry_eligible(&order, "confirmed", &intent, now),
            Err(ExpirySkipReason::NonPendingOrderState)
        );
    }

    #[test]
    fn expiry_rejects_when_fulfillment_already_started() {
        let now = Utc::now();
        let order = sample_order(
            Some("prepaid"),
            Some(PaymentStatus::Pending),
            FulfillmentStatus::InTransit,
        );
        let intent = sample_intent(PaymentIntentStatus::Pending, now - Duration::hours(1));
        assert_eq!(
            is_system_expiry_eligible(&order, "active_sale", &intent, now),
            Err(ExpirySkipReason::FulfillmentAlreadyStarted)
        );
    }

    #[test]
    fn expiry_rejects_fresh_or_non_pending_intent() {
        let now = Utc::now();
        let order = sample_order(
            Some("prepaid"),
            Some(PaymentStatus::Pending),
            FulfillmentStatus::NotCreated,
        );
        let fresh = sample_intent(PaymentIntentStatus::Pending, now + Duration::minutes(1));
        assert_eq!(
            is_system_expiry_eligible(&order, "active_sale", &fresh, now),
            Err(ExpirySkipReason::IntentNotExpired)
        );

        let non_pending =
            sample_intent(PaymentIntentStatus::NeedsReview, now - Duration::minutes(1));
        assert_eq!(
            is_system_expiry_eligible(&order, "active_sale", &non_pending, now),
            Err(ExpirySkipReason::IntentStatusNotPending)
        );
    }
}
