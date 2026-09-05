use proto::proto::core::{
    AdminMarkExchangeReceivedRequest, AdminUpdateExchangeStatusRequest, ExchangeRequestResponse,
    RequestExchangeRequest, SearchExchangeRequestsRequest,
};
use tracing::instrument;

use super::schema::{
    AdminMarkExchangeReceivedInput, AdminUpdateExchangeStatusInput, ExchangeRequest,
    RequestExchangeInput, SearchExchangeRequestsInput,
};
use crate::resolvers::{
    error::GqlError,
    utils::{connect_grpc_client, parse_i64},
};

fn exchange_response_to_gql(row: ExchangeRequestResponse) -> ExchangeRequest {
    ExchangeRequest {
        exchange_id: row.exchange_id.to_string(),
        order_id: row.order_id.to_string(),
        user_id: row.user_id.to_string(),
        order_detail_id: row.order_detail_id.to_string(),
        desired_variant_id: row.desired_variant_id.to_string(),
        quantity: row.quantity.to_string(),
        status: row.status,
        reason: row.reason,
        created_at: row.created_at,
        received_at: row.received_at,
        replacement_order_id: row.replacement_order_id.map(|v| v.to_string()),
    }
}

#[instrument(skip(user_id))]
pub(crate) async fn request_exchange(
    input: RequestExchangeInput,
    user_id: String,
) -> Result<Vec<ExchangeRequest>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .request_exchange(RequestExchangeRequest {
            order_id: parse_i64(&input.order_id, "order_id")?,
            user_id: parse_i64(&user_id, "user_id")?,
            order_detail_id: parse_i64(&input.order_detail_id, "order_detail_id")?,
            desired_variant_id: parse_i64(&input.desired_variant_id, "desired_variant_id")?,
            quantity: input
                .quantity
                .as_deref()
                .map(|qty| parse_i64(qty, "quantity"))
                .transpose()?,
            reason: input.reason,
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(exchange_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn search_exchange_requests(
    input: SearchExchangeRequestsInput,
) -> Result<Vec<ExchangeRequest>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .search_exchange_requests(SearchExchangeRequestsRequest {
            exchange_id: input
                .exchange_id
                .as_deref()
                .map(|v| parse_i64(v, "exchange_id"))
                .transpose()?,
            order_id: input
                .order_id
                .as_deref()
                .map(|v| parse_i64(v, "order_id"))
                .transpose()?,
            user_id: input
                .user_id
                .as_deref()
                .map(|v| parse_i64(v, "user_id"))
                .transpose()?,
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(exchange_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn admin_mark_exchange_received(
    input: AdminMarkExchangeReceivedInput,
) -> Result<Vec<ExchangeRequest>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .admin_mark_exchange_received(AdminMarkExchangeReceivedRequest {
            exchange_id: parse_i64(&input.exchange_id, "exchange_id")?,
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(exchange_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn admin_update_exchange_status(
    input: AdminUpdateExchangeStatusInput,
) -> Result<Vec<ExchangeRequest>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .admin_update_exchange_status(AdminUpdateExchangeStatusRequest {
            exchange_id: parse_i64(&input.exchange_id, "exchange_id")?,
            status: input.status,
            note: input.note,
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(exchange_response_to_gql)
        .collect())
}
