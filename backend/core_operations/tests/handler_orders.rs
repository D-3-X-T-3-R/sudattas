//! Unit tests for order handlers using SeaORM MockDatabase.

use core_db_entities::entity::sea_orm_active_enums::FulfillmentStatus;
use core_db_entities::entity::{order_details, order_status, orders, shipments};
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
        public_order_ref: "SUD-20990101-PLACEHOLDER".to_string(),
        user_id: 7,
        order_date: now,
        created_at: now,
        shipping_address_id: 11,
        total_amount: Some(Decimal::new(10_000, 2)),
        status_id: 2,
        payment_status: None,
        payment_method: None,
        currency: Some("INR".to_string()),
        updated_at: None,
        subtotal_minor: 8_000,
        items_total_minor_before_discount: Some(8_000),
        shipping_minor: Some(1_000),
        shipping_charge_minor: Some(1_000),
        tax_total_minor: Some(500),
        discount_total_minor: Some(500),
        items_total_minor_after_discount: Some(7_500),
        grand_total_minor: 9_000,
        applied_coupon_id: Some(1),
        applied_coupon_code: Some("SAVE10".to_string()),
        applied_discount_paise: Some(1_000),
        refund_settlement_status: None,
        fulfillment_status: FulfillmentStatus::NotCreated,
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
    assert!(
        o.public_order_ref.starts_with("SUD-") && o.public_order_ref.len() >= 18,
        "unexpected public_order_ref: {}",
        o.public_order_ref
    );
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
        public_order_ref: "SUD-20990101-HUPDILL01".to_string(),
        user_id: 7,
        order_date: now,
        created_at: now,
        shipping_address_id: 11,
        total_amount: Some(Decimal::new(10_000, 2)),
        status_id: 1, // from_status_id
        payment_status: None,
        payment_method: None,
        currency: Some("INR".to_string()),
        updated_at: None,
        subtotal_minor: 8_000,
        items_total_minor_before_discount: Some(8_000),
        shipping_minor: Some(1_000),
        shipping_charge_minor: Some(1_000),
        tax_total_minor: Some(500),
        discount_total_minor: Some(500),
        items_total_minor_after_discount: Some(7_500),
        grand_total_minor: 9_000,
        applied_coupon_id: None,
        applied_coupon_code: None,
        applied_discount_paise: None,
        refund_settlement_status: None,
        fulfillment_status: FulfillmentStatus::NotCreated,
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
async fn update_order_preserves_original_order_date() {
    use core_operations::handlers::orders::update_order;

    let original_order_date = chrono::Utc::now() - chrono::Duration::days(5);
    let existing_order = orders::Model {
        order_id: 1,
        order_number: Some("ORD-1".to_string()),
        public_order_ref: "SUD-20990101-HUPDILL02".to_string(),
        user_id: 7,
        order_date: original_order_date,
        created_at: original_order_date,
        shipping_address_id: 11,
        total_amount: Some(Decimal::new(10_000, 2)),
        status_id: 1,
        payment_status: None,
        payment_method: None,
        currency: Some("INR".to_string()),
        updated_at: None,
        subtotal_minor: 8_000,
        items_total_minor_before_discount: Some(8_000),
        shipping_minor: Some(1_000),
        shipping_charge_minor: Some(1_000),
        tax_total_minor: Some(500),
        discount_total_minor: Some(500),
        items_total_minor_after_discount: Some(7_500),
        grand_total_minor: 9_000,
        applied_coupon_id: None,
        applied_coupon_code: None,
        applied_discount_paise: None,
        refund_settlement_status: None,
        fulfillment_status: FulfillmentStatus::NotCreated,
    };

    let from_status = order_status::Model {
        status_id: 1,
        status_name: "pending".to_string(),
    };
    let to_status = order_status::Model {
        status_id: 6,
        status_name: "cancelled".to_string(),
    };
    let updated_order = orders::Model {
        status_id: 6,
        order_date: original_order_date,
        ..existing_order.clone()
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![existing_order]])
        .append_query_results(vec![vec![from_status.clone()]])
        .append_query_results(vec![vec![to_status.clone()]])
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![updated_order]])
        .append_query_results(vec![vec![to_status.clone()]])
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .append_query_results(vec![Vec::<order_details::Model>::new()])
        .append_query_results(vec![vec![from_status.clone()]])
        .append_query_results(vec![vec![to_status.clone()]])
        .append_query_results(vec![Vec::<order_status::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(UpdateOrderRequest {
        order_id: 1,
        user_id: 7,
        shipping_address_id: 11,
        total_amount_paise: 9_000,
        status_id: 6,
    });

    let result = update_order(&txn, req).await.expect("update_order");
    let OrdersResponse { items } = result.into_inner();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].order_date, original_order_date.to_string());
}

#[tokio::test]
async fn delete_order_not_found_yields_not_found_status() {
    use core_operations::handlers::orders::delete_order;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<orders::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(DeleteOrderRequest {
        order_id: 999,
        acting_user_id: None,
    });
    let result = delete_order(&txn, req).await;
    assert!(result.is_err());
    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn delete_order_acting_user_mismatch_yields_not_found() {
    use core_operations::handlers::orders::delete_order;

    let now = chrono::Utc::now();
    let model = orders::Model {
        order_id: 5,
        order_number: Some("ORD-5".to_string()),
        public_order_ref: "SUD-20990101-HDELMIS01".to_string(),
        user_id: 3,
        order_date: now,
        created_at: now,
        shipping_address_id: 20,
        total_amount: Some(Decimal::new(5_000, 2)),
        status_id: 1,
        payment_status: None,
        payment_method: None,
        currency: Some("INR".to_string()),
        updated_at: None,
        subtotal_minor: 4_000,
        items_total_minor_before_discount: Some(4_000),
        shipping_minor: Some(1_000),
        shipping_charge_minor: Some(1_000),
        tax_total_minor: None,
        discount_total_minor: None,
        items_total_minor_after_discount: Some(4_000),
        grand_total_minor: 5_000,
        applied_coupon_id: None,
        applied_coupon_code: None,
        applied_discount_paise: None,
        refund_settlement_status: None,
        fulfillment_status: FulfillmentStatus::NotCreated,
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(DeleteOrderRequest {
        order_id: 5,
        acting_user_id: Some(99),
    });
    let result = delete_order(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn delete_order_when_already_cancelled_returns_snapshot() {
    use core_operations::handlers::orders::delete_order;

    let now = chrono::Utc::now();
    let cancelled_sid = 7_i64;
    let model = orders::Model {
        order_id: 5,
        order_number: Some("ORD-5".to_string()),
        public_order_ref: "SUD-20990101-HDELCAN01".to_string(),
        user_id: 3,
        order_date: now,
        created_at: now,
        shipping_address_id: 20,
        total_amount: Some(Decimal::new(5_000, 2)),
        status_id: cancelled_sid,
        payment_status: None,
        payment_method: None,
        currency: Some("INR".to_string()),
        updated_at: None,
        subtotal_minor: 4_000,
        items_total_minor_before_discount: Some(4_000),
        shipping_minor: Some(1_000),
        shipping_charge_minor: Some(1_000),
        tax_total_minor: None,
        discount_total_minor: None,
        items_total_minor_after_discount: Some(4_000),
        grand_total_minor: 5_000,
        applied_coupon_id: None,
        applied_coupon_code: None,
        applied_discount_paise: None,
        refund_settlement_status: None,
        fulfillment_status: FulfillmentStatus::NotCreated,
    };
    let st = order_status::Model {
        status_id: cancelled_sid,
        status_name: "cancelled".to_string(),
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![model.clone()]])
        .append_query_results(vec![Vec::<shipments::Model>::new()])
        .append_query_results(vec![vec![st]])
        .append_query_results(vec![Vec::<order_status::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(DeleteOrderRequest {
        order_id: 5,
        acting_user_id: None,
    });
    let result = delete_order(&txn, req).await;
    assert!(result.is_ok(), "{result:?}");
    let OrdersResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].order_id, 5);
    assert_eq!(items[0].status_id, cancelled_sid);
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
        shiprocket_book: None,
        shiprocket_order_id: None,
        shiprocket_status_id: None,
        shiprocket_status_label: None,
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
