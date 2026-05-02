use crate::cancellation_saga;
use crate::handlers::db_errors::map_db_error_to_status;
use crate::handlers::order_events::create_order_event;
use crate::handlers::payment_intents::capture_payment;
use crate::handlers::payment_intents::finalize_order_paid;
use crate::handlers::refunds::create_refund;
use crate::handlers::shipments::{ensure_local_order_cancelled, update_cancelability_from_webhook};
use crate::order_state_machine;
use chrono::Utc;
use core_db_entities::entity::sea_orm_active_enums::{PaymentStatus, Status};
use core_db_entities::entity::webhook_events;
use core_db_entities::entity::{orders, payment_intents, refunds};
use proto::proto::core::{
    CapturePaymentRequest, CreateOrderEventRequest, CreateRefundRequest, IngestWebhookRequest,
    WebhookEventResponse, WebhookEventsResponse,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, DatabaseTransaction, DbBackend,
    EntityTrait, IntoActiveModel, QueryFilter, QueryOrder, Statement,
};
use std::time::Instant;
use tonic::{Request, Response, Status as TonicStatus};
use tracing::{info, warn};

use crate::handlers::shipments::apply_shiprocket_scan::{
    apply_shiprocket_scan_to_shipment, extract_scan_from_webhook_item,
    find_shipment_for_shiprocket_event, flatten_shiprocket_webhook_items,
};

pub async fn ingest_webhook(
    txn: &DatabaseTransaction,
    request: Request<IngestWebhookRequest>,
) -> Result<Response<WebhookEventsResponse>, TonicStatus> {
    let started = Instant::now();
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
            "ingest_webhook idempotent replay - returning existing event"
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

    // Process: payment.captured -> trigger capture_payment; shiprocket -> update Shipments from payload.
    let new_status = if req.event_type == "payment.captured" && req.signature_verified {
        match process_payment_captured(txn, &payload_json).await {
            Ok(_) => Status::Processed,
            Err(e) => {
                log::warn!("payment.captured processing failed: {}", e);
                crate::observability::record_webhook_processing_failed_total();
                Status::Failed
            }
        }
    } else if req.provider == "shiprocket" && req.signature_verified {
        match process_shiprocket_shipment_updates(txn, &payload_json, &req.event_type).await {
            Ok(_) => Status::Processed,
            Err(e) => {
                warn!("shiprocket webhook processing failed: {}", e);
                crate::observability::record_webhook_processing_failed_total();
                Status::Failed
            }
        }
    } else if req.provider == "razorpay"
        && req.signature_verified
        && razorpay_event_is_refund_like(&req.event_type)
    {
        match process_razorpay_refund_updates(txn, &payload_json, &req.event_type).await {
            Ok(_) => Status::Processed,
            Err(e) => {
                warn!("razorpay refund webhook processing failed: {}", e);
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

    let duration_sec = started.elapsed().as_secs_f64();
    let outcome = match updated.status {
        Some(Status::Processed) => "processed",
        Some(Status::Failed) => "failed",
        _ => "pending",
    };
    crate::observability::record_webhook_processing_duration_seconds(duration_sec, outcome);

    info!(
        webhook_id = %updated.webhook_id,
        provider = %updated.provider,
        event_type = %updated.event_type,
        status = ?updated.status,
        processing_duration_ms = (duration_sec * 1000.0).round() as i64,
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

    if matches!(intent.status, Status::Failed) {
        warn!(
            payment_intent_id = intent.intent_id,
            webhook_payment_id = %payment_id,
            "payment.captured received for failed/expired intent; marking needs_review"
        );
        txn.execute(Statement::from_sql_and_values(
            DbBackend::MySql,
            "UPDATE PaymentIntents SET status = 'needs_review' WHERE intent_id = ?",
            [intent.intent_id.into()],
        ))
        .await
        .map_err(map_db_error_to_status)?;
        if let Some(order_id) = intent.order_id {
            let _ = create_order_event(
                txn,
                Request::new(CreateOrderEventRequest {
                    order_id,
                    event_type: "late_payment_capture_needs_review".to_string(),
                    from_status: None,
                    to_status: Some("needs_review".to_string()),
                    actor_type: "system".to_string(),
                    message: Some(
                        "Captured payment arrived after system expiry; manual review required"
                            .to_string(),
                    ),
                }),
            )
            .await;
        }
        return Ok(());
    }

    // Phase 5: Verify amount and currency before treating as paid.
    let intent_paise = intent.amount_paise as i64;
    let order = match intent.order_id {
        Some(oid) => Some(
            orders::Entity::find_by_id(oid)
                .one(txn)
                .await
                .map_err(map_db_error_to_status)?
                .ok_or_else(|| {
                    TonicStatus::not_found(format!(
                        "Order {} referenced by payment intent {} was not found",
                        oid, intent.intent_id
                    ))
                })?,
        ),
        None => None,
    };
    let order_grand_paise: Option<i64> = order.as_ref().map(|o| o.grand_total_minor);
    let intent_currency = intent.currency.as_deref().unwrap_or("").to_uppercase();

    if let Some(order_row) = order.as_ref() {
        let status_name = order_state_machine::get_status_name(txn, order_row.status_id)
            .await?
            .unwrap_or_default();
        if status_name.eq_ignore_ascii_case("cancelled")
            || status_name.eq_ignore_ascii_case("refunded")
        {
            warn!(
                order_id = order_row.order_id,
                order_status = %status_name,
                payment_intent_id = intent.intent_id,
                webhook_payment_id = %payment_id,
                "payment.captured arrived for terminal order; marking payment intent needs_review"
            );
            txn.execute(Statement::from_sql_and_values(
                DbBackend::MySql,
                "UPDATE PaymentIntents SET status = 'needs_review' WHERE intent_id = ?",
                [intent.intent_id.into()],
            ))
            .await
            .map_err(map_db_error_to_status)?;
            let _ = create_order_event(
                txn,
                Request::new(CreateOrderEventRequest {
                    order_id: order_row.order_id,
                    event_type: "late_payment_capture_needs_review".to_string(),
                    from_status: Some(status_name),
                    to_status: Some("needs_review".to_string()),
                    actor_type: "system".to_string(),
                    message: Some(
                        "Captured payment arrived after order reached terminal state; manual review required"
                            .to_string(),
                    ),
                }),
            )
            .await;
            return Ok(());
        }
    }

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
            "payment.captured amount/currency mismatch - marking as needs_review"
        );
        let update_result = txn
            .execute(Statement::from_sql_and_values(
                DbBackend::MySql,
                "UPDATE PaymentIntents SET status = 'needs_review' WHERE intent_id = ?",
                [intent.intent_id.into()],
            ))
            .await
            .map_err(map_db_error_to_status)?;
        if update_result.rows_affected() != 1 {
            return Err(TonicStatus::internal(format!(
                "expected to mark exactly one PaymentIntent as needs_review, updated {} rows",
                update_result.rows_affected()
            )));
        }
        if let Some(order_id) = intent.order_id {
            if let Err(e) = order_state_machine::transition_order_status(
                txn,
                order_id,
                order_state_machine::OrderState::NeedsReview,
                "payment_mismatch",
                "system",
                Some("Amount/currency mismatch - needs review"),
                Some(PaymentStatus::NeedsReview),
            )
            .await
            {
                if e.code() != tonic::Code::InvalidArgument {
                    return Err(e);
                }
            }
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
        finalize_order_paid(
            txn,
            order_id,
            "payment_captured",
            "system",
            "Payment captured",
        )
        .await?;
    }

    Ok(())
}

async fn process_shiprocket_shipment_updates(
    txn: &DatabaseTransaction,
    payload: &serde_json::Value,
    event_type: &str,
) -> Result<(), TonicStatus> {
    let items = flatten_shiprocket_webhook_items(payload);
    for item in items {
        let Some(row) = find_shipment_for_shiprocket_event(txn, &item).await? else {
            warn!(
                ?item,
                "shiprocket webhook: no shipment matched awb / shiprocket_order_id"
            );
            continue;
        };
        let order_id = row.order_id;
        info!(
            order_id,
            shipment_id = row.shipment_id,
            awb = ?row.awb_code,
            shiprocket_order_id = ?row.shiprocket_order_id,
            "shiprocket webhook: matched shipment for order"
        );
        let (sid, lbl, scan) = extract_scan_from_webhook_item(&item);
        let cancel_like =
            shiprocket_event_or_payload_indicates_cancelled(event_type, &item, sid, lbl.as_deref());
        if sid.is_none() && scan.is_none() && !cancel_like {
            continue;
        }
        info!(
            order_id,
            shiprocket_status_id = ?sid,
            shiprocket_status_label = ?lbl,
            "shiprocket webhook: applying shipment scan/status update"
        );
        let updated = apply_shiprocket_scan_to_shipment(txn, row, sid, lbl.clone(), scan).await?;
        let updated_status = format!("{:?}", updated.shipment_status).to_lowercase();
        update_cancelability_from_webhook(txn, order_id, sid, Some(updated_status.as_str()))
            .await?;
        if shiprocket_status_indicates_cancelled_or_rto(sid) || cancel_like {
            info!(
                order_id,
                shiprocket_status_id = ?sid,
                event_type,
                cancel_like,
                "shiprocket webhook: cancellation/RTO detected, starting cancel+refund flow"
            );
            auto_transition_order_to_cancelled(txn, order_id).await?;
            cancellation_saga::run_order_settlement(txn, order_id).await?;
        }
        if shiprocket_status_indicates_delivered(sid) {
            auto_transition_order_to_delivered(txn, order_id).await;
        }
        if shiprocket_status_indicates_handover_to_courier(sid) {
            auto_transition_order_to_shipped(txn, order_id).await;
        }
    }
    Ok(())
}

fn shiprocket_status_indicates_handover_to_courier(status_id: Option<i32>) -> bool {
    matches!(
        status_id,
        // Picked up / handover / in-transit and beyond.
        Some(42 | 41 | 45 | 6 | 18 | 17 | 38 | 56)
    )
}

fn shiprocket_status_indicates_cancelled_or_rto(status_id: Option<i32>) -> bool {
    matches!(
        status_id,
        // Cancelled and return-to-origin lifecycle updates.
        Some(8 | 9 | 10 | 14 | 15 | 16)
    )
}

fn shiprocket_status_indicates_delivered(status_id: Option<i32>) -> bool {
    matches!(status_id, Some(7 | 23))
}

fn shiprocket_event_or_payload_indicates_cancelled(
    event_type: &str,
    item: &serde_json::Value,
    status_id: Option<i32>,
    status_label: Option<&str>,
) -> bool {
    if shiprocket_status_indicates_cancelled_or_rto(status_id) {
        return true;
    }
    let e = event_type.trim().to_lowercase();
    if e.contains("cancel") || e.contains("rto") {
        return true;
    }
    let mut hay = String::new();
    for key in [
        "shipment_status",
        "current_status",
        "status",
        "order_status",
        "status_label",
        "remarks",
        "message",
    ] {
        if let Some(v) = item.get(key).and_then(|x| x.as_str()) {
            hay.push(' ');
            hay.push_str(v);
        }
    }
    if let Some(lbl) = status_label {
        hay.push(' ');
        hay.push_str(lbl);
    }
    let h = hay.to_lowercase();
    h.contains("cancel") || h.contains("rto") || h.contains("returned")
}

fn razorpay_event_is_refund_like(event_type: &str) -> bool {
    let e = event_type.trim().to_lowercase();
    e == "refund.processed"
        || e == "refund.failed"
        || e == "refund.created"
        || e == "payment.refunded"
}

async fn process_razorpay_refund_updates(
    txn: &DatabaseTransaction,
    payload: &serde_json::Value,
    event_type: &str,
) -> Result<(), TonicStatus> {
    let refund_entity = payload
        .get("payload")
        .and_then(|p| p.get("refund"))
        .and_then(|r| r.get("entity"));

    let payment_entity = payload
        .get("payload")
        .and_then(|p| p.get("payment"))
        .and_then(|r| r.get("entity"));

    let payment_id = refund_entity
        .and_then(|x| x.get("payment_id"))
        .and_then(|x| x.as_str())
        .or_else(|| {
            payment_entity
                .and_then(|x| x.get("id"))
                .and_then(|x| x.as_str())
        })
        .unwrap_or("")
        .trim()
        .to_string();

    if payment_id.is_empty() {
        warn!(event_type, "razorpay refund webhook missing payment id");
        return Ok(());
    }

    let gateway_refund_id = refund_entity
        .and_then(|x| x.get("id"))
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .trim()
        .to_string();

    if gateway_refund_id.is_empty() {
        warn!(
            payment_id = %payment_id,
            event_type,
            "razorpay refund webhook missing refund id"
        );
        return Ok(());
    }

    let amount_paise = refund_entity
        .and_then(|x| x.get("amount"))
        .and_then(|x| x.as_i64())
        .unwrap_or(0)
        .max(0);
    if amount_paise <= 0 {
        warn!(
            payment_id = %payment_id,
            gateway_refund_id = %gateway_refund_id,
            event_type,
            "razorpay refund webhook has non-positive refund amount"
        );
        return Ok(());
    }

    let currency = refund_entity
        .and_then(|x| x.get("currency"))
        .and_then(|x| x.as_str())
        .map(|s| s.trim().to_uppercase())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "INR".to_string());

    let status_hint = refund_entity
        .and_then(|x| x.get("status"))
        .and_then(|x| x.as_str())
        .map(|s| s.trim().to_lowercase())
        .unwrap_or_else(|| event_type.trim().to_lowercase());
    let refund_status = map_refund_status(&status_hint);
    info!(
        event_type,
        payment_id = %payment_id,
        gateway_refund_id = %gateway_refund_id,
        amount_paise,
        currency = %currency,
        mapped_refund_status = ?refund_status,
        "razorpay refund webhook parsed"
    );

    let Some(intent) = payment_intents::Entity::find()
        .filter(payment_intents::Column::RazorpayPaymentId.eq(&payment_id))
        .order_by_desc(payment_intents::Column::IntentId)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
    else {
        warn!(
            payment_id = %payment_id,
            gateway_refund_id = %gateway_refund_id,
            "razorpay refund webhook: no payment intent found"
        );
        return Ok(());
    };

    let Some(order_id) = intent.order_id else {
        warn!(
            payment_id = %payment_id,
            gateway_refund_id = %gateway_refund_id,
            "razorpay refund webhook: payment intent has no order_id"
        );
        return Ok(());
    };
    info!(
        order_id,
        payment_intent_id = intent.intent_id,
        payment_id = %payment_id,
        gateway_refund_id = %gateway_refund_id,
        "razorpay refund webhook resolved order"
    );

    if refund_status == Status::Processed {
        // Reuse existing idempotent refund handler for persistence + auto transition to refunded.
        create_refund(
            txn,
            Request::new(CreateRefundRequest {
                order_id,
                gateway_refund_id: gateway_refund_id.clone(),
                amount_paise,
                currency: Some(currency),
                line_items_refunded_json: None,
            }),
        )
        .await?;
        info!(
            order_id,
            gateway_refund_id = %gateway_refund_id,
            amount_paise,
            "razorpay refund processed: refund recorded and order updated"
        );
        return Ok(());
    }

    // For pending/failed updates, upsert Refunds row and attach timeline events.
    upsert_refund_row(
        txn,
        order_id,
        &gateway_refund_id,
        amount_paise,
        &currency,
        refund_status.clone(),
    )
    .await?;

    let (evt, msg) = if refund_status == Status::Failed {
        ("refund_failed", "Refund failed at payment gateway")
    } else {
        (
            "refund_initiated",
            "Refund initiated; awaiting gateway confirmation",
        )
    };
    let _ = create_order_event(
        txn,
        Request::new(CreateOrderEventRequest {
            order_id,
            event_type: evt.to_string(),
            from_status: None,
            to_status: None,
            actor_type: "system".to_string(),
            message: Some(msg.to_string()),
        }),
    )
    .await;
    info!(
        order_id,
        gateway_refund_id = %gateway_refund_id,
        refund_status = ?refund_status,
        "razorpay refund update recorded on order timeline"
    );

    Ok(())
}

fn map_refund_status(raw: &str) -> Status {
    let s = raw.trim().to_lowercase();
    if s.contains("fail") {
        Status::Failed
    } else if s.contains("process") || s == "refund.processed" || s == "payment.refunded" {
        Status::Processed
    } else {
        Status::Pending
    }
}

async fn upsert_refund_row(
    txn: &DatabaseTransaction,
    order_id: i64,
    gateway_refund_id: &str,
    amount_paise: i64,
    currency: &str,
    status: Status,
) -> Result<(), TonicStatus> {
    if let Some(existing) = refunds::Entity::find()
        .filter(refunds::Column::GatewayRefundId.eq(gateway_refund_id))
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
    {
        let mut active = existing.into_active_model();
        active.status = ActiveValue::Set(Some(status));
        active.update(txn).await.map_err(map_db_error_to_status)?;
        return Ok(());
    }

    let row = refunds::ActiveModel {
        refund_id: ActiveValue::NotSet,
        order_id: ActiveValue::Set(order_id),
        gateway_refund_id: ActiveValue::Set(gateway_refund_id.to_string()),
        amount_paise: ActiveValue::Set((amount_paise.max(0).min(i32::MAX as i64)) as i32),
        currency: ActiveValue::Set(Some(currency.to_string())),
        status: ActiveValue::Set(Some(status)),
        line_items_refunded: ActiveValue::Set(None),
        created_at: ActiveValue::Set(Some(Utc::now())),
    };
    row.insert(txn).await.map_err(map_db_error_to_status)?;
    Ok(())
}

async fn auto_transition_order_to_cancelled(
    txn: &DatabaseTransaction,
    order_id: i64,
) -> Result<(), TonicStatus> {
    if let Err(e) = ensure_local_order_cancelled(txn, order_id).await {
        if e.code() == tonic::Code::InvalidArgument {
            // Idempotent duplicate: already cancelled / transition no longer needed.
            return Ok(());
        }
        warn!(
            order_id,
            error = %e,
            "shiprocket webhook: failed to auto-transition order to cancelled"
        );
        return Err(e);
    }
    info!(
        order_id,
        "shiprocket webhook: order auto-transitioned to cancelled"
    );
    Ok(())
}

async fn auto_transition_order_to_shipped(txn: &DatabaseTransaction, order_id: i64) {
    // Ensure flow compatibility when order is still "confirmed" (paid):
    // confirmed -> processing -> shipped
    if let Err(e) = order_state_machine::transition_order_status(
        txn,
        order_id,
        order_state_machine::OrderState::Processing,
        "shiprocket_handover",
        "system",
        Some("Shipment handed to courier"),
        None,
    )
    .await
    {
        if e.code() != tonic::Code::InvalidArgument {
            warn!(
                order_id,
                error = %e,
                "shiprocket webhook: failed to auto-transition order to processing"
            );
        }
    }

    if let Err(e) = order_state_machine::transition_order_status(
        txn,
        order_id,
        order_state_machine::OrderState::Shipped,
        "shiprocket_handover",
        "system",
        Some("Shipment handed to courier"),
        None,
    )
    .await
    {
        if e.code() != tonic::Code::InvalidArgument {
            warn!(
                order_id,
                error = %e,
                "shiprocket webhook: failed to auto-transition order to shipped"
            );
        }
    }
}

async fn auto_transition_order_to_delivered(txn: &DatabaseTransaction, order_id: i64) {
    // Ensure final transition remains valid when current state is still processing:
    // processing -> shipped -> delivered
    auto_transition_order_to_shipped(txn, order_id).await;

    if let Err(e) = order_state_machine::transition_order_status(
        txn,
        order_id,
        order_state_machine::OrderState::Delivered,
        "shiprocket_delivered",
        "system",
        Some("Shipment marked delivered by courier"),
        None,
    )
    .await
    {
        if e.code() != tonic::Code::InvalidArgument {
            warn!(
                order_id,
                error = %e,
                "shiprocket webhook: failed to auto-transition order to delivered"
            );
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn helper_detects_refund_events() {
        assert!(razorpay_event_is_refund_like("refund.processed"));
        assert!(razorpay_event_is_refund_like("refund.failed"));
        assert!(razorpay_event_is_refund_like("payment.refunded"));
        assert!(!razorpay_event_is_refund_like("payment.captured"));
    }

    #[test]
    fn helper_detects_cancel_from_event_or_payload_without_status_id() {
        let item = json!({
            "order_status": "Cancelled by seller",
            "message": "Shipment cancellation accepted"
        });
        assert!(shiprocket_event_or_payload_indicates_cancelled(
            "shiprocket.order.cancelled",
            &item,
            None,
            None
        ));
        assert!(shiprocket_event_or_payload_indicates_cancelled(
            "shiprocket.update",
            &item,
            None,
            None
        ));
    }

    #[test]
    fn helper_refund_status_mapping_handles_pending_processed_failed() {
        assert_eq!(map_refund_status("pending"), Status::Pending);
        assert_eq!(map_refund_status("processed"), Status::Processed);
        assert_eq!(map_refund_status("refund.processed"), Status::Processed);
        assert_eq!(map_refund_status("failed"), Status::Failed);
    }
}
