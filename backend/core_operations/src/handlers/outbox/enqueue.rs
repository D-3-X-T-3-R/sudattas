//! P1 Outbox: enqueue notification events (PaymentCaptured when order is paid, Shipped, Delivered, Refunded).
//! `OrderPlaced` is kept for constants / legacy rows; customer confirmation email is sent on `PaymentCaptured`.

use crate::handlers::db_errors::map_db_error_to_status;
use crate::notifications::admin_alert::AdminAlert;
use chrono::Utc;
use core_db_entities::entity::outbox_events;
use core_db_entities::entity::sea_orm_active_enums::Status as OutboxStatus;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use serde_json::Value;
use tonic::Status;

/// Event types for transactional notifications (emails/SMS).
pub const ORDER_PLACED: &str = "OrderPlaced";
pub const PAYMENT_CAPTURED: &str = "PaymentCaptured";
pub const SHIPPED: &str = "Shipped";
pub const DELIVERED: &str = "Delivered";
pub const REFUNDED: &str = "Refunded";
pub const INVOICE_GENERATED: &str = "InvoiceGenerated";
/// P2: Abandoned cart reminder (payload: cart_id, user_id?, session_id?, email?)
pub const ABANDONED_CART: &str = "AbandonedCart";
/// Admin alerts (payload: `AdminAlert::to_payload`), one event per channel so each retries independently.
pub const ADMIN_ORDER_EMAIL: &str = "AdminOrderEmail";
pub const ADMIN_ORDER_PUSH: &str = "AdminOrderPush";

/// Enqueue a notification event into the outbox. Same transaction as the business operation.
pub async fn enqueue_outbox_event(
    txn: &DatabaseTransaction,
    event_type: &str,
    aggregate_type: &str,
    aggregate_id: &str,
    payload: Value,
) -> Result<(), Status> {
    let row = outbox_events::ActiveModel {
        event_id: ActiveValue::NotSet,
        event_type: ActiveValue::Set(event_type.to_string()),
        aggregate_type: ActiveValue::Set(aggregate_type.to_string()),
        aggregate_id: ActiveValue::Set(aggregate_id.to_string()),
        payload: ActiveValue::Set(payload),
        status: ActiveValue::Set(OutboxStatus::Pending),
        created_at: ActiveValue::Set(Utc::now()),
        published_at: ActiveValue::Set(None),
    };
    row.insert(txn).await.map_err(map_db_error_to_status)?;
    Ok(())
}

/// Enqueue an admin alert on both channels (email + push), one outbox row each so they retry
/// independently. Best-effort: a failure is logged, never propagated, so an alert can't block
/// the business operation it describes.
pub async fn enqueue_admin_alert(
    txn: &DatabaseTransaction,
    order_id: i64,
    user_id: i64,
    alert: &AdminAlert,
) {
    let payload = alert.to_payload(order_id, user_id);
    for evt in [ADMIN_ORDER_EMAIL, ADMIN_ORDER_PUSH] {
        if let Err(e) =
            enqueue_outbox_event(txn, evt, "order", &order_id.to_string(), payload.clone()).await
        {
            tracing::error!(
                order_id,
                event = evt,
                error = %e,
                "failed to enqueue admin alert outbox event"
            );
        }
    }
}
