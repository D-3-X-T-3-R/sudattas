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
    let base = public_url.trim_end_matches('/');

    let cdn_url = req
        .url
        .clone()
        .unwrap_or_else(|| format!("{}/{}", base, req.key));
    let thumbnail_url = if req.url.is_some() {
        None
    } else {
        Some(format!("{}/thumbnails/{}", base, req.key))
    };
    let display_order = req.display_order.unwrap_or(0);

    let model = product_images::ActiveModel {
        image_id: ActiveValue::NotSet,
        product_id: ActiveValue::Set(req.product_id),
        display_order: ActiveValue::Set(display_order),
        url: ActiveValue::Set(cdn_url.clone()),
        created_at: ActiveValue::NotSet,
    }
    .insert(txn)
    .await
    .map_err(|e| tonic::Status::internal(e.to_string()))?;

    Ok(Response::new(ProductImagesResponse {
        items: vec![ProductImageResponse {
            image_id: model.image_id,
            product_id: model.product_id,
            image_base64: String::new(),
            alt_text: req.alt_text,
            url: Some(cdn_url),
            cdn_path: if req.key.is_empty() {
                None
            } else {
                Some(req.key)
            },
            thumbnail_url,
        }],
    }))
}
