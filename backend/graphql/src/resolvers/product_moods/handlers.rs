use proto::proto::core::{
    CreateProductMoodRequest, DeleteProductMoodRequest, ProductMoodResponse, ProductMoodsResponse,
    SearchProductMoodRequest, UpdateProductMoodRequest,
};
use tracing::instrument;

use super::schema::{
    DeleteProductMoodInput, NewProductMood, ProductMood, ProductMoodMutation,
    SearchProductMoodInput,
};
use crate::resolvers::{
    error::GqlError,
    utils::{connect_grpc_client, parse_i64, to_option_i64},
};

fn mood_response_to_gql(m: ProductMoodResponse) -> ProductMood {
    ProductMood {
        mood_id: m.mood_id.to_string(),
        mood_name: m.mood_name,
    }
}

fn moods_response_to_vec(resp: ProductMoodsResponse) -> Vec<ProductMood> {
    resp.items.into_iter().map(mood_response_to_gql).collect()
}

#[instrument]
pub(crate) async fn create_product_mood(
    input: NewProductMood,
) -> Result<Vec<ProductMood>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .create_product_mood(CreateProductMoodRequest {
            mood_name: input.mood_name,
        })
        .await?;
    Ok(moods_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn search_product_mood(
    input: SearchProductMoodInput,
) -> Result<Vec<ProductMood>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .search_product_mood(SearchProductMoodRequest {
            mood_id: to_option_i64(input.mood_id),
            mood_name: input.mood_name,
        })
        .await?;
    Ok(moods_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn update_product_mood(
    input: ProductMoodMutation,
) -> Result<Vec<ProductMood>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .update_product_mood(UpdateProductMoodRequest {
            mood_id: parse_i64(&input.mood_id, "mood_id")?,
            mood_name: input.mood_name,
        })
        .await?;
    Ok(moods_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn delete_product_mood(
    input: DeleteProductMoodInput,
) -> Result<Vec<ProductMood>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .delete_product_mood(DeleteProductMoodRequest {
            mood_id: parse_i64(&input.mood_id, "mood_id")?,
        })
        .await?;
    Ok(moods_response_to_vec(resp.into_inner()))
}
