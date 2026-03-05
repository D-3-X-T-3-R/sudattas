use proto::proto::core::{
    CreateProductRequest, DeleteProductRequest, GetProductsByIdRequest, GetRelatedProductsRequest,
    SearchProductRequest, SearchProductVariantRequest, UpdateProductRequest,
};

use tracing::instrument;

use super::schema::{GetRelatedProducts, NewProduct, Product, ProductMutation, SearchProduct};
use crate::resolvers::{
    convert,
    error::GqlError,
    utils::{connect_grpc_client, parse_i64, to_i64, to_option_i64},
};

#[instrument]
pub(crate) async fn create_product(product: NewProduct) -> Result<Vec<Product>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let name = product.name;
    let price_paise = parse_i64(&product.price_paise, "price_paise")?;
    let description = product.description;
    let category_id = parse_i64(&product.category_id, "category id")?;

    let response = client
        .create_product(CreateProductRequest {
            name,
            description: Some(description),
            price_paise,
            category_id,
        })
        .await?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(convert::product_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn search_product(search: SearchProduct) -> Result<Vec<Product>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let limit = crate::graphql_limits::cap_page_size(to_option_i64(search.limit));
    let response = client
        .search_product(SearchProductRequest {
            name: search.name,
            description: search.description,
            starting_price_paise: search
                .starting_price_paise
                .as_ref()
                .and_then(|s| s.parse().ok()),
            ending_price_paise: search
                .ending_price_paise
                .as_ref()
                .and_then(|s| s.parse().ok()),
            category_id: to_option_i64(search.category_id),
            product_id: to_option_i64(search.product_id),
            limit,
            offset: to_option_i64(search.offset),
        })
        .await?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(convert::product_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn delete_product(product_id: String) -> Result<Vec<Product>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let product_id = parse_i64(&product_id, "product id")?;

    let response = client
        .delete_product(DeleteProductRequest { product_id })
        .await?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(convert::product_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn update_product(product: ProductMutation) -> Result<Vec<Product>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let response = client
        .update_product(UpdateProductRequest {
            name: product.name,
            description: Some(product.description),
            price_paise: parse_i64(&product.price_paise, "price_paise")?,
            category_id: parse_i64(&product.category_id, "category id")?,
            product_id: to_i64(product.product_id),
        })
        .await?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(convert::product_response_to_gql)
        .collect())
}

/// Resolve product(s) for a variant (e.g. for cart/order detail line). Uses variant_id -> product_id.
#[instrument]
pub(crate) async fn get_products_for_variant(variant_id: &str) -> Result<Vec<Product>, GqlError> {
    let variant_id = parse_i64(variant_id, "variant id")?;
    let mut client = connect_grpc_client().await?;
    let variant_resp = client
        .search_product_variant(SearchProductVariantRequest { variant_id })
        .await?;
    let items = variant_resp.into_inner().items;
    let product_ids: Vec<i64> = items.into_iter().map(|v| v.product_id).collect();
    if product_ids.is_empty() {
        return Ok(Vec::new());
    }
    let resp = client
        .get_products_by_id(GetProductsByIdRequest { product_ids })
        .await?;
    Ok(resp
        .into_inner()
        .items
        .into_iter()
        .map(convert::product_response_to_gql)
        .collect())
}

/// P2 Recommendations: fetch related products for a given product.
#[instrument]
pub(crate) async fn get_related_products(
    input: GetRelatedProducts,
) -> Result<Vec<Product>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let product_id = parse_i64(&input.product_id, "product_id")?;
    let limit = to_option_i64(input.limit);
    let resp = client
        .get_related_products(GetRelatedProductsRequest { product_id, limit })
        .await?;
    Ok(resp
        .into_inner()
        .items
        .into_iter()
        .map(convert::product_response_to_gql)
        .collect())
}
