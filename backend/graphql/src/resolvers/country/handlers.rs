//! Country resolvers: gRPC Country APIs were removed; schema kept for compatibility, returns empty.

use tracing::instrument;

use super::schema::{Country, NewCountry, SearchCountry};
use crate::resolvers::error::GqlError;

#[instrument]
pub(crate) async fn create_country(_country: NewCountry) -> Result<Vec<Country>, GqlError> {
    Ok(vec![])
}

#[instrument]
pub(crate) async fn search_country(_search: SearchCountry) -> Result<Vec<Country>, GqlError> {
    Ok(vec![])
}

#[instrument]
pub(crate) async fn delete_country(_country_id: String) -> Result<Vec<Country>, GqlError> {
    Ok(vec![])
}
