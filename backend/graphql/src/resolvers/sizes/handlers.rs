use proto::proto::core::{
    CreateSizeRequest, DeleteSizeRequest, SearchSizeRequest, SizeResponse, SizesResponse,
    UpdateSizeRequest,
};
use tracing::instrument;

use super::schema::{DeleteSizeInput, NewSize, SearchSizeInput, Size, SizeMutation};
use crate::resolvers::{
    error::GqlError,
    utils::{connect_grpc_client, parse_i64},
};

fn size_response_to_gql(s: SizeResponse) -> Size {
    Size {
        size_id: s.size_id.to_string(),
        size_name: s.size_name,
    }
}

fn sizes_response_to_vec(resp: SizesResponse) -> Vec<Size> {
    resp.items.into_iter().map(size_response_to_gql).collect()
}

#[instrument]
pub(crate) async fn create_size(input: NewSize) -> Result<Vec<Size>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .create_size(CreateSizeRequest {
            size_name: input.size_name,
        })
        .await?;
    Ok(sizes_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn search_size(input: SearchSizeInput) -> Result<Vec<Size>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .search_size(SearchSizeRequest {
            size_id: parse_i64(&input.size_id, "size_id")?,
        })
        .await?;
    Ok(sizes_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn update_size(input: SizeMutation) -> Result<Vec<Size>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .update_size(UpdateSizeRequest {
            size_id: parse_i64(&input.size_id, "size_id")?,
            size_name: input.size_name,
        })
        .await?;
    Ok(sizes_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn delete_size(input: DeleteSizeInput) -> Result<Vec<Size>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .delete_size(DeleteSizeRequest {
            size_id: parse_i64(&input.size_id, "size_id")?,
        })
        .await?;
    Ok(sizes_response_to_vec(resp.into_inner()))
}
