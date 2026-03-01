use proto::proto::core::{
    CreateOrderDetailRequest, CreateOrderDetailsRequest, SearchOrderDetailRequest,
    UpdateOrderDetailRequest,
};

use tracing::instrument;

use super::schema::{NewOrderDetails, OrderDetails, OrderDetailsMutation, SearchOrderDetails};
use crate::resolvers::{
    convert,
    error::GqlError,
    utils::{connect_grpc_client, parse_i64, to_i64, to_option_i64},
};

#[instrument]
pub(crate) async fn create_order_detail(
    order_detail: NewOrderDetails,
) -> Result<Vec<OrderDetails>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let order_details: Vec<CreateOrderDetailRequest> = order_detail
        .order_details
        .into_iter()
        .map(|details| {
            let price_paise = parse_i64(&details.price_paise, "price_paise")?;
            Ok(CreateOrderDetailRequest {
                order_id: to_i64(details.order_id),
                product_id: to_i64(details.product_id),
                quantity: to_i64(details.quantity),
                price_paise,
                unit_price_minor: None,
                discount_minor: None,
                tax_minor: None,
                sku: None,
                title: None,
            })
        })
        .collect::<Result<Vec<_>, GqlError>>()?;
    let response = client
        .create_order_details(CreateOrderDetailsRequest { order_details })
        .await?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(convert::order_detail_response_to_gql)
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
            price_start_paise: search
                .price_start_paise
                .as_ref()
                .and_then(|s| s.parse().ok()),
            price_end_paise: search.price_end_paise.as_ref().and_then(|s| s.parse().ok()),
        })
        .await?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(convert::order_detail_response_to_gql)
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
            price_paise: parse_i64(&order_detail.price_paise, "price_paise")?,
        })
        .await?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(convert::order_detail_response_to_gql)
        .collect())
}
