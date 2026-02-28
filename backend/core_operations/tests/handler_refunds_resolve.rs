//! P1 Tests for create_refund (idempotent, invalid state, partial/full) and resolve_needs_review.

mod integration_common;

use core_db_entities::entity::order_status;
use core_db_entities::entity::orders;
use core_db_entities::entity::refunds;
use core_db_entities::entity::sea_orm_active_enums::Status as RefundStatus;
use proto::proto::core::{CreateRefundRequest, ResolveNeedsReviewRequest};
use sea_orm::entity::prelude::Decimal;
use sea_orm::{DatabaseBackend, MockDatabase, TransactionTrait};
use tonic::Request;

fn order_confirmed(id: i64, status_id: i64, grand_total_minor: i64) -> orders::Model {
    orders::Model {
        order_id: id,
        order_number: Some(format!("ORD-{}", id)),
        user_id: 1,
        order_date: chrono::Utc::now(),
        shipping_address_id: 1,
        total_amount: Decimal::new(grand_total_minor, 2),
        status_id,
        payment_status: None,
        currency: Some("INR".to_string()),
        updated_at: None,
        subtotal_minor: Some(grand_total_minor),
        shipping_minor: Some(0),
        tax_total_minor: Some(0),
        discount_total_minor: Some(0),
        grand_total_minor: Some(grand_total_minor),
        applied_coupon_id: None,
        applied_coupon_code: None,
        applied_discount_paise: None,
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
