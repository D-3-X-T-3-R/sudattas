//! Product status code<->id lookups, mirroring order_state_machine's get_status_id/get_status_name
//! for orders. Exists so callers never need to hardcode a ProductStatuses row id (1=draft,
//! 2=active, 3=archived today) — that mapping is seed data, not a compile-time constant, and
//! could legitimately differ across environments or after a re-seed.

use core_db_entities::entity::product_statuses;
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};

/// Look up a ProductStatuses id by its code (e.g. "active", "draft", "archived").
pub async fn get_status_id(
    txn: &DatabaseTransaction,
    code: &str,
) -> Result<Option<i64>, tonic::Status> {
    let row = product_statuses::Entity::find()
        .filter(product_statuses::Column::Code.eq(code))
        .one(txn)
        .await
        .map_err(|e| tonic::Status::internal(e.to_string()))?;
    Ok(row.map(|r| r.id))
}
