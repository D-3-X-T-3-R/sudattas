use proto::proto::core::{
    CreateFabricRequest, DeleteFabricRequest, FabricResponse, FabricsResponse, SearchFabricRequest,
    UpdateFabricRequest,
};
use tracing::instrument;

use super::schema::{DeleteFabricInput, Fabric, FabricMutation, NewFabric, SearchFabricInput};
use crate::resolvers::{
    error::GqlError,
    utils::{connect_grpc_client, parse_i64},
};

fn fabric_response_to_gql(f: FabricResponse) -> Fabric {
    Fabric {
        fabric_id: f.fabric_id.to_string(),
        fabric_name: f.fabric_name,
    }
}

fn fabrics_response_to_vec(resp: FabricsResponse) -> Vec<Fabric> {
    resp.items.into_iter().map(fabric_response_to_gql).collect()
}

#[instrument]
pub(crate) async fn create_fabric(input: NewFabric) -> Result<Vec<Fabric>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .create_fabric(CreateFabricRequest {
            fabric_name: input.fabric_name,
        })
        .await?;
    Ok(fabrics_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn search_fabric(input: SearchFabricInput) -> Result<Vec<Fabric>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .search_fabric(SearchFabricRequest {
            fabric_id: parse_i64(&input.fabric_id, "fabric_id")?,
        })
        .await?;
    Ok(fabrics_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn update_fabric(input: FabricMutation) -> Result<Vec<Fabric>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .update_fabric(UpdateFabricRequest {
            fabric_id: parse_i64(&input.fabric_id, "fabric_id")?,
            fabric_name: input.fabric_name,
        })
        .await?;
    Ok(fabrics_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn delete_fabric(input: DeleteFabricInput) -> Result<Vec<Fabric>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .delete_fabric(DeleteFabricRequest {
            fabric_id: parse_i64(&input.fabric_id, "fabric_id")?,
        })
        .await?;
    Ok(fabrics_response_to_vec(resp.into_inner()))
}
