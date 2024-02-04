use proto::proto::core::{CreateStateRequest, DeleteStateRequest, SearchStateRequest};

use tracing::instrument;

use super::schema::{NewState, SearchState, State};
use crate::resolvers::{
    error::{Code, GqlError},
    utils::{connect_grpc_client, to_option_i64},
};

#[instrument]
pub(crate) async fn create_state(state: NewState) -> Result<Vec<State>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let response = client
        .create_state(CreateStateRequest {
            state_name: state.state_name,
        })
        .await
        .map_err(|e| GqlError::new(&format!("gRPC request failed: {}", e), Code::Internal))?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(|state| State {
            state_name: state.state_name,
            state_id: state.state_id.to_string(),
        })
        .collect())
}

#[instrument]
pub(crate) async fn search_state(search: SearchState) -> Result<Vec<State>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let response = client
        .search_state(SearchStateRequest {
            state_name: search.state_name,
            state_id: to_option_i64(search.state_id),
        })
        .await
        .map_err(|e| GqlError::new(&format!("gRPC request failed: {}", e), Code::Internal))?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(|state| State {
            state_name: state.state_name,
            state_id: state.state_id.to_string(),
        })
        .collect())
}

#[instrument]
pub(crate) async fn delete_state(state_id: String) -> Result<Vec<State>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let state_id = state_id
        .parse::<i64>()
        .map_err(|_| GqlError::new("Failed to parse state id", Code::InvalidArgument))?;

    let response = client
        .delete_state(DeleteStateRequest { state_id })
        .await
        .map_err(|e| GqlError::new(&format!("gRPC request failed: {}", e), Code::Internal))?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(|state| State {
            state_name: state.state_name,
            state_id: state.state_id.to_string(),
        })
        .collect())
}
