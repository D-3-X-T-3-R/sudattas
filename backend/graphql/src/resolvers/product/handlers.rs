use proto::proto::core::{
    CreateProductRequest, DeleteProductRequest, SearchProductRequest, UpdateProductRequest,
};

use tracing::instrument;

use super::schema::{NewProduct, Product, ProductMutation, SearchProduct};
use crate::resolvers::{
    convert,
    error::{Code, GqlError},
    utils::{connect_grpc_client, to_f64, to_i64, to_option_f64, to_option_i64},
};

#[instrument]
pub(crate) async fn create_product(product: NewProduct) -> Result<Vec<Product>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let name = product.name;
    let stock_quantity = product
        .stock_quantity
        .parse::<i64>()
        .map_err(|_| GqlError::new("Failed to parse stock quantity", Code::InvalidArgument))?;
    let price = product
        .price
        .parse::<f64>()
        .map_err(|_| GqlError::new("Failed to parse price", Code::InvalidArgument))?;
    let description = product.description;
    let category_id = product
        .category_id
        .parse::<i64>()
        .map_err(|_| GqlError::new("Failed to parse category id", Code::InvalidArgument))?;

    let response = client
        .create_product(CreateProductRequest {
            name: name,
            description: Some(description),
            price: price,
            stock_quantity: Some(stock_quantity),
            category_id: Some(category_id),
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

    let response = client
        .search_product(SearchProductRequest {
            name: search.name,
            description: search.description,
            starting_price: to_option_f64(search.starting_price),
            ending_price: to_option_f64(search.ending_price),
            stock_quantity: to_option_i64(search.stock_quantity),
            category_id: to_option_i64(search.category_id),
            product_id: to_option_i64(search.product_id),
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

    let product_id = product_id
        .parse::<i64>()
        .map_err(|_| GqlError::new("Failed to parse product id", Code::InvalidArgument))?;

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
            price: to_f64(product.price),
            stock_quantity: to_option_i64(product.stock_quantity),
            category_id: to_option_i64(product.category_id),
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
