//! Verify the browser signature and retain a pending confirmation. Only authenticated
//! provider capture evidence may mark a prepaid order paid.

use crate::handlers::db_errors::map_db_error_to_status;

use core_db_entities::entity::payment_intents;
use core_db_entities::entity::sea_orm_active_enums::Status;
use hmac::{Hmac, Mac};
use proto::proto::core::{
    PaymentIntentResponse, VerifyRazorpayPaymentRequest, VerifyRazorpayPaymentResponse,
};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use sha2::Sha256;
use subtle::ConstantTimeEq;
use tonic::{Request, Response, Status as TonicStatus};
use tracing::warn;

type HmacSha256 = Hmac<Sha256>;

fn verify_signature(order_id: &str, payment_id: &str, signature: &str, secret: &str) -> bool {
    let payload = format!("{}|{}", order_id, payment_id);
    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };
    mac.update(payload.as_bytes());
    let computed = hex::encode(mac.finalize().into_bytes());
    let computed_bytes = computed.as_bytes();
    let signature_bytes = signature.as_bytes();
    if computed_bytes.len() != signature_bytes.len() {
        return false;
    }
    computed_bytes.ct_eq(signature_bytes).into()
}

fn intent_to_response(model: &payment_intents::Model) -> PaymentIntentResponse {
    PaymentIntentResponse {
        intent_id: model.intent_id,
        razorpay_order_id: model.razorpay_order_id.clone(),
        order_id: model.order_id,
        user_id: model.user_id,
        amount_paise: model.amount_paise as i64,
        currency: model.currency.clone(),
        status: format!("{:?}", model.status).to_lowercase(),
        razorpay_payment_id: model.razorpay_payment_id.clone(),
        created_at: model.created_at.map(|t| t.to_string()).unwrap_or_default(),
        expires_at: model.expires_at.to_string(),
        razorpay_key_id: None,
        gateway_fee_paise: model.gateway_fee_paise.map(i64::from),
        gateway_tax_paise: model.gateway_tax_paise.map(i64::from),
    }
}

pub async fn verify_razorpay_payment(
    txn: &DatabaseTransaction,
    request: Request<VerifyRazorpayPaymentRequest>,
) -> Result<Response<VerifyRazorpayPaymentResponse>, TonicStatus> {
    let req = request.into_inner();

    if req.razorpay_order_id.is_empty()
        || req.razorpay_payment_id.is_empty()
        || req.razorpay_signature.is_empty()
    {
        return Err(TonicStatus::invalid_argument(
            "razorpay_order_id, razorpay_payment_id and razorpay_signature are required",
        ));
    }

    let secret = match std::env::var("RAZORPAY_KEY_SECRET") {
        Ok(s) => s,
        Err(_) => {
            warn!("RAZORPAY_KEY_SECRET not set; cannot verify payment signature");
            return Err(TonicStatus::failed_precondition(
                "Razorpay not configured (KEY_SECRET missing)",
            ));
        }
    };

    // Always authenticate the callback, including replays of a captured payment.
    if !verify_signature(
        &req.razorpay_order_id,
        &req.razorpay_payment_id,
        &req.razorpay_signature,
        &secret,
    ) {
        return Ok(Response::new(VerifyRazorpayPaymentResponse {
            verified: false,
            payment_intent: None,
        }));
    }
    use sea_orm::{
        sea_query::LockType, ActiveModelTrait, ActiveValue, IntoActiveModel, QuerySelect,
    };
    let order = core_db_entities::entity::orders::Entity::find_by_id(req.order_id)
        .lock(LockType::Update)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| TonicStatus::not_found("Order not found"))?;
    let intent = payment_intents::Entity::find()
        .filter(payment_intents::Column::OrderId.eq(req.order_id))
        .filter(payment_intents::Column::RazorpayOrderId.eq(&req.razorpay_order_id))
        .lock(LockType::Update)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| TonicStatus::not_found("Payment intent not found"))?;
    if intent.user_id != Some(order.user_id)
        || intent
            .razorpay_payment_id
            .as_deref()
            .is_some_and(|id| id != req.razorpay_payment_id)
    {
        return Err(TonicStatus::failed_precondition(
            "Payment association conflict",
        ));
    }
    if matches!(
        intent.status,
        Status::Processed | Status::Failed | Status::NeedsReview
    ) {
        return Ok(Response::new(VerifyRazorpayPaymentResponse {
            verified: true,
            payment_intent: Some(intent_to_response(&intent)),
        }));
    }
    // Signature proves a Checkout response, not capture. Preserve the payment ID
    // for the reconciliation worker; frontend already maps client_verified to pending.
    let mut active = intent.into_active_model();
    active.razorpay_payment_id = ActiveValue::Set(Some(req.razorpay_payment_id));
    active.status = ActiveValue::Set(Status::ClientVerified);
    let updated = active.update(txn).await.map_err(map_db_error_to_status)?;
    Ok(Response::new(VerifyRazorpayPaymentResponse {
        verified: true,
        payment_intent: Some(intent_to_response(&updated)),
    }))
}
