//! P1 Observability: Prometheus metrics for GraphQL and HTTP boundary.
//!
//! Recorded here: request latency/error per endpoint, place_order/capture_payment outcomes,
//! webhook invalid signature. Install recorder in main and expose GET /metrics.

/// Labels for outcome (success vs error).
const OUTCOME_OK: &str = "ok";
const OUTCOME_ERROR: &str = "error";

/// GraphQL request duration in seconds (histogram).
pub fn record_graphql_request_duration_seconds(duration_sec: f64) {
    metrics::histogram!("graphql_request_duration_seconds", duration_sec);
}

/// GraphQL request count by outcome (counter).
pub fn record_graphql_request_total(success: bool) {
    let outcome = if success { OUTCOME_OK } else { OUTCOME_ERROR };
    metrics::counter!("graphql_requests_total", 1, "outcome" => outcome);
}

/// Place order: total and by outcome/reason (counter).
pub fn record_place_order_total(success: bool, reason: Option<&str>) {
    let outcome = if success { OUTCOME_OK } else { OUTCOME_ERROR };
    let reason_static: &'static str = match reason {
        Some("insufficient_stock") => "insufficient_stock",
        Some("idempotency") => "idempotency",
        _ => "error",
    };
    metrics::counter!("place_order_total", 1, "outcome" => outcome, "reason" => reason_static);
}

/// Payment capture: total and by outcome (counter).
pub fn record_capture_payment_total(success: bool) {
    let outcome = if success { OUTCOME_OK } else { OUTCOME_ERROR };
    metrics::counter!("capture_payment_total", 1, "outcome" => outcome);
}

/// Webhook: invalid or missing signature rejected at HTTP boundary (counter).
pub fn record_webhook_invalid_signature_total() {
    metrics::counter!("webhook_invalid_signature_total", 1);
}

/// Webhook: accepted and forwarded to gRPC (counter).
pub fn record_webhook_accepted_total() {
    metrics::counter!("webhook_accepted_total", 1);
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
    fn record_graphql_request_duration_does_not_panic() {
        install_test_recorder();
        record_graphql_request_duration_seconds(0.5);
        record_graphql_request_duration_seconds(1.0);
    }

    #[test]
    fn record_graphql_request_total_does_not_panic() {
        install_test_recorder();
        record_graphql_request_total(true);
        record_graphql_request_total(false);
    }

    #[test]
    fn record_place_order_total_does_not_panic() {
        install_test_recorder();
        record_place_order_total(true, None);
        record_place_order_total(false, Some("insufficient_stock"));
        record_place_order_total(false, Some("idempotency"));
        record_place_order_total(false, Some("other"));
    }

    #[test]
    fn record_capture_payment_total_does_not_panic() {
        install_test_recorder();
        record_capture_payment_total(true);
        record_capture_payment_total(false);
    }

    #[test]
    fn record_webhook_does_not_panic() {
        install_test_recorder();
        record_webhook_invalid_signature_total();
        record_webhook_accepted_total();
    }
}
