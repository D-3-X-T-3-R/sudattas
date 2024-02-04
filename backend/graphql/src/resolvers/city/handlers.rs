use proto::proto::core::{CreateCityRequest, DeleteCityRequest, SearchCityRequest};

use tracing::instrument;

use super::schema::{City, CityMutation, NewCity, SearchCity};
use crate::resolvers::{
    error::{Code, GqlError},
    utils::{connect_grpc_client, to_f64, to_i64, to_option_f64, to_option_i64},
};

#[instrument]
pub(crate) async fn create_city(city: NewCity) -> Result<Vec<City>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let response = client
        .create_city(CreateCityRequest {
            city_name: city.city_name,
        })
        .await
        .map_err(|e| GqlError::new(&format!("gRPC request failed: {}", e), Code::Internal))?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(|city| City {
            city_name: city.city_name,
            city_id: city.city_id.to_string(),
        })
        .collect())
}

#[instrument]
pub(crate) async fn search_city(search: SearchCity) -> Result<Vec<City>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let response = client
        .search_city(SearchCityRequest {
            city_name: search.city_name,
            city_id: to_option_i64(search.city_id),
        })
        .await
        .map_err(|e| GqlError::new(&format!("gRPC request failed: {}", e), Code::Internal))?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(|city| City {
            city_name: city.city_name,
            city_id: city.city_id.to_string(),
        })
        .collect())
}

#[instrument]
pub(crate) async fn delete_city(city_id: String) -> Result<Vec<City>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let city_id = city_id
        .parse::<i64>()
        .map_err(|_| GqlError::new("Failed to parse city id", Code::InvalidArgument))?;

    let response = client
        .delete_city(DeleteCityRequest { city_id })
        .await
        .map_err(|e| GqlError::new(&format!("gRPC request failed: {}", e), Code::Internal))?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(|city| City {
            city_name: city.city_name,
            city_id: city.city_id.to_string(),
        })
        .collect())
}
