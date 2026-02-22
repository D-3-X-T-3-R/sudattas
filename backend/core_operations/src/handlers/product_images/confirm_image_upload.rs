use crate::handlers::db_errors::map_db_error_to_status;
use chrono::Utc;
use core_db_entities::entity::product_images;
use proto::proto::core::{ConfirmImageUploadRequest, ProductImageResponse, ProductImagesResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn confirm_image_upload(
    txn: &DatabaseTransaction,
    request: Request<ConfirmImageUploadRequest>,
) -> Result<Response<ProductImagesResponse>, Status> {
    let req = request.into_inner();

    let public_url =
        std::env::var("R2_PUBLIC_URL").unwrap_or_else(|_| "https://images.example.com".to_string());

    let cdn_url = format!("{}/{}", public_url.trim_end_matches('/'), req.key);

    // Build a thumbnail URL convention: same path but in a /thumbnails/ subfolder.
    // (The actual thumbnail generation would be done by a worker or image CDN transform.)
    let thumbnail_url = format!(
        "{}/thumbnails/{}",
        public_url.trim_end_matches('/'),
        req.key
    );

    let image = product_images::ActiveModel {
        image_id: ActiveValue::NotSet,
        product_id: ActiveValue::Set(req.product_id),
        image_base64: ActiveValue::Set(None),
        url: ActiveValue::Set(Some(cdn_url.clone())),
        cdn_path: ActiveValue::Set(Some(req.key.clone())),
        thumbnail_url: ActiveValue::Set(Some(thumbnail_url.clone())),
        file_size_bytes: ActiveValue::NotSet,
        alt_text: ActiveValue::Set(req.alt_text),
        display_order: ActiveValue::Set(req.display_order),
        created_at: ActiveValue::Set(Some(Utc::now())),
    };

    match image.insert(txn).await {
        Ok(model) => Ok(Response::new(ProductImagesResponse {
            items: vec![ProductImageResponse {
                image_id: model.image_id,
                product_id: model.product_id,
                image_base64: String::new(),
                alt_text: model.alt_text,
                url: model.url,
                cdn_path: model.cdn_path,
                thumbnail_url: model.thumbnail_url,
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
