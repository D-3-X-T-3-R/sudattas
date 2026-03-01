//! Check that Razorpay API is reachable and credentials work.
//!
//! **Setup**
//! - Put `RAZORPAY_KEY_ID` and `RAZORPAY_KEY_SECRET` in `backend/.env` (or export them).
//! - From backend dir: `cargo test -p core_operations razorpay_connectivity -- --ignored`
//!
//! If you see "RAZORPAY_KEY_ID not set", ensure you run from `backend/` and that `backend/.env` exists with both keys.

#[tokio::test]
#[ignore = "requires RAZORPAY_KEY_ID and RAZORPAY_KEY_SECRET; run with --ignored"]
async fn razorpay_connectivity() {
    // Load .env from cwd or common relative paths (run from backend/ so .env is found)
    let _ = dotenvy::dotenv();
    let _ = dotenvy::from_path(".env");
    let _ = dotenvy::from_path("../.env");

    let order_id = core_operations::razorpay::create_order(
        100,
        "INR",
        "test_sudattas_check",
    )
    .await
    .expect("Razorpay create_order should succeed when keys are set");

    assert!(
        order_id.starts_with("order_"),
        "Razorpay should return order id starting with order_, got: {}",
        order_id
    );
}
