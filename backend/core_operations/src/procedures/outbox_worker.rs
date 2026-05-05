//! P1 Outbox worker: process pending outbox_events idempotently (deliver then mark processed).
//! On delivery failure the event is left Pending for retry on the next run.

use crate::handlers::db_errors::map_db_error_to_status;
use crate::notifications::delivery;
use chrono::{DateTime, Duration, Utc};
use core_db_entities::entity::outbox_events;
use core_db_entities::entity::sea_orm_active_enums::Status as OutboxStatus;
use sea_orm::{
    ColumnTrait, Condition, ConnectionTrait, DatabaseConnection, DbBackend, EntityTrait,
    QueryFilter, QueryOrder, QuerySelect, Statement, TransactionTrait,
};
use tonic::Status;
use tracing::warn;

fn is_stale_client_verified(event: &outbox_events::Model, stale_before: DateTime<Utc>) -> bool {
    event.status == OutboxStatus::ClientVerified
        && event
            .published_at
            .map(|ts| ts < stale_before)
            .unwrap_or(false)
}

/// Process up to `limit` pending outbox events: deliver each (stub or real email/SMS), then mark processed.
/// On delivery failure the event is left Pending and will be retried on the next run.
/// Idempotent: each event_id is processed once; after status = processed it is never selected again.
pub async fn process_pending_outbox_events(
    db: &DatabaseConnection,
    limit: u64,
) -> Result<usize, Status> {
    let reclaim_timeout_minutes = crate::order_policy::outbox_reclaim_timeout_minutes();
    let stale_before = Utc::now() - Duration::minutes(reclaim_timeout_minutes);
    let pending = outbox_events::Entity::find()
        .filter(
            Condition::any()
                .add(outbox_events::Column::Status.eq(OutboxStatus::Pending))
                .add(
                    Condition::all()
                        .add(outbox_events::Column::Status.eq(OutboxStatus::ClientVerified))
                        .add(outbox_events::Column::PublishedAt.lt(stale_before)),
                ),
        )
        .order_by_asc(outbox_events::Column::CreatedAt)
        .limit(limit)
        .all(db)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    let stale_client_verified_count = pending
        .iter()
        .filter(|row| is_stale_client_verified(row, stale_before))
        .count();
    if stale_client_verified_count > 0 {
        warn!(
            stale_client_verified_count,
            reclaim_timeout_minutes, "outbox: reclaiming stale client_verified events"
        );
    }

    let mut processed_count = 0;
    for row in pending {
        let claimable =
            row.status == OutboxStatus::Pending || is_stale_client_verified(&row, stale_before);
        if !claimable {
            continue;
        }

        // Claim first in a short transaction.
        let claim_txn = db.begin().await.map_err(map_db_error_to_status)?;
        let claim = claim_txn
            .execute(Statement::from_sql_and_values(
                DbBackend::MySql,
                r#"UPDATE OutboxEvents
                   SET status = 'client_verified',
                       published_at = UTC_TIMESTAMP()
                   WHERE event_id = ?
                     AND (
                         status = 'pending'
                         OR (
                             status = 'client_verified'
                             AND published_at IS NOT NULL
                             AND published_at < DATE_SUB(UTC_TIMESTAMP(), INTERVAL ? MINUTE)
                         )
                     )"#,
                [row.event_id.into(), reclaim_timeout_minutes.into()],
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
