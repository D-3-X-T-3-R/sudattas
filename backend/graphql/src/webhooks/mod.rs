use hmac::{Hmac, Mac};
use proto::proto::core::IngestWebhookRequest;
use sha2::Sha256;
use tracing::warn;
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
    computed == signature
}

/// When RAZORPAY_WEBHOOK_SECRET is set and provider is razorpay, signature is required and must be valid.
/// Returns Err((status, message)) to reject at HTTP boundary without calling gRPC.
fn enforce_signature_when_secret_set(
    provider: &str,
    signature_header: Option<&str>,
    body: &[u8],
) -> Result<bool, (warp::http::StatusCode, &'static str)> {
    let secret_configured = std::env::var("RAZORPAY_WEBHOOK_SECRET").is_ok();
    if provider != "razorpay" || !secret_configured {
        if provider == "razorpay" && !secret_configured {
            warn!(
                "RAZORPAY_WEBHOOK_SECRET not set; accepting webhook without signature verification"
            );
        }
        return Ok(false);
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
    let signature_verified =
        match enforce_signature_when_secret_set(&provider, signature_header.as_deref(), &body) {
            Ok(verified) => verified,
            Err((status, msg)) => return Ok(reply::with_status(msg, status)),
        };

    // Derive event_type and idempotency key from payload.
    let event_type = payload["event"].as_str().unwrap_or("unknown").to_string();

    let entity_id = payload["payload"]["payment"]["entity"]["id"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let webhook_id = if entity_id.is_empty() {
        format!("{}:{}", provider, hex::encode(&body[..body.len().min(32)]))
    } else {
        format!("{}:{}", provider, entity_id)
    };

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

    match client
        .ingest_webhook(IngestWebhookRequest {
            provider,
            event_type,
            webhook_id,
            payload_json: body_str,
            signature_verified,
            provider_event_id,
        })
        .await
    {
        Ok(_) => Ok(reply::with_status("OK", warp::http::StatusCode::OK)),
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

        let reply = handle_webhook("razorpay".to_string(), None, None, body.clone())
            .await
            .unwrap();
        assert_eq!(
            reply.into_response().status(),
            warp::http::StatusCode::UNAUTHORIZED
        );

        std::env::set_var("RAZORPAY_WEBHOOK_SECRET", "test_secret");
        let reply = handle_webhook(
            "razorpay".to_string(),
            Some("   ".to_string()),
            None,
            body.clone(),
        )
        .await
        .unwrap();
        assert_eq!(
            reply.into_response().status(),
            warp::http::StatusCode::UNAUTHORIZED
        );

        std::env::set_var("RAZORPAY_WEBHOOK_SECRET", "test_secret");
        let reply = handle_webhook(
            "razorpay".to_string(),
            Some("wrong_signature".to_string()),
            None,
            body.clone(),
        )
        .await
        .unwrap();
        assert_eq!(
            reply.into_response().status(),
            warp::http::StatusCode::UNAUTHORIZED
        );

        std::env::set_var("RAZORPAY_WEBHOOK_SECRET", "test_secret");
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        type HmacSha256 = Hmac<Sha256>;
        let mut mac = HmacSha256::new_from_slice(b"test_secret").unwrap();
        mac.update(&body);
        let sig = hex::encode(mac.finalize().into_bytes());
        let reply = handle_webhook("razorpay".to_string(), Some(sig), None, body)
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
}
