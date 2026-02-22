use proto::proto::core::{CreateCountryRequest, DeleteCountryRequest, SearchCountryRequest};

use tracing::instrument;

use super::schema::{Country, NewCountry, SearchCountry};
use crate::resolvers::{
    convert,
    error::GqlError,
    utils::{connect_grpc_client, parse_i64, to_option_i64},
};

#[instrument]
pub(crate) async fn create_country(country: NewCountry) -> Result<Vec<Country>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let response = client
        .create_country(CreateCountryRequest {
            country_name: country.country_name,
        })
        .await?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(convert::country_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn search_country(search: SearchCountry) -> Result<Vec<Country>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let response = client
        .search_country(SearchCountryRequest {
            country_name: search.country_name,
            country_id: to_option_i64(search.country_id),
        })
        .await?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(convert::country_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn delete_country(country_id: String) -> Result<Vec<Country>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let country_id = parse_i64(&country_id, "country id")?;

    let response = client
        .delete_country(DeleteCountryRequest { country_id })
        .await?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(convert::country_response_to_gql)
        .collect())
}
