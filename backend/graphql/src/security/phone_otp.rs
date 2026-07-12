use serde_json::Value;
use std::sync::OnceLock;
use std::time::Duration;

/// Shared HTTP client for Twilio Verify calls. `reqwest`'s defaults have no timeout,
/// and these OTP endpoints are public/unauthenticated — a hang here would otherwise
/// tie up a request handler indefinitely on a natural abuse target.
fn http_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("failed to build Twilio HTTP client")
    })
}

fn normalize_phone(raw: &str) -> Option<String> {
    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() == 10 {
        return Some(format!("+91{}", digits));
    }
    if digits.len() == 12 && digits.starts_with("91") {
        return Some(format!("+{}", digits));
    }
    if raw.trim().starts_with('+') && digits.len() >= 10 {
        return Some(format!("+{}", digits));
    }
    None
}

fn twilio_config() -> Result<(String, String, String), String> {
    let sid = std::env::var("TWILIO_ACCOUNT_SID").map_err(|_| "OTP_NOT_CONFIGURED".to_string())?;
    let token = std::env::var("TWILIO_AUTH_TOKEN").map_err(|_| "OTP_NOT_CONFIGURED".to_string())?;
    let service_sid =
        std::env::var("TWILIO_VERIFY_SERVICE_SID").map_err(|_| "OTP_NOT_CONFIGURED".to_string())?;
    Ok((sid, token, service_sid))
}

pub async fn request_sms_otp(phone: &str, channel: Option<&str>) -> Result<(), String> {
    let phone_e164 = normalize_phone(phone).ok_or_else(|| "INVALID_PHONE".to_string())?;
    let channel = match channel.unwrap_or("sms").trim().to_lowercase().as_str() {
        "sms" => "sms",
        "whatsapp" => "whatsapp",
        _ => return Err("INVALID_CHANNEL".to_string()),
    };
    let (sid, token, service_sid) = twilio_config()?;
    let client = http_client();
    let res = client
        .post(format!(
            "https://verify.twilio.com/v2/Services/{}/Verifications",
            service_sid
        ))
        .basic_auth(sid, Some(token))
        .form(&[("To", phone_e164.as_str()), ("Channel", channel)])
        .send()
        .await
        .map_err(|_| "OTP_SEND_FAILED".to_string())?;

    if !res.status().is_success() {
        return Err("OTP_SEND_FAILED".to_string());
    }
    Ok(())
}

pub async fn verify_sms_otp(phone: &str, code: &str) -> Result<bool, String> {
    let phone_e164 = normalize_phone(phone).ok_or_else(|| "INVALID_PHONE".to_string())?;
    if !code.chars().all(|c| c.is_ascii_digit()) || code.len() < 4 || code.len() > 8 {
        return Err("INVALID_OTP".to_string());
    }
    let (sid, token, service_sid) = twilio_config()?;
    let client = http_client();
    let res = client
        .post(format!(
            "https://verify.twilio.com/v2/Services/{}/VerificationCheck",
            service_sid
        ))
        .basic_auth(sid, Some(token))
        .form(&[("To", phone_e164.as_str()), ("Code", code)])
        .send()
        .await
        .map_err(|_| "OTP_VERIFY_FAILED".to_string())?;

    if !res.status().is_success() {
        return Ok(false);
    }
    let data: Value = res
        .json()
        .await
        .map_err(|_| "OTP_VERIFY_FAILED".to_string())?;
    Ok(data.get("status").and_then(Value::as_str) == Some("approved"))
}
