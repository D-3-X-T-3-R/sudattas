//! P2 Abandoned cart: find stale carts (user only), enqueue AbandonedCart if not opted out, mark sent.

use crate::handlers::outbox::{enqueue_outbox_event, ABANDONED_CART};
use chrono::{Duration, Utc};
use core_db_entities::entity::{cart, users};
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter, TransactionTrait,
};
use serde_json::json;
use std::collections::HashSet;
use tonic::Status;

/// Default delay after which a cart is considered abandoned (24 hours).
pub const DEFAULT_ABANDONED_DELAY_HOURS: i64 = 24;

/// Find carts not updated for at least `delay_hours`, with user_id set and not yet emailed.
/// For each such user (if not marketing_opt_out), enqueue one AbandonedCart event and mark
/// all their cart rows as abandoned_email_sent_at = now.
/// Returns the number of users for whom an event was enqueued.
pub async fn enqueue_abandoned_cart_events(
    db: &DatabaseConnection,
    delay_hours: i64,
) -> Result<usize, Status> {
    let cutoff = Utc::now() - Duration::hours(delay_hours);
    let txn = db
        .begin()
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    let rows = cart::Entity::find()
        .filter(cart::Column::UserId.is_not_null())
        .filter(cart::Column::UpdatedAt.lt(cutoff))
        .filter(cart::Column::AbandonedEmailSentAt.is_null())
        .all(&txn)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    let user_ids: Vec<i64> = rows
        .into_iter()
        .filter_map(|r| r.user_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let mut enqueued = 0;
    let now = Utc::now();
    for user_id in user_ids {
        let user = users::Entity::find_by_id(user_id)
            .one(&txn)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        let Some(u) = user else { continue };
        if u.marketing_opt_out == Some(1) {
            continue;
        }
        let email = u.email.clone();
        enqueue_outbox_event(
            &txn,
            ABANDONED_CART,
            "cart",
            &user_id.to_string(),
            json!({ "user_id": user_id, "email": email }),
        )
        .await?;
        enqueued += 1;
        let stmt = sea_orm::Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::MySql,
            "UPDATE Cart SET abandoned_email_sent_at = ? WHERE UserID = ? AND abandoned_email_sent_at IS NULL",
            [now.into(), user_id.into()],
        );
        txn.execute(stmt)
            .await
            .map_err(|e: sea_orm::DbErr| Status::internal(e.to_string()))?;
    }

    txn.commit()
        .await
        .map_err(|e| Status::internal(e.to_string()))?;
    Ok(enqueued)
}
