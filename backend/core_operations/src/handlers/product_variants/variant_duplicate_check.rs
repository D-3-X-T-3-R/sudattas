//! Shared duplicate-variant guard for create/update — a product should never end up with two
//! variants for the same (size, color) combination, since they'd represent the exact same real
//! item split across two DB rows with independently-tracked stock (e.g. four separate "Free
//! Size" rows on one product, each with its own quantity, rather than one row holding the real
//! total).

use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::product_variants;
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::Status;

/// Errors if `product_id` already has a variant matching `size_id`/`color_id` exactly (both
/// treated null-safely — `None` matches an existing `NULL`, not "any value"). `exclude_variant_id`
/// lets an update check against every *other* variant on the product without flagging itself.
pub async fn ensure_no_duplicate_variant(
    txn: &DatabaseTransaction,
    product_id: i64,
    size_id: Option<i64>,
    color_id: Option<i64>,
    exclude_variant_id: Option<i64>,
) -> Result<(), Status> {
    let mut query =
        product_variants::Entity::find().filter(product_variants::Column::ProductId.eq(product_id));

    query = match size_id {
        Some(id) => query.filter(product_variants::Column::SizeId.eq(id)),
        None => query.filter(product_variants::Column::SizeId.is_null()),
    };
    query = match color_id {
        Some(id) => query.filter(product_variants::Column::ColorId.eq(id)),
        None => query.filter(product_variants::Column::ColorId.is_null()),
    };
    if let Some(exclude_id) = exclude_variant_id {
        query = query.filter(product_variants::Column::VariantId.ne(exclude_id));
    }

    let duplicate = query.one(txn).await.map_err(map_db_error_to_status)?;
    if let Some(existing) = duplicate {
        return Err(Status::already_exists(format!(
            "This product already has a variant (#{}) with this exact size/color combination.",
            existing.variant_id
        )));
    }
    Ok(())
}
