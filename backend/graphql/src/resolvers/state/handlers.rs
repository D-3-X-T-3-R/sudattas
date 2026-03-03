//! State resolvers: gRPC State APIs were removed; schema kept for compatibility, returns empty.

use tracing::instrument;

use super::schema::{NewState, SearchState, State};
use crate::resolvers::error::GqlError;

#[instrument]
pub(crate) async fn create_state(_state: NewState) -> Result<Vec<State>, GqlError> {
    Ok(vec![])
}

#[instrument]
pub(crate) async fn search_state(_search: SearchState) -> Result<Vec<State>, GqlError> {
    Ok(vec![])
}

#[instrument]
pub(crate) async fn delete_state(_state_id: String) -> Result<Vec<State>, GqlError> {
    Ok(vec![])
}
