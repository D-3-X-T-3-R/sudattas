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
    sea_query::LockType, ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection,
    DatabaseTransaction, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder, QuerySelect,
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

fn is_valid_razorpay_order_id(value: &str) -> bool {
    value.starts_with("order_")
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

/// Resolves a Razorpay order id for a `CreatePaymentIntentRequest` that didn't supply one,
/// performing the Razorpay HTTP call (up to 15s, see `razorpay::http_client`) against a plain
/// read-only order lookup on `db` rather than an open `DatabaseTransaction`, so the round-trip
/// doesn't hold a pooled connection idle. Mirrors the "server-authoritative" branch of
/// [`create_payment_intent`] below; callers that resolve an order id this way should feed the
/// returned `(razorpay_order_id, amount_paise, currency)` back into the request before calling
/// [`create_payment_intent`], which will take the caller-supplied-id branch.
pub async fn resolve_server_created_razorpay_order(
    db: &DatabaseConnection,
    order_id: i64,
) -> Result<(String, i64, String), TonicStatus> {
    let order = orders::Entity::find_by_id(order_id)
        .one(db)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| TonicStatus::not_found(format!("Order {} not found", order_id)))?;

    let amount_paise = order.grand_total_minor;
    let currency = order.currency.as_deref().unwrap_or("INR").to_string();
    let receipt = format!("ord_{}", order_id);
    if receipt.len() > 40 {
        return Err(TonicStatus::invalid_argument(
            "Order receipt string exceeds Razorpay 40-char limit",
        ));
    }

    let razorpay_order_id = razorpay::create_order(amount_paise, &currency, &receipt)
        .await
        .map_err(|e| {
            TonicStatus::unavailable(format!(
                "Unable to create Razorpay order for order {}: {}",
                order_id, e
            ))
        })?;
    if !is_valid_razorpay_order_id(razorpay_order_id.as_str()) {
        return Err(TonicStatus::internal(format!(
            "Razorpay returned invalid order id '{}' for order {}",
            razorpay_order_id, order_id
        )));
    }
    Ok((razorpay_order_id, amount_paise, currency))
}

pub async fn create_payment_intent(
    txn: &DatabaseTransaction,
    request: Request<CreatePaymentIntentRequest>,
) -> Result<Response<PaymentIntentsResponse>, TonicStatus> {
    let req = request.into_inner();
    let requested_gateway_order_id = req
        .razorpay_order_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let existing_intent = payment_intents::Entity::find()
        .filter(payment_intents::Column::OrderId.eq(req.order_id))
        .order_by_desc(payment_intents::Column::IntentId)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?;

    if let Some(existing) = existing_intent {
        let existing_is_invalid = !is_valid_razorpay_order_id(existing.razorpay_order_id.as_str());
        let request_has_valid_override = requested_gateway_order_id
            .map(is_valid_razorpay_order_id)
            .unwrap_or(false);
        let can_refresh_invalid_existing = existing_is_invalid
            && !matches!(existing.status, Status::Processed)
            && (requested_gateway_order_id.is_none() || request_has_valid_override);
        if !can_refresh_invalid_existing {
            return Ok(Response::new(PaymentIntentsResponse {
                items: vec![model_to_response(existing)],
            }));
        }
    }

    let (razorpay_order_id, amount_paise, currency) = match requested_gateway_order_id {
        Some(s) => {
            if !is_valid_razorpay_order_id(s) {
                return Err(TonicStatus::invalid_argument(
                    "razorpay_order_id must start with 'order_'",
                ));
            }
            let currency = req.currency.unwrap_or_else(|| "INR".to_string());
            (s.to_string(), req.amount_paise, currency)
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

            let razorpay_order_id = razorpay::create_order(amount_paise, &currency, &receipt)
                .await
                .map_err(|e| {
                    TonicStatus::unavailable(format!(
                        "Unable to create Razorpay order for order {}: {}",
                        req.order_id, e
                    ))
                })?;
            if !is_valid_razorpay_order_id(razorpay_order_id.as_str()) {
                return Err(TonicStatus::internal(format!(
                    "Razorpay returned invalid order id '{}' for order {}",
                    razorpay_order_id, req.order_id
                )));
            }
            (razorpay_order_id, amount_paise, currency)
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
