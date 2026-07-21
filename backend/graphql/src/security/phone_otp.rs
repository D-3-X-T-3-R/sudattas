use governor::{Quota, RateLimiter};
use serde_json::Value;
use std::num::NonZeroU32;
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

fn env_nonzero_u32(key: &str, default: u32) -> NonZeroU32 {
    let value = std::env::var(key)
        .ok()
        .and_then(|raw| raw.trim().parse::<u32>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(default);
    NonZeroU32::new(value).unwrap_or_else(|| NonZeroU32::new(default.max(1)).unwrap())
}

/// Per-phone-number OTP send limiter, independent of the generic per-IP/session GraphQL rate
/// limiter in `main.rs`. Without this, a single phone number can be SMS-bombed by rotating
/// source IPs/sessions since the generic limiter keys on caller identity, not on the target phone.
fn otp_send_limiter() -> &'static governor::DefaultKeyedRateLimiter<String> {
    static LIMITER: OnceLock<governor::DefaultKeyedRateLimiter<String>> = OnceLock::new();
    LIMITER.get_or_init(|| {
        let burst = env_nonzero_u32("OTP_SEND_MAX_PER_HOUR", 5);
        RateLimiter::keyed(Quota::per_hour(burst))
    })
}

/// Per-phone-number OTP verify (code-guess) limiter: brute-force protection for the phone number
/// being verified, independent of Twilio Verify's own throttling, which is otherwise the only
/// defense against repeated code guesses.
fn otp_verify_limiter() -> &'static governor::DefaultKeyedRateLimiter<String> {
    static LIMITER: OnceLock<governor::DefaultKeyedRateLimiter<String>> = OnceLock::new();
    LIMITER.get_or_init(|| {
        let burst = env_nonzero_u32("OTP_VERIFY_MAX_PER_15_MIN", 5);
        let period = Duration::from_secs(900) / burst.get();
        let quota = Quota::with_period(period)
            .expect("OTP_VERIFY_MAX_PER_15_MIN period must be non-zero")
            .allow_burst(burst);
        RateLimiter::keyed(quota)
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
    if otp_send_limiter().check_key(&phone_e164).is_err() {
        return Err("OTP_RATE_LIMITED".to_string());
    }
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
    if otp_verify_limiter().check_key(&phone_e164).is_err() {
        return Err("OTP_RATE_LIMITED".to_string());
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
