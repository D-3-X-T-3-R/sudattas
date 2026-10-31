//! P1 Tests for create_refund (idempotent, invalid state, partial/full) and resolve_needs_review.

mod integration_common;

use core_db_entities::entity::order_events;
use core_db_entities::entity::order_status;
use core_db_entities::entity::orders;
use core_db_entities::entity::refund_attempts;
use core_db_entities::entity::refunds;
use core_db_entities::entity::sea_orm_active_enums::{
    ActorType, FulfillmentStatus, Status as RefundStatus,
};
use proto::proto::core::{
    CreateRefundRequest, ResolveNeedsReviewRequest, ResolveRefundAttemptNeedsReviewRequest,
    SearchRefundAttemptsRequest,
};
use sea_orm::entity::prelude::Decimal;
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, TransactionTrait};
use tonic::Request;

fn order_confirmed(id: i64, status_id: i64, grand_total_minor: i64) -> orders::Model {
    orders::Model {
        order_id: id,
        order_number: Some(format!("ORD-{}", id)),
        public_order_ref: format!("SUD-20990101-R{id:010}"),
        user_id: 1,
        order_date: chrono::Utc::now(),
        created_at: chrono::Utc::now(),
        cancel_window_ends_at: None,
        earliest_booking_at: None,
        pickup_target_at: None,
        pickup_target_reason: None,
        pickup_target_set_by: None,
        pickup_target_updated_at: None,
        shipping_address_id: 1,
        total_amount: Some(Decimal::new(grand_total_minor, 2)),
        status_id,
        payment_status: None,
        payment_method: None,
        currency: Some("INR".to_string()),
        updated_at: None,
        subtotal_minor: grand_total_minor,
        items_total_minor_before_discount: Some(grand_total_minor),
        shipping_minor: Some(0),
        shipping_charge_minor: Some(0),
        tax_total_minor: Some(0),
        discount_total_minor: Some(0),
        items_total_minor_after_discount: Some(grand_total_minor),
        grand_total_minor,
        invoice_id: None,
        invoice_number: None,
        invoice_generated_at: None,
        invoice_storage_path: None,
        applied_coupon_id: None,
        applied_coupon_code: None,
        applied_discount_paise: None,
        refund_settlement_status: None,
        fulfillment_status: FulfillmentStatus::NotCreated,
    }
}

fn status_row(status_id: i64, name: &str) -> order_status::Model {
    order_status::Model {
        status_id,
        status_name: name.to_string(),
    }
}

fn refund_model(
    refund_id: i64,
    order_id: i64,
    gateway_refund_id: &str,
    amount_paise: i32,
) -> refunds::Model {
    refunds::Model {
        refund_id,
        order_id,
        gateway_refund_id: gateway_refund_id.to_string(),
        amount_paise,
        currency: Some("INR".to_string()),
        status: Some(RefundStatus::Processed),
        line_items_refunded: None,
        created_at: Some(chrono::Utc::now()),
    }
}

fn refund_attempt(attempt_id: i64, order_id: i64, status: &str) -> refund_attempts::Model {
    refund_attempts::Model {
        attempt_id,
        order_id,
        payment_intent_id: None,
        razorpay_payment_id: None,
        amount_requested_paise: 5_000,
        amount_sent_to_gateway_paise: 5_000,
        gateway_refund_id: None,
        status: status.to_string(),
        provider_error: Some("gateway timeout".to_string()),
        idempotency_key: format!("itest_attempt_{attempt_id}"),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        attempt_count: 3,
    }
}

fn order_event_row(event_id: i64, order_id: i64) -> order_events::Model {
    order_events::Model {
        event_id,
        order_id,
        event_type: "refund_attempt_needs_review_resolved".to_string(),
        from_status: None,
        to_status: None,
        actor_type: ActorType::Admin,
        message: Some("resolved".to_string()),
        created_at: Some(chrono::Utc::now()),
    }
}

#[tokio::test]
async fn create_refund_idempotent_returns_existing() {
    use core_operations::handlers::refunds::create_refund;

    let order = order_confirmed(10, 2, 10_000);
    let status = status_row(2, "confirmed");
    let existing_refund = refund_model(1, 10, "gw_refund_123", 5_000);

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![order]])
        .append_query_results(vec![vec![status]])
        .append_query_results(vec![vec![existing_refund.clone()]])
        .into_connection();

    let txn = db.begin().await.expect("begin");
    let req = Request::new(CreateRefundRequest {
        order_id: 10,
        gateway_refund_id: "gw_refund_123".to_string(),
        amount_paise: 5_000,
        currency: None,
        line_items_refunded_json: None,
    });

    let result = create_refund(&txn, req).await;
    assert!(
        result.is_ok(),
        "idempotent create_refund should return existing: {:?}",
        result.err()
    );
    let res = result.unwrap().into_inner();
    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].refund_id, 1);
    assert_eq!(res.items[0].gateway_refund_id, "gw_refund_123");
    assert_eq!(res.items[0].amount_paise, 5_000);
}

#[tokio::test]
async fn create_refund_rejects_non_refundable_state() {
    use core_operations::handlers::refunds::create_refund;

    let order = order_confirmed(10, 1, 10_000);
    let status = status_row(1, "pending");

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![order]])
        .append_query_results(vec![vec![status]])
        .into_connection();

    let txn = db.begin().await.expect("begin");
    let req = Request::new(CreateRefundRequest {
        order_id: 10,
        gateway_refund_id: "gw_new".to_string(),
        amount_paise: 5_000,
        currency: None,
        line_items_refunded_json: None,
    });

    let result = create_refund(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::FailedPrecondition);
}

#[tokio::test]
async fn create_refund_rejects_empty_gateway_refund_id() {
    use core_operations::handlers::refunds::create_refund;

    let db = MockDatabase::new(DatabaseBackend::MySql).into_connection();
    let txn = db.begin().await.expect("begin");
    let req = Request::new(CreateRefundRequest {
        order_id: 10,
        gateway_refund_id: "".to_string(),
        amount_paise: 1_000,
        currency: None,
        line_items_refunded_json: None,
    });

    let result = create_refund(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn resolve_needs_review_rejects_non_needs_review() {
    use core_operations::handlers::orders::resolve_needs_review;

    let order = order_confirmed(10, 2, 10_000);
    let status = status_row(2, "confirmed");

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![order]])
        .append_query_results(vec![vec![status]])
        .into_connection();

    let txn = db.begin().await.expect("begin");
    let req = Request::new(ResolveNeedsReviewRequest {
        order_id: 10,
        resolution: "paid".to_string(),
        actor_id: "admin_1".to_string(),
    });

    let result = resolve_needs_review(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::FailedPrecondition);
}

#[tokio::test]
async fn resolve_needs_review_rejects_invalid_resolution() {
    use core_operations::handlers::orders::resolve_needs_review;

    let order = order_confirmed(10, 99, 10_000);
    let status = status_row(99, "needs_review");

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![order]])
        .append_query_results(vec![vec![status]])
        .into_connection();

    let txn = db.begin().await.expect("begin");
    let req = Request::new(ResolveNeedsReviewRequest {
        order_id: 10,
        resolution: "invalid".to_string(),
        actor_id: "admin_1".to_string(),
    });

    let result = resolve_needs_review(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn search_refund_attempts_returns_matching_rows() {
    use core_operations::handlers::refunds::search_refund_attempts;

    let attempt = refund_attempt(7, 42, "needs_review");
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![attempt]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(SearchRefundAttemptsRequest {
        attempt_id: None,
        order_id: Some(42),
        status: Some("needs_review".to_string()),
    });

    let result = search_refund_attempts(&txn, req).await;
    assert!(result.is_ok(), "{result:?}");
    let items = result.unwrap().into_inner().items;
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].attempt_id, 7);
    assert_eq!(items[0].order_id, 42);
    assert_eq!(items[0].status, "needs_review");
    assert_eq!(items[0].provider_error.as_deref(), Some("gateway timeout"));
}

#[tokio::test]
async fn search_refund_attempts_returns_empty_when_none_match() {
    use core_operations::handlers::refunds::search_refund_attempts;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<refund_attempts::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(SearchRefundAttemptsRequest {
        attempt_id: None,
        order_id: Some(999),
        status: None,
    });

    let result = search_refund_attempts(&txn, req).await;
    assert!(result.is_ok(), "{result:?}");
    assert!(result.unwrap().into_inner().items.is_empty());
}

#[tokio::test]
async fn resolve_refund_attempt_needs_review_not_found() {
    use core_operations::handlers::refunds::resolve_refund_attempt_needs_review;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<refund_attempts::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(ResolveRefundAttemptNeedsReviewRequest {
        attempt_id: 999,
        resolution: "retry".to_string(),
        actor_id: "admin_1".to_string(),
    });

    let result = resolve_refund_attempt_needs_review(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn resolve_refund_attempt_needs_review_rejects_non_needs_review() {
    use core_operations::handlers::refunds::resolve_refund_attempt_needs_review;

    let attempt = refund_attempt(1, 10, "processed");

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![attempt]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(ResolveRefundAttemptNeedsReviewRequest {
        attempt_id: 1,
        resolution: "retry".to_string(),
        actor_id: "admin_1".to_string(),
    });

    let result = resolve_refund_attempt_needs_review(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::FailedPrecondition);
}

#[tokio::test]
async fn resolve_refund_attempt_needs_review_rejects_invalid_resolution() {
    use core_operations::handlers::refunds::resolve_refund_attempt_needs_review;

    let attempt = refund_attempt(1, 10, "needs_review");

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![attempt]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(ResolveRefundAttemptNeedsReviewRequest {
        attempt_id: 1,
        resolution: "bogus".to_string(),
        actor_id: "admin_1".to_string(),
    });

    let result = resolve_refund_attempt_needs_review(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn resolve_refund_attempt_needs_review_retry_resets_attempt_for_worker() {
    use core_operations::handlers::refunds::resolve_refund_attempt_needs_review;

    let attempt = refund_attempt(1, 10, "needs_review");

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![attempt]])
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![order_event_row(1, 10)]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(ResolveRefundAttemptNeedsReviewRequest {
        attempt_id: 1,
        resolution: "retry".to_string(),
        actor_id: "admin_1".to_string(),
    });

    let result = resolve_refund_attempt_needs_review(&txn, req).await;
    assert!(result.is_ok(), "retry should succeed: {:?}", result.err());
    let resp = result.unwrap().into_inner();
    assert!(resp.success);
    assert!(
        resp.message.contains("pending_external") || resp.message.to_lowercase().contains("retry"),
        "message should describe the retry reset: {}",
        resp.message
    );

    txn.commit().await.expect("commit");
    let logs = db.into_transaction_log();
    let sql: Vec<String> = logs
        .iter()
        .flat_map(|txn| txn.statements().iter().map(|stmt| stmt.sql.to_lowercase()))
        .collect();
    let reset_stmt = sql
        .iter()
        .find(|s| s.contains("update refundattempts") && s.contains("attempt_id = ?"))
        .expect("expected the RefundAttempts reset UPDATE to run");
    assert!(
        reset_stmt.contains("attempt_count = 0"),
        "retry must reset attempt_count so the worker retries fresh: {}",
        reset_stmt
    );
}

#[tokio::test]
async fn resolve_refund_attempt_needs_review_mark_settled_does_not_touch_attempt_count() {
    use core_operations::handlers::refunds::resolve_refund_attempt_needs_review;

    let attempt = refund_attempt(2, 11, "needs_review");

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![attempt]])
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 2,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![order_event_row(2, 11)]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(ResolveRefundAttemptNeedsReviewRequest {
        attempt_id: 2,
        resolution: "mark_settled".to_string(),
        actor_id: "admin_2".to_string(),
    });

    let result = resolve_refund_attempt_needs_review(&txn, req).await;
    assert!(
        result.is_ok(),
        "mark_settled should succeed: {:?}",
        result.err()
    );
    assert!(result.unwrap().into_inner().success);

    txn.commit().await.expect("commit");
    let logs = db.into_transaction_log();
    let sql: Vec<String> = logs
        .iter()
        .flat_map(|txn| txn.statements().iter().map(|stmt| stmt.sql.to_lowercase()))
        .collect();
    let settle_stmt = sql
        .iter()
        .find(|s| s.contains("update refundattempts") && s.contains("attempt_id = ?"))
        .expect("expected the RefundAttempts settle UPDATE to run");
    assert!(
        !settle_stmt.contains("attempt_count"),
        "mark_settled must not reset attempt_count (it's a manual resolution, not a retry): {}",
        settle_stmt
    );
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_resolve_refund_attempt_needs_review_not_found() {
    use core_operations::handlers::refunds::resolve_refund_attempt_needs_review;
    use sea_orm::{Database, TransactionTrait};

    let db = Database::connect(&integration_common::test_db_url())
        .await
        .expect("connect");
    let txn = db.begin().await.expect("begin");
    let req = Request::new(ResolveRefundAttemptNeedsReviewRequest {
        attempt_id: 999_999,
        resolution: "retry".to_string(),
        actor_id: "admin_test".to_string(),
    });
    let result = resolve_refund_attempt_needs_review(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_create_refund_order_not_found() {
    use core_operations::handlers::refunds::create_refund;
    use sea_orm::{Database, TransactionTrait};

    let db = Database::connect(&integration_common::test_db_url())
        .await
        .expect("connect");
    let txn = db.begin().await.expect("begin");
    let req = Request::new(CreateRefundRequest {
        order_id: 999_999,
        gateway_refund_id: "gw_inexistent".to_string(),
        amount_paise: 100,
        currency: None,
        line_items_refunded_json: None,
    });
    let result = create_refund(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_resolve_needs_review_order_not_found() {
    use core_operations::handlers::orders::resolve_needs_review;
    use sea_orm::{Database, TransactionTrait};

    let db = Database::connect(&integration_common::test_db_url())
        .await
        .expect("connect");
    let txn = db.begin().await.expect("begin");
    let req = Request::new(ResolveNeedsReviewRequest {
        order_id: 999_999,
        resolution: "paid".to_string(),
        actor_id: "admin_test".to_string(),
    });
    let result = resolve_needs_review(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}
