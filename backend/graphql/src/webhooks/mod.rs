use hmac::{Hmac, Mac};
use proto::proto::core::IngestWebhookRequest;
use sha2::Sha256;
use tracing::warn;
use warp::hyper::body::Bytes;
use warp::reply::{self, Reply};

use crate::resolvers::utils::connect_grpc_client;

type HmacSha256 = Hmac<Sha256>;

/// Verify Razorpay HMAC-SHA256 signature.
/// Razorpay signs the raw request body with the webhook secret.
fn verify_razorpay_signature(body: &[u8], signature: &str) -> bool {
    let secret = match std::env::var("RAZORPAY_WEBHOOK_SECRET") {
        Ok(s) => s,
        Err(_) => {
            warn!("RAZORPAY_WEBHOOK_SECRET not set; skipping signature verification");
            return false;
        }
    };

    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };
    mac.update(body);
    let computed = hex::encode(mac.finalize().into_bytes());
    computed == signature
}

pub async fn handle_webhook(
    provider: String,
    signature_header: Option<String>,
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

    let signature_verified = match (&provider as &str, &signature_header) {
        ("razorpay", Some(sig)) => verify_razorpay_signature(&body, sig),
        _ => false,
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
        })
        .await
    {
        Ok(_) => Ok(reply::with_status("OK", warp::http::StatusCode::OK)),
        Err(e) => {
            warn!("ingest_webhook gRPC error: {:?}", e);
            Ok(reply::with_status(
                "Internal error",
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}
