use proto::proto::core::{
    DeleteOrderRequest, PlaceOrderRequest, SearchOrderRequest, UpdateOrderRequest,
};

use tracing::instrument;

use super::schema::{NewOrder, Order, OrderMutation, SearchOrder};
use crate::resolvers::{
    convert,
    error::GqlError,
    grpc_client,
    utils::{connect_grpc_client, to_f64, to_i64, to_option_i64},
};

#[instrument(skip(user_id))]
pub(crate) async fn place_order(
    order: NewOrder,
    user_id: String,
    request_id: Option<&str>,
) -> Result<Vec<Order>, GqlError> {
    let mut client = grpc_client::connect_grpc_client_with_metadata(request_id).await?;

    let response = client
        .place_order(PlaceOrderRequest {
            user_id: to_i64(Some(user_id)),
            shipping_address_id: to_i64(order.shipping_address_id),
            coupon_code: order.coupon_code,
        })
        .await?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(convert::order_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn search_order(search: SearchOrder) -> Result<Vec<Order>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let response = client
        .search_order(SearchOrderRequest {
            user_id: to_i64(search.user_id),
            order_id: to_option_i64(search.order_id),
            order_date_start: to_option_i64(search.order_date_start),
            order_date_end: to_option_i64(search.order_date_end),
            status_id: to_option_i64(search.status_id),
            limit: to_option_i64(search.limit),
            offset: to_option_i64(search.offset),
        })
        .await?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(convert::order_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn delete_order(order_id: String) -> Result<Vec<Order>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let response = client
        .delete_order(DeleteOrderRequest {
            order_id: to_i64(order_id),
        })
        .await?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(convert::order_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn update_order(order: OrderMutation) -> Result<Vec<Order>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let response = client
        .update_order(UpdateOrderRequest {
            order_id: to_i64(order.order_id),
            user_id: to_i64(order.user_id),
            status_id: to_i64(order.status_id),
            shipping_address_id: to_i64(order.shipping_address_id),
            total_amount: to_f64(order.total_amount),
        })
        .await?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(convert::order_response_to_gql)
        .collect())
}
