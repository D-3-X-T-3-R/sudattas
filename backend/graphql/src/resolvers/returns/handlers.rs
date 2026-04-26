use proto::proto::core::{
    AdminMarkReturnReceivedRequest, AdminUpdateReturnStatusRequest, RequestReturnRequest,
    ReturnRequestItemInput as ProtoReturnRequestItemInput, ReturnRequestItemResponse,
    ReturnRequestResponse, SearchReturnRequestsRequest,
};
use tracing::instrument;

use super::schema::{
    AdminMarkReturnReceivedInput, AdminUpdateReturnStatusInput, RequestReturnInput, ReturnRequest,
    ReturnRequestItem, SearchReturnRequestsInput,
};
use crate::resolvers::{
    error::GqlError,
    utils::{connect_grpc_client, parse_i64},
};

fn return_item_response_to_gql(row: ReturnRequestItemResponse) -> ReturnRequestItem {
    ReturnRequestItem {
        return_id: row.return_id.to_string(),
        order_detail_id: row.order_detail_id.to_string(),
        quantity: row.quantity.to_string(),
        refund_amount_minor: row.refund_amount_minor.to_string(),
        status: row.status,
    }
}

fn return_response_to_gql(row: ReturnRequestResponse) -> ReturnRequest {
    ReturnRequest {
        return_id: row.return_id.to_string(),
        order_id: row.order_id.to_string(),
        user_id: row.user_id.to_string(),
        status: row.status,
        reason: row.reason,
        created_at: row.created_at,
        received_at: row.received_at,
        refund_attempt_id: row.refund_attempt_id.map(|v| v.to_string()),
        items: row
            .items
            .into_iter()
            .map(return_item_response_to_gql)
            .collect(),
    }
}

#[instrument(skip(user_id))]
pub(crate) async fn request_return(
    input: RequestReturnInput,
    user_id: String,
) -> Result<Vec<ReturnRequest>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .request_return(RequestReturnRequest {
            order_id: parse_i64(&input.order_id, "order_id")?,
            user_id: parse_i64(&user_id, "user_id")?,
            reason: input.reason,
            items: input
                .items
                .into_iter()
                .map(|item| {
                    Ok(ProtoReturnRequestItemInput {
                        order_detail_id: parse_i64(&item.order_detail_id, "order_detail_id")?,
                        quantity: item
                            .quantity
                            .as_deref()
                            .map(|qty| parse_i64(qty, "quantity"))
                            .transpose()?,
                    })
                })
                .collect::<Result<Vec<_>, GqlError>>()?,
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(return_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn search_return_requests(
    input: SearchReturnRequestsInput,
) -> Result<Vec<ReturnRequest>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .search_return_requests(SearchReturnRequestsRequest {
            return_id: input
                .return_id
                .as_deref()
                .map(|v| parse_i64(v, "return_id"))
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
        .map(return_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn admin_mark_return_received(
    input: AdminMarkReturnReceivedInput,
) -> Result<Vec<ReturnRequest>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .admin_mark_return_received(AdminMarkReturnReceivedRequest {
            return_id: parse_i64(&input.return_id, "return_id")?,
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(return_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn admin_update_return_status(
    input: AdminUpdateReturnStatusInput,
) -> Result<Vec<ReturnRequest>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .admin_update_return_status(AdminUpdateReturnStatusRequest {
            return_id: parse_i64(&input.return_id, "return_id")?,
            status: input.status,
            note: input.note,
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(return_response_to_gql)
        .collect())
}
