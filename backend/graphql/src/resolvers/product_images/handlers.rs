use proto::proto::core::{
    ConfirmImageUploadRequest, DeleteProductImageRequest, GetPresignedUploadUrlRequest,
    SearchProductImageRequest, SyncProductImageItem, SyncProductImagesRequest,
    UpdateProductImageRequest,
};

use tracing::instrument;

use super::schema::{
    ConfirmImageUpload, GetPresignedUploadUrl, PresignedUploadUrl, ProductImage,
    ProductImageMutation, SearchProductImage, SyncProductImagesInput,
};
use crate::resolvers::{
    convert,
    error::GqlError,
    utils::{connect_grpc_client, parse_i64, to_i64, to_option_i64},
};

#[instrument]
pub(crate) async fn search_product_image(
    search: SearchProductImage,
) -> Result<Vec<ProductImage>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let response = client
        .search_product_image(SearchProductImageRequest {
            image_id: to_option_i64(search.image_id),
            product_id: to_option_i64(search.product_id),
            alt_text: search.alt_text,
        })
        .await?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(convert::product_image_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn delete_product_image(image_id: String) -> Result<Vec<ProductImage>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let response = client
        .delete_product_image(DeleteProductImageRequest {
            image_id: to_i64(image_id),
        })
        .await?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(convert::product_image_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn update_product_image(
    product_image: ProductImageMutation,
) -> Result<Vec<ProductImage>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let response = client
        .update_product_image(UpdateProductImageRequest {
            image_base64: product_image.image_base64,
            alt_text: product_image.alt_text,
            image_id: to_i64(product_image.image_id),
            product_id: to_i64(product_image.product_id),
        })
        .await?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(convert::product_image_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn get_presigned_upload_url(
    input: GetPresignedUploadUrl,
) -> Result<Vec<PresignedUploadUrl>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .get_presigned_upload_url(GetPresignedUploadUrlRequest {
            product_id: parse_i64(&input.product_id, "product id")?,
            filename: input.filename,
            content_type: input.content_type,
            display_order: input.display_order,
        })
        .await?;
    let r = response.into_inner();
    Ok(vec![PresignedUploadUrl {
        upload_url: r.upload_url,
        key: r.key,
        cdn_url: r.cdn_url,
    }])
}

#[instrument]
pub(crate) async fn confirm_image_upload(
    input: ConfirmImageUpload,
) -> Result<Vec<ProductImage>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .confirm_image_upload(ConfirmImageUploadRequest {
            product_id: parse_i64(&input.product_id, "product id")?,
            key: input.key,
            alt_text: input.alt_text,
            display_order: input.display_order,
            url: input.url,
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(convert::product_image_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn sync_product_images(
    input: SyncProductImagesInput,
) -> Result<Vec<ProductImage>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let product_id = parse_i64(&input.product_id, "product id")?;
    let items: Vec<SyncProductImageItem> = input
        .items
        .into_iter()
        .map(|i| SyncProductImageItem {
            image_id: i.image_id.and_then(|id| id.parse().ok()),
            key: i.key,
            url: i.url,
        })
        .collect();
    let response = client
        .sync_product_images(SyncProductImagesRequest { product_id, items })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(convert::product_image_response_to_gql)
        .collect())
}
