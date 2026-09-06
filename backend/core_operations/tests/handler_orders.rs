//! Unit tests for order handlers using SeaORM MockDatabase.

use core_db_entities::entity::sea_orm_active_enums::FulfillmentStatus;
use core_db_entities::entity::{order_details, order_status, orders};
use proto::proto::core::{
    AdminMarkOrderDeliveredRequest, AdminMarkOrderShippedRequest, CancelOrderItemsRequest,
    CreateOrderRequest, DeleteOrderRequest, OrdersResponse, UpdateOrderRequest,
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
        cancel_window_ends_at: None,
        earliest_booking_at: None,
        pickup_target_at: None,
        pickup_target_reason: None,
        pickup_target_set_by: None,
        pickup_target_updated_at: None,
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
        invoice_id: None,
        invoice_number: None,
        invoice_generated_at: None,
        invoice_storage_path: None,
        applied_coupon_id: Some(1),
        applied_coupon_code: Some("SAVE10".to_string()),
        applied_discount_paise: Some(1_000),
        refund_settlement_status: None,
        fulfillment_status: FulfillmentStatus::NotCreated,
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![
            MockExecResult {
                last_insert_id: 1,
                rows_affected: 1,
            },
            MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            },
        ])
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
        payment_method: "cod".to_string(),
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
        cancel_window_ends_at: Some(now + chrono::Duration::hours(1)),
        earliest_booking_at: None,
        pickup_target_at: None,
        pickup_target_reason: None,
        pickup_target_set_by: None,
        pickup_target_updated_at: None,
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
        invoice_id: None,
        invoice_number: None,
        invoice_generated_at: None,
        invoice_storage_path: None,
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
        cancel_window_ends_at: None,
        earliest_booking_at: None,
        pickup_target_at: None,
        pickup_target_reason: None,
        pickup_target_set_by: None,
        pickup_target_updated_at: None,
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
        invoice_id: None,
        invoice_number: None,
        invoice_generated_at: None,
        invoice_storage_path: None,
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
        cancel_window_ends_at: None,
        earliest_booking_at: None,
        pickup_target_at: None,
        pickup_target_reason: None,
        pickup_target_set_by: None,
        pickup_target_updated_at: None,
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
        invoice_id: None,
        invoice_number: None,
        invoice_generated_at: None,
        invoice_storage_path: None,
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
        cancel_window_ends_at: None,
        earliest_booking_at: None,
        pickup_target_at: None,
        pickup_target_reason: None,
        pickup_target_set_by: None,
        pickup_target_updated_at: None,
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
        invoice_id: None,
        invoice_number: None,
        invoice_generated_at: None,
        invoice_storage_path: None,
        applied_coupon_id: None,
        applied_coupon_code: None,
        applied_discount_paise: None,
        refund_settlement_status: None,
        fulfillment_status: FulfillmentStatus::NotCreated,
    };
    let mut cancel_window_row = std::collections::BTreeMap::new();
    cancel_window_row.insert(
        "cancel_window_ends_at",
        (now + chrono::Duration::hours(1)).into(),
    );

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![model.clone()]])
        .append_query_results(vec![vec![cancel_window_row]])
        .append_query_results(vec![Vec::<order_details::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(DeleteOrderRequest {
        order_id: 5,
        // Customer path (matches model.user_id: 3) — deliberately not the admin/service path
        // (acting_user_id: None), since admin now skips the cancel-window query this test mocks
        // entirely (see the admin-override tests below); this test is about the "no active items
        // left to cancel" early-return behavior, not the admin/customer distinction.
        acting_user_id: Some(3),
    });
    let result = delete_order(&txn, req).await;
    assert!(result.is_ok(), "{result:?}");
    let OrdersResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].order_id, 5);
    assert_eq!(items[0].status_id, cancelled_sid);
}

/// A minimal order past `NotCreated` fulfillment (i.e. a shipment has been booked) but not yet
/// delivered — the case the admin-cancel-window bypass is meant to unlock.
fn shipped_order_model(order_id: i64, user_id: i64, status_id: i64) -> orders::Model {
    let now = chrono::Utc::now();
    orders::Model {
        order_id,
        order_number: Some(format!("ORD-{order_id}")),
        public_order_ref: format!("SUD-20990101-SHIP{order_id:04}"),
        user_id,
        order_date: now,
        // Cancel window is already closed — the whole point of these tests is that admin must
        // be able to bypass this, not that it happens to still be open.
        created_at: now - chrono::Duration::days(30),
        cancel_window_ends_at: Some(now - chrono::Duration::days(29)),
        earliest_booking_at: None,
        pickup_target_at: None,
        pickup_target_reason: None,
        pickup_target_set_by: None,
        pickup_target_updated_at: None,
        shipping_address_id: 20,
        total_amount: Some(Decimal::new(5_000, 2)),
        status_id,
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
        invoice_id: None,
        invoice_number: None,
        invoice_generated_at: None,
        invoice_storage_path: None,
        applied_coupon_id: None,
        applied_coupon_code: None,
        applied_discount_paise: None,
        refund_settlement_status: None,
        fulfillment_status: FulfillmentStatus::Booked,
    }
}

#[tokio::test]
async fn cancel_order_items_customer_still_blocked_by_shipment_in_progress() {
    use core_operations::handlers::orders::cancel_order_items;

    let order = shipped_order_model(5, 3, 2);
    let status_row = order_status::Model {
        status_id: 2,
        status_name: "shipped".to_string(),
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![order]])
        .append_query_results(vec![vec![status_row]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    // Customer path (acting_user_id matches order.user_id) — must be entirely unaffected by the
    // admin bypass: no shipment/logistics query should even be attempted.
    let req = Request::new(CancelOrderItemsRequest {
        order_id: 5,
        order_detail_ids: vec![1],
        acting_user_id: Some(3),
    });
    let result = cancel_order_items(&txn, req).await;
    let status = result.expect_err("customer cancel of a shipped order must still be blocked");
    assert_eq!(status.code(), tonic::Code::FailedPrecondition);
    assert!(status.message().contains("Cancellation window closed"));
}

#[tokio::test]
async fn cancel_order_items_admin_still_blocked_once_pickup_is_in_progress() {
    use core_operations::handlers::orders::cancel_order_items;

    let order = shipped_order_model(5, 3, 2);
    let status_row = order_status::Model {
        status_id: 2,
        status_name: "shipped".to_string(),
    };
    let mut shipment_row = std::collections::BTreeMap::new();
    shipment_row.insert("shipment_id", 42_i64.into());
    shipment_row.insert("order_id", 5_i64.into());
    // A real Shiprocket order exists — this test is specifically about the pickup-in-progress
    // cutoff, not the "no provider reference at all" case (see
    // cancel_order_via_logistics_returns_none_when_no_provider_reference in
    // handler_shipments_coupons.rs for that one).
    shipment_row.insert("shiprocket_order_id", "SR-9001".to_string().into());
    shipment_row.insert("logistics_status", "pickup_scheduled".to_string().into());
    // Pickup already in progress — the same cutoff that disables a customer's own cancel button.
    shipment_row.insert("can_customer_cancel", false.into());

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![order]])
        .append_query_results(vec![vec![status_row]])
        .append_query_results(vec![vec![shipment_row]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(CancelOrderItemsRequest {
        order_id: 5,
        order_detail_ids: vec![1],
        acting_user_id: None,
    });
    let result = cancel_order_items(&txn, req).await;
    let status = result.expect_err("admin must not force-cancel past an in-progress pickup either");
    assert_eq!(status.code(), tonic::Code::FailedPrecondition);
    assert!(status
        .message()
        .contains("pickup/logistics is already in progress"));
}

#[tokio::test]
async fn admin_mark_order_shipped_order_not_found_returns_booking_precondition() {
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
    let result = admin_mark_order_shipped(&txn, &db, req).await;
    assert!(result.is_err());
    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::FailedPrecondition);
    assert!(
        status.message().contains("Shipment is not booked yet"),
        "unexpected message: {}",
        status.message()
    );
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

#[tokio::test]
async fn create_order_invalid_payment_method_returns_invalid_argument() {
    use core_operations::handlers::orders::create_order;

    let db = MockDatabase::new(DatabaseBackend::MySql).into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(CreateOrderRequest {
        user_id: 7,
        shipping_address_id: 11,
        status_id: 2,
        total_amount_paise: 9_000,
        subtotal_minor: None,
        shipping_minor: None,
        tax_total_minor: None,
        discount_total_minor: None,
        grand_total_minor: None,
        applied_coupon_id: None,
        applied_coupon_code: None,
        applied_discount_paise: None,
        payment_method: "bank_transfer".to_string(),
    });

    let result = create_order(&txn, req).await;
    assert!(result.is_err());
    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::InvalidArgument);
    assert!(status.message().contains("cod") && status.message().contains("prepaid"));
}

#[tokio::test]
async fn create_order_prepaid_records_captured_immediately() {
    use core_operations::handlers::orders::create_order;

    let now = chrono::Utc::now();
    let model = orders::Model {
        order_id: 1,
        order_number: Some("ORD-1".to_string()),
        public_order_ref: "SUD-20990101-PLACEHOLDER".to_string(),
        user_id: 7,
        order_date: now,
        created_at: now,
        cancel_window_ends_at: None,
        earliest_booking_at: None,
        pickup_target_at: None,
        pickup_target_reason: None,
        pickup_target_set_by: None,
        pickup_target_updated_at: None,
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
        invoice_id: None,
        invoice_number: None,
        invoice_generated_at: None,
        invoice_storage_path: None,
        applied_coupon_id: None,
        applied_coupon_code: None,
        applied_discount_paise: None,
        refund_settlement_status: None,
        fulfillment_status: FulfillmentStatus::NotCreated,
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![
            MockExecResult {
                last_insert_id: 1,
                rows_affected: 1,
            },
            MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            },
        ])
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
        applied_coupon_id: None,
        applied_coupon_code: None,
        applied_discount_paise: None,
        payment_method: "PREPAID".to_string(),
    });

    let result = create_order(&txn, req).await;
    assert!(result.is_ok(), "err: {:?}", result.err());
    txn.commit().await.expect("commit");

    let logs = db.into_transaction_log();
    let inserts: Vec<String> = logs
        .iter()
        .flat_map(|t| t.statements().iter().map(|s| format!("{:?}", s)))
        .filter(|s| s.to_lowercase().contains("insert into"))
        .collect();
    assert!(
        inserts.iter().any(|s| s.contains("prepaid") && s.contains("captured")),
        "expected the INSERT to bind payment_method='prepaid' and payment_status='captured' (normalized/derived from case-insensitive input), got: {:?}",
        inserts
    );
}

#[tokio::test]
async fn place_order_admin_invalid_payment_method_returns_invalid_argument() {
    use core_operations::procedures::orders::place_order_admin;
    use proto::proto::core::{PlaceOrderAdminLineItem, PlaceOrderAdminRequest};

    let db = MockDatabase::new(DatabaseBackend::MySql).into_connection();

    let req = Request::new(PlaceOrderAdminRequest {
        user_id: 7,
        shipping_address_id: 11,
        payment_method: "bank_transfer".to_string(),
        line_items: vec![PlaceOrderAdminLineItem {
            variant_id: 1,
            quantity: 1,
            price_paise: 1000,
        }],
        shipping_minor: None,
        applied_coupon_id: None,
        applied_coupon_code: None,
        applied_discount_paise: None,
    });

    let result = place_order_admin(&db, req).await;
    assert!(result.is_err());
    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::InvalidArgument);
    assert!(status.message().contains("cod") && status.message().contains("prepaid"));
}

#[tokio::test]
async fn place_order_admin_no_line_items_returns_invalid_argument() {
    use core_operations::procedures::orders::place_order_admin;
    use proto::proto::core::PlaceOrderAdminRequest;

    let db = MockDatabase::new(DatabaseBackend::MySql).into_connection();

    let req = Request::new(PlaceOrderAdminRequest {
        user_id: 7,
        shipping_address_id: 11,
        payment_method: "cod".to_string(),
        line_items: vec![],
        shipping_minor: None,
        applied_coupon_id: None,
        applied_coupon_code: None,
        applied_discount_paise: None,
    });

    let result = place_order_admin(&db, req).await;
    assert!(result.is_err());
    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::InvalidArgument);
    assert!(
        status.message().contains("line item"),
        "unexpected message: {}",
        status.message()
    );
}
