//! Soft delete: sets a product's status to "archived" via the ProductStatuses lookup table —
//! never a hardcoded id. Distinct from delete_product (unused, would fail on FK constraints)
//! and permanently_delete_product (irreversible hard delete with full cascade + R2 purge).

use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::products;
use proto::proto::core::{ArchiveProductRequest, ProductResponse, ProductsResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn archive_product(
    txn: &DatabaseTransaction,
    request: Request<ArchiveProductRequest>,
) -> Result<Response<ProductsResponse>, Status> {
    let req = request.into_inner();

    // A missing "archived" row is a real misconfiguration, not a normal empty-result case like
    // search/sitemap — this is a single explicit admin action, so fail loudly instead of
    // silently no-op-ing.
    let archived_status_id = crate::product_state::get_status_id(txn, "archived")
        .await?
        .ok_or_else(|| Status::internal("ProductStatuses is missing an 'archived' row"))?;

    let product = products::ActiveModel {
        product_id: ActiveValue::Set(req.product_id),
        product_status_id: ActiveValue::Set(Some(archived_status_id)),
        ..Default::default()
    };

    match product.update(txn).await {
        Ok(model) => {
            let price_paise = model.price_paise as i64;
            let response = ProductsResponse {
                items: vec![ProductResponse {
                    name: model.name,
                    product_id: model.product_id,
                    description: model.description,
                    price_paise,
                    category_id: model.category_id,
                    sku: model.sku,
                    slug: model.slug,
                    fabric: model.fabric,
                    weave: model.weave,
                    occasion: model.occasion,
                    has_blouse_piece: model.has_blouse_piece.map(|v| v != 0),
                    care_instructions: model.care_instructions,
                    product_status_id: model.product_status_id,
                    meta_title: model.meta_title,
                    meta_description: model.meta_description,
                }],
            };
            Ok(Response::new(response))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
