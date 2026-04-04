use proto::proto::core::{
    CreateProductMoodMappingRequest, DeleteProductMoodMappingRequest, ProductMoodMappingResponse,
    ProductMoodMappingsResponse, SearchProductMoodMappingRequest,
};
use tracing::instrument;

use super::schema::{
    DeleteProductMoodMappingInput, NewProductMoodMapping, ProductMoodMapping,
    SearchProductMoodMappingInput,
};
use crate::resolvers::{
    error::GqlError,
    utils::{connect_grpc_client, parse_i64},
};

fn mapping_response_to_gql(m: ProductMoodMappingResponse) -> ProductMoodMapping {
    ProductMoodMapping {
        product_id: m.product_id.to_string(),
        mood_id: m.mood_id.to_string(),
    }
}

fn mappings_response_to_vec(resp: ProductMoodMappingsResponse) -> Vec<ProductMoodMapping> {
    resp.items
        .into_iter()
        .map(mapping_response_to_gql)
        .collect()
}

#[instrument]
pub(crate) async fn create_product_mood_mapping(
    input: NewProductMoodMapping,
) -> Result<Vec<ProductMoodMapping>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .create_product_mood_mapping(CreateProductMoodMappingRequest {
            product_id: parse_i64(&input.product_id, "product_id")?,
            mood_id: parse_i64(&input.mood_id, "mood_id")?,
        })
        .await?;
    Ok(mappings_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn search_product_mood_mapping(
    input: SearchProductMoodMappingInput,
) -> Result<Vec<ProductMoodMapping>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let mood_id_opt = match &input.mood_id {
        None => None,
        Some(s) => Some(parse_i64(s, "mood_id")?),
    };
    let resp = client
        .search_product_mood_mapping(SearchProductMoodMappingRequest {
            product_id: parse_i64(&input.product_id, "product_id")?,
            mood_id: mood_id_opt,
        })
        .await?;
    Ok(mappings_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn delete_product_mood_mapping(
    input: DeleteProductMoodMappingInput,
) -> Result<Vec<ProductMoodMapping>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .delete_product_mood_mapping(DeleteProductMoodMappingRequest {
            product_id: parse_i64(&input.product_id, "product_id")?,
            mood_id: parse_i64(&input.mood_id, "mood_id")?,
        })
        .await?;
    Ok(mappings_response_to_vec(resp.into_inner()))
}
