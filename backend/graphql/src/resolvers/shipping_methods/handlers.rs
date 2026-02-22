use proto::proto::core::{
    CreateShippingMethodRequest, DeleteShippingMethodRequest, SearchShippingMethodRequest,
    ShippingMethodResponse, UpdateShippingMethodRequest,
};
use tracing::instrument;

use super::schema::{NewShippingMethod, SearchShippingMethod, ShippingMethod, ShippingMethodMutation};
use crate::resolvers::{
    error::GqlError,
    utils::{connect_grpc_client, parse_i64},
};

fn method_response_to_gql(m: ShippingMethodResponse) -> ShippingMethod {
    ShippingMethod {
        method_id: m.method_id.to_string(),
        method_name: m.method_name,
        cost: m.cost,
        estimated_delivery_time: m.estimated_delivery_time,
    }
}

#[instrument]
pub(crate) async fn search_shipping_method(
    input: SearchShippingMethod,
) -> Result<Vec<ShippingMethod>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .search_shipping_method(SearchShippingMethodRequest {
            method_id: input
                .method_id
                .as_deref()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(method_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn create_shipping_method(
    input: NewShippingMethod,
) -> Result<Vec<ShippingMethod>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .create_shipping_method(CreateShippingMethodRequest {
            method_name: input.method_name,
            cost: input.cost,
            estimated_delivery_time: input.estimated_delivery_time,
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(method_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn update_shipping_method(
    input: ShippingMethodMutation,
) -> Result<Vec<ShippingMethod>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .update_shipping_method(UpdateShippingMethodRequest {
            method_id: parse_i64(&input.method_id, "method id")?,
            method_name: input.method_name,
            cost: input.cost,
            estimated_delivery_time: input.estimated_delivery_time,
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(method_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn delete_shipping_method(
    method_id: String,
) -> Result<Vec<ShippingMethod>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .delete_shipping_method(DeleteShippingMethodRequest {
            method_id: parse_i64(&method_id, "method id")?,
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(method_response_to_gql)
        .collect())
}
