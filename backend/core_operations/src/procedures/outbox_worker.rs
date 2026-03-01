//! P1 Outbox worker: process pending outbox_events idempotently (deliver then mark processed).
//! On delivery failure the event is left Pending for retry on the next run.

use crate::handlers::db_errors::map_db_error_to_status;
use crate::notifications::delivery;
use core_db_entities::entity::outbox_events;
use core_db_entities::entity::sea_orm_active_enums::Status as OutboxStatus;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, QuerySelect, TransactionTrait,
};
use tonic::Status;
use tracing::warn;

/// Process up to `limit` pending outbox events: deliver each (stub or real email/SMS), then mark processed.
/// On delivery failure the event is left Pending and will be retried on the next run.
/// Idempotent: each event_id is processed once; after status = processed it is never selected again.
pub async fn process_pending_outbox_events(
    db: &DatabaseConnection,
    limit: u64,
) -> Result<usize, Status> {
    let conn = db;
    let pending = outbox_events::Entity::find()
        .filter(outbox_events::Column::Status.eq(OutboxStatus::Pending))
        .order_by_asc(outbox_events::Column::CreatedAt)
        .limit(limit)
        .all(conn)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    let mut processed_count = 0;
    for row in &pending {
        let txn = conn
            .begin()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        if let Err(e) = delivery::deliver_event(row).await {
            warn!(
                event_id = row.event_id,
                event_type = row.event_type,
                error = %e.message(),
                "outbox: delivery failed, event left Pending for retry"
            );
            let _ = txn.rollback().await;
            continue;
        }

        let mut active: outbox_events::ActiveModel = row.clone().into();
        active.status = ActiveValue::Set(OutboxStatus::Processed);
        active.published_at = ActiveValue::Set(Some(chrono::Utc::now()));
        if let Err(e) = active.update(&txn).await.map_err(map_db_error_to_status) {
            warn!(
                event_id = row.event_id,
                "outbox: update failed: {}",
                e.message()
            );
            let _ = txn.rollback().await;
            continue;
        }
        if txn.commit().await.is_err() {
            continue;
        }
        processed_count += 1;
    }

    Ok(processed_count)
}
