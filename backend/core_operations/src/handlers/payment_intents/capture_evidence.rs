//! The only prepaid paid-transition entry point. Call with authenticated provider
//! evidence (raw-body verified webhook or authenticated read-only provider GET).
use super::finalize_order_paid;
use crate::handlers::db_errors::map_db_error_to_status as db_err;
use core_db_entities::entity::sea_orm_active_enums::{PaymentStatus, Status as IntentStatus};
use core_db_entities::entity::{orders, payment_intents};
use sea_orm::{
    sea_query::LockType, ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait,
    DatabaseTransaction, DbBackend, EntityTrait, IntoActiveModel, QueryFilter, QuerySelect,
    Statement,
};
use tonic::Status;

pub async fn apply_captured_evidence(
    txn: &DatabaseTransaction,
    gateway: &str,
    payment: &str,
    amount: i64,
    currency: &str,
) -> Result<(), Status> {
    if payment.is_empty() || gateway.is_empty() || amount <= 0 || currency != "INR" {
        return Err(Status::invalid_argument("Incomplete capture evidence"));
    }
    // Discovery only; every decision below uses current reads under locks.
    let found = payment_intents::Entity::find()
        .filter(payment_intents::Column::RazorpayOrderId.eq(gateway))
        .one(txn)
        .await
        .map_err(db_err)?
        .ok_or_else(|| Status::not_found("Payment intent not found"))?;
    let order_id = found
        .order_id
        .ok_or_else(|| Status::failed_precondition("Payment has no internal order"))?;
    // All callback/capture/refund/expiry paths acquire order before intent.
    let order = orders::Entity::find_by_id(order_id)
        .lock(LockType::Update)
        .one(txn)
        .await
        .map_err(db_err)?
        .ok_or_else(|| Status::not_found("Order not found"))?;
    let intent = payment_intents::Entity::find_by_id(found.intent_id)
        .lock(LockType::Update)
        .one(txn)
        .await
        .map_err(db_err)?
        .ok_or_else(|| Status::not_found("Intent not found"))?;
    let association_ok = intent.order_id == Some(order_id)
        && intent.user_id == Some(order.user_id)
        && intent.razorpay_order_id == gateway
        && intent.amount_paise as i64 == amount
        && order.grand_total_minor == amount
        && intent.currency.as_deref() == Some(currency)
        && order.currency.as_deref() == Some(currency)
        && intent
            .razorpay_payment_id
            .as_deref()
            .is_none_or(|id| id == payment);
    let status = crate::order_state_machine::get_status_name(txn, order.status_id)
        .await?
        .unwrap_or_default();
    // A duplicate capture after a refund/cancellation is historical evidence,
    // not a new late payment. Do not undo the refund or reopen fulfilment.
    if association_ok
        && intent.status == IntentStatus::Processed
        && intent.razorpay_payment_id.as_deref() == Some(payment)
        && (order.payment_status == Some(PaymentStatus::Captured)
            || matches!(status.as_str(), "refunded" | "cancelled"))
    {
        return Ok(());
    }
    let restored = txn.query_one(Statement::from_sql_and_values(DbBackend::MySql,
        "SELECT order_id FROM OrderInventoryRestores WHERE order_id=? LIMIT 1 FOR UPDATE",
        [order_id.into()])).await.map_err(db_err)?.is_some()
        || txn.query_one(Statement::from_sql_and_values(DbBackend::MySql,
        "SELECT order_id FROM OrderInventoryRestoreItems WHERE order_id=? LIMIT 1 FOR UPDATE",
        [order_id.into()])).await.map_err(db_err)?.is_some();
    if !association_ok
        || matches!(
            intent.status,
            IntentStatus::Failed | IntentStatus::NeedsReview
        )
        || matches!(
            status.as_str(),
            "cancelled"
                | "refunded"
                | "needs_review"
                | "partially_cancelled"
                | "cancel_pending_logistics"
        )
        || restored
    {
        let mut active = intent.into_active_model();
        active.status = ActiveValue::Set(IntentStatus::NeedsReview);
        active.update(txn).await.map_err(db_err)?;
        txn.execute(Statement::from_sql_and_values(
            DbBackend::MySql,
            "UPDATE Orders SET payment_status='needs_review' WHERE OrderID=?",
            [order_id.into()],
        ))
        .await
        .map_err(db_err)?;
        crate::handlers::order_events::create_order_event(txn,tonic::Request::new(proto::proto::core::CreateOrderEventRequest {
            order_id,event_type:"capture_needs_reconciliation".into(),from_status:None,to_status:Some("needs_review".into()),
            actor_type:"system".into(),message:Some("Capture conflicts with association or released reservation; fulfilment blocked".into())
        })).await?;
        return Ok(());
    }
    if order.payment_method.as_deref() == Some("cod") {
        return Err(Status::failed_precondition("Unexpected capture for COD"));
    }
    let mut active = intent.into_active_model();
    active.razorpay_payment_id = ActiveValue::Set(Some(payment.into()));
    active.status = ActiveValue::Set(IntentStatus::Processed);
    active.update(txn).await.map_err(db_err)?;
    // Captured legacy orders are adopted without a second ledger posting.
    finalize_order_paid(
        txn,
        order_id,
        "payment_captured",
        "system",
        "Verified provider capture",
    )
    .await?;
    Ok(())
}
