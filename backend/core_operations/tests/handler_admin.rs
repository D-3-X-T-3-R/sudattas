//! P1 Admin operations: order search (optional user_id), coupon create/update, audit search.

mod integration_common;

use core_db_entities::entity::coupons;
use core_db_entities::entity::{order_events, orders};
use proto::proto::core::{
    CreateCouponRequest, SearchOrderEventsRequest, SearchOrderRequest, UpdateCouponRequest,
};
use sea_orm::{DatabaseBackend, MockDatabase, TransactionTrait};
use tonic::Request;

#[tokio::test]
async fn search_order_accepts_optional_user_id() {
    use core_operations::handlers::orders::search_order;

    let empty_orders: Vec<orders::Model> = vec![];
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![empty_orders])
        .into_connection();
    let txn = db.begin().await.expect("begin");
    let req = Request::new(SearchOrderRequest {
        order_id: None,
        user_id: None,
        order_date_start: None,
        order_date_end: None,
        status_id: None,
        limit: Some(10),
        offset: None,
    });
    let result = search_order(&txn, req).await;
    assert!(
        result.is_ok(),
        "search_order with no user_id (admin) should succeed: {:?}",
        result.err()
    );
    assert!(result.unwrap().into_inner().items.is_empty());
}

#[tokio::test]
async fn create_coupon_rejects_invalid_discount_type() {
    use core_operations::handlers::coupons::create_coupon;

    let db = MockDatabase::new(DatabaseBackend::MySql).into_connection();
    let txn = db.begin().await.expect("begin");
    let req = Request::new(CreateCouponRequest {
        code: "TEST10".to_string(),
        discount_type: "invalid".to_string(),
        discount_value: 10,
        min_order_value_paise: None,
        usage_limit: None,
        max_uses_per_customer: None,
        starts_at: "2025-01-01T00:00:00Z".to_string(),
        ends_at: None,
    });
    let result = create_coupon(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn search_order_events_returns_list() {
    use core_operations::handlers::order_events::search_order_events;

    let empty_events: Vec<order_events::Model> = vec![];
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![empty_events])
        .into_connection();
    let txn = db.begin().await.expect("begin");
    let req = Request::new(SearchOrderEventsRequest {
        order_id: None,
        limit: Some(10),
        offset: Some(0),
    });
    let result = search_order_events(&txn, req).await;
    assert!(result.is_ok());
    assert!(result.unwrap().into_inner().items.is_empty());
}

#[tokio::test]
async fn update_coupon_rejects_invalid_status() {
    use core_db_entities::entity::sea_orm_active_enums::{CouponStatus, DiscountType};
    use core_operations::handlers::coupons::update_coupon;

    let coupon = coupons::Model {
        coupon_id: 1,
        code: "TEST".to_string(),
        discount_type: DiscountType::Percentage,
        discount_value: 10,
        min_order_value_paise: None,
        usage_limit: None,
        usage_count: Some(0),
        max_uses_per_customer: None,
        coupon_status: Some(CouponStatus::Active),
        starts_at: chrono::Utc::now(),
        ends_at: None,
        created_at: None,
    };
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![coupon]])
        .into_connection();
    let txn = db.begin().await.expect("begin");
    let req = Request::new(UpdateCouponRequest {
        coupon_id: 1,
        status: Some("invalid".to_string()),
        usage_limit: None,
        ends_at: None,
    });
    let result = update_coupon(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::InvalidArgument);
}
