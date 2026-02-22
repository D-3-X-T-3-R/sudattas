use crate::handlers::db_errors::map_db_error_to_status;
use crate::handlers::payment_intents::capture_payment;
use chrono::Utc;
use core_db_entities::entity::payment_intents;
use core_db_entities::entity::sea_orm_active_enums::Status;
use core_db_entities::entity::webhook_events;
use proto::proto::core::{
    CapturePaymentRequest, IngestWebhookRequest, WebhookEventResponse, WebhookEventsResponse,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseTransaction, EntityTrait,
    IntoActiveModel, QueryFilter,
};
use tonic::{Request, Response, Status as TonicStatus};

pub async fn ingest_webhook(
    txn: &DatabaseTransaction,
    request: Request<IngestWebhookRequest>,
) -> Result<Response<WebhookEventsResponse>, TonicStatus> {
    let req = request.into_inner();

    // Idempotency: if we've already seen this webhook_id, return it as-is.
    if let Some(existing) = webhook_events::Entity::find()
        .filter(webhook_events::Column::WebhookId.eq(&req.webhook_id))
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
    {
        return Ok(Response::new(WebhookEventsResponse {
            items: vec![model_to_response(existing)],
        }));
    }

    // Persist with Pending status.
    let payload_json: serde_json::Value =
        serde_json::from_str(&req.payload_json).unwrap_or(serde_json::Value::Null);

    let event = webhook_events::ActiveModel {
        event_id: ActiveValue::NotSet,
        provider: ActiveValue::Set(req.provider.clone()),
        event_type: ActiveValue::Set(req.event_type.clone()),
        webhook_id: ActiveValue::Set(req.webhook_id.clone()),
        payload: ActiveValue::Set(payload_json.clone()),
        status: ActiveValue::Set(Some(Status::Pending)),
        received_at: ActiveValue::Set(Some(Utc::now())),
    };

    let inserted = event.insert(txn).await.map_err(map_db_error_to_status)?;

    // Process: payment.captured → trigger capture_payment.
    let new_status = if req.event_type == "payment.captured" && req.signature_verified {
        match process_payment_captured(txn, &payload_json).await {
            Ok(_) => Status::Processed,
            Err(e) => {
                log::warn!("payment.captured processing failed: {}", e);
                Status::Failed
            }
        }
    } else if req.signature_verified {
        // Other known events: mark processed (no additional logic needed yet).
        Status::Processed
    } else {
        Status::Failed
    };

    // Update status.
    let mut active = inserted.clone().into_active_model();
    active.status = ActiveValue::Set(Some(new_status));
    let updated = active.update(txn).await.map_err(map_db_error_to_status)?;

    Ok(Response::new(WebhookEventsResponse {
        items: vec![model_to_response(updated)],
    }))
}

async fn process_payment_captured(
    txn: &DatabaseTransaction,
    payload: &serde_json::Value,
) -> Result<(), TonicStatus> {
    let payment_id = payload["payload"]["payment"]["entity"]["id"]
        .as_str()
        .ok_or_else(|| TonicStatus::invalid_argument("Missing payment id in webhook payload"))?;

    // Razorpay order_id in the payload is their own "order_xxx" string — not our internal ID.
    // Look up the payment intent by razorpay_order_id to find our intent_id.
    let razorpay_order_id = payload["payload"]["payment"]["entity"]["order_id"]
        .as_str()
        .unwrap_or("");

    let intent = payment_intents::Entity::find()
        .filter(payment_intents::Column::RazorpayOrderId.eq(razorpay_order_id))
        .one(txn)
        .await
        .map_err(|e| TonicStatus::internal(e.to_string()))?
        .ok_or_else(|| {
            TonicStatus::not_found(format!(
                "No payment intent found for razorpay_order_id={}",
                razorpay_order_id
            ))
        })?;

    capture_payment(
        txn,
        tonic::Request::new(CapturePaymentRequest {
            intent_id: intent.intent_id,
            razorpay_payment_id: payment_id.to_string(),
        }),
    )
    .await?;

    Ok(())
}

pub fn model_to_response(m: webhook_events::Model) -> WebhookEventResponse {
    WebhookEventResponse {
        event_id: m.event_id,
        provider: m.provider,
        event_type: m.event_type,
        webhook_id: m.webhook_id,
        status: m
            .status
            .map(|s| format!("{:?}", s).to_lowercase())
            .unwrap_or_default(),
        received_at: m.received_at.map(|t| t.to_string()).unwrap_or_default(),
    }
}
