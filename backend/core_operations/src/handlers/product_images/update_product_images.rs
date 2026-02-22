use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::product_images;
use proto::proto::core::{ProductImageResponse, ProductImagesResponse, UpdateProductImageRequest};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn update_product_image(
    txn: &DatabaseTransaction,
    request: Request<UpdateProductImageRequest>,
) -> Result<Response<ProductImagesResponse>, Status> {
    let req = request.into_inner();

    let product_images = product_images::ActiveModel {
        image_id: ActiveValue::Set(req.image_id),
        product_id: ActiveValue::Set(req.product_id),
        image_base64: ActiveValue::Set(Some(req.image_base64)),
        alt_text: ActiveValue::Set(req.alt_text),
        url: ActiveValue::NotSet,
        cdn_path: ActiveValue::NotSet,
        thumbnail_url: ActiveValue::NotSet,
        file_size_bytes: ActiveValue::NotSet,
        display_order: ActiveValue::NotSet,
        created_at: ActiveValue::NotSet,
    };
    match product_images.update(txn).await {
        Ok(model) => {
            let response = ProductImagesResponse {
                items: vec![ProductImageResponse {
                    image_id: model.image_id,
                    product_id: model.product_id,
                    image_base64: model.image_base64.unwrap_or_default(),
                    alt_text: model.alt_text,
                    url: model.url,
                    cdn_path: model.cdn_path,
                    thumbnail_url: model.thumbnail_url,
                }],
            };
            Ok(Response::new(response))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
