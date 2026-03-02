//! Verify client-returned Razorpay signature after checkout; mark intent as ClientVerified.
//! Webhook remains the final authority for Paid.

use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::payment_intents;
use core_db_entities::entity::sea_orm_active_enums::Status;
use hmac::{Hmac, Mac};
use proto::proto::core::{
    PaymentIntentResponse, VerifyRazorpayPaymentRequest, VerifyRazorpayPaymentResponse,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseTransaction, EntityTrait, IntoActiveModel,
    QueryFilter,
};
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

    let intent = payment_intents::Entity::find()
        .filter(payment_intents::Column::OrderId.eq(req.order_id))
        .filter(payment_intents::Column::RazorpayOrderId.eq(&req.razorpay_order_id))
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| {
            TonicStatus::not_found(format!(
                "Payment intent for order {} with razorpay_order_id {} not found",
                req.order_id, req.razorpay_order_id
            ))
        })?;

    if !verify_signature(
        &req.razorpay_order_id,
        &req.razorpay_payment_id,
        &req.razorpay_signature,
        &secret,
    ) {
        crate::observability::record_payment_verify_invalid_signature_total();
        warn!(
            order_id = req.order_id,
            "verify_razorpay_payment: invalid signature rejected"
        );
        return Ok(Response::new(VerifyRazorpayPaymentResponse {
            verified: false,
            payment_intent: None,
        }));
    }

    let mut active = intent.clone().into_active_model();
    active.status = ActiveValue::Set(Status::ClientVerified);
    active.razorpay_payment_id = ActiveValue::Set(Some(req.razorpay_payment_id.clone()));

    let updated: payment_intents::Model =
        active.update(txn).await.map_err(map_db_error_to_status)?;

    Ok(Response::new(VerifyRazorpayPaymentResponse {
        verified: true,
        payment_intent: Some(intent_to_response(&updated)),
    }))
}
