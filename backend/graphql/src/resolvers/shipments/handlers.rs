use proto::proto::core::{
    CreateShipmentRequest, GetShipmentRequest, ShipmentResponse, UpdateShipmentRequest,
};
use tracing::instrument;

use super::schema::{GetShipment, NewShipment, Shipment, UpdateShipment};
use crate::resolvers::{
    error::GqlError,
    utils::{connect_grpc_client, parse_i64},
};

fn shipment_response_to_gql(s: ShipmentResponse) -> Shipment {
    Shipment {
        shipment_id: s.shipment_id.to_string(),
        order_id: s.order_id.to_string(),
        shiprocket_order_id: s.shiprocket_order_id,
        awb_code: s.awb_code,
        carrier: s.carrier,
        status: s.status,
        created_at: s.created_at,
        delivered_at: s.delivered_at,
    }
}

#[instrument]
pub(crate) async fn create_shipment(input: NewShipment) -> Result<Vec<Shipment>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .create_shipment(CreateShipmentRequest {
            order_id: parse_i64(&input.order_id, "order id")?,
            shiprocket_order_id: input.shiprocket_order_id,
            awb_code: input.awb_code,
            carrier: input.carrier,
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(shipment_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn update_shipment(input: UpdateShipment) -> Result<Vec<Shipment>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .update_shipment(UpdateShipmentRequest {
            shipment_id: parse_i64(&input.shipment_id, "shipment id")?,
            shiprocket_order_id: input.shiprocket_order_id,
            awb_code: input.awb_code,
            carrier: input.carrier,
            status: input.status,
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(shipment_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn get_shipment(input: GetShipment) -> Result<Vec<Shipment>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .get_shipment(GetShipmentRequest {
            shipment_id: input.shipment_id.as_deref().and_then(|s| s.parse().ok()),
            order_id: input.order_id.as_deref().and_then(|s| s.parse().ok()),
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(shipment_response_to_gql)
        .collect())
}
