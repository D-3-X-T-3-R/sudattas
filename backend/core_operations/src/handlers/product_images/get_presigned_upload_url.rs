use std::path::Path;
use std::time::Duration;

use aws_sdk_s3::presigning::PresigningConfig;
use core_db_entities::entity::{product_categories, product_images, products};
use proto::proto::core::{GetPresignedUploadUrlRequest, PresignedUploadUrlResponse};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use super::r2_client::build_r2_client;
use crate::handlers::db_errors::map_db_error_to_status;

/// Generate a presigned upload URL for a product image.
/// The R2 object key follows:
/// `product/<category_name>/<product_id>_<sku>_<index>_<unique>.<extension>`
/// The `<unique>` suffix (8 hex chars) avoids collisions when two uploads get the same index (e.g. concurrent requests).
pub async fn get_presigned_upload_url(
    db: &DatabaseConnection,
    request: Request<GetPresignedUploadUrlRequest>,
) -> Result<Response<PresignedUploadUrlResponse>, Status> {
    let req = request.into_inner();

    // Look up product metadata (category_id and sku) for folder/filename structure.
    let product = products::Entity::find_by_id(req.product_id)
        .one(db)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::not_found(format!("Product {} not found", req.product_id)))?;

    // Look up category name for folder structure.
    let category = product_categories::Entity::find_by_id(product.category_id)
        .one(db)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| {
            Status::not_found(format!(
                "Category {} not found for product {}",
                product.category_id, product.product_id
            ))
        })?;

    // Derive safe segments for category and SKU for the filename.
    let raw_category = category.name;
    let safe_category: String = raw_category
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();

    // Derive a safe SKU segment for the filename.
    let raw_sku = product
        .sku
        .clone()
        .unwrap_or_else(|| format!("p{}", product.product_id));
    let safe_sku: String = raw_sku
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();

    // Use display_order when provided (avoids race when uploading multiple images in parallel).
    // Otherwise compute next index from max(display_order) + 1 for this product.
    let next_index: i32 = if let Some(order) = req.display_order {
        order + 1 // 0-based order → 1-based index for filename
    } else {
        let rows = product_images::Entity::find()
            .filter(product_images::Column::ProductId.eq(product.product_id))
            .all(db)
            .await
            .map_err(map_db_error_to_status)?;
        rows.into_iter().map(|m| m.display_order).max().unwrap_or(0) + 1
    };

    // Preserve the original file extension, if present.
    let ext = Path::new(&req.filename)
        .extension()
        .and_then(|os| os.to_str())
        .unwrap_or("");

    // Unique suffix so two uploads never get the same key (avoids overwrite when index collides).
    let unique: String = Uuid::new_v4()
        .simple()
        .to_string()
        .chars()
        .take(8)
        .collect();

    let key = if ext.is_empty() {
        format!(
            "product/{}/{}_{}_{}_{}",
            safe_category, product.product_id, safe_sku, next_index, unique
        )
    } else {
        format!(
            "product/{}/{}_{}_{}_{}.{}",
            safe_category, product.product_id, safe_sku, next_index, unique, ext
        )
    };

    let (client, bucket, public_url) = build_r2_client()
        .ok_or_else(|| Status::failed_precondition("R2 not configured; check R2_* env vars"))?;

    let presigning_config = PresigningConfig::builder()
        .expires_in(Duration::from_secs(15 * 60)) // 15-minute window
        .build()
        .map_err(|e| Status::internal(e.to_string()))?;

    let presigned = client
        .put_object()
        .bucket(&bucket)
        .key(&key)
        .content_type(&req.content_type)
        .presigned(presigning_config)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    let cdn_url = format!("{}/{}", public_url.trim_end_matches('/'), key);

    Ok(Response::new(PresignedUploadUrlResponse {
        upload_url: presigned.uri().to_string(),
        key,
        cdn_url,
    }))
}
