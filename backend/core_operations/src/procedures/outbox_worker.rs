//! P1 Outbox worker: process pending outbox_events idempotently (publish then mark processed).

use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::outbox_events;
use core_db_entities::entity::sea_orm_active_enums::Status as OutboxStatus;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, QuerySelect, TransactionTrait,
};
use tonic::Status;
use tracing::info;

/// Process up to `limit` pending outbox events: "publish" each (stub: log) then mark processed.
/// Idempotent: each event_id is processed once; after status = processed it is never selected again.
pub async fn process_pending_outbox_events(
    db: &DatabaseConnection,
    limit: u64,
) -> Result<usize, Status> {
    let txn = db
        .begin()
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    let pending = outbox_events::Entity::find()
        .filter(outbox_events::Column::Status.eq(OutboxStatus::Pending))
        .order_by_asc(outbox_events::Column::CreatedAt)
        .limit(limit)
        .all(&txn)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    let count = pending.len();
    for row in &pending {
        // Stub: log instead of sending email/SMS; replace with real delivery later.
        info!(
            event_id = row.event_id,
            event_type = row.event_type,
            aggregate_id = %row.aggregate_id,
            "outbox: publish (stub)"
        );
        let mut active: outbox_events::ActiveModel = row.clone().into();
        active.status = ActiveValue::Set(OutboxStatus::Processed);
        active.published_at = ActiveValue::Set(Some(chrono::Utc::now()));
        active.update(&txn).await.map_err(map_db_error_to_status)?;
    }

    txn.commit()
        .await
        .map_err(|e| Status::internal(e.to_string()))?;
    Ok(count)
}
