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

/// Admin authorization denied at GraphQL resolver boundary.
pub fn record_admin_authz_denied_total() {
    metrics::counter!("graphql_admin_authz_denied_total", 1);
}

/// Admin authorization denied with reason label (e.g. not_admin, not_jwt, role_lookup_failed).
pub fn record_admin_authz_denied_reason_total(reason: &'static str) {
    metrics::counter!("graphql_admin_authz_denied_reason_total", 1, "reason" => reason);
}

/// Admin role resolution attempt outcome/source.
/// source: cache | db | env_fallback | none
/// outcome: success | failure
pub fn record_admin_role_resolution_total(source: &'static str, outcome: &'static str) {
    metrics::counter!(
        "graphql_admin_role_resolution_total",
        1,
        "source" => source,
        "outcome" => outcome
    );
}

/// Auth rejection at HTTP boundary before GraphQL execution.
pub fn record_auth_rejection_total(kind: &'static str) {
    metrics::counter!("graphql_auth_rejection_total", 1, "kind" => kind);
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

    #[test]
    fn record_admin_authz_denied_does_not_panic() {
        install_test_recorder();
        record_admin_authz_denied_total();
        record_admin_authz_denied_reason_total("not_admin");
    }

    #[test]
    fn record_admin_role_resolution_does_not_panic() {
        install_test_recorder();
        record_admin_role_resolution_total("cache", "success");
        record_admin_role_resolution_total("db", "failure");
    }

    #[test]
    fn record_auth_rejection_does_not_panic() {
        install_test_recorder();
        record_auth_rejection_total("unauthorized");
        record_auth_rejection_total("csrf");
        record_auth_rejection_total("rate_limited");
    }
}
