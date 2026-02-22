use crate::handlers::db_errors::map_db_error_to_status;
use chrono::Utc;
use core_db_entities::entity::payment_intents;
use core_db_entities::entity::sea_orm_active_enums::Status;
use proto::proto::core::{
    CreatePaymentIntentRequest, PaymentIntentResponse, PaymentIntentsResponse,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status as TonicStatus};

pub async fn create_payment_intent(
    txn: &DatabaseTransaction,
    request: Request<CreatePaymentIntentRequest>,
) -> Result<Response<PaymentIntentsResponse>, TonicStatus> {
    let req = request.into_inner();

    let currency = req.currency.unwrap_or_else(|| "INR".to_string());
    let expires_at = Utc::now() + chrono::Duration::hours(24);

    let intent = payment_intents::ActiveModel {
        intent_id: ActiveValue::NotSet,
        razorpay_order_id: ActiveValue::Set(req.razorpay_order_id),
        order_id: ActiveValue::Set(Some(req.order_id)),
        user_id: ActiveValue::Set(Some(req.user_id)),
        amount_paise: ActiveValue::Set(req.amount_paise as i32),
        currency: ActiveValue::Set(Some(currency)),
        status: ActiveValue::Set(Some(Status::Pending)),
        razorpay_payment_id: ActiveValue::Set(None),
        metadata: ActiveValue::Set(None),
        created_at: ActiveValue::Set(Some(Utc::now())),
        expires_at: ActiveValue::Set(expires_at),
    };

    match intent.insert(txn).await {
        Ok(model) => Ok(Response::new(PaymentIntentsResponse {
            items: vec![PaymentIntentResponse {
                intent_id: model.intent_id,
                razorpay_order_id: model.razorpay_order_id,
                order_id: model.order_id,
                user_id: model.user_id,
                amount_paise: model.amount_paise as i64,
                currency: model.currency,
                status: model
                    .status
                    .map(|s| format!("{:?}", s).to_lowercase())
                    .unwrap_or_else(|| "pending".to_string()),
                razorpay_payment_id: model.razorpay_payment_id,
                created_at: model.created_at.map(|t| t.to_string()).unwrap_or_default(),
                expires_at: model.expires_at.to_string(),
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
