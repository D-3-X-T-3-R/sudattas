//! Admin lookup — unlike get_payment_intent (a single record by known id), this is a real
//! filtered/paginated browse so admin can actually find a payment intent without already
//! knowing its intent_id/order_id.

use crate::handlers::db_errors::map_db_error_to_status;
use crate::razorpay;
use core_db_entities::entity::payment_intents;
use core_db_entities::entity::sea_orm_active_enums::Status as PaymentIntentStatus;
use proto::proto::core::{PaymentIntentResponse, PaymentIntentsResponse, SearchPaymentIntentRequest};
use sea_orm::{
    ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder, QuerySelect, QueryTrait,
};
use tonic::{Request, Response, Status};

const DEFAULT_LIMIT: u64 = 50;
const MAX_LIMIT: u64 = 200;

fn parse_status(code: &str) -> Result<PaymentIntentStatus, Status> {
    match code {
        "pending" => Ok(PaymentIntentStatus::Pending),
        "processed" => Ok(PaymentIntentStatus::Processed),
        "failed" => Ok(PaymentIntentStatus::Failed),
        "needs_review" => Ok(PaymentIntentStatus::NeedsReview),
        "client_verified" => Ok(PaymentIntentStatus::ClientVerified),
        other => Err(Status::invalid_argument(format!(
            "Unknown payment intent status: {other}"
        ))),
    }
}

pub async fn search_payment_intent(
    txn: &DatabaseTransaction,
    request: Request<SearchPaymentIntentRequest>,
) -> Result<Response<PaymentIntentsResponse>, Status> {
    let req = request.into_inner();

    let status_filter = req.status.as_deref().map(parse_status).transpose()?;
    let limit = req
        .limit
        .map(|l| l as u64)
        .unwrap_or(DEFAULT_LIMIT)
        .min(MAX_LIMIT);

    let query = payment_intents::Entity::find()
        .apply_if(req.order_id, |q, v| {
            q.filter(payment_intents::Column::OrderId.eq(v))
        })
        .apply_if(req.user_id, |q, v| {
            q.filter(payment_intents::Column::UserId.eq(v))
        })
        .apply_if(req.razorpay_order_id, |q, v| {
            q.filter(payment_intents::Column::RazorpayOrderId.eq(v))
        })
        .apply_if(req.razorpay_payment_id, |q, v| {
            q.filter(payment_intents::Column::RazorpayPaymentId.eq(v))
        })
        .apply_if(status_filter, |q, v| {
            q.filter(payment_intents::Column::Status.eq(v))
        })
        .order_by_desc(payment_intents::Column::IntentId)
        .limit(limit)
        .apply_if(req.offset, |q, v| q.offset(v as u64));

    let results = query.all(txn).await.map_err(map_db_error_to_status)?;

    let key_id = razorpay::key_id_for_frontend();
    let items = results
        .into_iter()
        .map(|model| PaymentIntentResponse {
            intent_id: model.intent_id,
            razorpay_order_id: model.razorpay_order_id,
            order_id: model.order_id,
            user_id: model.user_id,
            amount_paise: model.amount_paise as i64,
            currency: model.currency,
            status: format!("{:?}", model.status).to_lowercase(),
            razorpay_payment_id: model.razorpay_payment_id,
            created_at: model.created_at.map(|t| t.to_string()).unwrap_or_default(),
            expires_at: model.expires_at.to_string(),
            razorpay_key_id: key_id.clone(),
            gateway_fee_paise: model.gateway_fee_paise.map(i64::from),
            gateway_tax_paise: model.gateway_tax_paise.map(i64::from),
        })
        .collect();

    Ok(Response::new(PaymentIntentsResponse { items }))
}
