//! P1 Outbox: enqueue notification events (OrderPlaced, PaymentCaptured, Shipped, Delivered, Refunded).

use crate::handlers::db_errors::map_db_error_to_status;
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
