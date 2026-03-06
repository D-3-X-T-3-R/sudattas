use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::products;
use proto::proto::core::{ProductResponse, ProductsResponse, UpdateProductRequest};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn update_product(
    txn: &DatabaseTransaction,
    request: Request<UpdateProductRequest>,
) -> Result<Response<ProductsResponse>, Status> {
    let req = request.into_inner();

    let products = products::ActiveModel {
        name: ActiveValue::Set(req.name),
        description: ActiveValue::Set(req.description),
        product_id: ActiveValue::Set(req.product_id),
        category_id: ActiveValue::Set(req.category_id),
        sku: req
            .sku
            .map(|s| ActiveValue::Set(Some(s)))
            .unwrap_or(ActiveValue::NotSet),
        slug: req
            .slug
            .map(|s| ActiveValue::Set(Some(s)))
            .unwrap_or(ActiveValue::NotSet),
        price_paise: ActiveValue::Set(req.price_paise as i32),
        fabric: req
            .fabric
            .map(|s| ActiveValue::Set(Some(s)))
            .unwrap_or(ActiveValue::NotSet),
        weave: req
            .weave
            .map(|s| ActiveValue::Set(Some(s)))
            .unwrap_or(ActiveValue::NotSet),
        occasion: req
            .occasion
            .map(|s| ActiveValue::Set(Some(s)))
            .unwrap_or(ActiveValue::NotSet),
        has_blouse_piece: req
            .has_blouse_piece
            .map(|b| ActiveValue::Set(Some(if b { 1 } else { 0 })))
            .unwrap_or(ActiveValue::NotSet),
        care_instructions: req
            .care_instructions
            .map(|s| ActiveValue::Set(Some(s)))
            .unwrap_or(ActiveValue::NotSet),
        product_status_id: req
            .product_status_id
            .map(|id| ActiveValue::Set(Some(id)))
            .unwrap_or(ActiveValue::NotSet),
        created_at: ActiveValue::NotSet,
        updated_at: ActiveValue::NotSet,
    };
    match products.update(txn).await {
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
                }],
            };
            Ok(Response::new(response))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
