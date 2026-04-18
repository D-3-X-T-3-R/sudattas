//! Full-order cancellation settlement and inventory restore (exactly-once guard).
//!
//! Shared by customer cancel, Shiprocket cancel/RTO webhooks, and logistics retry jobs.

use crate::handlers::db_errors::map_db_error_to_status;
use crate::handlers::order_events::create_order_event;
use crate::handlers::refunds::create_refund;
use crate::handlers::shipments::logistics_workflow::load_shipment_for_order;
use crate::observability;
use crate::razorpay;
use core_db_entities::entity::sea_orm_active_enums::{PaymentStatus, Status as DbStatus};
use core_db_entities::entity::{order_details, orders, payment_intents, refund_attempts, refunds};
use proto::proto::core::{CreateOrderEventRequest, CreateRefundRequest};
use sea_orm::{
    sea_query::LockType, ColumnTrait, ConnectionTrait, DatabaseTransaction, DbBackend, EntityTrait,
    QueryFilter, QueryOrder, QuerySelect, Statement,
};
use tonic::{Request, Status as TonicStatus};
use tracing::{info, warn};

async fn set_order_refund_settlement_status(
    txn: &DatabaseTransaction,
    order_id: i64,
    status: &str,
) -> Result<(), TonicStatus> {
    txn.execute(Statement::from_sql_and_values(
        DbBackend::MySql,
        "UPDATE Orders SET refund_settlement_status = ?, updated_at = UTC_TIMESTAMP() WHERE OrderID = ?",
        [status.into(), order_id.into()],
    ))
    .await
    .map_err(map_db_error_to_status)?;
    Ok(())
}

async fn insert_refund_attempt_no_intent(
    txn: &DatabaseTransaction,
    order_id: i64,
    amount_requested_paise: i64,
    status: &str,
    provider_error: &str,
    idempotency_key: &str,
) -> Result<(), TonicStatus> {
    txn.execute(Statement::from_sql_and_values(
        DbBackend::MySql,
        r#"INSERT INTO RefundAttempts (
            order_id, payment_intent_id, razorpay_payment_id,
            amount_requested_paise, amount_sent_to_gateway_paise,
            gateway_refund_id, status, provider_error, idempotency_key
        ) VALUES (?, NULL, NULL, ?, 0, NULL, ?, ?, ?)"#,
        [
            order_id.into(),
            amount_requested_paise.into(),
            status.into(),
            provider_error.into(),
            idempotency_key.into(),
        ],
    ))
    .await
    .map_err(map_db_error_to_status)?;
    Ok(())
}

async fn insert_refund_attempt_submitted(
    txn: &DatabaseTransaction,
    order_id: i64,
    payment_intent_id: i64,
    razorpay_payment_id: &str,
    amount_requested_paise: i64,
    amount_sent_to_gateway_paise: i64,
    idempotency_key: &str,
) -> Result<i64, TonicStatus> {
    let ins = txn
        .execute(Statement::from_sql_and_values(
            DbBackend::MySql,
            r#"INSERT INTO RefundAttempts (
            order_id, payment_intent_id, razorpay_payment_id,
            amount_requested_paise, amount_sent_to_gateway_paise,
            gateway_refund_id, status, provider_error, idempotency_key
        ) VALUES (?,?,?,?,?, NULL, 'submitted', NULL, ?)"#,
            [
                order_id.into(),
                payment_intent_id.into(),
                razorpay_payment_id.into(),
                amount_requested_paise.into(),
                amount_sent_to_gateway_paise.into(),
                idempotency_key.into(),
            ],
        ))
        .await
        .map_err(map_db_error_to_status)?;
    Ok(ins.last_insert_id() as i64)
}

async fn mark_refund_attempt_processed(
    txn: &DatabaseTransaction,
    attempt_id: i64,
    gateway_refund_id: &str,
) -> Result<(), TonicStatus> {
    txn.execute(Statement::from_sql_and_values(
        DbBackend::MySql,
        r#"UPDATE RefundAttempts
           SET status = 'processed',
               gateway_refund_id = ?,
               provider_error = NULL,
               updated_at = UTC_TIMESTAMP()
           WHERE attempt_id = ?"#,
        [gateway_refund_id.into(), attempt_id.into()],
    ))
    .await
    .map_err(map_db_error_to_status)?;
    Ok(())
}

async fn mark_refund_attempt_failed(
    txn: &DatabaseTransaction,
    attempt_id: i64,
    provider_error: &str,
) -> Result<(), TonicStatus> {
    txn.execute(Statement::from_sql_and_values(
        DbBackend::MySql,
        r#"UPDATE RefundAttempts
           SET status = 'failed',
               provider_error = ?,
               updated_at = UTC_TIMESTAMP()
           WHERE attempt_id = ?"#,
        [provider_error.into(), attempt_id.into()],
    ))
    .await
    .map_err(map_db_error_to_status)?;
    Ok(())
}

async fn has_submitted_attempt(
    txn: &DatabaseTransaction,
    order_id: i64,
) -> Result<bool, TonicStatus> {
    let row = refund_attempts::Entity::find()
        .filter(refund_attempts::Column::OrderId.eq(order_id))
        .filter(refund_attempts::Column::Status.eq("submitted"))
        .order_by_desc(refund_attempts::Column::AttemptId)
        .lock(LockType::Update)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?;
    Ok(row.is_some())
}

async fn has_attempt_for_idempotency_key(
    txn: &DatabaseTransaction,
    order_id: i64,
    idempotency_key: &str,
) -> Result<bool, TonicStatus> {
    let row = refund_attempts::Entity::find()
        .filter(refund_attempts::Column::OrderId.eq(order_id))
        .filter(refund_attempts::Column::IdempotencyKey.eq(idempotency_key))
        .order_by_desc(refund_attempts::Column::AttemptId)
        .lock(LockType::Update)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?;
    Ok(row.is_some())
}

/// Restore catalog inventory for a commercially cancelled order at most once (DB-backed guard).
pub async fn restore_inventory_once(
    txn: &DatabaseTransaction,
    order_id: i64,
) -> Result<(), TonicStatus> {
    let ins = txn
        .execute(Statement::from_sql_and_values(
            DbBackend::MySql,
            "INSERT IGNORE INTO OrderInventoryRestores (order_id) VALUES (?)",
            [order_id.into()],
        ))
        .await
        .map_err(map_db_error_to_status)?;
    if ins.rows_affected() == 0 {
        return Ok(());
    }
    let details = order_details::Entity::find()
        .filter(order_details::Column::OrderId.eq(order_id))
        .all(txn)
        .await
        .map_err(map_db_error_to_status)?;
    for d in &details {
        txn.execute(Statement::from_sql_and_values(
            DbBackend::MySql,
            r#"UPDATE Inventory SET QuantityAvailable = QuantityAvailable + ? WHERE VariantID = ?"#,
            [d.quantity.into(), d.variant_id.into()],
        ))
        .await
        .map_err(map_db_error_to_status)?;
    }
    Ok(())
}

/// Single shared path: Razorpay full-order refund after commercial cancel.
pub async fn run_full_order_settlement(
    txn: &DatabaseTransaction,
    order_id: i64,
) -> Result<(), TonicStatus> {
    info!(order_id, "refund execution started");
    let order = orders::Entity::find_by_id(order_id)
        .lock(LockType::Update)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| TonicStatus::not_found("order not found"))?;

    let grand = order.grand_total_minor.max(0);

    let settled_processed: i64 = refunds::Entity::find()
        .filter(refunds::Column::OrderId.eq(order_id))
        .filter(refunds::Column::Status.eq(DbStatus::Processed))
        .all(txn)
        .await
        .map_err(map_db_error_to_status)?
        .iter()
        .map(|r| r.amount_paise as i64)
        .sum();

    if grand <= 0 {
        set_order_refund_settlement_status(txn, order_id, "refund_not_applicable").await?;
        return Ok(());
    }

    if settled_processed >= grand {
        info!(order_id, "refund skipped - already processed");
        set_order_refund_settlement_status(txn, order_id, "refund_processed").await?;
        return Ok(());
    }

    if order
        .refund_settlement_status
        .as_deref()
        .is_some_and(|s| s.eq_ignore_ascii_case("refund_processed"))
    {
        info!(order_id, "refund skipped - already processed");
        set_order_refund_settlement_status(txn, order_id, "refund_processed").await?;
        return Ok(());
    }

    if order
        .refund_settlement_status
        .as_deref()
        .is_some_and(|s| s.eq_ignore_ascii_case("refund_pending"))
    {
        info!(order_id, "refund skipped - already pending");
        return Ok(());
    }

    if has_submitted_attempt(txn, order_id).await? {
        info!(order_id, "refund skipped - already pending");
        return Ok(());
    }

    if !matches!(order.payment_status, Some(PaymentStatus::Captured)) {
        set_order_refund_settlement_status(txn, order_id, "refund_not_applicable").await?;
        return Ok(());
    }

    let intent = match payment_intents::Entity::find()
        .filter(payment_intents::Column::OrderId.eq(order_id))
        .filter(payment_intents::Column::Status.eq(DbStatus::Processed))
        .filter(payment_intents::Column::RazorpayPaymentId.is_not_null())
        .order_by_desc(payment_intents::Column::IntentId)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
    {
        Some(i) => i,
        None => {
            set_order_refund_settlement_status(txn, order_id, "refund_failed").await?;
            insert_refund_attempt_no_intent(
                txn,
                order_id,
                grand,
                "failed",
                "No canonical processed payment intent with razorpay_payment_id",
                &format!("no_intent:{order_id}"),
            )
            .await?;
            let _ = create_order_event(
                txn,
                Request::new(CreateOrderEventRequest {
                    order_id,
                    event_type: "refund_failed".to_string(),
                    from_status: None,
                    to_status: None,
                    actor_type: "system".to_string(),
                    message: Some(
                        "Refund could not start: no canonical captured payment intent".to_string(),
                    ),
                }),
            )
            .await;
            return Ok(());
        }
    };

    let payment_id = intent
        .razorpay_payment_id
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| TonicStatus::internal("payment id missing on intent"))?;

    let remaining = grand.saturating_sub(settled_processed);
    let intent_cap = intent.amount_paise as i64;
    let gateway_cap = razorpay::fetch_payment_amount_paise(payment_id)
        .await
        .unwrap_or(intent_cap);
    let basis_cap = intent_cap.min(gateway_cap).max(0);
    let send_amount = remaining.min(basis_cap).max(0);

    if send_amount <= 0 {
        set_order_refund_settlement_status(txn, order_id, "refund_not_applicable").await?;
        return Ok(());
    }

    let idem = format!("refund_{order_id}_{payment_id}");
    if has_attempt_for_idempotency_key(txn, order_id, &idem).await? {
        info!(order_id, idempotency_key = %idem, "refund skipped - attempt already exists");
        return Ok(());
    }
    set_order_refund_settlement_status(txn, order_id, "refund_pending").await?;
    let attempt_id = insert_refund_attempt_submitted(
        txn,
        order_id,
        intent.intent_id,
        payment_id,
        remaining,
        send_amount,
        &idem,
    )
    .await?;
    info!(
        order_id,
        payment_id,
        idempotency_key = %idem,
        attempt_id,
        "refund request sent"
    );
    info!(
        order_id,
        idempotency_key = %idem,
        "refund request deduped via idempotency key"
    );

    match razorpay::create_refund(payment_id, send_amount, &idem).await {
        Ok(gateway_refund) => {
            let rid = gateway_refund.refund_id.clone();
            let st = gateway_refund
                .status
                .clone()
                .unwrap_or_else(|| "pending".to_string())
                .to_lowercase();
            if let Some(shipment) = load_shipment_for_order(txn, order_id, true).await? {
                txn.execute(Statement::from_sql_and_values(
                    DbBackend::MySql,
                    r#"UPDATE Shipments
                       SET razorpay_refund_id = ?,
                           refund_status = ?,
                           refund_initiated_at = UTC_TIMESTAMP()
                       WHERE shipment_id = ?"#,
                    [
                        rid.clone().into(),
                        st.clone().into(),
                        shipment.shipment_id.into(),
                    ],
                ))
                .await
                .map_err(map_db_error_to_status)?;
            }
            observability::log_operational_event(
                "refund_created",
                &[
                    ("order_id", order_id.to_string()),
                    ("refund_id", rid.clone()),
                    ("refund_status", st.clone()),
                ],
            );
            create_refund(
                txn,
                Request::new(CreateRefundRequest {
                    order_id,
                    gateway_refund_id: rid.clone(),
                    amount_paise: send_amount,
                    currency: order.currency.clone(),
                    line_items_refunded_json: None,
                }),
            )
            .await
            .map_err(|e| {
                warn!(order_id, error = %e, "create_refund after gateway success failed");
                e
            })?;
            if let Err(e) = mark_refund_attempt_processed(txn, attempt_id, rid.as_str()).await {
                warn!(
                    order_id,
                    error = ?e,
                    "RefundAttempts insert failed after refund was recorded (gateway refund already created)"
                );
            }
            let settled_after: i64 = refunds::Entity::find()
                .filter(refunds::Column::OrderId.eq(order_id))
                .filter(refunds::Column::Status.eq(DbStatus::Processed))
                .all(txn)
                .await
                .map_err(map_db_error_to_status)?
                .iter()
                .map(|r| r.amount_paise as i64)
                .sum();
            if settled_after >= grand {
                set_order_refund_settlement_status(txn, order_id, "refund_processed").await?;
            } else {
                set_order_refund_settlement_status(txn, order_id, "refund_pending").await?;
            }
            let _ = create_order_event(
                txn,
                Request::new(CreateOrderEventRequest {
                    order_id,
                    event_type: "refund_initiated".to_string(),
                    from_status: None,
                    to_status: None,
                    actor_type: "system".to_string(),
                    message: Some(format!(
                        "Refund initiated for {} paise after order cancellation",
                        send_amount
                    )),
                }),
            )
            .await;
            Ok(())
        }
        Err(err) => {
            observability::record_refund_failure_total("gateway_error");
            let err_msg = err.clone();
            mark_refund_attempt_failed(txn, attempt_id, err_msg.as_str()).await?;
            set_order_refund_settlement_status(txn, order_id, "refund_failed").await?;
            let _ = create_order_event(
                txn,
                Request::new(CreateOrderEventRequest {
                    order_id,
                    event_type: "refund_failed".to_string(),
                    from_status: None,
                    to_status: None,
                    actor_type: "system".to_string(),
                    message: Some(format!("Razorpay refund failed: {err_msg}")),
                }),
            )
            .await;
            Ok(())
        }
    }
}
