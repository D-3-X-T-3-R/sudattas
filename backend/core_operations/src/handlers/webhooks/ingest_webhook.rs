use crate::handlers::db_errors::map_db_error_to_status;
use crate::handlers::payment_intents::capture_payment;
use crate::order_state_machine;
use chrono::Utc;
use core_db_entities::entity::sea_orm_active_enums::{PaymentStatus, Status};
use core_db_entities::entity::webhook_events;
use core_db_entities::entity::{coupon_redemptions, orders, payment_intents};
use proto::proto::core::{
    CapturePaymentRequest, IngestWebhookRequest, WebhookEventResponse, WebhookEventsResponse,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, DatabaseTransaction, DbBackend,
    EntityTrait, IntoActiveModel, QueryFilter, Statement,
};
use tonic::{Request, Response, Status as TonicStatus};
use tracing::{info, warn};

pub async fn ingest_webhook(
    txn: &DatabaseTransaction,
    request: Request<IngestWebhookRequest>,
) -> Result<Response<WebhookEventsResponse>, TonicStatus> {
    let req = request.into_inner();

    info!(
        provider = %req.provider,
        event_type = %req.event_type,
        webhook_id = %req.webhook_id,
        signature_verified = req.signature_verified,
        "ingest_webhook received event"
    );

    // Idempotency: if we've already seen this webhook_id, return it as-is.
    if let Some(existing) = webhook_events::Entity::find()
        .filter(webhook_events::Column::WebhookId.eq(&req.webhook_id))
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
    {
        info!(
            webhook_id = %req.webhook_id,
            "ingest_webhook idempotent replay – returning existing event"
        );
        return Ok(Response::new(WebhookEventsResponse {
            items: vec![model_to_response(existing)],
        }));
    }

    // Phase 6 replay protection: reject duplicate provider_event_id (e.g. x-razorpay-event-id).
    if let Some(ref peid) = req.provider_event_id {
        let peid = peid.trim();
        if !peid.is_empty()
            && webhook_events::Entity::find()
                .filter(webhook_events::Column::ProviderEventId.eq(peid))
                .one(txn)
                .await
                .map_err(map_db_error_to_status)?
                .is_some()
        {
            return Err(TonicStatus::already_exists(format!(
                "Replay: provider_event_id already processed: {}",
                peid
            )));
        }
    }

    // Persist with Pending status.
    let payload_json: serde_json::Value =
        serde_json::from_str(&req.payload_json).unwrap_or(serde_json::Value::Null);

    let provider_event_id_value = req
        .provider_event_id
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(String::from);

    let event = webhook_events::ActiveModel {
        event_id: ActiveValue::NotSet,
        provider: ActiveValue::Set(req.provider.clone()),
        event_type: ActiveValue::Set(req.event_type.clone()),
        webhook_id: ActiveValue::Set(req.webhook_id.clone()),
        provider_event_id: ActiveValue::Set(provider_event_id_value),
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
                crate::observability::record_webhook_processing_failed_total();
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

    info!(
        webhook_id = %updated.webhook_id,
        provider = %updated.provider,
        event_type = %updated.event_type,
        status = ?updated.status,
        "ingest_webhook completed"
    );

    Ok(Response::new(WebhookEventsResponse {
        items: vec![model_to_response(updated)],
    }))
}

async fn process_payment_captured(
    txn: &DatabaseTransaction,
    payload: &serde_json::Value,
) -> Result<(), TonicStatus> {
    let entity = &payload["payload"]["payment"]["entity"];
    let payment_id = entity["id"]
        .as_str()
        .ok_or_else(|| TonicStatus::invalid_argument("Missing payment id in webhook payload"))?;

    // Razorpay: amount is in smallest currency unit (paise for INR).
    let webhook_amount_paise: i64 = entity["amount"].as_i64().unwrap_or(0);
    let webhook_currency: String = entity["currency"].as_str().unwrap_or("").to_uppercase();

    let razorpay_order_id = entity["order_id"].as_str().unwrap_or("");

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

    info!(
        webhook_payment_id = %payment_id,
        razorpay_order_id = %razorpay_order_id,
        payment_intent_id = intent.intent_id,
        "process_payment_captured resolved payment_intent for webhook"
    );

    // Phase 5: Verify amount and currency before treating as paid.
    let intent_paise = intent.amount_paise as i64;
    let order = match intent.order_id {
        Some(oid) => orders::Entity::find_by_id(oid)
            .one(txn)
            .await
            .ok()
            .flatten(),
        None => None,
    };
    let order_grand_paise: Option<i64> = order.as_ref().and_then(|o| o.grand_total_minor);
    let intent_currency = intent.currency.as_deref().unwrap_or("").to_uppercase();

    // When intent has no order, verify only webhook vs intent; when it has an order, require order grand total to match too.
    let amount_ok =
        webhook_amount_paise == intent_paise && order_grand_paise.is_none_or(|g| g == intent_paise);
    let currency_ok = !webhook_currency.is_empty() && webhook_currency == intent_currency;

    if !amount_ok || !currency_ok {
        crate::observability::record_payment_mismatch_total();
        warn!(
            payment_intent_id = intent.intent_id,
            webhook_amount_paise = webhook_amount_paise,
            intent_paise = intent_paise,
            order_grand_paise = ?order_grand_paise,
            webhook_currency = %webhook_currency,
            intent_currency = %intent_currency,
            "payment.captured amount/currency mismatch – marking as needs_review"
        );
        let _ = txn
            .execute(Statement::from_sql_and_values(
                DbBackend::MySql,
                "UPDATE payment_intents SET status = 'needs_review' WHERE intent_id = ?",
                [intent.intent_id.into()],
            ))
            .await;
        if let Some(order_id) = intent.order_id {
            let _ = order_state_machine::transition_order_status(
                txn,
                order_id,
                order_state_machine::OrderState::NeedsReview,
                "payment_mismatch",
                "system",
                Some("Amount/currency mismatch – needs review"),
                Some(PaymentStatus::NeedsReview),
            )
            .await;
        }
        return Ok(());
    }

    capture_payment(
        txn,
        tonic::Request::new(CapturePaymentRequest {
            intent_id: intent.intent_id,
            razorpay_payment_id: payment_id.to_string(),
        }),
    )
    .await?;

    if let Some(order_id) = intent.order_id {
        let _ = order_state_machine::transition_order_status(
            txn,
            order_id,
            order_state_machine::OrderState::Paid,
            "payment_captured",
            "system",
            Some("Payment captured"),
            Some(PaymentStatus::Captured),
        )
        .await;
    }

    // Phase 4: increment coupon usage_count only on verified payment (not on place_order).
    // P1: record redemption for per-customer usage tracking.
    if let Some(order_id) = intent.order_id {
        if let Ok(Some(order)) = orders::Entity::find_by_id(order_id).one(txn).await {
            if let Some(coupon_id) = order.applied_coupon_id {
                let res = txn
                    .execute(Statement::from_sql_and_values(
                        DbBackend::MySql,
                        r#"UPDATE coupons SET usage_count = COALESCE(usage_count, 0) + 1
                           WHERE coupon_id = ? AND (usage_limit IS NULL OR COALESCE(usage_count, 0) < usage_limit)"#,
                        [coupon_id.into()],
                    ))
                    .await;
                if let Ok(result) = res {
                    if result.rows_affected() > 0 {
                        info!(
                            coupon_id = coupon_id,
                            "coupon usage_count incremented (within limit)"
                        );
                        let redemption = coupon_redemptions::ActiveModel {
                            redemption_id: ActiveValue::NotSet,
                            coupon_id: ActiveValue::Set(coupon_id),
                            user_id: ActiveValue::Set(order.user_id),
                            order_id: ActiveValue::Set(order_id),
                            redeemed_at: ActiveValue::Set(Some(Utc::now())),
                        };
                        if redemption.insert(txn).await.is_err() {
                            warn!(
                                coupon_id = coupon_id,
                                order_id = order_id,
                                "coupon_redemptions insert failed (non-fatal)"
                            );
                        }
                    }
                }
            }
        }
    }

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
