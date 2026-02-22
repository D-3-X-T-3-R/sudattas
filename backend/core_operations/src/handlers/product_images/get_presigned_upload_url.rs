use aws_sdk_s3::presigning::PresigningConfig;
use chrono::Utc;
use proto::proto::core::{GetPresignedUploadUrlRequest, PresignedUploadUrlResponse};
use std::time::Duration;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use super::r2_client::build_r2_client;

pub async fn get_presigned_upload_url(
    request: Request<GetPresignedUploadUrlRequest>,
) -> Result<Response<PresignedUploadUrlResponse>, Status> {
    let req = request.into_inner();

    let (client, bucket, public_url) = build_r2_client()
        .ok_or_else(|| Status::failed_precondition("R2 not configured; check R2_* env vars"))?;

    // Unique key: products/{product_id}/{uuid}/{filename}
    let key = format!(
        "products/{}/{}/{}",
        req.product_id,
        Uuid::new_v4(),
        req.filename
    );

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

/// Unused but kept to satisfy the import of Utc if needed elsewhere.
#[allow(dead_code)]
fn _unused() {
    let _ = Utc::now();
}
