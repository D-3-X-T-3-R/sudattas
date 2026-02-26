use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::payment_intents;
use core_db_entities::entity::sea_orm_active_enums::Status;
use proto::proto::core::{CapturePaymentRequest, PaymentIntentResponse, PaymentIntentsResponse};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseTransaction, EntityTrait, IntoActiveModel,
    QueryFilter,
};
use tonic::{Request, Response, Status as TonicStatus};
use tracing::info;

pub async fn capture_payment(
    txn: &DatabaseTransaction,
    request: Request<CapturePaymentRequest>,
) -> Result<Response<PaymentIntentsResponse>, TonicStatus> {
    let req = request.into_inner();

    // Basic validation: gateway payment id must be present.
    if req.razorpay_payment_id.is_empty() {
        return Err(TonicStatus::invalid_argument(
            "razorpay_payment_id is required for capture",
        ));
    }

    let intent = payment_intents::Entity::find_by_id(req.intent_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| {
            TonicStatus::not_found(format!("Payment intent {} not found", req.intent_id))
        })?;

    info!(
        payment_intent_id = intent.intent_id,
        razorpay_payment_id = %req.razorpay_payment_id,
        "capture_payment invoked"
    );

    // If this intent already has a gateway payment id attached, enforce idempotency/conflict
    // semantics:
    //
    // - Same gateway id: treat as idempotent replay and return current state without changing it.
    // - Different gateway id: conflicting capture for the same intent → reject.
    if let Some(existing_gateway_id) = &intent.razorpay_payment_id {
        if existing_gateway_id == &req.razorpay_payment_id {
            // Idempotent replay: return the current intent as-is.
            info!(
                payment_intent_id = intent.intent_id,
                razorpay_payment_id = %req.razorpay_payment_id,
                "capture_payment idempotent replay"
            );
            return Ok(Response::new(PaymentIntentsResponse {
                items: vec![PaymentIntentResponse {
                    intent_id: intent.intent_id,
                    razorpay_order_id: intent.razorpay_order_id.clone(),
                    order_id: intent.order_id,
                    user_id: intent.user_id,
                    amount_paise: intent.amount_paise as i64,
                    currency: intent.currency.clone(),
                    status: intent
                        .status
                        .map(|s| format!("{:?}", s).to_lowercase())
                        .unwrap_or_else(|| "processed".to_string()),
                    razorpay_payment_id: intent.razorpay_payment_id.clone(),
                    created_at: intent.created_at.map(|t| t.to_string()).unwrap_or_default(),
                    expires_at: intent.expires_at.to_string(),
                }],
            }));
        } else {
            // Conflicting payment ids for the same intent: mark as NeedsReview at the caller.
            info!(
                payment_intent_id = intent.intent_id,
                existing_razorpay_payment_id = %existing_gateway_id,
                new_razorpay_payment_id = %req.razorpay_payment_id,
                "capture_payment conflicting gateway payment id for same intent"
            );
            return Err(TonicStatus::failed_precondition(
                "Conflicting gateway payment id for this intent; mark as NeedsReview",
            ));
        }
    }

    // Ensure this gateway payment id is not already used for a different intent.
    let existing_for_gateway = payment_intents::Entity::find()
        .filter(payment_intents::Column::RazorpayPaymentId.eq(req.razorpay_payment_id.clone()))
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?;

    if let Some(other) = existing_for_gateway {
        if other.intent_id != intent.intent_id {
            info!(
                payment_intent_id = other.intent_id,
                new_intent_id = intent.intent_id,
                razorpay_payment_id = %req.razorpay_payment_id,
                "capture_payment gateway payment id already used for different intent"
            );
            return Err(TonicStatus::failed_precondition(
                "Gateway payment id already used for a different intent; mark as NeedsReview",
            ));
        }
    }

    let mut model = intent.into_active_model();
    model.status = ActiveValue::Set(Some(Status::Processed));
    model.razorpay_payment_id = ActiveValue::Set(Some(req.razorpay_payment_id));

    match model.update(txn).await {
        Ok(updated) => Ok(Response::new(PaymentIntentsResponse {
            items: vec![PaymentIntentResponse {
                intent_id: updated.intent_id,
                razorpay_order_id: updated.razorpay_order_id,
                order_id: updated.order_id,
                user_id: updated.user_id,
                amount_paise: updated.amount_paise as i64,
                currency: updated.currency,
                status: updated
                    .status
                    .map(|s| format!("{:?}", s).to_lowercase())
                    .unwrap_or_else(|| "processed".to_string()),
                razorpay_payment_id: updated.razorpay_payment_id,
                created_at: updated
                    .created_at
                    .map(|t| t.to_string())
                    .unwrap_or_default(),
                expires_at: updated.expires_at.to_string(),
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
