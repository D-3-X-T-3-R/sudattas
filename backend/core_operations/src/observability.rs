//! P1 Observability: Prometheus metrics for gRPC/core operations.
//!
//! Recorded here: payment mismatch (NeedsReview), capture conflict, webhook processing failure,
//! inventory update failure. Install recorder in the gRPC server binary and expose GET /metrics.

/// Payment amount/currency mismatch (webhook vs intent/order) – order marked NeedsReview.
pub fn record_payment_mismatch_total() {
    ::metrics::counter!("payment_mismatch_total", 1);
}

/// Payment capture conflict (e.g. same intent different gateway id) – NeedsReview.
pub fn record_payment_capture_conflict_total() {
    ::metrics::counter!("payment_capture_conflict_total", 1);
}

/// Client-returned Razorpay signature verification failed (reject, log security event).
pub fn record_payment_verify_invalid_signature_total() {
    ::metrics::counter!("payment_verify_invalid_signature_total", 1);
}

/// Webhook event processing failed (e.g. payment.captured handler error).
pub fn record_webhook_processing_failed_total() {
    ::metrics::counter!("webhook_processing_failed_total", 1);
}

/// Webhook event processing latency in seconds.
pub fn record_webhook_processing_duration_seconds(duration_sec: f64, outcome: &'static str) {
    ::metrics::histogram!(
        "webhook_processing_duration_seconds",
        duration_sec,
        "outcome" => outcome
    );
}

/// Inventory decrement failed (insufficient stock) in place_order.
pub fn record_inventory_update_failure_total() {
    ::metrics::counter!("inventory_update_failure_total", 1);
}

/// A supervised background worker task (or the metrics/health server) died — panicked or was
/// cancelled — and is not automatically restarted. Paired with a log::error! at each call site;
/// this counter exists so the death is visible on a dashboard even if log-based alerting isn't
/// wired up.
pub fn record_worker_died_total(worker: &'static str) {
    ::metrics::counter!("worker_died_total", 1, "worker" => worker);
}

/// Checkout/payment verification failed after the user returned from Razorpay.
pub fn record_payment_verification_failed_total(reason: &'static str) {
    ::metrics::counter!("payment_verification_failed_total", 1, "reason" => reason);
}

/// Shiprocket booking failure while auto-fulfilling a paid order.
pub fn record_shiprocket_booking_failure_total(reason: &'static str) {
    ::metrics::counter!("shiprocket_booking_failed_total", 1, "reason" => reason);
}

/// Shiprocket cancellation attempt failed and the order moved to retry state.
pub fn record_shiprocket_cancel_failure_total(reason: &'static str) {
    ::metrics::counter!("shiprocket_cancel_failed_total", 1, "reason" => reason);
}

/// Razorpay refund creation failed.
pub fn record_refund_failure_total(reason: &'static str) {
    ::metrics::counter!("refund_failure_total", 1, "reason" => reason);
}

/// Background stale-order expiry batch failed.
pub fn record_stale_order_expiry_failure_total() {
    ::metrics::counter!("stale_order_expiry_failure_total", 1);
}

/// Background cancel-pending-logistics batch failed.
pub fn record_cancel_pending_logistics_failure_total() {
    ::metrics::counter!("cancel_pending_logistics_failure_total", 1);
}

/// Background outbox/email backlog processing failed.
pub fn record_outbox_worker_failure_total() {
    ::metrics::counter!("outbox_worker_failure_total", 1);
}

/// Count of orders still stuck in pending beyond the operator threshold.
pub fn record_stuck_pending_orders_gauge(count: f64) {
    ::metrics::gauge!("stuck_pending_orders", count);
}

/// Count of orders waiting for logistics cancellation retries.
pub fn record_cancel_pending_logistics_backlog_gauge(count: f64) {
    ::metrics::gauge!("cancel_pending_logistics_backlog", count);
}

/// Orders with recorded gateway refund failure (needs ops follow-up).
pub fn record_refund_failed_orders_gauge(count: f64) {
    ::metrics::gauge!("refund_failed_orders", count);
}

/// Count of outbox/email events that are still pending delivery.
pub fn record_outbox_backlog_gauge(count: f64) {
    ::metrics::gauge!("outbox_backlog", count);
}

/// Oldest pending outbox age in seconds.
pub fn record_outbox_pending_max_age_seconds_gauge(age_sec: f64) {
    ::metrics::gauge!("outbox_pending_max_age_seconds", age_sec);
}

/// Number of pending outbox rows with at least one failed publish attempt.
pub fn record_outbox_retry_backlog_gauge(count: f64) {
    ::metrics::gauge!("outbox_retry_backlog", count);
}

/// Number of webhook events that failed processing.
pub fn record_webhook_failed_backlog_gauge(count: f64) {
    ::metrics::gauge!("webhook_failed_backlog", count);
}

/// Oldest pending webhook age in seconds.
pub fn record_webhook_pending_max_age_seconds_gauge(age_sec: f64) {
    ::metrics::gauge!("webhook_pending_max_age_seconds", age_sec);
}

/// Number of refund attempts that are still pending/submitted beyond expected processing time.
pub fn record_refund_attempts_stuck_gauge(count: f64) {
    ::metrics::gauge!("refund_attempts_stuck", count);
}

/// Number of shipments in booking/cancel persistence retry states.
pub fn record_shipments_retry_backlog_gauge(count: f64) {
    ::metrics::gauge!("shipments_retry_backlog", count);
}

/// Count of active payment intents that remain pending past the intended expiry window.
pub fn record_stuck_payment_intents_gauge(count: f64) {
    ::metrics::gauge!("stuck_payment_intents", count);
}

/// Count of stale idempotency keys that remained pending beyond retry timeout.
pub fn record_stale_idempotency_pending_gauge(count: f64) {
    ::metrics::gauge!("stale_idempotency_pending", count);
}

/// Invoice existed but corresponding outbox event was missing and repaired.
pub fn record_invoice_outbox_repair_total() {
    ::metrics::counter!("invoice_outbox_repair_total", 1);
}

/// Emit a structured operational event without secrets/tokens.
pub fn log_operational_event(event: &'static str, fields: &[(&str, String)]) {
    let rendered = fields
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join(" ");
    if rendered.is_empty() {
        tracing::info!(event, "operational event");
    } else {
        tracing::info!(event, fields = %rendered, "operational event");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    static RECORDER_INIT: std::sync::Once = std::sync::Once::new();

    fn install_test_recorder() {
        RECORDER_INIT.call_once(|| {
            let _ = metrics_exporter_prometheus::PrometheusBuilder::new().install_recorder();
        });
    }

    #[test]
    fn record_payment_mismatch_does_not_panic() {
        install_test_recorder();
        record_payment_mismatch_total();
    }

    #[test]
    fn record_payment_capture_conflict_does_not_panic() {
        install_test_recorder();
        record_payment_capture_conflict_total();
    }

    #[test]
    fn record_payment_verify_invalid_signature_does_not_panic() {
        install_test_recorder();
        record_payment_verify_invalid_signature_total();
    }

    #[test]
    fn record_webhook_processing_failed_does_not_panic() {
        install_test_recorder();
        record_webhook_processing_failed_total();
    }

    #[test]
    fn record_webhook_processing_duration_does_not_panic() {
        install_test_recorder();
        record_webhook_processing_duration_seconds(0.02, "processed");
        record_webhook_processing_duration_seconds(0.15, "failed");
    }

    #[test]
    fn record_inventory_update_failure_does_not_panic() {
        install_test_recorder();
        record_inventory_update_failure_total();
    }

    #[test]
    fn record_additional_launch_metrics_do_not_panic() {
        install_test_recorder();
        record_payment_verification_failed_total("invalid_signature");
        record_shiprocket_booking_failure_total("provider_unavailable");
        record_shiprocket_cancel_failure_total("provider_unavailable");
        record_refund_failure_total("gateway_error");
        record_stale_order_expiry_failure_total();
        record_cancel_pending_logistics_failure_total();
        record_outbox_worker_failure_total();
        record_stuck_pending_orders_gauge(2.0);
        record_cancel_pending_logistics_backlog_gauge(3.0);
        record_refund_failed_orders_gauge(1.0);
        record_outbox_backlog_gauge(4.0);
        record_outbox_pending_max_age_seconds_gauge(120.0);
        record_outbox_retry_backlog_gauge(2.0);
        record_webhook_failed_backlog_gauge(1.0);
        record_webhook_pending_max_age_seconds_gauge(300.0);
        record_refund_attempts_stuck_gauge(3.0);
        record_shipments_retry_backlog_gauge(2.0);
        record_stuck_payment_intents_gauge(1.0);
        record_stale_idempotency_pending_gauge(1.0);
        record_invoice_outbox_repair_total();
        log_operational_event(
            "order_placed",
            &[("order_id", "42".to_string()), ("user_id", "9".to_string())],
        );
    }
}
