use proto::proto::core::{
    CreateProductMoodRequest, DeleteProductMoodRequest, ProductMoodResponse, ProductMoodsResponse,
    SearchProductMoodRequest, ShopHighlightMoodsRequest, UpdateProductMoodRequest,
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

/// Storefront: up to `max_moods` distinct moods from the newest `recent_product_limit` products.
#[instrument]
pub(crate) async fn shop_highlight_moods(
    recent_product_limit: Option<i32>,
    max_moods: Option<i32>,
) -> Result<Vec<ProductMood>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .shop_highlight_moods(ShopHighlightMoodsRequest {
            recent_product_limit: recent_product_limit.map(|v| v as i64),
            max_moods: max_moods.map(|v| v as i64),
        })
        .await?;
    let out: Vec<ProductMood> = resp
        .into_inner()
        .items
        .into_iter()
        .map(|i| ProductMood {
            mood_id: i.mood_id.to_string(),
            mood_name: i.mood_name,
        })
        .collect();
    Ok(out)
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
