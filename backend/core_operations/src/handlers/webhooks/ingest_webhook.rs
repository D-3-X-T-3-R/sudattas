use crate::cancellation_saga;
use crate::handlers::db_errors::map_db_error_to_status;

use crate::handlers::shipments::{ensure_local_order_cancelled, update_cancelability_from_webhook};
use crate::order_state_machine;
use chrono::{DateTime, Duration, Utc};
use core_db_entities::entity::payment_intents;
use core_db_entities::entity::sea_orm_active_enums::Status;
use core_db_entities::entity::webhook_events;
use hmac::{Hmac, Mac};
use proto::proto::core::{IngestWebhookRequest, WebhookEventResponse, WebhookEventsResponse};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, DatabaseTransaction, DbBackend,
    EntityTrait, IntoActiveModel, QueryFilter, QuerySelect, Statement, TransactionTrait,
};
use sha2::Sha256;
use std::time::Instant;
use subtle::ConstantTimeEq;
use tonic::{Request, Response, Status as TonicStatus};
use tracing::{info, warn};

async fn persist_verified_body(
    txn: &DatabaseTransaction,
    event: &webhook_events::Model,
    req: &IngestWebhookRequest,
) -> Result<(), TonicStatus> {
    if !event_identity_matches(event, req) {
        return Err(TonicStatus::already_exists("Webhook identity conflict"));
    }
    let hash = crate::handlers::idempotency::compute_request_hash(&req.payload_json);
    txn.execute(Statement::from_sql_and_values(DbBackend::MySql,
        "INSERT INTO WebhookVerificationV2(event_id,raw_body,body_sha256,verified_at) VALUES (?,?,?,UTC_TIMESTAMP(6)) ON DUPLICATE KEY UPDATE event_id=event_id",
        [event.event_id.into(),req.payload_json.clone().into(),hash.clone().into()])).await.map_err(map_db_error_to_status)?;
    let row = txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::MySql,
            "SELECT body_sha256 FROM WebhookVerificationV2 WHERE event_id=? FOR UPDATE",
            [event.event_id.into()],
        ))
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| TonicStatus::internal("Webhook provenance missing"))?;
    let saved: String = row
        .try_get("", "body_sha256")
        .map_err(|e| TonicStatus::internal(e.to_string()))?;
    if saved != hash {
        return Err(TonicStatus::already_exists(
            "Webhook ID reused with different raw body",
        ));
    }
    Ok(())
}

async fn apply_atomic(
    txn: &DatabaseTransaction,
    req: &IngestWebhookRequest,
    payload: &serde_json::Value,
    verified: bool,
    event_id: i64,
) -> Result<Status, TonicStatus> {
    if !verified {
        return Err(TonicStatus::permission_denied("Unverified webhook"));
    }
    let row = txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::MySql,
            "SELECT attempts,exhausted FROM WebhookVerificationV2 WHERE event_id=? FOR UPDATE",
            [event_id.into()],
        ))
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| TonicStatus::permission_denied("No verification provenance"))?;
    let attempts: i32 = row
        .try_get("", "attempts")
        .map_err(|e| TonicStatus::internal(e.to_string()))?;
    if attempts >= 5 {
        txn.execute(Statement::from_sql_and_values(DbBackend::MySql,
            "UPDATE WebhookVerificationV2 SET exhausted=TRUE,last_error='retry budget exhausted; operator review required' WHERE event_id=?",
            [event_id.into()])).await.map_err(map_db_error_to_status)?;
        tracing::error!(
            event_id,
            "webhook retry exhausted; operator reconciliation required"
        );
        return Ok(Status::NeedsReview);
    }
    txn.execute(Statement::from_sql_and_values(DbBackend::MySql,
        "UPDATE WebhookVerificationV2 SET attempts=attempts+1,next_attempt_at=DATE_ADD(UTC_TIMESTAMP(6),INTERVAL 60 SECOND) WHERE event_id=?",
        [event_id.into()])).await.map_err(map_db_error_to_status)?;
    // Inbox/provenance are outside this savepoint. A failed business operation
    // cannot commit its intent/order/invoice/ledger writes with a Failed inbox row.
    let business = txn.begin().await.map_err(map_db_error_to_status)?;
    let status = process_webhook_payload(&business, req, payload, true).await;
    if status == Status::Processed {
        business.commit().await.map_err(map_db_error_to_status)?;
    } else {
        business.rollback().await.map_err(map_db_error_to_status)?;
    }
    Ok(status)
}

type HmacSha256 = Hmac<Sha256>;

use crate::handlers::shipments::apply_shiprocket_scan::{
    apply_shiprocket_scan_to_shipment, extract_scan_from_webhook_item,
    find_shipment_for_shiprocket_event, flatten_shiprocket_webhook_items,
};

fn normalize_provider_event_id(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn parse_payload_or_null(payload_json: &str) -> serde_json::Value {
    serde_json::from_str(payload_json).unwrap_or(serde_json::Value::Null)
}

fn processing_status_outcome(status: Option<&Status>) -> &'static str {
    match status {
        Some(Status::Processed) => "processed",
        Some(Status::Failed) => "failed",
        Some(Status::ClientVerified) => "in_progress",
        _ => "pending",
    }
}

fn status_canonical_value(status: &Status) -> &'static str {
    match status {
        Status::Pending => "pending",
        Status::Processed => "processed",
        Status::Failed => "failed",
        Status::NeedsReview => "needs_review",
        Status::ClientVerified => "client_verified",
    }
}

fn is_fresh_in_progress(
    status: Option<&Status>,
    received_at: Option<DateTime<Utc>>,
    stale_before: DateTime<Utc>,
) -> bool {
    matches!(
        status,
        Some(s) if *s == Status::Pending || *s == Status::ClientVerified
    ) && received_at.map(|ts| ts >= stale_before).unwrap_or(false)
}

fn is_stale_in_progress(
    status: Option<&Status>,
    received_at: Option<DateTime<Utc>>,
    stale_before: DateTime<Utc>,
) -> bool {
    matches!(
        status,
        Some(s) if *s == Status::Pending || *s == Status::ClientVerified
    ) && received_at.map(|ts| ts < stale_before).unwrap_or(true)
}

fn is_replayable_failure(status: Option<&Status>) -> bool {
    matches!(
        status,
        Some(s) if *s == Status::Failed || *s == Status::NeedsReview
    )
}

fn verify_razorpay_signature(payload_json: &str, signature: &str) -> bool {
    let secret = match std::env::var("RAZORPAY_WEBHOOK_SECRET") {
        Ok(s) if !s.trim().is_empty() => s,
        _ => return false,
    };
    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };
    mac.update(payload_json.as_bytes());
    let computed = hex::encode(mac.finalize().into_bytes());
    let computed_bytes = computed.as_bytes();
    let signature_bytes = signature.as_bytes();
    computed_bytes.len() == signature_bytes.len() && computed_bytes.ct_eq(signature_bytes).into()
}

fn verify_shiprocket_token(token: &str) -> bool {
    match std::env::var("SHIPROCKET_WEBHOOK_SECRET") {
        Ok(secret) if !secret.trim().is_empty() => {
            let expected = secret.trim().as_bytes();
            let provided = token.as_bytes();
            expected.len() == provided.len() && expected.ct_eq(provided).into()
        }
        _ => false,
    }
}

/// Independently re-derives signature validity from the raw signature/token material forwarded by
/// the HTTP layer, instead of trusting the caller-supplied `signature_verified` flag. Closes the
/// defense-in-depth gap where a misconfigured/bypassed gRPC auth layer could otherwise forge
/// "verified" webhooks. Returns `None` when no raw signature material was forwarded (e.g. internal
/// admin-triggered replay of an already-ingested event), in which case the caller-supplied flag is
/// used as-is.
fn independently_verify_signature(req: &IngestWebhookRequest) -> Option<bool> {
    let raw_signature = req
        .raw_signature
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())?;
    match req.provider.as_str() {
        "razorpay" => Some(verify_razorpay_signature(&req.payload_json, raw_signature)),
        "shiprocket" => Some(verify_shiprocket_token(raw_signature)),
        _ => None,
    }
}

/// `trust_flag_if_no_raw_signature`: true only for internal replay of an already-verified,
/// already-ingested event (replay_webhook_by_id, which constructs its own request and
/// intentionally omits raw_signature). `ingest_webhook` must pass false: its real caller (the
/// HTTP webhook layer) always forwards raw_signature now, so a request that somehow lacks it
/// should be treated as unverified rather than trusted.
fn resolve_effective_verified(
    req: &IngestWebhookRequest,
    trust_flag_if_no_raw_signature: bool,
) -> bool {
    match independently_verify_signature(req) {
        Some(verified) => {
            if verified != req.signature_verified {
                warn!(
                    provider = %req.provider,
                    event_type = %req.event_type,
                    webhook_id = %req.webhook_id,
                    caller_claimed = req.signature_verified,
                    independently_verified = verified,
                    "ingest_webhook: caller-supplied signature_verified disagreed with server-side re-verification"
                );
            }
            verified
        }
        None if trust_flag_if_no_raw_signature => req.signature_verified,
        None => false,
    }
}

fn event_identity_matches(existing: &webhook_events::Model, req: &IngestWebhookRequest) -> bool {
    existing.provider.eq_ignore_ascii_case(&req.provider)
        && existing.event_type.eq_ignore_ascii_case(&req.event_type)
}

async fn claim_existing_webhook_for_replay(
    txn: &DatabaseTransaction,
    event_id: i64,
    reclaim_timeout_minutes: i64,
) -> Result<bool, TonicStatus> {
    let claim = txn
        .execute(Statement::from_sql_and_values(
            DbBackend::MySql,
            r#"UPDATE WebhookEvents
               SET status = 'client_verified',
                   received_at = UTC_TIMESTAMP()
               WHERE event_id = ?
                 AND (
                     status IN ('failed', 'needs_review')
                     OR (
                         status IN ('pending', 'client_verified')
                         AND (
                             received_at IS NULL
                             OR received_at < DATE_SUB(UTC_TIMESTAMP(), INTERVAL ? MINUTE)
                         )
                     )
                 )"#,
            [event_id.into(), reclaim_timeout_minutes.into()],
        ))
        .await
        .map_err(map_db_error_to_status)?;
    Ok(claim.rows_affected() == 1)
}

fn choose_replay_payload(
    existing: &webhook_events::Model,
    incoming_payload: &serde_json::Value,
) -> serde_json::Value {
    if !incoming_payload.is_null() {
        incoming_payload.clone()
    } else {
        existing.payload.clone()
    }
}

async fn process_webhook_payload(
    txn: &DatabaseTransaction,
    req: &IngestWebhookRequest,
    payload_json: &serde_json::Value,
    verified: bool,
) -> Status {
    if req.event_type == "payment.captured" && verified {
        match process_payment_captured(txn, payload_json).await {
            Ok(_) => Status::Processed,
            Err(e) => {
                warn!("payment.captured processing failed: {}", e);
                crate::observability::record_webhook_processing_failed_total();
                Status::Failed
            }
        }
    } else if req.provider == "shiprocket" && verified {
        match process_shiprocket_shipment_updates(txn, payload_json, &req.event_type).await {
            Ok(_) => Status::Processed,
            Err(e) => {
                warn!("shiprocket webhook processing failed: {}", e);
                crate::observability::record_webhook_processing_failed_total();
                Status::Failed
            }
        }
    } else if req.provider == "razorpay"
        && verified
        && razorpay_event_is_refund_like(&req.event_type)
    {
        match process_razorpay_refund_updates(txn, payload_json, &req.event_type).await {
            Ok(_) => Status::Processed,
            Err(e) => {
                warn!("razorpay refund webhook processing failed: {}", e);
                crate::observability::record_webhook_processing_failed_total();
                Status::Failed
            }
        }
    } else if verified {
        Status::Processed
    } else {
        Status::Failed
    }
}

async fn finalize_webhook_row(
    txn: &DatabaseTransaction,
    row: webhook_events::Model,
    payload_json: &serde_json::Value,
    status: Status,
) -> Result<webhook_events::Model, TonicStatus> {
    let mut active = row.into_active_model();
    active.payload = ActiveValue::Set(payload_json.clone());
    active.status = ActiveValue::Set(Some(status));
    active.update(txn).await.map_err(map_db_error_to_status)
}

/// Window controlling how long an in-progress webhook row is treated as "still being worked on"
/// before it's eligible for reclaim/replay. `reclaim_timeout_minutes` and `stale_before` are
/// always computed together (`stale_before = now - reclaim_timeout_minutes`) and passed together
/// at every call site, so they're grouped here to keep `handle_existing_webhook`'s argument count
/// under clippy's `too_many_arguments` threshold.
struct ReplayWindow {
    reclaim_timeout_minutes: i64,
    stale_before: DateTime<Utc>,
}

async fn handle_existing_webhook(
    txn: &DatabaseTransaction,
    req: &IngestWebhookRequest,
    existing: webhook_events::Model,
    incoming_payload: &serde_json::Value,
    window: ReplayWindow,
    dedupe_key: &str,
    verified: bool,
) -> Result<webhook_events::Model, TonicStatus> {
    if matches!(existing.status, Some(Status::Processed)) {
        info!(
            webhook_id = %existing.webhook_id,
            provider_event_id = ?existing.provider_event_id,
            dedupe_key,
            "webhook duplicate already processed; returning idempotent no-op"
        );
        return Ok(existing);
    }

    if is_fresh_in_progress(
        existing.status.as_ref(),
        existing.received_at,
        window.stale_before,
    ) {
        info!(
            webhook_id = %existing.webhook_id,
            provider_event_id = ?existing.provider_event_id,
            dedupe_key,
            "webhook duplicate currently in progress and fresh; returning existing row without reprocessing"
        );
        return Ok(existing);
    }

    if !event_identity_matches(&existing, req) {
        return Err(TonicStatus::already_exists(format!(
            "Webhook conflict for dedupe key {}: existing provider/event_type = {}/{}",
            dedupe_key, existing.provider, existing.event_type
        )));
    }

    if is_stale_in_progress(
        existing.status.as_ref(),
        existing.received_at,
        window.stale_before,
    ) {
        let age_seconds = existing
            .received_at
            .map(|ts| (Utc::now() - ts).num_seconds())
            .unwrap_or(-1);
        warn!(
            webhook_id = %existing.webhook_id,
            provider_event_id = ?existing.provider_event_id,
            dedupe_key,
            reclaim_timeout_minutes = window.reclaim_timeout_minutes,
            age_seconds,
            status = ?existing.status.as_ref(),
            "webhook stale in-progress row reclaimed for replay"
        );
    } else if is_replayable_failure(existing.status.as_ref()) {
        info!(
            webhook_id = %existing.webhook_id,
            provider_event_id = ?existing.provider_event_id,
            dedupe_key,
            status = ?existing.status.as_ref(),
            "webhook failed row replay requested"
        );
    } else {
        return Ok(existing);
    }

    let claimed =
        claim_existing_webhook_for_replay(txn, existing.event_id, window.reclaim_timeout_minutes)
            .await?;
    if !claimed {
        let latest = webhook_events::Entity::find_by_id(existing.event_id)
            .one(txn)
            .await
            .map_err(map_db_error_to_status)?
            .ok_or_else(|| {
                TonicStatus::internal(format!(
                    "webhook event {} disappeared during replay claim",
                    existing.event_id
                ))
            })?;
        return Ok(latest);
    }

    let replay_payload = choose_replay_payload(&existing, incoming_payload);
    let replay_status =
        apply_atomic(txn, req, &replay_payload, verified, existing.event_id).await?;
    finalize_webhook_row(txn, existing, &replay_payload, replay_status).await
}

pub async fn replay_webhook_by_id(
    txn: &DatabaseTransaction,
    webhook_id: &str,
) -> Result<Response<WebhookEventsResponse>, TonicStatus> {
    let mut rows = webhook_events::Entity::find()
        .filter(webhook_events::Column::WebhookId.eq(webhook_id))
        .lock(sea_orm::sea_query::LockType::Update)
        .all(txn)
        .await
        .map_err(map_db_error_to_status)?;
    if rows.len() != 1 {
        return Err(TonicStatus::failed_precondition(
            "Webhook ID missing or ambiguous; use internal event ID",
        ));
    }
    replay_verified_row(txn, rows.remove(0)).await
}

pub async fn replay_verified_event_by_id(
    txn: &DatabaseTransaction,
    event_id: i64,
) -> Result<Response<WebhookEventsResponse>, TonicStatus> {
    let existing = webhook_events::Entity::find_by_id(event_id)
        .lock(sea_orm::sea_query::LockType::Update)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| TonicStatus::not_found("Webhook event missing"))?;
    replay_verified_row(txn, existing).await
}

async fn replay_verified_row(
    txn: &DatabaseTransaction,
    existing: webhook_events::Model,
) -> Result<Response<WebhookEventsResponse>, TonicStatus> {
    let proof = txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::MySql,
            "SELECT raw_body,body_sha256 FROM WebhookVerificationV2 WHERE event_id=? FOR UPDATE",
            [existing.event_id.into()],
        ))
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| {
            TonicStatus::permission_denied(
                "Legacy or unverified event cannot be replayed; obtain an authenticated redelivery",
            )
        })?;
    let raw: String = proof
        .try_get("", "raw_body")
        .map_err(|e| TonicStatus::internal(e.to_string()))?;
    let hash: String = proof
        .try_get("", "body_sha256")
        .map_err(|e| TonicStatus::internal(e.to_string()))?;
    if crate::handlers::idempotency::compute_request_hash(&raw) != hash {
        return Err(TonicStatus::permission_denied(
            "Webhook provenance corrupted",
        ));
    }
    let req = IngestWebhookRequest {
        provider: existing.provider.clone(),
        event_type: existing.event_type.clone(),
        webhook_id: existing.webhook_id.clone(),
        payload_json: raw,
        signature_verified: true,
        provider_event_id: existing.provider_event_id.clone(),
        raw_signature: None,
    };
    let reclaim_timeout_minutes = crate::order_policy::webhook_reclaim_timeout_minutes();
    let stale_before = Utc::now() - Duration::minutes(reclaim_timeout_minutes);
    let verified = resolve_effective_verified(&req, true);
    let updated = handle_existing_webhook(
        txn,
        &req,
        existing,
        &parse_payload_or_null(&req.payload_json),
        ReplayWindow {
            reclaim_timeout_minutes,
            stale_before,
        },
        "webhook_id",
        verified,
    )
    .await?;
    Ok(Response::new(WebhookEventsResponse {
        items: vec![model_to_response(updated)],
    }))
}

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

    let reclaim_timeout_minutes = crate::order_policy::webhook_reclaim_timeout_minutes();
    let stale_before = Utc::now() - Duration::minutes(reclaim_timeout_minutes);
    let payload_json = parse_payload_or_null(&req.payload_json);
    let verified = resolve_effective_verified(&req, false);
    if !verified {
        return Err(TonicStatus::permission_denied("Webhook signature rejected"));
    }
    if payload_json.is_null() {
        return Err(TonicStatus::invalid_argument("Invalid webhook JSON"));
    }
    if req.provider == "razorpay" && payload_json["event"].as_str() != Some(req.event_type.as_str())
    {
        return Err(TonicStatus::invalid_argument(
            "Webhook event type does not match signed body",
        ));
    }

    // Idempotency + replay by webhook_id.
    if let Some(existing) = webhook_events::Entity::find()
        .filter(webhook_events::Column::Provider.eq(&req.provider))
        .filter(webhook_events::Column::WebhookId.eq(&req.webhook_id))
        .lock(sea_orm::sea_query::LockType::Update)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
    {
        persist_verified_body(txn, &existing, &req).await?;
        let updated = handle_existing_webhook(
            txn,
            &req,
            existing,
            &payload_json,
            ReplayWindow {
                reclaim_timeout_minutes,
                stale_before,
            },
            "webhook_id",
            verified,
        )
        .await?;
        let duration_sec = started.elapsed().as_secs_f64();
        crate::observability::record_webhook_processing_duration_seconds(
            duration_sec,
            processing_status_outcome(updated.status.as_ref()),
        );
        return Ok(Response::new(WebhookEventsResponse {
            items: vec![model_to_response(updated)],
        }));
    }

    // Replay protection + safe reprocessing by provider_event_id.
    if let Some(peid) = normalize_provider_event_id(req.provider_event_id.as_deref()) {
        if let Some(existing) = webhook_events::Entity::find()
            .filter(webhook_events::Column::ProviderEventId.eq(peid.as_str()))
            .lock(sea_orm::sea_query::LockType::Update)
            .one(txn)
            .await
            .map_err(map_db_error_to_status)?
        {
            persist_verified_body(txn, &existing, &req).await?;
            let updated = handle_existing_webhook(
                txn,
                &req,
                existing,
                &payload_json,
                ReplayWindow {
                    reclaim_timeout_minutes,
                    stale_before,
                },
                "provider_event_id",
                verified,
            )
            .await?;
            let duration_sec = started.elapsed().as_secs_f64();
            crate::observability::record_webhook_processing_duration_seconds(
                duration_sec,
                processing_status_outcome(updated.status.as_ref()),
            );
            return Ok(Response::new(WebhookEventsResponse {
                items: vec![model_to_response(updated)],
            }));
        }
    }

    // Persist with Pending status.
    let event = webhook_events::ActiveModel {
        event_id: ActiveValue::NotSet,
        provider: ActiveValue::Set(req.provider.clone()),
        event_type: ActiveValue::Set(req.event_type.clone()),
        webhook_id: ActiveValue::Set(req.webhook_id.clone()),
        provider_event_id: ActiveValue::Set(normalize_provider_event_id(
            req.provider_event_id.as_deref(),
        )),
        payload: ActiveValue::Set(payload_json.clone()),
        status: ActiveValue::Set(Some(Status::Pending)),
        received_at: ActiveValue::Set(Some(Utc::now())),
    };

    let inserted = event.insert(txn).await.map_err(map_db_error_to_status)?;

    persist_verified_body(txn, &inserted, &req).await?;
    let new_status = apply_atomic(txn, &req, &payload_json, verified, inserted.event_id).await?;
    let updated = finalize_webhook_row(txn, inserted, &payload_json, new_status).await?;

    let duration_sec = started.elapsed().as_secs_f64();
    crate::observability::record_webhook_processing_duration_seconds(
        duration_sec,
        processing_status_outcome(updated.status.as_ref()),
    );

    info!(
        webhook_id = %updated.webhook_id,
        provider = %updated.provider,
        event_type = %updated.event_type,
        status = ?updated.status.as_ref(),
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
    if entity["status"].as_str() != Some("captured") || entity["captured"].as_bool() != Some(true) {
        return Err(TonicStatus::failed_precondition(
            "Captured payment evidence is incomplete or contradictory",
        ));
    }
    crate::handlers::payment_intents::capture_evidence::apply_captured_evidence(
        txn,
        entity["order_id"].as_str().unwrap_or(""),
        entity["id"].as_str().unwrap_or(""),
        entity["amount"].as_i64().unwrap_or(0),
        entity["currency"].as_str().unwrap_or(""),
    )
    .await
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
    let r = &payload["payload"]["refund"]["entity"];
    let payment = r["payment_id"]
        .as_str()
        .ok_or_else(|| TonicStatus::failed_precondition("Refund has no payment association"))?;
    let id = r["id"]
        .as_str()
        .ok_or_else(|| TonicStatus::failed_precondition("Refund ID missing"))?;
    let amount = r["amount"]
        .as_i64()
        .ok_or_else(|| TonicStatus::failed_precondition("Refund amount missing"))?;
    let currency = r["currency"]
        .as_str()
        .ok_or_else(|| TonicStatus::failed_precondition("Refund currency missing"))?;
    let status = r["status"].as_str().unwrap_or(match event_type {
        "refund.processed" => "processed",
        "refund.failed" => "failed",
        "refund.created" => "pending",
        _ => "unknown",
    });
    let intent = payment_intents::Entity::find()
        .filter(payment_intents::Column::RazorpayPaymentId.eq(payment))
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| {
            TonicStatus::failed_precondition("Refund payment missing; reconciliation required")
        })?;
    let order = intent
        .order_id
        .ok_or_else(|| TonicStatus::failed_precondition("Refund internal order missing"))?;
    crate::handlers::refunds::evidence::apply(txn, order, payment, id, amount, currency, status)
        .await?;
    Ok(())
}
#[cfg(test)]
fn map_refund_status(raw: &str) -> Status {
    match raw.trim().to_ascii_lowercase().as_str() {
        "processed" | "refund.processed" | "payment.refunded" => Status::Processed,
        "failed" | "refund.failed" => Status::Failed,
        "pending" | "refund.created" => Status::Pending,
        _ => Status::NeedsReview,
    }
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
            .as_ref()
            .map(status_canonical_value)
            .map(str::to_string)
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

    #[test]
    fn helper_classifies_fresh_vs_stale_in_progress_rows() {
        let now = Utc::now();
        let stale_before = now - Duration::minutes(12);
        assert!(is_fresh_in_progress(
            Some(&Status::Pending),
            Some(now - Duration::minutes(1)),
            stale_before
        ));
        assert!(!is_stale_in_progress(
            Some(&Status::Pending),
            Some(now - Duration::minutes(1)),
            stale_before
        ));

        assert!(is_stale_in_progress(
            Some(&Status::ClientVerified),
            Some(now - Duration::minutes(30)),
            stale_before
        ));
        assert!(!is_fresh_in_progress(
            Some(&Status::ClientVerified),
            Some(now - Duration::minutes(30)),
            stale_before
        ));
    }

    #[test]
    fn helper_detects_replayable_failure_statuses() {
        assert!(is_replayable_failure(Some(&Status::Failed)));
        assert!(is_replayable_failure(Some(&Status::NeedsReview)));
        assert!(!is_replayable_failure(Some(&Status::Processed)));
        assert!(!is_replayable_failure(Some(&Status::Pending)));
    }

    #[test]
    fn helper_processing_outcome_labels() {
        assert_eq!(
            processing_status_outcome(Some(&Status::Processed)),
            "processed"
        );
        assert_eq!(processing_status_outcome(Some(&Status::Failed)), "failed");
        assert_eq!(
            processing_status_outcome(Some(&Status::ClientVerified)),
            "in_progress"
        );
        assert_eq!(processing_status_outcome(Some(&Status::Pending)), "pending");
    }

    #[test]
    fn helper_model_to_response_uses_canonical_status_values() {
        let model = webhook_events::Model {
            event_id: 1,
            provider: "razorpay".to_string(),
            event_type: "noop.event".to_string(),
            webhook_id: "wh_1".to_string(),
            provider_event_id: None,
            payload: serde_json::json!({}),
            status: Some(Status::ClientVerified),
            received_at: None,
        };

        let response = model_to_response(model);
        assert_eq!(response.status, "client_verified");
    }
}
