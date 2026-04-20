use chrono::{DateTime, Duration, Utc};

fn parse_positive_i64_env(key: &str, default_value: i64) -> i64 {
    std::env::var(key)
        .ok()
        .and_then(|raw| raw.trim().parse::<i64>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(default_value)
}

pub fn cancel_window_hours() -> i64 {
    parse_positive_i64_env("CANCEL_WINDOW_HOURS", 12)
}

pub fn pickup_delay_hours() -> i64 {
    parse_positive_i64_env("PICKUP_DELAY_HOURS", 48)
}

pub fn free_shipping_threshold_minor() -> i64 {
    parse_positive_i64_env("FREE_SHIPPING_THRESHOLD_MINOR", i64::MAX)
}

pub fn cancel_window_deadline(order_created_at: DateTime<Utc>) -> DateTime<Utc> {
    order_created_at + Duration::hours(cancel_window_hours())
}

pub fn is_within_cancel_window(order_created_at: DateTime<Utc>, now: DateTime<Utc>) -> bool {
    now < cancel_window_deadline(order_created_at)
}

