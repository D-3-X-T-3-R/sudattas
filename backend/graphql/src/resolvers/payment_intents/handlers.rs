use proto::proto::core::{
    CapturePaymentRequest, CreatePaymentIntentRequest, GetPaymentIntentRequest,
    PaymentIntentResponse,
};
use tracing::instrument;

use super::schema::{CapturePayment, GetPaymentIntent, NewPaymentIntent, PaymentIntent};
use crate::resolvers::{
    error::GqlError,
    utils::{connect_grpc_client, parse_i64},
};

fn payment_intent_response_to_gql(p: PaymentIntentResponse) -> PaymentIntent {
    PaymentIntent {
        intent_id: p.intent_id.to_string(),
        razorpay_order_id: p.razorpay_order_id,
        order_id: p.order_id.map(|v| v.to_string()),
        user_id: p.user_id.map(|v| v.to_string()),
        amount_paise: p.amount_paise.to_string(),
        currency: p.currency,
        status: p.status,
        razorpay_payment_id: p.razorpay_payment_id,
        created_at: p.created_at,
        expires_at: p.expires_at,
    }
}

#[instrument]
pub(crate) async fn create_payment_intent(
    input: NewPaymentIntent,
) -> Result<Vec<PaymentIntent>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .create_payment_intent(CreatePaymentIntentRequest {
            order_id: parse_i64(&input.order_id, "order id")?,
            user_id: parse_i64(&input.user_id, "user id")?,
            amount_paise: parse_i64(&input.amount_paise, "amount_paise")?,
            currency: input.currency,
            razorpay_order_id: input.razorpay_order_id,
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(payment_intent_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn capture_payment(input: CapturePayment) -> Result<Vec<PaymentIntent>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .capture_payment(CapturePaymentRequest {
            intent_id: parse_i64(&input.intent_id, "intent id")?,
            razorpay_payment_id: input.razorpay_payment_id,
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(payment_intent_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn get_payment_intent(
    input: GetPaymentIntent,
) -> Result<Vec<PaymentIntent>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .get_payment_intent(GetPaymentIntentRequest {
            intent_id: input.intent_id.as_deref().and_then(|s| s.parse().ok()),
            order_id: input.order_id.as_deref().and_then(|s| s.parse().ok()),
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(payment_intent_response_to_gql)
        .collect())
}
