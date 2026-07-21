use hmac::{Hmac, Mac};
use proto::proto::core::IngestWebhookRequest;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use tracing::{info, warn};
use warp::hyper::body::Bytes;
use warp::reply::{self, Reply};

use crate::resolvers::utils::connect_grpc_client;

type HmacSha256 = Hmac<Sha256>;

/// Verify Razorpay HMAC-SHA256 signature using the raw body and webhook secret.
fn verify_razorpay_signature(body: &[u8], signature: &str) -> bool {
    let secret = match std::env::var("RAZORPAY_WEBHOOK_SECRET") {
        Ok(s) => s,
        Err(_) => return false,
    };
    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };
    mac.update(body);
    let computed = hex::encode(mac.finalize().into_bytes());
    let computed_bytes = computed.as_bytes();
    let signature_bytes = signature.as_bytes();
    computed_bytes.len() == signature_bytes.len() && computed_bytes.ct_eq(signature_bytes).into()
}

/// When RAZORPAY_WEBHOOK_SECRET is set and provider is razorpay, signature is required and must be valid.
/// Returns Err((status, message)) to reject at HTTP boundary without calling gRPC.
/// Optional shared secret: send header `X-Shiprocket-Token: <secret>` (or configure URL-only in dev).
fn verify_shiprocket_webhook_token(token: Option<&str>) -> bool {
    match std::env::var("SHIPROCKET_WEBHOOK_SECRET") {
        Ok(secret) if !secret.trim().is_empty() => {
            let expected = secret.trim().as_bytes();
            let provided = token.map(str::trim).unwrap_or("").as_bytes();
            let ok = expected.len() == provided.len() && expected.ct_eq(provided).into();
            if !ok {
                warn!("Shiprocket webhook rejected: X-Shiprocket-Token missing or wrong");
            }
            ok
        }
        _ => {
            warn!("SHIPROCKET_WEBHOOK_SECRET not set; rejecting Shiprocket webhook");
            false
        }
    }
}

fn enforce_signature_when_secret_set(
    provider: &str,
    signature_header: Option<&str>,
    body: &[u8],
) -> Result<bool, (warp::http::StatusCode, &'static str)> {
    let secret_configured = std::env::var("RAZORPAY_WEBHOOK_SECRET")
        .ok()
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);
    if provider != "razorpay" {
        return Ok(false);
    }
    if !secret_configured {
        warn!("RAZORPAY_WEBHOOK_SECRET not set; rejecting Razorpay webhook");
        return Err((
            warp::http::StatusCode::SERVICE_UNAVAILABLE,
            "Webhook secret is not configured",
        ));
    }
    let sig = signature_header.map(|s| s.trim()).unwrap_or("");
    if sig.is_empty() {
        return Err((warp::http::StatusCode::UNAUTHORIZED, "Missing signature"));
    }
    if !verify_razorpay_signature(body, sig) {
        return Err((warp::http::StatusCode::UNAUTHORIZED, "Invalid signature"));
    }
    Ok(true)
}

pub async fn handle_webhook(
    provider: String,
    signature_header: Option<String>,
    provider_event_id: Option<String>,
    shiprocket_token: Option<String>,
    body: Bytes,
) -> Result<impl Reply, std::convert::Infallible> {
    let body_str = match std::str::from_utf8(&body) {
        Ok(s) => s.to_string(),
        Err(_) => {
            return Ok(reply::with_status(
                "Invalid UTF-8 body",
                warp::http::StatusCode::BAD_REQUEST,
            ))
        }
    };

    let payload: serde_json::Value = match serde_json::from_str(&body_str) {
        Ok(v) => v,
        Err(_) => {
            return Ok(reply::with_status(
                "Invalid JSON",
                warp::http::StatusCode::BAD_REQUEST,
            ))
        }
    };

    // Phase 6: When secret is set, reject invalid/missing signature at HTTP boundary; do not call gRPC.
    let signature_verified = if provider == "shiprocket" {
        if verify_shiprocket_webhook_token(shiprocket_token.as_deref()) {
            true
        } else {
            crate::metrics::record_webhook_invalid_signature_total();
            return Ok(reply::with_status(
                "Missing or invalid webhook token",
                warp::http::StatusCode::UNAUTHORIZED,
            ));
        }
    } else {
        match enforce_signature_when_secret_set(&provider, signature_header.as_deref(), &body) {
            Ok(verified) => verified,
            Err((status, msg)) => {
                crate::metrics::record_webhook_invalid_signature_total();
                return Ok(reply::with_status(msg, status));
            }
        }
    };

    // Derive event_type and idempotency key from payload.
    let (event_type, webhook_id) = if provider == "shiprocket" {
        let et = payload["event"]
            .as_str()
            .or_else(|| payload["event_type"].as_str())
            .unwrap_or("shiprocket.tracking")
            .to_string();
        let mut hasher = Sha256::new();
        hasher.update(&body);
        let wh = format!("shiprocket:{}", hex::encode(hasher.finalize()));
        (et, wh)
    } else {
        let et = payload["event"].as_str().unwrap_or("unknown").to_string();
        let entity_id = payload["payload"]["payment"]["entity"]["id"]
            .as_str()
            .unwrap_or("")
            .to_string();
        // Idempotency key must be unique per Razorpay delivery. Using only payment_id makes
        // payment.authorized and payment.captured collide (same webhook_id → captured dropped).
        let header_peid = provider_event_id
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty());
        let wh = if let Some(peid) = header_peid {
            format!("{}:{}", provider, peid)
        } else if entity_id.is_empty() {
            format!("{}:{}", provider, hex::encode(&body[..body.len().min(32)]))
        } else {
            format!("{}:{}:{}", provider, et, entity_id)
        };
        (et, wh)
    };

    let provider_event_id_resolved = provider_event_id.or_else(|| {
        if provider == "shiprocket" {
            payload
                .get("id")
                .and_then(|x| x.as_str())
                .map(std::string::ToString::to_string)
        } else {
            None
        }
    });

    info!(
        provider = %provider,
        event_type = %event_type,
        webhook_id = %webhook_id,
        signature_verified,
        provider_event_id = ?provider_event_id_resolved,
        "webhook accepted at HTTP layer, forwarding to core operations"
    );

    crate::metrics::record_webhook_accepted_total();

    let mut client = match connect_grpc_client().await {
        Ok(c) => c,
        Err(e) => {
            warn!("gRPC connect failed in webhook handler: {:?}", e);
            return Ok(reply::with_status(
                "Service unavailable",
                warp::http::StatusCode::SERVICE_UNAVAILABLE,
            ));
        }
    };

    let raw_signature = if provider == "shiprocket" {
        shiprocket_token
    } else {
        signature_header
    };

    match client
        .ingest_webhook(IngestWebhookRequest {
            provider,
            event_type,
            webhook_id,
            payload_json: body_str,
            signature_verified,
            provider_event_id: provider_event_id_resolved,
            raw_signature,
        })
        .await
    {
        Ok(_) => {
            info!("webhook processing accepted by core operations");
            Ok(reply::with_status("OK", warp::http::StatusCode::OK))
        }
        Err(e) => {
            let (status, msg) = if e.code() == proto::tonic::Code::AlreadyExists {
                (
                    warp::http::StatusCode::CONFLICT,
                    "Replay: event already processed",
                )
            } else {
                warn!("ingest_webhook gRPC error: {:?}", e);
                (
                    warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal error",
                )
            };
            Ok(reply::with_status(msg, status))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use warp::Reply;

    fn minimal_razorpay_body() -> Bytes {
        let json =
            r#"{"event":"payment.captured","payload":{"payment":{"entity":{"id":"pay_test"}}}}"#;
        Bytes::from(json.to_string())
    }

    /// Phase 6: signature validation (missing, empty, invalid → 401; valid → passes to gRPC). Single test to avoid env races.
    #[tokio::test]
    async fn webhook_signature_validation_when_secret_set() {
        std::env::set_var("RAZORPAY_WEBHOOK_SECRET", "test_secret");
        let body = minimal_razorpay_body();

        let reply = handle_webhook("razorpay".to_string(), None, None, None, body.clone())
            .await
            .unwrap();
        let status = reply.into_response().status();
        assert!(
            status == warp::http::StatusCode::UNAUTHORIZED
                || status == warp::http::StatusCode::SERVICE_UNAVAILABLE
        );

        std::env::set_var("RAZORPAY_WEBHOOK_SECRET", "test_secret");
        let reply = handle_webhook(
            "razorpay".to_string(),
            Some("   ".to_string()),
            None,
            None,
            body.clone(),
        )
        .await
        .unwrap();
        let status = reply.into_response().status();
        assert!(
            status == warp::http::StatusCode::UNAUTHORIZED
                || status == warp::http::StatusCode::SERVICE_UNAVAILABLE
        );

        std::env::set_var("RAZORPAY_WEBHOOK_SECRET", "test_secret");
        let reply = handle_webhook(
            "razorpay".to_string(),
            Some("wrong_signature".to_string()),
            None,
            None,
            body.clone(),
        )
        .await
        .unwrap();
        let status = reply.into_response().status();
        assert!(
            status == warp::http::StatusCode::UNAUTHORIZED
                || status == warp::http::StatusCode::SERVICE_UNAVAILABLE
        );

        std::env::set_var("RAZORPAY_WEBHOOK_SECRET", "test_secret");
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        type HmacSha256 = Hmac<Sha256>;
        let mut mac = HmacSha256::new_from_slice(b"test_secret").unwrap();
        mac.update(&body);
        let sig = hex::encode(mac.finalize().into_bytes());
        let reply = handle_webhook("razorpay".to_string(), Some(sig), None, None, body)
            .await
            .unwrap();
        let status = reply.into_response().status();
        std::env::remove_var("RAZORPAY_WEBHOOK_SECRET");
        assert!(
            status == warp::http::StatusCode::SERVICE_UNAVAILABLE
                || status == warp::http::StatusCode::OK
                || status == warp::http::StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn webhook_rejects_when_secret_missing() {
        std::env::remove_var("RAZORPAY_WEBHOOK_SECRET");
        let reply = handle_webhook(
            "razorpay".to_string(),
            Some("sig".to_string()),
            None,
            None,
            minimal_razorpay_body(),
        )
        .await
        .unwrap();
        let status = reply.into_response().status();
        assert!(
            status == warp::http::StatusCode::SERVICE_UNAVAILABLE
                || status == warp::http::StatusCode::UNAUTHORIZED,
            "expected fail-closed status when webhook secret is unavailable, got {status}"
        );
    }

    #[tokio::test]
    async fn webhook_invalid_signature_retry_stays_fail_closed() {
        std::env::set_var("RAZORPAY_WEBHOOK_SECRET", "test_secret");
        let body = minimal_razorpay_body();

        for _ in 0..2 {
            let reply = handle_webhook(
                "razorpay".to_string(),
                Some("bad_signature".to_string()),
                Some("evt_retry_fail_closed".to_string()),
                None,
                body.clone(),
            )
            .await
            .unwrap();
            let status = reply.into_response().status();
            assert!(
                status == warp::http::StatusCode::UNAUTHORIZED
                    || status == warp::http::StatusCode::SERVICE_UNAVAILABLE,
                "invalid signature retry must remain fail-closed, got {status}"
            );
        }

        std::env::remove_var("RAZORPAY_WEBHOOK_SECRET");
    }
}
