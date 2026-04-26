//! P1 Outbox worker: process pending outbox_events idempotently (deliver then mark processed).
//! On delivery failure the event is left Pending for retry on the next run.

use crate::handlers::db_errors::map_db_error_to_status;
use crate::notifications::delivery;
use core_db_entities::entity::outbox_events;
use core_db_entities::entity::sea_orm_active_enums::Status as OutboxStatus;
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseConnection, DbBackend, EntityTrait, QueryFilter,
    QueryOrder, QuerySelect, Statement, TransactionTrait,
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
    let pending = outbox_events::Entity::find()
        .filter(outbox_events::Column::Status.eq(OutboxStatus::Pending))
        .order_by_asc(outbox_events::Column::CreatedAt)
        .limit(limit)
        .all(db)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    let mut processed_count = 0;
    for row in pending {
        // Claim first in a short transaction.
        let claim_txn = db.begin().await.map_err(map_db_error_to_status)?;
        let claim = claim_txn
            .execute(Statement::from_sql_and_values(
                DbBackend::MySql,
                r#"UPDATE OutboxEvents
                   SET status = 'client_verified',
                       published_at = UTC_TIMESTAMP()
                   WHERE event_id = ?
                     AND status = 'pending'"#,
                [row.event_id.into()],
            ))
            .await
            .map_err(map_db_error_to_status)?;
        if claim.rows_affected() != 1 {
            claim_txn.rollback().await.ok();
            continue;
        }
        claim_txn.commit().await.map_err(map_db_error_to_status)?;

        // Deliver only after durable claim commit.
        if let Err(e) = delivery::deliver_event(db, &row).await {
            warn!(
                event_id = row.event_id,
                event_type = row.event_type,
                error = %e.message(),
                "outbox: delivery failed, event returned to Pending for retry"
            );
            let fail_txn = db.begin().await.map_err(map_db_error_to_status)?;
            fail_txn
                .execute(Statement::from_sql_and_values(
                    DbBackend::MySql,
                    r#"UPDATE OutboxEvents
                       SET status = 'pending',
                           published_at = NULL
                       WHERE event_id = ?
                         AND status = 'client_verified'"#,
                    [row.event_id.into()],
                ))
                .await
                .map_err(map_db_error_to_status)?;
            fail_txn.commit().await.map_err(map_db_error_to_status)?;
            continue;
        }

        let success_txn = db.begin().await.map_err(map_db_error_to_status)?;
        let updated = success_txn
            .execute(Statement::from_sql_and_values(
                DbBackend::MySql,
                r#"UPDATE OutboxEvents
                   SET status = 'processed',
                       published_at = UTC_TIMESTAMP()
                   WHERE event_id = ?
                     AND status = 'client_verified'"#,
                [row.event_id.into()],
            ))
            .await
            .map_err(map_db_error_to_status)?;
        if updated.rows_affected() != 1 {
            success_txn.rollback().await.ok();
            continue;
        }
        success_txn.commit().await.map_err(map_db_error_to_status)?;
        processed_count += 1;
    }

    Ok(processed_count)
}
