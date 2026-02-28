use proto::proto::core::{
    CreateShippingAddressRequest, DeleteShippingAddressRequest, GetShippingAddressRequest,
    ShippingAddressResponse, UpdateShippingAddressRequest,
};
use tracing::instrument;

use super::schema::{NewShippingAddress, ShippingAddress, ShippingAddressMutation};
use crate::resolvers::{
    error::GqlError,
    utils::{connect_grpc_client, parse_i64},
};
use crate::validation;

fn address_response_to_gql(a: ShippingAddressResponse) -> ShippingAddress {
    ShippingAddress {
        shipping_address_id: a.shipping_address_id.to_string(),
        country_id: a.country_id.to_string(),
        state_id: a.state_id.to_string(),
        city_id: a.city_id.to_string(),
        road: a.road,
        apartment_no_or_name: a.apartment_no_or_name,
    }
}

#[instrument]
pub(crate) async fn get_shipping_addresses() -> Result<Vec<ShippingAddress>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .get_shipping_address(GetShippingAddressRequest {})
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(address_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn create_shipping_address(
    input: NewShippingAddress,
) -> Result<Vec<ShippingAddress>, GqlError> {
    validation::validate_address_road(&input.road)?;
    let mut client = connect_grpc_client().await?;
    let response = client
        .create_shipping_address(CreateShippingAddressRequest {
            country_id: parse_i64(&input.country_id, "country id")?,
            state_id: parse_i64(&input.state_id, "state id")?,
            city_id: parse_i64(&input.city_id, "city id")?,
            road: input.road,
            apartment_no_or_name: input.apartment_no_or_name,
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(address_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn update_shipping_address(
    input: ShippingAddressMutation,
) -> Result<Vec<ShippingAddress>, GqlError> {
    validation::validate_address_road(&input.road)?;
    let mut client = connect_grpc_client().await?;
    let response = client
        .update_shipping_address(UpdateShippingAddressRequest {
            shipping_address_id: parse_i64(&input.shipping_address_id, "shipping address id")?,
            country_id: parse_i64(&input.country_id, "country id")?,
            state_id: parse_i64(&input.state_id, "state id")?,
            city_id: parse_i64(&input.city_id, "city id")?,
            road: input.road,
            apartment_no_or_name: input.apartment_no_or_name,
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(address_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn delete_shipping_address(
    shipping_address_id: String,
) -> Result<Vec<ShippingAddress>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .delete_shipping_address(DeleteShippingAddressRequest {
            shipping_address_id: parse_i64(&shipping_address_id, "shipping address id")?,
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(address_response_to_gql)
        .collect())
}
