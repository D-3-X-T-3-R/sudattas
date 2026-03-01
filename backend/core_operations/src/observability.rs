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

/// Inventory decrement failed (insufficient stock) in place_order.
pub fn record_inventory_update_failure_total() {
    ::metrics::counter!("inventory_update_failure_total", 1);
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
    fn record_inventory_update_failure_does_not_panic() {
        install_test_recorder();
        record_inventory_update_failure_total();
    }
}
