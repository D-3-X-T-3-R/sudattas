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

pub fn return_window_days() -> i64 {
    parse_positive_i64_env("RETURN_WINDOW_DAYS", 7)
}

pub fn worker_reclaim_timeout_minutes() -> i64 {
    parse_positive_i64_env("WORKER_RECLAIM_TIMEOUT_MINUTES", 12)
}

pub fn shipment_booking_reclaim_timeout_minutes() -> i64 {
    parse_positive_i64_env(
        "SHIPMENT_BOOKING_RECLAIM_TIMEOUT_MINUTES",
        worker_reclaim_timeout_minutes(),
    )
}

pub fn outbox_reclaim_timeout_minutes() -> i64 {
    parse_positive_i64_env(
        "OUTBOX_RECLAIM_TIMEOUT_MINUTES",
        worker_reclaim_timeout_minutes(),
    )
}

pub fn refund_reclaim_timeout_minutes() -> i64 {
    parse_positive_i64_env(
        "REFUND_RECLAIM_TIMEOUT_MINUTES",
        worker_reclaim_timeout_minutes(),
    )
}

pub fn cancel_window_deadline(order_created_at: DateTime<Utc>) -> DateTime<Utc> {
    order_created_at + Duration::hours(cancel_window_hours())
}

pub fn earliest_booking_deadline(order_created_at: DateTime<Utc>) -> DateTime<Utc> {
    cancel_window_deadline(order_created_at)
}

pub fn default_pickup_target(order_created_at: DateTime<Utc>) -> DateTime<Utc> {
    order_created_at + Duration::hours(pickup_delay_hours())
}

pub fn is_before_deadline(now: DateTime<Utc>, deadline: DateTime<Utc>) -> bool {
    now < deadline
}

pub fn is_booking_open(now: DateTime<Utc>, earliest_booking_at: DateTime<Utc>) -> bool {
    now >= earliest_booking_at
}

pub fn is_within_cancel_window(order_created_at: DateTime<Utc>, now: DateTime<Utc>) -> bool {
    is_before_deadline(now, cancel_window_deadline(order_created_at))
}

pub fn return_window_deadline(delivered_at: DateTime<Utc>) -> DateTime<Utc> {
    delivered_at + Duration::days(return_window_days())
}

pub fn is_within_return_window(delivered_at: DateTime<Utc>, now: DateTime<Utc>) -> bool {
    now <= return_window_deadline(delivered_at)
}
