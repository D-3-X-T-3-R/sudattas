use proto::proto::core::{CreateCountryRequest, DeleteCountryRequest, SearchCountryRequest};

use tracing::instrument;

use super::schema::{Country, NewCountry, SearchCountry};
use crate::resolvers::{
    error::{Code, GqlError},
    utils::{connect_grpc_client, to_option_i64},
};

#[instrument]
pub(crate) async fn create_country(country: NewCountry) -> Result<Vec<Country>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let response = client
        .create_country(CreateCountryRequest {
            country_name: country.country_name,
        })
        .await
        .map_err(|e| GqlError::new(&format!("gRPC request failed: {}", e), Code::Internal))?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(|country| Country {
            country_name: country.country_name,
            country_id: country.country_id.to_string(),
        })
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
        .await
        .map_err(|e| GqlError::new(&format!("gRPC request failed: {}", e), Code::Internal))?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(|country| Country {
            country_name: country.country_name,
            country_id: country.country_id.to_string(),
        })
        .collect())
}

#[instrument]
pub(crate) async fn delete_country(country_id: String) -> Result<Vec<Country>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let country_id = country_id
        .parse::<i64>()
        .map_err(|_| GqlError::new("Failed to parse country id", Code::InvalidArgument))?;

    let response = client
        .delete_country(DeleteCountryRequest { country_id })
        .await
        .map_err(|e| GqlError::new(&format!("gRPC request failed: {}", e), Code::Internal))?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(|country| Country {
            country_name: country.country_name,
            country_id: country.country_id.to_string(),
        })
        .collect())
}
