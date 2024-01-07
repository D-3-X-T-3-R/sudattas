use proto::proto::core::{
    CreateOrderDetailRequest, CreateOrderDetailsRequest, SearchOrderDetailRequest,
    UpdateOrderDetailRequest,
};

use tracing::instrument;

use super::schema::{NewOrderDetails, OrderDetails, OrderDetailsMutation, SearchOrderDetails};
use crate::resolvers::{
    error::{Code, GqlError},
    utils::{connect_grpc_client, to_f64, to_i64, to_option_f64, to_option_i64},
};

#[instrument]
pub(crate) async fn create_order_detail(
    order_detail: NewOrderDetails,
) -> Result<Vec<OrderDetails>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let order_details = order_detail
        .order_details
        .into_iter()
        .map(|details| CreateOrderDetailRequest {
            order_id: to_i64(details.order_id),
            product_id: to_i64(details.product_id),
            quantity: to_i64(details.quantity),
            price: to_f64(details.price),
        })
        .collect();
    let response = client
        .create_order_details(CreateOrderDetailsRequest { order_details })
        .await
        .map_err(|e| GqlError::new(&format!("gRPC request failed: {}", e), Code::Internal))?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(|order_detail| OrderDetails {
            order_detail_id: order_detail.order_detail_id.to_string(),
            order_id: order_detail.order_id.to_string(),
            product_id: order_detail.product_id.to_string(),
            quantity: order_detail.quantity.to_string(),
            price: order_detail.price.to_string(),
        })
        .collect())
}

#[instrument]
pub(crate) async fn search_order_detail(
    search: SearchOrderDetails,
) -> Result<Vec<OrderDetails>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let response = client
        .search_order_detail(SearchOrderDetailRequest {
            order_detail_id: to_option_i64(search.order_detail_id),
            order_id: to_option_i64(search.order_id),
            product_id: to_option_i64(search.product_id),
            quantity: to_option_i64(search.quantity),
            price_start: to_option_f64(search.price_start),
            price_end: to_option_f64(search.price_end),
        })
        .await
        .map_err(|e| GqlError::new(&format!("gRPC request failed: {}", e), Code::Internal))?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(|order_detail| OrderDetails {
            order_detail_id: order_detail.order_detail_id.to_string(),
            order_id: order_detail.order_id.to_string(),
            product_id: order_detail.product_id.to_string(),
            quantity: order_detail.quantity.to_string(),
            price: order_detail.price.to_string(),
        })
        .collect())
}

#[instrument]
pub(crate) async fn update_order_detail(
    order_detail: OrderDetailsMutation,
) -> Result<Vec<OrderDetails>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let response = client
        .update_order_detail(UpdateOrderDetailRequest {
            order_detail_id: to_i64(order_detail.order_detail_id),
            order_id: to_i64(order_detail.order_id),
            product_id: to_i64(order_detail.product_id),
            quantity: to_i64(order_detail.quantity),
            price: to_f64(order_detail.price),
        })
        .await
        .map_err(|e| GqlError::new(&format!("gRPC request failed: {}", e), Code::Internal))?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(|order_detail| OrderDetails {
            order_detail_id: order_detail.order_detail_id.to_string(),
            order_id: order_detail.order_id.to_string(),
            product_id: order_detail.product_id.to_string(),
            quantity: order_detail.quantity.to_string(),
            price: order_detail.price.to_string(),
        })
        .collect())
}
