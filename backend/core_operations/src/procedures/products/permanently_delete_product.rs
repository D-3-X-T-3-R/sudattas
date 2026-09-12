//! Admin: permanently (irreversibly) delete a product — distinct from `deleteProduct`, which
//! is currently unused and would fail on any real product (the FK constraints on
//! ProductVariants/ProductImages/Reviews/Wishlist are all `NO ACTION`, i.e. RESTRICT). This is
//! the second tier of the two-tier deletion model: Archive (soft, via `product_status_id`,
//! already existed) for the normal case, this for "actually get rid of it."
//!
//! Refuses outright if the product has ever been ordered — deleting a variant with order
//! history would either violate the same FK constraint or, worse, silently corrupt historical
//! order data if the constraint were ever loosened. Archiving is the correct action for a
//! product that's been sold before; this path is for a mistake entry or genuinely retired
//! product with no order history.
//!
//! Phased like `place_order`: the DB cascade runs and commits first (the part that must be
//! atomic and correct), then the R2 image purge runs as a best-effort cleanup afterward — a
//! leaked storage object is a minor cost issue, not a correctness one, so it doesn't hold the
//! DB transaction open across the external HTTP calls to Cloudflare.

use crate::handlers::db_errors::map_db_error_to_status;
use crate::handlers::product_images::r2_client::build_r2_client;
use core_db_entities::entity::{
    cart, inventory, inventory_log, order_details, product_images, product_variants, products,
    reviews, wishlist,
};
use proto::proto::core::{PermanentlyDeleteProductRequest, PermanentlyDeleteProductResponse};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, TransactionTrait};
use tonic::{Request, Response, Status};
use tracing::warn;

pub async fn permanently_delete_product(
    db: &DatabaseConnection,
    request: Request<PermanentlyDeleteProductRequest>,
) -> Result<Response<PermanentlyDeleteProductResponse>, Status> {
    let req = request.into_inner();
    let product_id = req.product_id;

    let txn = db.begin().await.map_err(map_db_error_to_status)?;

    let product = products::Entity::find_by_id(product_id)
        .one(&txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::not_found(format!("Product with ID {product_id} not found.")))?;

    let variant_ids: Vec<i64> = product_variants::Entity::find()
        .filter(product_variants::Column::ProductId.eq(product_id))
        .all(&txn)
        .await
        .map_err(map_db_error_to_status)?
        .into_iter()
        .map(|v| v.variant_id)
        .collect();

    if !variant_ids.is_empty() {
        let has_order_history = order_details::Entity::find()
            .filter(order_details::Column::VariantId.is_in(variant_ids.clone()))
            .one(&txn)
            .await
            .map_err(map_db_error_to_status)?
            .is_some();
        if has_order_history {
            return Err(Status::failed_precondition(
                "This product has order history and can't be permanently removed — archive it \
                 instead to hide it from the storefront while keeping order records intact.",
            ));
        }
    }

    // Collect image URLs before deleting the rows — needed for the R2 purge after commit.
    let image_urls: Vec<String> = product_images::Entity::find()
        .filter(product_images::Column::ProductId.eq(product_id))
        .all(&txn)
        .await
        .map_err(map_db_error_to_status)?
        .into_iter()
        .map(|i| i.url)
        .collect();
    let images_deleted = image_urls.len() as i32;

    // Delete in dependency order: variant-scoped children first, then variants, then the
    // product-scoped tables, then the product row itself. ProductMoodMapping cascades
    // automatically (ON DELETE CASCADE) so it needs no explicit step here.
    if !variant_ids.is_empty() {
        cart::Entity::delete_many()
            .filter(cart::Column::VariantId.is_in(variant_ids.clone()))
            .exec(&txn)
            .await
            .map_err(map_db_error_to_status)?;

        inventory_log::Entity::delete_many()
            .filter(inventory_log::Column::VariantId.is_in(variant_ids.clone()))
            .exec(&txn)
            .await
            .map_err(map_db_error_to_status)?;

        inventory::Entity::delete_many()
            .filter(inventory::Column::VariantId.is_in(variant_ids.clone()))
            .exec(&txn)
            .await
            .map_err(map_db_error_to_status)?;
    }

    let variants_deleted = product_variants::Entity::delete_many()
        .filter(product_variants::Column::ProductId.eq(product_id))
        .exec(&txn)
        .await
        .map_err(map_db_error_to_status)?
        .rows_affected as i32;

    reviews::Entity::delete_many()
        .filter(reviews::Column::ProductId.eq(product_id))
        .exec(&txn)
        .await
        .map_err(map_db_error_to_status)?;

    wishlist::Entity::delete_many()
        .filter(wishlist::Column::ProductId.eq(product_id))
        .exec(&txn)
        .await
        .map_err(map_db_error_to_status)?;

    product_images::Entity::delete_many()
        .filter(product_images::Column::ProductId.eq(product_id))
        .exec(&txn)
        .await
        .map_err(map_db_error_to_status)?;

    products::Entity::delete_by_id(product_id)
        .exec(&txn)
        .await
        .map_err(map_db_error_to_status)?;

    txn.commit().await.map_err(map_db_error_to_status)?;

    let mut images_purge_failed = 0i32;
    if !image_urls.is_empty() {
        match build_r2_client() {
            Some((client, bucket, public_url)) => {
                let prefix = format!("{}/", public_url.trim_end_matches('/'));
                for url in &image_urls {
                    let Some(key) = url.strip_prefix(&prefix) else {
                        // Not an R2 object under our configured public URL (e.g. a legacy or
                        // external URL) — nothing in our bucket to purge for it.
                        continue;
                    };
                    if let Err(e) = client.delete_object().bucket(&bucket).key(key).send().await {
                        // Walk the full source chain — the top-level Display (`{e}`) alone is
                        // just "dispatch failure" with no detail about the actual cause.
                        let mut chain = format!("{e}");
                        let mut source = std::error::Error::source(&e);
                        while let Some(s) = source {
                            chain.push_str(&format!(" -> {s}"));
                            source = s.source();
                        }
                        warn!(
                            product_id,
                            key,
                            error = %chain,
                            "permanently_delete_product: R2 purge failed for one image; product is still fully deleted from the DB"
                        );
                        images_purge_failed += 1;
                    }
                }
            }
            None => {
                warn!(
                    product_id,
                    image_count = image_urls.len(),
                    "permanently_delete_product: R2 not configured; skipping image purge"
                );
                images_purge_failed = image_urls.len() as i32;
            }
        }
    }

    Ok(Response::new(PermanentlyDeleteProductResponse {
        product_id,
        name: product.name,
        variants_deleted,
        images_deleted,
        images_purge_failed,
    }))
}
