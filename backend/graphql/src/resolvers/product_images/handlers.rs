use proto::proto::core::{
    AddProductImageRequest, DeleteProductImageRequest, ProductImageRequest,
    SearchProductImageRequest, UpdateProductImageRequest,
};

use tracing::instrument;

use super::schema::{NewProductImage, ProductImage, ProductImageMutation, SearchProductImage};
use crate::resolvers::{
    error::{Code, GqlError},
    utils::{connect_grpc_client, to_i64, to_option_i64},
};

#[instrument]
pub(crate) async fn add_product_image(
    product_image: NewProductImage,
) -> Result<Vec<ProductImage>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let response = client
        .add_product_image(AddProductImageRequest {
            product_id: to_i64(product_image.product_id),
            product_images: product_image
                .product_images
                .into_iter()
                .map(|pi| ProductImageRequest {
                    image_base64: pi.image_base64,
                    alt_text: pi.alt_text,
                })
                .collect(),
        })
        .await
        .map_err(|e| GqlError::new(&format!("gRPC request failed: {}", e), Code::Internal))?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(|product_image| ProductImage {
            image_id: product_image.image_id.to_string(),
            product_id: product_image.product_id.to_string(),
            image_base64: product_image.image_base64.to_string(),
            alt_text: product_image.alt_text,
        })
        .collect())
}

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
        .await
        .map_err(|e| GqlError::new(&format!("gRPC request failed: {}", e), Code::Internal))?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(|product_image| ProductImage {
            image_id: product_image.image_id.to_string(),
            product_id: product_image.product_id.to_string(),
            image_base64: product_image.image_base64.to_string(),
            alt_text: product_image.alt_text,
        })
        .collect())
}

#[instrument]
pub(crate) async fn delete_product_image(image_id: String) -> Result<Vec<ProductImage>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let response = client
        .delete_product_image(DeleteProductImageRequest {
            image_id: to_i64(image_id),
        })
        .await
        .map_err(|e| GqlError::new(&format!("gRPC request failed: {}", e), Code::Internal))?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(|product_image| ProductImage {
            image_id: product_image.image_id.to_string(),
            product_id: product_image.product_id.to_string(),
            image_base64: product_image.image_base64.to_string(),
            alt_text: product_image.alt_text,
        })
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
        .await
        .map_err(|e| GqlError::new(&format!("gRPC request failed: {}", e), Code::Internal))?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(|product_image| ProductImage {
            image_id: product_image.image_id.to_string(),
            product_id: product_image.product_id.to_string(),
            image_base64: product_image.image_base64.to_string(),
            alt_text: product_image.alt_text,
        })
        .collect())
}
