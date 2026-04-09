use proto::proto::core::{
    CreateShippingAddressRequest, DeleteShippingAddressRequest, GetShippingAddressRequest,
    ShippingAddressResponse, UpdateShippingAddressRequest,
};
use tracing::instrument;

use super::schema::{NewShippingAddress, ShippingAddress, ShippingAddressMutation};
use crate::resolvers::{
    error::Code,
    error::GqlError,
    utils::{connect_grpc_client, parse_i64},
};
use crate::validation::{validate_address_road, validate_phone, validate_postal_code};

fn parse_optional_user_id(input: Option<&str>) -> Result<Option<i64>, GqlError> {
    match input {
        Some(raw) if !raw.trim().is_empty() => {
            Ok(Some(parse_i64(raw.trim(), "shipping address user_id")?))
        }
        _ => Ok(None),
    }
}

fn address_response_to_gql(a: ShippingAddressResponse) -> ShippingAddress {
    ShippingAddress {
        shipping_address_id: a.shipping_address_id.to_string(),
        user_id: a.user_id.map(|u| u.to_string()),
        is_default: a.is_default,
        country: a.country,
        state_region: a.state_region,
        city: a.city,
        postal_code: a.postal_code,
        road: a.road,
        apartment_no_or_name: a.apartment_no_or_name,
        recipient_name: a.recipient_name,
        phone_number: a.phone_number,
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
    let road = input
        .road
        .as_deref()
        .ok_or_else(|| GqlError::new("Address road is required", Code::InvalidArgument))?;
    validate_address_road(road)?;
    validate_postal_code(&input.postal_code)?;
    validate_phone(input.phone_number.as_deref())?;

    let mut client = connect_grpc_client().await?;
    let user_id = parse_optional_user_id(input.user_id.as_deref())?;
    let response = client
        .create_shipping_address(CreateShippingAddressRequest {
            user_id,
            is_default: input.is_default.unwrap_or(false),
            country: input.country,
            state_region: input.state_region,
            city: input.city,
            postal_code: input.postal_code,
            road: input.road,
            apartment_no_or_name: input.apartment_no_or_name,
            recipient_name: input.recipient_name,
            phone_number: input.phone_number,
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
    let road = input
        .road
        .as_deref()
        .ok_or_else(|| GqlError::new("Address road is required", Code::InvalidArgument))?;
    validate_address_road(road)?;
    validate_postal_code(&input.postal_code)?;
    validate_phone(input.phone_number.as_deref())?;

    let mut client = connect_grpc_client().await?;
    let user_id = parse_optional_user_id(input.user_id.as_deref())?;
    let response = client
        .update_shipping_address(UpdateShippingAddressRequest {
            shipping_address_id: parse_i64(&input.shipping_address_id, "shipping address id")?,
            user_id,
            is_default: input.is_default.unwrap_or(false),
            country: input.country,
            state_region: input.state_region,
            city: input.city,
            postal_code: input.postal_code,
            road: input.road,
            apartment_no_or_name: input.apartment_no_or_name,
            recipient_name: input.recipient_name,
            phone_number: input.phone_number,
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
