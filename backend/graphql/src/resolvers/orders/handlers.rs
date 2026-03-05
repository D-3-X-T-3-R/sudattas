use proto::proto::core::{
    AdminMarkOrderDeliveredRequest, AdminMarkOrderShippedRequest, CreateOrderRequest,
    DeleteOrderRequest, PlaceOrderRequest, SearchOrderRequest, UpdateOrderRequest,
};
use tracing::instrument;

use super::schema::{
    AdminMarkOrderDeliveredInput, AdminMarkOrderShippedInput, CreateOrderInput, NewOrder, Order,
    OrderMutation, SearchOrder,
};
use crate::resolvers::{
    convert,
    error::GqlError,
    grpc_client,
    utils::{connect_grpc_client, parse_i64, to_i64, to_option_i64},
};

#[instrument(skip(user_id))]
pub(crate) async fn place_order(
    order: NewOrder,
    user_id: String,
    request_id: Option<&str>,
    idempotency_key: Option<&str>,
) -> Result<Vec<Order>, GqlError> {
    let mut client = grpc_client::connect_grpc_client_with_metadata(request_id).await?;

    let mut req = proto::tonic::Request::new(PlaceOrderRequest {
        user_id: to_i64(Some(user_id)),
        shipping_address_id: to_i64(order.shipping_address_id),
        coupon_code: order.coupon_code,
    });

    if let Some(key) = idempotency_key {
        if let Ok(value) = key.parse() {
            req.metadata_mut().insert("idempotency-key", value);
        }
    }

    let response = client.place_order(req).await?;

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
            user_id: to_option_i64(search.user_id),
            order_id: to_option_i64(search.order_id),
            order_date_start: to_option_i64(search.order_date_start),
            order_date_end: to_option_i64(search.order_date_end),
            status_id: to_option_i64(search.status_id),
            limit: crate::graphql_limits::cap_page_size(to_option_i64(search.limit)),
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
            total_amount_paise: parse_i64(&order.total_amount_paise, "total_amount_paise")?,
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
pub(crate) async fn create_order_admin(input: CreateOrderInput) -> Result<Vec<Order>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .create_order(CreateOrderRequest {
            user_id: parse_i64(&input.user_id, "user_id")?,
            shipping_address_id: parse_i64(&input.shipping_address_id, "shipping_address_id")?,
            status_id: parse_i64(&input.status_id, "status_id")?,
            total_amount_paise: parse_i64(&input.total_amount_paise, "total_amount_paise")?,
            subtotal_minor: to_option_i64(input.subtotal_minor),
            shipping_minor: to_option_i64(input.shipping_minor),
            tax_total_minor: to_option_i64(input.tax_total_minor),
            discount_total_minor: to_option_i64(input.discount_total_minor),
            grand_total_minor: to_option_i64(input.grand_total_minor),
            applied_coupon_id: to_option_i64(input.applied_coupon_id),
            applied_coupon_code: input.applied_coupon_code,
            applied_discount_paise: input
                .applied_discount_paise
                .as_deref()
                .map(|s| s.parse::<i32>())
                .transpose()
                .map_err(|_| {
                    GqlError::new(
                        "Failed to parse applied_discount_paise",
                        crate::resolvers::error::Code::InvalidArgument,
                    )
                })?,
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
pub(crate) async fn admin_mark_order_shipped(
    input: AdminMarkOrderShippedInput,
) -> Result<bool, GqlError> {
    let mut client = connect_grpc_client().await?;
    let _ = client
        .admin_mark_order_shipped(AdminMarkOrderShippedRequest {
            order_id: parse_i64(&input.order_id, "order_id")?,
            awb_code: input.awb_code,
            carrier: input.carrier,
        })
        .await?;
    Ok(true)
}

#[instrument]
pub(crate) async fn admin_mark_order_delivered(
    input: AdminMarkOrderDeliveredInput,
) -> Result<bool, GqlError> {
    let mut client = connect_grpc_client().await?;
    let _ = client
        .admin_mark_order_delivered(AdminMarkOrderDeliveredRequest {
            order_id: parse_i64(&input.order_id, "order_id")?,
        })
        .await?;
    Ok(true)
}
