//! Razorpay API client for server-authoritative order creation.
//! Requires RAZORPAY_KEY_ID and RAZORPAY_KEY_SECRET env vars (when not set, create_order returns Err).

use reqwest::Client;
use serde::Deserialize;

const RAZORPAY_ORDERS_URL: &str = "https://api.razorpay.com/v1/orders";
const RAZORPAY_PAYMENTS_URL: &str = "https://api.razorpay.com/v1/payments";

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
        .post(RAZORPAY_ORDERS_URL)
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
    std::env::var("RAZORPAY_KEY_ID").ok()
}

/// Create a Razorpay refund for a captured payment and return gateway refund details.
/// Amount is in paise.
pub async fn create_refund(
    payment_id: &str,
    amount_paise: i64,
) -> Result<RazorpayRefundResult, String> {
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

    let body = serde_json::json!({
        "amount": amount_paise
    });

    let client = Client::new();
    let url = format!("{}/{}/refund", RAZORPAY_PAYMENTS_URL, pid);
    let res = client
        .post(&url)
        .basic_auth(&key_id, Some(&key_secret))
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
