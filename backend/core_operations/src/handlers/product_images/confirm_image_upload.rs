use core_db_entities::entity::product_images;
use proto::proto::core::{ConfirmImageUploadRequest, ProductImageResponse, ProductImagesResponse};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter,
};
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
    let thumbnail_url = format!(
        "{}/thumbnails/{}",
        public_url.trim_end_matches('/'),
        req.key
    );

    let order_key = req.display_order.unwrap_or(0).to_string();

    // One row per product: merge new URL into existing urls JSON, or insert new row.
    let existing = product_images::Entity::find()
        .filter(product_images::Column::ProductId.eq(req.product_id))
        .one(txn)
        .await
        .map_err(|e| tonic::Status::internal(e.to_string()))?;

    let model = if let Some(existing) = existing {
        let mut urls_map = match existing.urls.as_object() {
            Some(m) => m.clone(),
            None => serde_json::Map::new(),
        };
        urls_map.insert(order_key, serde_json::Value::String(cdn_url.clone()));
        let urls_json = serde_json::Value::Object(urls_map);
        let active = product_images::ActiveModel {
            image_id: ActiveValue::Set(existing.image_id),
            product_id: ActiveValue::Set(existing.product_id),
            urls: ActiveValue::Set(urls_json),
            created_at: ActiveValue::NotSet,
        };
        active
            .update(txn)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?
    } else {
        let mut urls_map = serde_json::Map::new();
        urls_map.insert(order_key, serde_json::Value::String(cdn_url.clone()));
        let urls_json = serde_json::Value::Object(urls_map);
        let image = product_images::ActiveModel {
            image_id: ActiveValue::NotSet,
            product_id: ActiveValue::Set(req.product_id),
            urls: ActiveValue::Set(urls_json),
            created_at: ActiveValue::NotSet,
        };
        image
            .insert(txn)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?
    };

    Ok(Response::new(ProductImagesResponse {
        items: vec![ProductImageResponse {
            image_id: model.image_id,
            product_id: model.product_id,
            image_base64: String::new(),
            alt_text: req.alt_text,
            url: Some(cdn_url),
            cdn_path: Some(req.key),
            thumbnail_url: Some(thumbnail_url),
        }],
    }))
}
