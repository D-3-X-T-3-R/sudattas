use proto::proto::core::{
    CreateOccasionRequest, DeleteOccasionRequest, OccasionResponse, OccasionsResponse,
    SearchOccasionRequest, UpdateOccasionRequest,
};
use tracing::instrument;

use super::schema::{
    DeleteOccasionInput, NewOccasion, Occasion, OccasionMutation, SearchOccasionInput,
};
use crate::resolvers::{
    error::GqlError,
    utils::{connect_grpc_client, parse_i64},
};

fn occasion_response_to_gql(o: OccasionResponse) -> Occasion {
    Occasion {
        occasion_id: o.occasion_id.to_string(),
        occasion_name: o.occasion_name,
    }
}

fn occasions_response_to_vec(resp: OccasionsResponse) -> Vec<Occasion> {
    resp.items
        .into_iter()
        .map(occasion_response_to_gql)
        .collect()
}

#[instrument]
pub(crate) async fn create_occasion(input: NewOccasion) -> Result<Vec<Occasion>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .create_occasion(CreateOccasionRequest {
            occasion_name: input.occasion_name,
        })
        .await?;
    Ok(occasions_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn search_occasion(input: SearchOccasionInput) -> Result<Vec<Occasion>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .search_occasion(SearchOccasionRequest {
            occasion_id: parse_i64(&input.occasion_id, "occasion_id")?,
        })
        .await?;
    Ok(occasions_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn update_occasion(input: OccasionMutation) -> Result<Vec<Occasion>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .update_occasion(UpdateOccasionRequest {
            occasion_id: parse_i64(&input.occasion_id, "occasion_id")?,
            occasion_name: input.occasion_name,
        })
        .await?;
    Ok(occasions_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn delete_occasion(input: DeleteOccasionInput) -> Result<Vec<Occasion>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .delete_occasion(DeleteOccasionRequest {
            occasion_id: parse_i64(&input.occasion_id, "occasion_id")?,
        })
        .await?;
    Ok(occasions_response_to_vec(resp.into_inner()))
}
