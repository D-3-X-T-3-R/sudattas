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

fn address_response_to_gql(a: ShippingAddressResponse) -> ShippingAddress {
    ShippingAddress {
        shipping_address_id: a.shipping_address_id.to_string(),
        user_id: a.user_id.map(|u| u.to_string()),
        country: a.country,
        state_region: a.state_region,
        city: a.city,
        postal_code: a.postal_code,
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
    let mut client = connect_grpc_client().await?;
    let response = client
        .create_shipping_address(CreateShippingAddressRequest {
            user_id: input.user_id.as_deref().and_then(|s| s.parse().ok()),
            country: input.country,
            state_region: input.state_region,
            city: input.city,
            postal_code: input.postal_code,
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
    let mut client = connect_grpc_client().await?;
    let response = client
        .update_shipping_address(UpdateShippingAddressRequest {
            shipping_address_id: parse_i64(&input.shipping_address_id, "shipping address id")?,
            user_id: input.user_id.as_deref().and_then(|s| s.parse().ok()),
            country: input.country,
            state_region: input.state_region,
            city: input.city,
            postal_code: input.postal_code,
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
