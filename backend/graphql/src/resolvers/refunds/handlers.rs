use proto::proto::core::{
    CreateRefundRequest, RefundResponse, RefundsResponse, ResolveNeedsReviewRequest,
};
use tracing::instrument;

use super::schema::{NewRefund, Refund, ResolveNeedsReviewInput};
use crate::resolvers::{
    error::GqlError,
    utils::{connect_grpc_client, parse_i64},
};

fn refund_response_to_gql(r: RefundResponse) -> Refund {
    Refund {
        refund_id: r.refund_id.to_string(),
        order_id: r.order_id.to_string(),
        gateway_refund_id: r.gateway_refund_id,
        amount_paise: r.amount_paise.to_string(),
        currency: r.currency,
        status: r.status,
        created_at: r.created_at,
    }
}

fn refunds_response_to_vec(resp: RefundsResponse) -> Vec<Refund> {
    resp.items.into_iter().map(refund_response_to_gql).collect()
}

#[instrument]
pub(crate) async fn create_refund(input: NewRefund) -> Result<Vec<Refund>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .create_refund(CreateRefundRequest {
            order_id: parse_i64(&input.order_id, "order_id")?,
            gateway_refund_id: input.gateway_refund_id,
            amount_paise: parse_i64(&input.amount_paise, "amount_paise")?,
            currency: input.currency,
            line_items_refunded_json: input.line_items_refunded_json,
        })
        .await?;
    Ok(refunds_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn resolve_needs_review(input: ResolveNeedsReviewInput) -> Result<bool, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .resolve_needs_review(ResolveNeedsReviewRequest {
            order_id: parse_i64(&input.order_id, "order_id")?,
            resolution: input.resolution,
            actor_id: input.actor_id,
        })
        .await?;
    Ok(resp.into_inner().success)
}
