use proto::proto::core::{
    CreateShippingZoneRequest, DeleteShippingZoneRequest, SearchShippingZoneRequest,
    UpdateShippingZoneRequest,
};

use tracing::instrument;

use super::schema::{NewShippingZone, SearchShippingZone, ShippingZone, ShippingZoneMutation};
use crate::resolvers::{
    error::{Code, GqlError},
    utils::{connect_grpc_client, to_i32, to_i64, to_option_i32, to_option_i64},
};

#[instrument]
pub(crate) async fn create_shipping_zone(
    shipping_zone: NewShippingZone,
) -> Result<Vec<ShippingZone>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let response = client
        .create_shipping_zone(CreateShippingZoneRequest {
            zip_code: to_i32(shipping_zone.zip_code),
            description: shipping_zone.description,
        })
        .await
        .map_err(|e| GqlError::new(&format!("gRPC request failed: {}", e), Code::Internal))?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(|shipping_zone| ShippingZone {
            zone_id: shipping_zone.zone_id.to_string(),
            zip_code: shipping_zone.zip_code.to_string(),
            description: shipping_zone.description,
        })
        .collect())
}

#[instrument]
pub(crate) async fn search_shipping_zone(
    search: SearchShippingZone,
) -> Result<Vec<ShippingZone>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let response = client
        .search_shipping_zone(SearchShippingZoneRequest {
            zone_id: to_option_i64(search.zone_id),
            zip_code: to_option_i32(search.zip_code),
        })
        .await
        .map_err(|e| GqlError::new(&format!("gRPC request failed: {}", e), Code::Internal))?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(|shipping_zone| ShippingZone {
            zone_id: shipping_zone.zone_id.to_string(),
            zip_code: shipping_zone.zip_code.to_string(),
            description: shipping_zone.description,
        })
        .collect())
}

#[instrument]
pub(crate) async fn delete_shipping_zone(zone_id: String) -> Result<Vec<ShippingZone>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let response = client
        .delete_shipping_zone(DeleteShippingZoneRequest {
            zone_id: to_i64(zone_id),
        })
        .await
        .map_err(|e| GqlError::new(&format!("gRPC request failed: {}", e), Code::Internal))?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(|shipping_zone| ShippingZone {
            zone_id: shipping_zone.zone_id.to_string(),
            zip_code: shipping_zone.zip_code.to_string(),
            description: shipping_zone.description,
        })
        .collect())
}

#[instrument]
pub(crate) async fn update_shipping_zone(
    shipping_zone: ShippingZoneMutation,
) -> Result<Vec<ShippingZone>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let response = client
        .update_shipping_zone(UpdateShippingZoneRequest {
            zone_id: to_i64(shipping_zone.zone_id),
            zip_code: to_i32(shipping_zone.zip_code),
            description: shipping_zone.description,
        })
        .await
        .map_err(|e| GqlError::new(&format!("gRPC request failed: {}", e), Code::Internal))?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(|shipping_zone| ShippingZone {
            zone_id: shipping_zone.zone_id.to_string(),
            zip_code: shipping_zone.zip_code.to_string(),
            description: shipping_zone.description,
        })
        .collect())
}
