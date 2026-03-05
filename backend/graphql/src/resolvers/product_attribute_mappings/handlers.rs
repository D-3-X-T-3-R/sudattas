use proto::proto::core::{
    CreateProductAttributeMappingRequest, DeleteProductAttributeMappingRequest,
    ProductAttributeMappingResponse, ProductAttributeMappingsResponse,
    SearchProductAttributeMappingRequest,
};
use tracing::instrument;

use super::schema::{
    DeleteProductAttributeMappingInput, NewProductAttributeMapping, ProductAttributeMapping,
    SearchProductAttributeMappingInput,
};
use crate::resolvers::{
    error::GqlError,
    utils::{connect_grpc_client, parse_i64},
};

fn mapping_response_to_gql(m: ProductAttributeMappingResponse) -> ProductAttributeMapping {
    ProductAttributeMapping {
        product_id: m.product_id.to_string(),
        attribute_id: m.attribute_id.to_string(),
    }
}

fn mappings_response_to_vec(
    resp: ProductAttributeMappingsResponse,
) -> Vec<ProductAttributeMapping> {
    resp.items
        .into_iter()
        .map(mapping_response_to_gql)
        .collect()
}

#[instrument]
pub(crate) async fn create_product_attribute_mapping(
    input: NewProductAttributeMapping,
) -> Result<Vec<ProductAttributeMapping>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .create_product_attribute_mapping(CreateProductAttributeMappingRequest {
            product_id: parse_i64(&input.product_id, "product_id")?,
            attribute_id: parse_i64(&input.attribute_id, "attribute_id")?,
        })
        .await?;
    Ok(mappings_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn search_product_attribute_mapping(
    input: SearchProductAttributeMappingInput,
) -> Result<Vec<ProductAttributeMapping>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .search_product_attribute_mapping(SearchProductAttributeMappingRequest {
            product_id: parse_i64(&input.product_id, "product_id")?,
            attribute_id: parse_i64(&input.attribute_id, "attribute_id")?,
        })
        .await?;
    Ok(mappings_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn delete_product_attribute_mapping(
    input: DeleteProductAttributeMappingInput,
) -> Result<Vec<ProductAttributeMapping>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .delete_product_attribute_mapping(DeleteProductAttributeMappingRequest {
            product_id: parse_i64(&input.product_id, "product_id")?,
            attribute_id: parse_i64(&input.attribute_id, "attribute_id")?,
        })
        .await?;
    Ok(mappings_response_to_vec(resp.into_inner()))
}
