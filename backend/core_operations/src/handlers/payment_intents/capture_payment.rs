//! Explicit capture recovery requires authoritative provider evidence.
use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::payment_intents;
use proto::proto::core::{CapturePaymentRequest, GetPaymentIntentRequest, PaymentIntentsResponse};
use sea_orm::{DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn capture_payment(
    txn: &DatabaseTransaction,
    request: Request<CapturePaymentRequest>,
) -> Result<Response<PaymentIntentsResponse>, Status> {
    let req = request.into_inner();
    let intent = payment_intents::Entity::find_by_id(req.intent_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::not_found("Payment intent not found"))?;
    let evidence = crate::razorpay::fetch_payment(&req.razorpay_payment_id)
        .await
        .map_err(|_| {
            Status::unavailable(
                "Payment confirmation unavailable; retry confirmation, not purchase",
            )
        })?;
    if evidence.id != req.razorpay_payment_id || evidence.order_id != intent.razorpay_order_id {
        return Err(Status::failed_precondition(
            "Provider payment association mismatch",
        ));
    }
    if evidence.status != "captured" || !evidence.captured {
        return Err(Status::failed_precondition("Payment is not captured"));
    }
    super::capture_evidence::apply_captured_evidence(
        txn,
        &evidence.order_id,
        &evidence.id,
        evidence.amount,
        &evidence.currency,
    )
    .await?;
    super::get_payment_intent(
        txn,
        Request::new(GetPaymentIntentRequest {
            intent_id: Some(req.intent_id),
            order_id: None,
        }),
    )
    .await
}
