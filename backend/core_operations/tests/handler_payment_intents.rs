//! Tests for payment_intents handlers (idempotent capture semantics) using SeaORM MockDatabase.

use core_db_entities::entity::payment_intents;
use core_db_entities::entity::sea_orm_active_enums::Status as PaymentStatus;
use proto::proto::core::CapturePaymentRequest;
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, TransactionTrait};
use tonic::Request;

/// Helper to build a payment_intents model for tests.
fn make_intent(
    intent_id: i64,
    razorpay_order_id: &str,
    razorpay_payment_id: Option<&str>,
    status: Option<PaymentStatus>,
) -> payment_intents::Model {
    payment_intents::Model {
        intent_id,
        razorpay_order_id: razorpay_order_id.to_string(),
        order_id: None,
        user_id: None,
        amount_paise: 10_000,
        currency: Some("INR".to_string()),
        status,
        razorpay_payment_id: razorpay_payment_id.map(|s| s.to_string()),
        metadata: None,
        created_at: None,
        expires_at: chrono::Utc::now(),
    }
}

#[tokio::test]
async fn capture_payment_is_idempotent_for_same_gateway_id() {
    use core_operations::handlers::payment_intents::capture_payment;

    // First query: find_by_id returns an intent with an existing gateway id.
    // Second query (by RazorpayPaymentId) returns the same intent.
    let existing_intent = make_intent(
        1,
        "order_123",
        Some("pay_abc"),
        Some(PaymentStatus::Processed),
    );

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![
            vec![existing_intent.clone()], // find_by_id
            vec![existing_intent.clone()], // find by RazorpayPaymentId
        ])
        // No-op update (rows_affected=1) even though handler should not change anything
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .into_connection();

    let txn = db.begin().await.expect("begin transaction");

    let req = Request::new(CapturePaymentRequest {
        intent_id: 1,
        razorpay_payment_id: "pay_abc".to_string(),
    });

    let result = capture_payment(&txn, req).await;
    assert!(
        result.is_ok(),
        "capture_payment should be idempotent for same gateway id: {:?}",
        result.err()
    );

    let resp = result.unwrap().into_inner();
    assert_eq!(resp.items.len(), 1);
    let item = &resp.items[0];
    assert_eq!(item.intent_id, 1);
    assert_eq!(item.razorpay_payment_id.as_deref(), Some("pay_abc"));
}

#[tokio::test]
async fn capture_payment_rejects_conflicting_gateway_id_for_same_intent() {
    use core_operations::handlers::payment_intents::capture_payment;

    // Existing intent already has a different gateway payment id.
    let existing_intent = make_intent(
        1,
        "order_123",
        Some("pay_original"),
        Some(PaymentStatus::Processed),
    );

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![
            vec![existing_intent], // find_by_id
        ])
        .into_connection();

    let txn = db.begin().await.expect("begin transaction");

    let req = Request::new(CapturePaymentRequest {
        intent_id: 1,
        razorpay_payment_id: "pay_conflict".to_string(),
    });

    let result = capture_payment(&txn, req).await;
    assert!(
        result.is_err(),
        "capture_payment should fail for conflicting gateway id"
    );
    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::FailedPrecondition);
}

#[tokio::test]
async fn capture_payment_rejects_gateway_id_reused_for_different_intent() {
    use core_operations::handlers::payment_intents::capture_payment;

    // First query: find_by_id returns intent 1 with no gateway id yet.
    // Second query: find by RazorpayPaymentId returns intent 2, meaning the
    // gateway payment id is already associated with a different intent.
    let intent_without_gateway = make_intent(1, "order_123", None, Some(PaymentStatus::Pending));
    let intent_with_gateway = make_intent(
        2,
        "order_456",
        Some("pay_reused"),
        Some(PaymentStatus::Processed),
    );

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![
            vec![intent_without_gateway], // find_by_id
            vec![intent_with_gateway],    // find by RazorpayPaymentId
        ])
        .into_connection();

    let txn = db.begin().await.expect("begin transaction");

    let req = Request::new(CapturePaymentRequest {
        intent_id: 1,
        razorpay_payment_id: "pay_reused".to_string(),
    });

    let result = capture_payment(&txn, req).await;
    assert!(
        result.is_err(),
        "capture_payment should fail when gateway id is already used for another intent"
    );
    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::FailedPrecondition);
}
