use crate::handlers::db_errors::map_db_error_to_status;
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

    let order_key = req.display_order.unwrap_or(1).to_string();
    let mut urls_map = serde_json::Map::new();
    urls_map.insert(
        order_key.clone(),
        serde_json::Value::String(cdn_url.clone()),
    );
    let urls_json = serde_json::Value::Object(urls_map);
    let image = product_images::ActiveModel {
        image_id: ActiveValue::NotSet,
        product_id: ActiveValue::Set(req.product_id),
        urls: ActiveValue::Set(urls_json),
        created_at: ActiveValue::NotSet,
    };

    match image.insert(txn).await {
        Ok(model) => {
            let key = req.display_order.unwrap_or(1).to_string();
            let url = model
                .urls
                .get(key.as_str())
                .or_else(|| model.urls.get("1"))
                .and_then(|v| v.as_str())
                .map(String::from);
            Ok(Response::new(ProductImagesResponse {
                items: vec![ProductImageResponse {
                    image_id: model.image_id,
                    product_id: model.product_id,
                    image_base64: String::new(),
                    alt_text: req.alt_text,
                    url,
                    cdn_path: Some(req.key),
                    thumbnail_url: Some(thumbnail_url),
                }],
            }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
