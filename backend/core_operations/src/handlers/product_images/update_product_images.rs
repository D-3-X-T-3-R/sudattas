use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::product_images;
use proto::proto::core::{ProductImageResponse, ProductImagesResponse, UpdateProductImageRequest};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn update_product_image(
    txn: &DatabaseTransaction,
    request: Request<UpdateProductImageRequest>,
) -> Result<Response<ProductImagesResponse>, Status> {
    let req = request.into_inner();
    let existing = product_images::Entity::find_by_id(req.image_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::not_found(format!("ProductImage {} not found", req.image_id)))?;

    let model = product_images::ActiveModel {
        image_id: ActiveValue::Set(existing.image_id),
        product_id: ActiveValue::Set(req.product_id),
        urls: ActiveValue::Set(existing.urls),
        created_at: ActiveValue::NotSet,
    };
    match model.update(txn).await {
        Ok(updated) => {
            let url = updated
                .urls
                .get("1")
                .or_else(|| updated.urls.get("0"))
                .and_then(|v| v.as_str())
                .map(String::from);
            let response = ProductImagesResponse {
                items: vec![ProductImageResponse {
                    image_id: updated.image_id,
                    product_id: updated.product_id,
                    image_base64: String::new(),
                    alt_text: None,
                    url,
                    cdn_path: None,
                    thumbnail_url: None,
                }],
            };
            Ok(Response::new(response))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
