use proto::proto::core::{
    CreateWeaveRequest, DeleteWeaveRequest, SearchWeaveRequest, UpdateWeaveRequest, WeaveResponse,
    WeavesResponse,
};
use tracing::instrument;

use super::schema::{DeleteWeaveInput, NewWeave, SearchWeaveInput, Weave, WeaveMutation};
use crate::resolvers::{
    error::GqlError,
    utils::{connect_grpc_client, parse_i64},
};

fn weave_response_to_gql(w: WeaveResponse) -> Weave {
    Weave {
        weave_id: w.weave_id.to_string(),
        weave_name: w.weave_name,
    }
}

fn weaves_response_to_vec(resp: WeavesResponse) -> Vec<Weave> {
    resp.items.into_iter().map(weave_response_to_gql).collect()
}

#[instrument]
pub(crate) async fn create_weave(input: NewWeave) -> Result<Vec<Weave>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .create_weave(CreateWeaveRequest {
            weave_name: input.weave_name,
        })
        .await?;
    Ok(weaves_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn search_weave(input: SearchWeaveInput) -> Result<Vec<Weave>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .search_weave(SearchWeaveRequest {
            weave_id: parse_i64(&input.weave_id, "weave_id")?,
        })
        .await?;
    Ok(weaves_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn update_weave(input: WeaveMutation) -> Result<Vec<Weave>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .update_weave(UpdateWeaveRequest {
            weave_id: parse_i64(&input.weave_id, "weave_id")?,
            weave_name: input.weave_name,
        })
        .await?;
    Ok(weaves_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn delete_weave(input: DeleteWeaveInput) -> Result<Vec<Weave>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .delete_weave(DeleteWeaveRequest {
            weave_id: parse_i64(&input.weave_id, "weave_id")?,
        })
        .await?;
    Ok(weaves_response_to_vec(resp.into_inner()))
}

