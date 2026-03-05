//! Unit tests for order handlers using SeaORM MockDatabase.

use core_db_entities::entity::{order_status, orders};
use proto::proto::core::{
    AdminMarkOrderDeliveredRequest, AdminMarkOrderShippedRequest, CreateOrderRequest,
    DeleteOrderRequest, OrdersResponse, UpdateOrderRequest,
};
use rust_decimal::Decimal;
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, TransactionTrait};
use tonic::Request;

#[tokio::test]
async fn create_order_inserts_and_returns_created_model() {
    use core_operations::handlers::orders::create_order;

    let now = chrono::Utc::now();
    let model = orders::Model {
        order_id: 1,
        order_number: Some("ORD-1".to_string()),
        user_id: 7,
        order_date: now,
        shipping_address_id: 11,
        total_amount: Some(Decimal::new(10_000, 2)),
        status_id: 2,
        payment_status: None,
        payment_method: None,
        currency: Some("INR".to_string()),
        updated_at: None,
        subtotal_minor: 8_000,
        shipping_minor: Some(1_000),
        tax_total_minor: Some(500),
        discount_total_minor: Some(500),
        grand_total_minor: 9_000,
        applied_coupon_id: Some(1),
        applied_coupon_code: Some("SAVE10".to_string()),
        applied_discount_paise: Some(1_000),
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(CreateOrderRequest {
        user_id: 7,
        shipping_address_id: 11,
        status_id: 2,
        total_amount_paise: 9_000,
        subtotal_minor: Some(8_000),
        shipping_minor: Some(1_000),
        tax_total_minor: Some(500),
        discount_total_minor: Some(500),
        grand_total_minor: Some(9_000),
        applied_coupon_id: Some(1),
        applied_coupon_code: Some("SAVE10".to_string()),
        applied_discount_paise: Some(1_000),
    });

    let result = create_order(&txn, req).await;
    assert!(result.is_ok());
    let OrdersResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    let o = &items[0];
    assert_eq!(o.order_id, 1);
    assert_eq!(o.user_id, 7);
    assert_eq!(o.shipping_address_id, 11);
    assert_eq!(o.status_id, 2);
    assert_eq!(o.total_amount_paise, 9_000);
}

#[tokio::test]
async fn update_order_not_found_yields_not_found_status() {
    use core_operations::handlers::orders::update_order;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<orders::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(UpdateOrderRequest {
        order_id: 99,
        user_id: 7,
        shipping_address_id: 11,
        total_amount_paise: 9_000,
        status_id: 2,
    });

    let result = update_order(&txn, req).await;
    assert!(result.is_err());
    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn update_order_illegal_state_transition_returns_invalid_argument() {
    use core_operations::handlers::orders::update_order;

    let now = chrono::Utc::now();
    let existing_order = orders::Model {
        order_id: 1,
        order_number: Some("ORD-1".to_string()),
        user_id: 7,
        order_date: now,
        shipping_address_id: 11,
        total_amount: Some(Decimal::new(10_000, 2)),
        status_id: 1, // from_status_id
        payment_status: None,
        payment_method: None,
        currency: Some("INR".to_string()),
        updated_at: None,
        subtotal_minor: 8_000,
        shipping_minor: Some(1_000),
        tax_total_minor: Some(500),
        discount_total_minor: Some(500),
        grand_total_minor: 9_000,
        applied_coupon_id: None,
        applied_coupon_code: None,
        applied_discount_paise: None,
    };

    let from_status = order_status::Model {
        status_id: 1,
        status_name: "pending".to_string(),
    };
    let to_status = order_status::Model {
        status_id: 2,
        status_name: "processing".to_string(),
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![existing_order]])
        .append_query_results(vec![vec![from_status]])
        .append_query_results(vec![vec![to_status]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(UpdateOrderRequest {
        order_id: 1,
        user_id: 7,
        shipping_address_id: 11,
        total_amount_paise: 9_000,
        status_id: 2,
    });

    let result = update_order(&txn, req).await;
    assert!(result.is_err());
    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::InvalidArgument);
    assert!(
        status.message().contains("Illegal order state transition"),
        "expected illegal transition message, got {}",
        status.message()
    );
}

#[tokio::test]
async fn delete_order_deletes_existing_and_returns_response() {
    use core_operations::handlers::orders::delete_order;

    let now = chrono::Utc::now();
    let model = orders::Model {
        order_id: 5,
        order_number: Some("ORD-5".to_string()),
        user_id: 3,
        order_date: now,
        shipping_address_id: 20,
        total_amount: Some(Decimal::new(5_000, 2)),
        status_id: 1,
        payment_status: None,
        payment_method: None,
        currency: Some("INR".to_string()),
        updated_at: None,
        subtotal_minor: 4_000,
        shipping_minor: Some(1_000),
        tax_total_minor: None,
        discount_total_minor: None,
        grand_total_minor: 5_000,
        applied_coupon_id: None,
        applied_coupon_code: None,
        applied_discount_paise: None,
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![model]])
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(DeleteOrderRequest { order_id: 5 });
    let result = delete_order(&txn, req).await;
    assert!(result.is_ok());
    let OrdersResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    let o = &items[0];
    assert_eq!(o.order_id, 5);
    assert_eq!(o.user_id, 3);
    assert_eq!(o.shipping_address_id, 20);
}

#[tokio::test]
async fn delete_order_not_found_yields_not_found_status() {
    use core_operations::handlers::orders::delete_order;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<orders::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(DeleteOrderRequest { order_id: 999 });
    let result = delete_order(&txn, req).await;
    assert!(result.is_err());
    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn admin_mark_order_shipped_order_not_found_propagates_not_found() {
    use core_operations::handlers::orders::admin_mark_shipped::admin_mark_order_shipped;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<orders::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(AdminMarkOrderShippedRequest {
        order_id: 123,
        awb_code: None,
        carrier: None,
    });
    let result = admin_mark_order_shipped(&txn, req).await;
    assert!(result.is_err());
    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn admin_mark_order_delivered_order_not_found_propagates_not_found() {
    use core_operations::handlers::orders::admin_mark_delivered::admin_mark_order_delivered;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<orders::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(AdminMarkOrderDeliveredRequest { order_id: 123 });
    let result = admin_mark_order_delivered(&txn, req).await;
    assert!(result.is_err());
    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::NotFound);
}
