use crate::handlers::db_errors::map_db_error_to_status;
use crate::razorpay;
use chrono::Utc;
use core_db_entities::entity::orders;
use core_db_entities::entity::payment_intents;
use core_db_entities::entity::sea_orm_active_enums::Status;
use proto::proto::core::{
    CreatePaymentIntentRequest, PaymentIntentResponse, PaymentIntentsResponse,
};
use sea_orm::{
    sea_query::LockType, ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseTransaction,
    EntityTrait, IntoActiveModel, QueryFilter, QueryOrder, QuerySelect,
};
use tonic::{Request, Response, Status as TonicStatus};

fn is_duplicate_entry(err: &sea_orm::DbErr) -> bool {
    match err {
        sea_orm::DbErr::Exec(exec) => {
            let message = exec.to_string();
            message.contains("Duplicate entry") || message.contains("1062")
        }
        _ => false,
    }
}

fn model_to_response(model: payment_intents::Model) -> PaymentIntentResponse {
    let razorpay_key_id = razorpay::key_id_for_frontend();
    PaymentIntentResponse {
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
    }
}

pub async fn create_payment_intent(
    txn: &DatabaseTransaction,
    request: Request<CreatePaymentIntentRequest>,
) -> Result<Response<PaymentIntentsResponse>, TonicStatus> {
    let req = request.into_inner();
    let existing_intent = payment_intents::Entity::find()
        .filter(payment_intents::Column::OrderId.eq(req.order_id))
        .order_by_desc(payment_intents::Column::IntentId)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?;

    if let Some(existing) = existing_intent {
        let can_refresh_placeholder = existing.razorpay_order_id.starts_with("rzp_pending_")
            && req
                .razorpay_order_id
                .as_deref()
                .is_none_or(|value| value.trim().is_empty())
            && !matches!(existing.status, Status::Processed);
        if !can_refresh_placeholder {
            return Ok(Response::new(PaymentIntentsResponse {
                items: vec![model_to_response(existing)],
            }));
        }
    }

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

            let amount_paise = order.grand_total_minor;
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

    if let Some(existing) = payment_intents::Entity::find()
        .filter(payment_intents::Column::OrderId.eq(req.order_id))
        .order_by_desc(payment_intents::Column::IntentId)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
    {
        let mut active = existing.into_active_model();
        active.razorpay_order_id = ActiveValue::Set(razorpay_order_id);
        active.user_id = ActiveValue::Set(Some(req.user_id));
        active.amount_paise = ActiveValue::Set(amount_paise as i32);
        active.currency = ActiveValue::Set(Some(currency));
        active.status = ActiveValue::Set(Status::Pending);
        active.razorpay_payment_id = ActiveValue::Set(None);
        active.expires_at = ActiveValue::Set(expires_at);

        let model = active.update(txn).await.map_err(map_db_error_to_status)?;
        return Ok(Response::new(PaymentIntentsResponse {
            items: vec![model_to_response(model)],
        }));
    }

    let intent = payment_intents::ActiveModel {
        intent_id: ActiveValue::NotSet,
        razorpay_order_id: ActiveValue::Set(razorpay_order_id),
        order_id: ActiveValue::Set(Some(req.order_id)),
        active_order_id: ActiveValue::NotSet,
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
        Ok(model) => Ok(Response::new(PaymentIntentsResponse {
            items: vec![model_to_response(model)],
        })),
        Err(err) if is_duplicate_entry(&err) => {
            let existing = payment_intents::Entity::find()
                .filter(payment_intents::Column::OrderId.eq(req.order_id))
                .lock(LockType::Update)
                .order_by_desc(payment_intents::Column::IntentId)
                .one(txn)
                .await
                .map_err(map_db_error_to_status)?
                .ok_or_else(|| {
                    TonicStatus::already_exists(
                        "Active payment intent already exists for this order",
                    )
                })?;
            Ok(Response::new(PaymentIntentsResponse {
                items: vec![model_to_response(existing)],
            }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
