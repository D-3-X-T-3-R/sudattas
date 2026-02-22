use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::payment_intents;
use proto::proto::core::{GetPaymentIntentRequest, PaymentIntentResponse, PaymentIntentsResponse};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn get_payment_intent(
    txn: &DatabaseTransaction,
    request: Request<GetPaymentIntentRequest>,
) -> Result<Response<PaymentIntentsResponse>, Status> {
    let req = request.into_inner();

    let query = payment_intents::Entity::find();

    let query = if let Some(intent_id) = req.intent_id {
        query.filter(payment_intents::Column::IntentId.eq(intent_id))
    } else if let Some(order_id) = req.order_id {
        query.filter(payment_intents::Column::OrderId.eq(order_id))
    } else {
        return Err(Status::invalid_argument("Either intent_id or order_id must be set"));
    };

    let results = query.all(txn).await.map_err(map_db_error_to_status)?;

    let items = results
        .into_iter()
        .map(|model| PaymentIntentResponse {
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
        })
        .collect();

    Ok(Response::new(PaymentIntentsResponse { items }))
}
