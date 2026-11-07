//! Reverses archive_product: sets a product's status back to "active" via the ProductStatuses
//! lookup table — never a hardcoded id. Touches nothing else on the product.

use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::products;
use proto::proto::core::{ActivateProductRequest, ProductResponse, ProductsResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn activate_product(
    txn: &DatabaseTransaction,
    request: Request<ActivateProductRequest>,
) -> Result<Response<ProductsResponse>, Status> {
    let req = request.into_inner();

    // A missing "active" row is a real misconfiguration, not a normal empty-result case like
    // search/sitemap — this is a single explicit admin action, so fail loudly instead of
    // silently no-op-ing.
    let active_status_id = crate::product_state::get_status_id(txn, "active")
        .await?
        .ok_or_else(|| Status::internal("ProductStatuses is missing an 'active' row"))?;

    let product = products::ActiveModel {
        product_id: ActiveValue::Set(req.product_id),
        product_status_id: ActiveValue::Set(Some(active_status_id)),
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
