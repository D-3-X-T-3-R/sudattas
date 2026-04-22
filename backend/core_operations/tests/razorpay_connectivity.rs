//! Check that Razorpay client wiring works with a local mock endpoint.

mod provider_test_gate;

use std::sync::{Arc, Mutex};
use warp::Filter;

#[tokio::test]
async fn razorpay_connectivity() {
    if !provider_test_gate::should_run_provider_dependent_test("razorpay_connectivity") {
        return;
    }

    let seen_auth: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let seen_auth_filter = Arc::clone(&seen_auth);

    let orders = warp::path!("v1" / "orders")
        .and(warp::post())
        .and(warp::header::optional::<String>("authorization"))
        .and(warp::body::json())
        .map(move |auth: Option<String>, body: serde_json::Value| {
            if let Ok(mut slot) = seen_auth_filter.lock() {
                *slot = auth;
            }
            assert_eq!(body.get("amount").and_then(|v| v.as_i64()), Some(100));
            assert_eq!(body.get("currency").and_then(|v| v.as_str()), Some("INR"));
            assert_eq!(
                body.get("receipt").and_then(|v| v.as_str()),
                Some("test_sudattas_check")
            );
            warp::reply::json(&serde_json::json!({ "id": "order_test_123" }))
        });

    let (addr, server) = warp::serve(orders).bind_ephemeral(([127, 0, 0, 1], 0));
    tokio::task::spawn(server);

    std::env::set_var("RAZORPAY_KEY_ID", "rzp_test_key");
    std::env::set_var("RAZORPAY_KEY_SECRET", "rzp_test_secret");
    std::env::set_var("RAZORPAY_API_BASE", format!("http://{}/v1", addr));

    let order_id = core_operations::razorpay::create_order(100, "INR", "test_sudattas_check")
        .await
        .expect("Razorpay create_order should succeed against local mock");

    assert_eq!(order_id, "order_test_123");

    let auth = seen_auth
        .lock()
        .expect("auth lock")
        .clone()
        .unwrap_or_default();
    assert!(
        auth.starts_with("Basic "),
        "expected basic auth header to be sent"
    );
}
