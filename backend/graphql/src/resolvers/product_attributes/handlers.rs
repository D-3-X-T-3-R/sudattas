use proto::proto::core::{
    CreateProductAttributeRequest, DeleteProductAttributeRequest, ProductAttributeResponse,
    ProductAttributesResponse, SearchProductAttributeRequest, UpdateProductAttributeRequest,
};
use tracing::instrument;

use super::schema::{
    DeleteProductAttributeInput, NewProductAttribute, ProductAttribute, ProductAttributeMutation,
    SearchProductAttributeInput,
};
use crate::resolvers::{
    error::GqlError,
    utils::{connect_grpc_client, parse_i64, to_option_i64},
};

fn attr_response_to_gql(a: ProductAttributeResponse) -> ProductAttribute {
    ProductAttribute {
        attribute_id: a.attribute_id.to_string(),
        attribute_name: a.attribute_name,
        attribute_value: a.attribute_value,
    }
}

fn attrs_response_to_vec(resp: ProductAttributesResponse) -> Vec<ProductAttribute> {
    resp.items.into_iter().map(attr_response_to_gql).collect()
}

#[instrument]
pub(crate) async fn create_product_attribute(
    input: NewProductAttribute,
) -> Result<Vec<ProductAttribute>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .create_product_attribute(CreateProductAttributeRequest {
            attribute_name: input.attribute_name,
            attribute_value: input.attribute_value,
        })
        .await?;
    Ok(attrs_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn search_product_attribute(
    input: SearchProductAttributeInput,
) -> Result<Vec<ProductAttribute>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .search_product_attribute(SearchProductAttributeRequest {
            attribute_id: to_option_i64(input.attribute_id),
            attribute_name: input.attribute_name,
            attribute_value: input.attribute_value,
        })
        .await?;
    Ok(attrs_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn update_product_attribute(
    input: ProductAttributeMutation,
) -> Result<Vec<ProductAttribute>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .update_product_attribute(UpdateProductAttributeRequest {
            attribute_id: parse_i64(&input.attribute_id, "attribute_id")?,
            attribute_name: input.attribute_name,
            attribute_value: input.attribute_value,
        })
        .await?;
    Ok(attrs_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn delete_product_attribute(
    input: DeleteProductAttributeInput,
) -> Result<Vec<ProductAttribute>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .delete_product_attribute(DeleteProductAttributeRequest {
            attribute_id: parse_i64(&input.attribute_id, "attribute_id")?,
        })
        .await?;
    Ok(attrs_response_to_vec(resp.into_inner()))
}
