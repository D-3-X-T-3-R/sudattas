use proto::proto::core::{
    CreateRefundRequest, GetRefundsRequest, RefundAttemptResponse, RefundResponse, RefundsResponse,
    ResolveNeedsReviewRequest, ResolveRefundAttemptNeedsReviewRequest, SearchRefundAttemptsRequest,
};
use tracing::instrument;

use super::schema::{
    GetRefund, NewRefund, Refund, RefundAttempt, ResolveNeedsReviewInput,
    ResolveRefundAttemptNeedsReviewInput, SearchRefundAttemptsInput,
};
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
        line_items_refunded_json: r.line_items_refunded_json,
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

#[instrument]
pub(crate) async fn resolve_refund_attempt_needs_review(
    input: ResolveRefundAttemptNeedsReviewInput,
) -> Result<bool, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .resolve_refund_attempt_needs_review(ResolveRefundAttemptNeedsReviewRequest {
            attempt_id: parse_i64(&input.attempt_id, "attempt_id")?,
            resolution: input.resolution,
            actor_id: input.actor_id,
        })
        .await?;
    Ok(resp.into_inner().success)
}

#[instrument]
pub(crate) async fn get_refunds(input: GetRefund) -> Result<Vec<Refund>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .get_refunds(GetRefundsRequest {
            refund_id: input
                .refund_id
                .as_deref()
                .map(|s| parse_i64(s, "refund_id"))
                .transpose()?,
            order_id: input
                .order_id
                .as_deref()
                .map(|s| parse_i64(s, "order_id"))
                .transpose()?,
            gateway_refund_id: input
                .gateway_refund_id
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
        })
        .await?;
    Ok(refunds_response_to_vec(resp.into_inner()))
}

fn refund_attempt_response_to_gql(r: RefundAttemptResponse) -> RefundAttempt {
    RefundAttempt {
        attempt_id: r.attempt_id.to_string(),
        order_id: r.order_id.to_string(),
        payment_intent_id: r.payment_intent_id.map(|v| v.to_string()),
        razorpay_payment_id: r.razorpay_payment_id,
        amount_requested_paise: r.amount_requested_paise.to_string(),
        amount_sent_to_gateway_paise: r.amount_sent_to_gateway_paise.to_string(),
        gateway_refund_id: r.gateway_refund_id,
        status: r.status,
        provider_error: r.provider_error,
        created_at: r.created_at,
        updated_at: r.updated_at,
        attempt_count: r.attempt_count,
    }
}

#[instrument]
pub(crate) async fn search_refund_attempts(
    input: SearchRefundAttemptsInput,
) -> Result<Vec<RefundAttempt>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .search_refund_attempts(SearchRefundAttemptsRequest {
            attempt_id: input
                .attempt_id
                .as_deref()
                .map(|s| parse_i64(s, "attempt_id"))
                .transpose()?,
            order_id: input
                .order_id
                .as_deref()
                .map(|s| parse_i64(s, "order_id"))
                .transpose()?,
            status: input.status,
        })
        .await?;
    Ok(resp
        .into_inner()
        .items
        .into_iter()
        .map(refund_attempt_response_to_gql)
        .collect())
}
