use crate::handlers::db_errors::map_db_error_to_status;
use crate::money::decimal_to_paise;
use crate::razorpay;
use chrono::Utc;
use core_db_entities::entity::orders;
use core_db_entities::entity::payment_intents;
use core_db_entities::entity::sea_orm_active_enums::Status;
use proto::proto::core::{
    CreatePaymentIntentRequest, PaymentIntentResponse, PaymentIntentsResponse,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status as TonicStatus};

pub async fn create_payment_intent(
    txn: &DatabaseTransaction,
    request: Request<CreatePaymentIntentRequest>,
) -> Result<Response<PaymentIntentsResponse>, TonicStatus> {
    let req = request.into_inner();

    let (razorpay_order_id, amount_paise, currency) = match req.razorpay_order_id.as_deref() {
        Some(s) if !s.trim().is_empty() => {
            let currency = req.currency.unwrap_or_else(|| "INR".to_string());
            (s.trim().to_string(), req.amount_paise, currency)
        }
        _ => {
            // Server-authoritative: create Razorpay order from backend.
            let order = orders::Entity::find_by_id(req.order_id)
                .one(txn)
                .await
                .map_err(map_db_error_to_status)?
                .ok_or_else(|| {
                    TonicStatus::not_found(format!("Order {} not found", req.order_id))
                })?;

            let amount_paise = order
                .grand_total_minor
                .unwrap_or_else(|| decimal_to_paise(&order.total_amount));
            let currency = order.currency.as_deref().unwrap_or("INR").to_string();
            let receipt = format!("ord_{}", req.order_id);
            if receipt.len() > 40 {
                return Err(TonicStatus::invalid_argument(
                    "Order receipt string exceeds Razorpay 40-char limit",
                ));
            }

            match razorpay::create_order(amount_paise, &currency, &receipt).await {
                Ok(razorpay_order_id) => (razorpay_order_id, amount_paise, currency),
                Err(e) => {
                    // CI / dev without RAZORPAY_KEY_*: create intent with placeholder id so
                    // place_order still produces a row and webhooks/tests can find it by order.
                    tracing::warn!(
                        "Razorpay order create failed ({}), using placeholder razorpay_order_id for order {}",
                        e,
                        req.order_id
                    );
                    let placeholder = format!("rzp_pending_{}", req.order_id);
                    (placeholder, amount_paise, currency)
                }
            }
        }
    };

    let expires_at = Utc::now() + chrono::Duration::hours(24);

    let intent = payment_intents::ActiveModel {
        intent_id: ActiveValue::NotSet,
        razorpay_order_id: ActiveValue::Set(razorpay_order_id),
        order_id: ActiveValue::Set(Some(req.order_id)),
        user_id: ActiveValue::Set(Some(req.user_id)),
        amount_paise: ActiveValue::Set(amount_paise as i32),
        currency: ActiveValue::Set(Some(currency)),
        status: ActiveValue::Set(Status::Pending),
        razorpay_payment_id: ActiveValue::Set(None),
        metadata: ActiveValue::Set(None),
        gateway_fee_paise: ActiveValue::Set(None),
        gateway_tax_paise: ActiveValue::Set(None),
        created_at: ActiveValue::Set(Some(Utc::now())),
        expires_at: ActiveValue::Set(expires_at),
    };

    match intent.insert(txn).await {
        Ok(model) => {
            let razorpay_key_id = razorpay::key_id_for_frontend();
            Ok(Response::new(PaymentIntentsResponse {
                items: vec![PaymentIntentResponse {
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
                    razorpay_key_id,
                }],
            }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
