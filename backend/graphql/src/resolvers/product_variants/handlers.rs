use proto::proto::core::{
    CreateProductVariantRequest, DeleteProductVariantRequest, ProductVariantResponse,
    ProductVariantsResponse, UpdateProductVariantRequest,
};
use tracing::instrument;

use super::schema::{
    DeleteProductVariantInput, NewProductVariant, ProductVariant, ProductVariantMutation,
};
use crate::resolvers::{
    error::GqlError,
    utils::{connect_grpc_client, parse_i64, to_option_i64},
};

fn variant_response_to_gql(v: ProductVariantResponse) -> ProductVariant {
    ProductVariant {
        variant_id: v.variant_id.to_string(),
        product_id: v.product_id.to_string(),
        size_id: v.size_id.map(|id| id.to_string()),
        color_id: v.color_id.map(|id| id.to_string()),
        additional_price_paise: v.additional_price_paise.map(|p| p.to_string()),
    }
}

fn variants_response_to_vec(resp: ProductVariantsResponse) -> Vec<ProductVariant> {
    resp.items
        .into_iter()
        .map(variant_response_to_gql)
        .collect()
}

#[instrument]
pub(crate) async fn create_product_variant(
    input: NewProductVariant,
) -> Result<Vec<ProductVariant>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .create_product_variant(CreateProductVariantRequest {
            product_id: parse_i64(&input.product_id, "product_id")?,
            size_id: to_option_i64(input.size_id),
            color_id: to_option_i64(input.color_id),
            additional_price_paise: input
                .additional_price_paise
                .as_deref()
                .map(|s| s.parse::<i64>())
                .transpose()
                .map_err(|_| {
                    GqlError::new(
                        "Failed to parse additional_price_paise",
                        crate::resolvers::error::Code::InvalidArgument,
                    )
                })?,
        })
        .await?;
    Ok(variants_response_to_vec(response.into_inner()))
}

#[instrument]
pub(crate) async fn update_product_variant(
    input: ProductVariantMutation,
) -> Result<Vec<ProductVariant>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .update_product_variant(UpdateProductVariantRequest {
            variant_id: parse_i64(&input.variant_id, "variant_id")?,
            product_id: input
                .product_id
                .as_deref()
                .map(|s| parse_i64(s, "product_id"))
                .transpose()?,
            size_id: to_option_i64(input.size_id),
            color_id: to_option_i64(input.color_id),
            additional_price_paise: input
                .additional_price_paise
                .as_deref()
                .map(|s| s.parse::<i64>())
                .transpose()
                .map_err(|_| {
                    GqlError::new(
                        "Failed to parse additional_price_paise",
                        crate::resolvers::error::Code::InvalidArgument,
                    )
                })?,
        })
        .await?;
    Ok(variants_response_to_vec(response.into_inner()))
}

#[instrument]
pub(crate) async fn delete_product_variant(
    input: DeleteProductVariantInput,
) -> Result<Vec<ProductVariant>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .delete_product_variant(DeleteProductVariantRequest {
            variant_id: parse_i64(&input.variant_id, "variant_id")?,
        })
        .await?;
    Ok(variants_response_to_vec(response.into_inner()))
}
