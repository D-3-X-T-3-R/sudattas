//! Razorpay API client for server-authoritative order creation.
//! Requires RAZORPAY_KEY_ID and RAZORPAY_KEY_SECRET env vars (when not set, create_order returns Err).
//! Optional override: `RAZORPAY_API_BASE` (defaults to `https://api.razorpay.com/v1`).

use crate::load_env_once;
use reqwest::Client;
use serde::Deserialize;
use tracing::info;

/// First characters of key id for logs (never log full key id or secret).
fn mask_razorpay_key_id(key_id: &str) -> String {
    let prefix: String = key_id.chars().take(14).collect();
    format!("{prefix}…")
}

fn razorpay_mode_label(key_id: &str) -> &'static str {
    if key_id.starts_with("rzp_live_") {
        "live"
    } else if key_id.starts_with("rzp_test_") {
        "test"
    } else {
        "unknown"
    }
}

fn razorpay_api_base() -> String {
    load_env_once();
    std::env::var("RAZORPAY_API_BASE")
        .unwrap_or_else(|_| "https://api.razorpay.com/v1".to_string())
        .trim_end_matches('/')
        .to_string()
}

fn razorpay_orders_url() -> String {
    format!("{}/orders", razorpay_api_base())
}

fn razorpay_payments_url() -> String {
    format!("{}/payments", razorpay_api_base())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct RazorpayOrderResponse {
    id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct RazorpayRefundResponse {
    id: String,
    status: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RazorpayRefundResult {
    pub refund_id: String,
    pub status: Option<String>,
}

/// Create a Razorpay order; returns the Razorpay order ID (e.g. "order_xxx").
/// Amount must be in paise (Razorpay minimum 100 for INR).
pub async fn create_order(
    amount_paise: i64,
    currency: &str,
    receipt: &str,
) -> Result<String, String> {
    load_env_once();
    let key_id = std::env::var("RAZORPAY_KEY_ID").map_err(|_| "RAZORPAY_KEY_ID not set")?;
    let key_secret =
        std::env::var("RAZORPAY_KEY_SECRET").map_err(|_| "RAZORPAY_KEY_SECRET not set")?;

    if amount_paise < 100 {
        return Err("Razorpay minimum amount is 100 paise (INR 1.00)".to_string());
    }

    let body = serde_json::json!({
        "amount": amount_paise,
        "currency": currency,
        "receipt": receipt,
    });

    let client = Client::new();
    let res = client
        .post(razorpay_orders_url())
        .basic_auth(&key_id, Some(&key_secret))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Razorpay request failed: {}", e))?;

    let status = res.status();
    let text = res
        .text()
        .await
        .map_err(|e| format!("Razorpay response read failed: {}", e))?;

    if !status.is_success() {
        return Err(format!("Razorpay API error {}: {}", status, text));
    }

    let parsed: RazorpayOrderResponse = serde_json::from_str(&text)
        .map_err(|e| format!("Razorpay response parse failed: {}", e))?;
    Ok(parsed.id)
}

/// Returns RAZORPAY_KEY_ID if set (for frontend Checkout; never expose secret).
pub fn key_id_for_frontend() -> Option<String> {
    load_env_once();
    std::env::var("RAZORPAY_KEY_ID").ok()
}

/// Create a Razorpay refund for a captured payment and return gateway refund details.
/// Amount is in paise.
pub async fn create_refund(
    payment_id: &str,
    amount_paise: i64,
    refund_idempotency_key: &str,
) -> Result<RazorpayRefundResult, String> {
    load_env_once();
    let key_id = std::env::var("RAZORPAY_KEY_ID").map_err(|_| "RAZORPAY_KEY_ID not set")?;
    let key_secret =
        std::env::var("RAZORPAY_KEY_SECRET").map_err(|_| "RAZORPAY_KEY_SECRET not set")?;

    let pid = payment_id.trim();
    if pid.is_empty() {
        return Err("payment_id is required".to_string());
    }
    if amount_paise <= 0 {
        return Err("amount_paise must be positive".to_string());
    }
    let idem = refund_idempotency_key.trim();
    if idem.is_empty() {
        return Err("refund_idempotency_key is required".to_string());
    }
    if !idem
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(
            "refund_idempotency_key may only contain alphanumeric characters, underscores, and hyphens"
                .to_string(),
        );
    }

    let body = serde_json::json!({
        "amount": amount_paise
    });

    let api_base = razorpay_api_base();
    let payments_collection_url = razorpay_payments_url();
    let url = format!("{}/{}/refund", payments_collection_url, pid);
    let body_for_log = serde_json::to_string(&body)
        .unwrap_or_else(|e| format!(r#"{{"error":"failed to serialize body for log: {e}"}}"#));
    info!(
        razorpay_refund_api_base = %api_base,
        razorpay_refund_payments_url = %payments_collection_url,
        razorpay_refund_post_url = %url,
        razorpay_refund_key_id_prefix = %mask_razorpay_key_id(&key_id),
        razorpay_refund_mode = %razorpay_mode_label(&key_id),
        razorpay_refund_request_body = %body_for_log,
        razorpay_refund_idempotency_key = %idem,
        razorpay_refund_payment_id = %pid,
        razorpay_refund_payment_id_len = pid.len(),
        razorpay_refund_payment_id_had_outer_whitespace = (pid != payment_id),
        "razorpay create_refund outbound request (diagnostic; secret not logged)"
    );

    let client = Client::new();
    let res = client
        .post(&url)
        .basic_auth(&key_id, Some(&key_secret))
        .header("X-Refund-Idempotency", idem)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Razorpay refund request failed: {}", e))?;

    let status = res.status();
    let text = res
        .text()
        .await
        .map_err(|e| format!("Razorpay refund response read failed: {}", e))?;

    if !status.is_success() {
        return Err(format!("Razorpay refund API error {}: {}", status, text));
    }

    let parsed: RazorpayRefundResponse = serde_json::from_str(&text)
        .map_err(|e| format!("Razorpay refund response parse failed: {}", e))?;
    Ok(RazorpayRefundResult {
        refund_id: parsed.id,
        status: parsed.status,
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct RazorpayPaymentFetchResponse {
    amount: i64,
}

/// Fetch captured payment amount in paise from Razorpay (authoritative cap for refunds).
pub async fn fetch_payment_amount_paise(payment_id: &str) -> Result<i64, String> {
    load_env_once();
    let key_id = std::env::var("RAZORPAY_KEY_ID").map_err(|_| "RAZORPAY_KEY_ID not set")?;
    let key_secret =
        std::env::var("RAZORPAY_KEY_SECRET").map_err(|_| "RAZORPAY_KEY_SECRET not set")?;

    let pid = payment_id.trim();
    if pid.is_empty() {
        return Err("payment_id is required".to_string());
    }

    let url = format!("{}/{}", razorpay_payments_url(), pid);
    let client = Client::new();
    let res = client
        .get(&url)
        .basic_auth(&key_id, Some(&key_secret))
        .send()
        .await
        .map_err(|e| format!("Razorpay payment fetch failed: {}", e))?;

    let status = res.status();
    let text = res
        .text()
        .await
        .map_err(|e| format!("Razorpay payment response read failed: {}", e))?;

    if !status.is_success() {
        return Err(format!("Razorpay payment API error {}: {}", status, text));
    }

    let parsed: RazorpayPaymentFetchResponse = serde_json::from_str(&text)
        .map_err(|e| format!("Razorpay payment response parse failed: {}", e))?;
    Ok(parsed.amount)
}
