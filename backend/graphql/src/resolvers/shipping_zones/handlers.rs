use proto::proto::core::{
    CreateShippingZoneRequest, DeleteShippingZoneRequest, SearchShippingZoneRequest,
    ShippingZoneResponse, UpdateShippingZoneRequest,
};
use tracing::instrument;

use super::schema::{NewShippingZone, SearchShippingZone, ShippingZone, ShippingZoneMutation};
use crate::resolvers::{
    error::GqlError,
    utils::{connect_grpc_client, parse_i64},
};

fn zone_response_to_gql(z: ShippingZoneResponse) -> ShippingZone {
    ShippingZone {
        zone_id: z.zone_id.to_string(),
        zone_name: z.zone_name,
        description: z.description,
    }
}

#[instrument]
pub(crate) async fn search_shipping_zone(
    input: SearchShippingZone,
) -> Result<Vec<ShippingZone>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .search_shipping_zone(SearchShippingZoneRequest {
            zone_id: input
                .zone_id
                .as_deref()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(zone_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn create_shipping_zone(
    input: NewShippingZone,
) -> Result<Vec<ShippingZone>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .create_shipping_zone(CreateShippingZoneRequest {
            zone_name: input.zone_name,
            description: input.description,
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(zone_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn update_shipping_zone(
    input: ShippingZoneMutation,
) -> Result<Vec<ShippingZone>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .update_shipping_zone(UpdateShippingZoneRequest {
            zone_id: parse_i64(&input.zone_id, "zone id")?,
            zone_name: input.zone_name,
            description: input.description,
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(zone_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn delete_shipping_zone(zone_id: String) -> Result<Vec<ShippingZone>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .delete_shipping_zone(DeleteShippingZoneRequest {
            zone_id: parse_i64(&zone_id, "zone id")?,
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(zone_response_to_gql)
        .collect())
}
