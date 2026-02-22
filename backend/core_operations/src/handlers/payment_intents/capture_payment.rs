use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::payment_intents;
use core_db_entities::entity::sea_orm_active_enums::Status;
use proto::proto::core::{CapturePaymentRequest, PaymentIntentResponse, PaymentIntentsResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction, EntityTrait, IntoActiveModel};
use tonic::{Request, Response, Status as TonicStatus};

pub async fn capture_payment(
    txn: &DatabaseTransaction,
    request: Request<CapturePaymentRequest>,
) -> Result<Response<PaymentIntentsResponse>, TonicStatus> {
    let req = request.into_inner();

    let intent = payment_intents::Entity::find_by_id(req.intent_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| {
            TonicStatus::not_found(format!("Payment intent {} not found", req.intent_id))
        })?;

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
