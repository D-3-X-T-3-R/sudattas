//! Integration tests for prepaid post-delivery returns with partial-item support.
//!
//! Run with:
//! `cargo test --test integration_returns -- --ignored --test-threads=1 --nocapture`

mod integration_common;

use chrono::Utc;
use core_db_entities::entity::sea_orm_active_enums::{PaymentStatus, Status as DbStatus};
use core_db_entities::entity::{
    inventory, order_details, order_inventory_restore_items, order_status, orders, payment_intents,
    refund_attempts, refunds, return_request_items, return_requests, shipping_addresses,
    user_roles,
};
use core_operations::cancellation_saga;
use core_operations::order_state_machine;
use core_operations::procedures::orders::place_order;
use integration_common::test_db_url;
use proto::proto::core::{
    AdminMarkOrderDeliveredRequest, AdminMarkOrderShippedRequest, AdminMarkReturnReceivedRequest,
    CancelOrderItemsRequest, RequestReturnRequest, ReturnRequestItemInput, UpdateOrderRequest,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, Database, DatabaseBackend,
    DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, Statement, TransactionTrait,
};
use tonic::{Code, Request};

#[derive(Debug, Clone)]
struct OrderFixture {
    order_id: i64,
    user_id: i64,
    shipping_id: i64,
    total_paise: i64,
    order_detail_id: i64,
    order_detail_qty: i64,
    line_total_minor: i64,
    variant_id: i64,
}

async fn ensure_order_status(txn: &sea_orm::DatabaseTransaction, name: &str) -> i64 {
    if let Ok(Some(id)) = order_state_machine::get_status_id(txn, name).await {
        return id;
    }
    let row = order_status::ActiveModel {
        status_id: ActiveValue::NotSet,
        status_name: ActiveValue::Set(name.to_string()),
    }
    .insert(txn)
    .await
    .expect("insert order status");
    row.status_id
}

async fn ensure_return_tables(conn: &DatabaseConnection) {
    conn.execute(Statement::from_string(
        DatabaseBackend::MySql,
        r#"
CREATE TABLE IF NOT EXISTS ReturnRequests (
    return_id BIGINT NOT NULL AUTO_INCREMENT,
    order_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'requested',
    reason VARCHAR(512) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    received_at TIMESTAMP NULL DEFAULT NULL,
    refund_attempt_id BIGINT NULL,
    PRIMARY KEY (return_id),
    KEY idx_return_requests_order (order_id),
    KEY idx_return_requests_user (user_id),
    KEY idx_return_requests_status (status),
    KEY idx_return_requests_refund_attempt (refund_attempt_id),
    CONSTRAINT fk_return_requests_order
        FOREIGN KEY (order_id) REFERENCES Orders (OrderID),
    CONSTRAINT fk_return_requests_user
        FOREIGN KEY (user_id) REFERENCES Users (UserID),
    CONSTRAINT fk_return_requests_refund_attempt
        FOREIGN KEY (refund_attempt_id) REFERENCES RefundAttempts (attempt_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
        "#,
    ))
    .await
    .expect("ensure ReturnRequests table");

    conn.execute(Statement::from_string(
        DatabaseBackend::MySql,
        r#"
CREATE TABLE IF NOT EXISTS ReturnRequestItems (
    return_id BIGINT NOT NULL,
    order_detail_id BIGINT NOT NULL,
    quantity BIGINT NOT NULL,
    refund_amount_minor BIGINT NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'requested',
    PRIMARY KEY (return_id, order_detail_id),
    KEY idx_return_items_order_detail (order_detail_id),
    KEY idx_return_items_status (status),
    CONSTRAINT fk_return_items_return
        FOREIGN KEY (return_id) REFERENCES ReturnRequests (return_id)
        ON DELETE CASCADE,
    CONSTRAINT fk_return_items_order_detail
        FOREIGN KEY (order_detail_id) REFERENCES OrderDetails (OrderDetailID)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
        "#,
    ))
    .await
    .expect("ensure ReturnRequestItems table");
}

async fn make_order_booking_eligible(txn: &sea_orm::DatabaseTransaction, order_id: i64) {
    txn.execute(Statement::from_sql_and_values(
        DatabaseBackend::MySql,
        r#"UPDATE Orders
           SET earliest_booking_at = DATE_SUB(UTC_TIMESTAMP(), INTERVAL 1 HOUR),
               cancel_window_ends_at = DATE_SUB(UTC_TIMESTAMP(), INTERVAL 1 HOUR),
               payment_status = 'captured'
           WHERE OrderID = ?"#,
        [order_id.into()],
    ))
    .await
    .expect("make order booking eligible");
}

async fn place_order_fixture(
    txn: &sea_orm::DatabaseTransaction,
    tag: i64,
    payment_mode: Option<&str>,
    quantity: i64,
) -> OrderFixture {
    let _ = ensure_order_status(txn, "pending").await;

    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(format!("itest_returns_role_{tag}")),
    }
    .insert(txn)
    .await
    .expect("insert role");

    let user = core_operations::handlers::users::create_user(
        txn,
        Request::new(proto::proto::core::CreateUserRequest {
            username: format!("itest_returns_user_{tag}"),
            email: format!("itest_returns_{tag}@example.com"),
            auth_provider: "email".to_string(),
            password_plain: Some("StrongPass123!".to_string()),
            google_sub: None,
            full_name: None,
            address: None,
            phone: None,
            role_id: Some(role.role_id),
        }),
    )
    .await
    .expect("create user")
    .into_inner()
    .items
    .into_iter()
    .next()
    .expect("user response");
    let user_id = user.user_id;

    let shipping = shipping_addresses::ActiveModel {
        shipping_address_id: ActiveValue::NotSet,
        user_id: ActiveValue::Set(Some(user_id)),
        is_default: ActiveValue::Set(0),
        country: ActiveValue::Set("IN".to_string()),
        state_region: ActiveValue::Set("KA".to_string()),
        city: ActiveValue::Set("Bengaluru".to_string()),
        postal_code: ActiveValue::Set("560001".to_string()),
        road: ActiveValue::Set(Some("MG Road".to_string())),
        apartment_no_or_name: ActiveValue::Set(None),
        recipient_name: ActiveValue::Set(Some("Test User".to_string())),
        phone_number: ActiveValue::Set(Some("+919999999999".to_string())),
    }
    .insert(txn)
    .await
    .expect("insert shipping");

    let category = core_db_entities::entity::product_categories::ActiveModel {
        category_id: ActiveValue::NotSet,
        name: ActiveValue::Set(format!("itest_returns_cat_{tag}")),
    }
    .insert(txn)
    .await
    .expect("insert category");

    let product = core_db_entities::entity::products::ActiveModel {
        product_id: ActiveValue::NotSet,
        sku: ActiveValue::Set(None),
        name: ActiveValue::Set("Return test product".to_string()),
        slug: ActiveValue::Set(None),
        description: ActiveValue::Set(None),
        price_paise: ActiveValue::Set(150_000),
        category_id: ActiveValue::Set(category.category_id),
        fabric: ActiveValue::Set(None),
        weave: ActiveValue::Set(None),
        occasion: ActiveValue::Set(None),
        has_blouse_piece: ActiveValue::Set(None),
        care_instructions: ActiveValue::Set(None),
        product_status_id: ActiveValue::Set(None),
        created_at: ActiveValue::Set(Some(Utc::now())),
        updated_at: ActiveValue::Set(None),
    }
    .insert(txn)
    .await
    .expect("insert product");

    let variant = core_db_entities::entity::product_variants::ActiveModel {
        variant_id: ActiveValue::NotSet,
        product_id: ActiveValue::Set(product.product_id),
        size_id: ActiveValue::Set(None),
        color_id: ActiveValue::Set(None),
        additional_price: ActiveValue::Set(Some(0)),
    }
    .insert(txn)
    .await
    .expect("insert variant");

    inventory::ActiveModel {
        inventory_id: ActiveValue::NotSet,
        variant_id: ActiveValue::Set(Some(variant.variant_id)),
        quantity_available: ActiveValue::Set(Some(10)),
        quantity_reserved: ActiveValue::Set(Some(0)),
        reorder_level: ActiveValue::Set(None),
        updated_at: ActiveValue::Set(Some(Utc::now())),
    }
    .insert(txn)
    .await
    .expect("insert inventory");

    let cart_item = core_operations::handlers::cart::create_cart_item(
        txn,
        Request::new(proto::proto::core::CreateCartItemRequest {
            user_id: Some(user_id),
            variant_id: variant.variant_id,
            quantity,
            session_id: None,
        }),
    )
    .await
    .expect("create cart item")
    .into_inner()
    .items
    .into_iter()
    .next()
    .expect("cart row");

    let placed = place_order(
        txn,
        Request::new(proto::proto::core::PlaceOrderRequest {
            shipping_address_id: shipping.shipping_address_id,
            user_id,
            coupon_code: None,
            selected_cart_ids: vec![cart_item.cart_id],
            payment_mode: payment_mode.map(str::to_string),
        }),
    )
    .await
    .expect("place order")
    .into_inner()
    .items
    .into_iter()
    .next()
    .expect("order row");

    let detail = order_details::Entity::find()
        .filter(order_details::Column::OrderId.eq(placed.order_id))
        .one(txn)
        .await
        .expect("query order detail")
        .expect("order detail exists");

    OrderFixture {
        order_id: placed.order_id,
        user_id,
        shipping_id: shipping.shipping_address_id,
        total_paise: placed.total_amount_paise,
        order_detail_id: detail.order_detail_id,
        order_detail_qty: detail.quantity,
        line_total_minor: detail.line_total_minor,
        variant_id: detail.variant_id,
    }
}

async fn transition_order_to_delivered(txn: &sea_orm::DatabaseTransaction, fx: &OrderFixture) {
    let confirmed_id = ensure_order_status(txn, "confirmed").await;
    let processing_id = ensure_order_status(txn, "processing").await;

    core_operations::handlers::orders::update_order(
        txn,
        Request::new(UpdateOrderRequest {
            order_id: fx.order_id,
            user_id: fx.user_id,
            shipping_address_id: fx.shipping_id,
            total_amount_paise: fx.total_paise,
            status_id: confirmed_id,
        }),
    )
    .await
    .expect("order -> confirmed");

    core_operations::handlers::orders::update_order(
        txn,
        Request::new(UpdateOrderRequest {
            order_id: fx.order_id,
            user_id: fx.user_id,
            shipping_address_id: fx.shipping_id,
            total_amount_paise: fx.total_paise,
            status_id: processing_id,
        }),
    )
    .await
    .expect("order -> processing");

    make_order_booking_eligible(txn, fx.order_id).await;

    core_operations::handlers::orders::admin_mark_order_shipped(
        txn,
        Request::new(AdminMarkOrderShippedRequest {
            order_id: fx.order_id,
            awb_code: Some(format!("AWB-{}", fx.order_id)),
            carrier: Some("TestCarrier".to_string()),
            shiprocket_book: None,
            shiprocket_order_id: None,
            shiprocket_status_id: None,
            shiprocket_status_label: None,
        }),
    )
    .await
    .expect("order -> shipped");

    core_operations::handlers::orders::admin_mark_order_delivered(
        txn,
        Request::new(AdminMarkOrderDeliveredRequest {
            order_id: fx.order_id,
        }),
    )
    .await
    .expect("order -> delivered");
}

async fn ensure_captured_payment_intent(
    txn: &sea_orm::DatabaseTransaction,
    fx: &OrderFixture,
    tag: i64,
) -> i64 {
    let row = payment_intents::ActiveModel {
        intent_id: ActiveValue::NotSet,
        razorpay_order_id: ActiveValue::Set(format!("order_ret_{tag}")),
        order_id: ActiveValue::Set(Some(fx.order_id)),
        active_order_id: ActiveValue::NotSet,
        user_id: ActiveValue::Set(Some(fx.user_id)),
        amount_paise: ActiveValue::Set(i32::try_from(fx.total_paise).expect("amount fits i32")),
        currency: ActiveValue::Set(Some("INR".to_string())),
        status: ActiveValue::Set(DbStatus::Processed),
        razorpay_payment_id: ActiveValue::Set(Some(format!("pay_ret_{tag}"))),
        metadata: ActiveValue::Set(None),
        created_at: ActiveValue::Set(Some(Utc::now())),
        expires_at: ActiveValue::Set(Utc::now() + chrono::Duration::days(1)),
        gateway_fee_paise: ActiveValue::Set(None),
        gateway_tax_paise: ActiveValue::Set(None),
    }
    .insert(txn)
    .await
    .expect("insert processed payment intent");

    txn.execute(Statement::from_sql_and_values(
        DatabaseBackend::MySql,
        "UPDATE Orders SET payment_status = 'captured' WHERE OrderID = ?",
        [fx.order_id.into()],
    ))
    .await
    .expect("mark order payment captured");
    row.intent_id
}

async fn request_return_for_fixture(
    txn: &sea_orm::DatabaseTransaction,
    fx: &OrderFixture,
    quantity: i64,
    reason: &str,
) -> proto::proto::core::ReturnRequestResponse {
    core_operations::handlers::returns::request_return(
        txn,
        Request::new(RequestReturnRequest {
            order_id: fx.order_id,
            user_id: fx.user_id,
            items: vec![ReturnRequestItemInput {
                order_detail_id: fx.order_detail_id,
                quantity: Some(quantity),
            }],
            reason: reason.to_string(),
        }),
    )
    .await
    .expect("request return")
    .into_inner()
    .items
    .into_iter()
    .next()
    .expect("return row")
}

async fn inventory_available_for_variant(
    txn: &sea_orm::DatabaseTransaction,
    variant_id: i64,
) -> i64 {
    let row = inventory::Entity::find()
        .filter(inventory::Column::VariantId.eq(Some(variant_id)))
        .one(txn)
        .await
        .expect("query inventory")
        .expect("inventory row");
    row.quantity_available.unwrap_or(0)
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn prepaid_delivered_can_request_full_return_within_window_and_no_immediate_refund_attempt() {
    let db = Database::connect(&test_db_url()).await.expect("connect db");
    ensure_return_tables(&db).await;
    let txn = db.begin().await.expect("begin txn");
    let tag = Utc::now().timestamp_millis();
    let fx = place_order_fixture(&txn, tag, None, 1).await;
    transition_order_to_delivered(&txn, &fx).await;

    let ret = request_return_for_fixture(&txn, &fx, 1, "Size issue").await;
    assert_eq!(ret.status.to_lowercase(), "requested");
    assert_eq!(ret.items.len(), 1);
    assert_eq!(ret.items[0].quantity, 1);

    let attempts = refund_attempts::Entity::find()
        .filter(refund_attempts::Column::OrderId.eq(fx.order_id))
        .count(&txn)
        .await
        .expect("count refund attempts");
    assert_eq!(attempts, 0, "return request must not create refund attempt");
    txn.rollback().await.ok();
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn prepaid_delivered_partial_return_uses_line_net_total_and_no_shipping_refund_component() {
    let db = Database::connect(&test_db_url()).await.expect("connect db");
    ensure_return_tables(&db).await;
    let txn = db.begin().await.expect("begin txn");
    let tag = Utc::now().timestamp_millis();
    let fx = place_order_fixture(&txn, tag, None, 2).await;
    transition_order_to_delivered(&txn, &fx).await;

    let ret = request_return_for_fixture(&txn, &fx, 1, "Color mismatch").await;
    let item = ret.items.first().expect("return item");
    let expected_partial = fx.line_total_minor / fx.order_detail_qty.max(1);
    assert_eq!(item.refund_amount_minor, expected_partial);
    txn.rollback().await.ok();
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn cod_delivered_cannot_request_return() {
    let db = Database::connect(&test_db_url()).await.expect("connect db");
    ensure_return_tables(&db).await;
    let txn = db.begin().await.expect("begin txn");
    let tag = Utc::now().timestamp_millis();
    let fx = place_order_fixture(&txn, tag, None, 1).await;
    transition_order_to_delivered(&txn, &fx).await;
    txn.execute(Statement::from_sql_and_values(
        DatabaseBackend::MySql,
        "UPDATE Orders SET payment_method = 'cod' WHERE OrderID = ?",
        [fx.order_id.into()],
    ))
    .await
    .expect("set order payment method cod");

    let err = core_operations::handlers::returns::request_return(
        &txn,
        Request::new(RequestReturnRequest {
            order_id: fx.order_id,
            user_id: fx.user_id,
            items: vec![ReturnRequestItemInput {
                order_detail_id: fx.order_detail_id,
                quantity: Some(1),
            }],
            reason: "Not needed".to_string(),
        }),
    )
    .await
    .expect_err("cod return must fail");
    assert_eq!(err.code(), Code::FailedPrecondition);
    assert!(
        err.message()
            .to_ascii_lowercase()
            .contains("prepaid orders"),
        "unexpected message: {}",
        err.message()
    );
    txn.rollback().await.ok();
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn prepaid_not_delivered_cannot_request_return_and_post_window_is_rejected() {
    let db = Database::connect(&test_db_url()).await.expect("connect db");
    ensure_return_tables(&db).await;
    let txn = db.begin().await.expect("begin txn");
    let tag = Utc::now().timestamp_millis();
    let fx = place_order_fixture(&txn, tag, None, 1).await;

    let not_delivered = core_operations::handlers::returns::request_return(
        &txn,
        Request::new(RequestReturnRequest {
            order_id: fx.order_id,
            user_id: fx.user_id,
            items: vec![ReturnRequestItemInput {
                order_detail_id: fx.order_detail_id,
                quantity: Some(1),
            }],
            reason: "Too long".to_string(),
        }),
    )
    .await
    .expect_err("not delivered should fail");
    assert_eq!(not_delivered.code(), Code::FailedPrecondition);
    assert!(not_delivered
        .message()
        .to_ascii_lowercase()
        .contains("after delivery"));

    transition_order_to_delivered(&txn, &fx).await;
    txn.execute(Statement::from_sql_and_values(
        DatabaseBackend::MySql,
        r#"UPDATE OrderEvents
           SET created_at = DATE_SUB(UTC_TIMESTAMP(), INTERVAL 8 DAY)
           WHERE order_id = ?
             AND LOWER(COALESCE(to_status, '')) = 'delivered'"#,
        [fx.order_id.into()],
    ))
    .await
    .expect("backdate delivered transition");

    let post_window = core_operations::handlers::returns::request_return(
        &txn,
        Request::new(RequestReturnRequest {
            order_id: fx.order_id,
            user_id: fx.user_id,
            items: vec![ReturnRequestItemInput {
                order_detail_id: fx.order_detail_id,
                quantity: Some(1),
            }],
            reason: "Late request".to_string(),
        }),
    )
    .await
    .expect_err("window closed should fail");
    assert_eq!(post_window.code(), Code::FailedPrecondition);
    assert!(post_window
        .message()
        .to_ascii_lowercase()
        .contains("window has closed"));
    txn.rollback().await.ok();
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn duplicate_item_return_request_is_rejected() {
    let db = Database::connect(&test_db_url()).await.expect("connect db");
    ensure_return_tables(&db).await;
    let txn = db.begin().await.expect("begin txn");
    let tag = Utc::now().timestamp_millis();
    let fx = place_order_fixture(&txn, tag, None, 1).await;
    transition_order_to_delivered(&txn, &fx).await;
    let _ = request_return_for_fixture(&txn, &fx, 1, "Wrong fit").await;

    let dup = core_operations::handlers::returns::request_return(
        &txn,
        Request::new(RequestReturnRequest {
            order_id: fx.order_id,
            user_id: fx.user_id,
            items: vec![ReturnRequestItemInput {
                order_detail_id: fx.order_detail_id,
                quantity: Some(1),
            }],
            reason: "Duplicate".to_string(),
        }),
    )
    .await
    .expect_err("duplicate return should fail");
    assert_eq!(dup.code(), Code::FailedPrecondition);
    assert!(dup
        .message()
        .to_ascii_lowercase()
        .contains("already part of an existing return request"));
    txn.rollback().await.ok();
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn admin_mark_received_creates_durable_refund_attempt_and_inventory_not_restocked() {
    let db = Database::connect(&test_db_url()).await.expect("connect db");
    ensure_return_tables(&db).await;
    let txn = db.begin().await.expect("begin txn");
    let tag = Utc::now().timestamp_millis();
    let fx = place_order_fixture(&txn, tag, None, 1).await;
    transition_order_to_delivered(&txn, &fx).await;
    ensure_captured_payment_intent(&txn, &fx, tag).await;

    let before_inventory = inventory_available_for_variant(&txn, fx.variant_id).await;
    let ret = request_return_for_fixture(&txn, &fx, 1, "Defect").await;
    let after_request_inventory = inventory_available_for_variant(&txn, fx.variant_id).await;
    assert_eq!(
        before_inventory, after_request_inventory,
        "inventory must not change on return request"
    );

    let received = core_operations::handlers::returns::admin_mark_return_received(
        &txn,
        Request::new(AdminMarkReturnReceivedRequest {
            return_id: ret.return_id,
        }),
    )
    .await
    .expect("mark received")
    .into_inner()
    .items
    .into_iter()
    .next()
    .expect("return row after receive");
    assert_eq!(received.status.to_lowercase(), "received");
    assert!(received.refund_attempt_id.is_some());

    let after_received_inventory = inventory_available_for_variant(&txn, fx.variant_id).await;
    assert_eq!(
        before_inventory, after_received_inventory,
        "inventory must not be restocked at store receipt"
    );

    let attempt_id = received.refund_attempt_id.expect("attempt id set");
    let attempt = refund_attempts::Entity::find_by_id(attempt_id)
        .one(&txn)
        .await
        .expect("query attempt")
        .expect("attempt exists");
    assert_eq!(attempt.status, "pending_external");
    assert!(
        attempt.idempotency_key.starts_with("return_"),
        "return refund attempt must be tagged distinctly from cancellation attempts"
    );

    let refunds_count = refunds::Entity::find()
        .filter(refunds::Column::OrderId.eq(fx.order_id))
        .count(&txn)
        .await
        .expect("count refunds");
    assert_eq!(
        refunds_count, 0,
        "mark received should only enqueue durable attempt, no immediate refund row"
    );
    txn.rollback().await.ok();
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn refund_worker_processes_return_refund_exactly_once() {
    let db = Database::connect(&test_db_url()).await.expect("connect db");
    ensure_return_tables(&db).await;
    let setup_txn = db.begin().await.expect("begin setup txn");
    let tag = Utc::now().timestamp_millis();
    let fx = place_order_fixture(&setup_txn, tag, None, 1).await;
    transition_order_to_delivered(&setup_txn, &fx).await;
    ensure_captured_payment_intent(&setup_txn, &fx, tag).await;
    let requested = request_return_for_fixture(&setup_txn, &fx, 1, "Wrong design").await;
    let received = core_operations::handlers::returns::admin_mark_return_received(
        &setup_txn,
        Request::new(AdminMarkReturnReceivedRequest {
            return_id: requested.return_id,
        }),
    )
    .await
    .expect("mark received")
    .into_inner()
    .items
    .into_iter()
    .next()
    .expect("received row");
    let attempt_id = received.refund_attempt_id.expect("attempt id");
    let gateway_refund_id = format!("gw_ret_once_{tag}");
    setup_txn
        .execute(Statement::from_sql_and_values(
            DatabaseBackend::MySql,
            r#"UPDATE RefundAttempts
               SET status = 'submitted',
                   gateway_refund_id = ?
               WHERE attempt_id = ?"#,
            [gateway_refund_id.clone().into(), attempt_id.into()],
        ))
        .await
        .expect("prepare submitted attempt");
    setup_txn.commit().await.expect("commit setup");

    let first_processed = cancellation_saga::process_pending_refund_attempts(&db, 25)
        .await
        .expect("worker run 1");
    let second_processed = cancellation_saga::process_pending_refund_attempts(&db, 25)
        .await
        .expect("worker run 2");
    assert!(
        first_processed >= 1,
        "first worker pass must process the attempt"
    );
    assert_eq!(
        second_processed, 0,
        "second worker pass must be idempotent for processed return refund attempts"
    );

    let verify_txn = db.begin().await.expect("begin verify");
    let refund_rows = refunds::Entity::find()
        .filter(refunds::Column::OrderId.eq(fx.order_id))
        .filter(refunds::Column::GatewayRefundId.eq(gateway_refund_id.clone()))
        .all(&verify_txn)
        .await
        .expect("query refunds");
    assert_eq!(refund_rows.len(), 1, "refund must be recorded exactly once");

    let attempt = refund_attempts::Entity::find_by_id(attempt_id)
        .one(&verify_txn)
        .await
        .expect("query attempt")
        .expect("attempt exists");
    assert_eq!(attempt.status, "processed");
    verify_txn.rollback().await.ok();
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn multi_step_partial_returns_preserve_exact_line_total_refund() {
    let db = Database::connect(&test_db_url()).await.expect("connect db");
    ensure_return_tables(&db).await;
    let txn = db.begin().await.expect("begin txn");
    let tag = Utc::now().timestamp_millis();
    let fx = place_order_fixture(&txn, tag, None, 3).await;
    transition_order_to_delivered(&txn, &fx).await;

    txn.execute(Statement::from_sql_and_values(
        DatabaseBackend::MySql,
        r#"UPDATE OrderDetails
           SET line_total_minor = 100,
               unit_price_minor = 34,
               discount_minor = 2
           WHERE OrderDetailID = ?"#,
        [fx.order_detail_id.into()],
    ))
    .await
    .expect("force non-divisible line total");

    let first = request_return_for_fixture(&txn, &fx, 2, "Partial 1").await;
    let second = request_return_for_fixture(&txn, &fx, 1, "Partial 2").await;
    assert_eq!(first.items[0].refund_amount_minor, 66);
    assert_eq!(
        second.items[0].refund_amount_minor, 34,
        "final quantity return should get remaining line refund after rounding"
    );

    let sum_refunds: i64 = return_request_items::Entity::find()
        .filter(return_request_items::Column::OrderDetailId.eq(fx.order_detail_id))
        .all(&txn)
        .await
        .expect("query return items")
        .iter()
        .map(|row| row.refund_amount_minor)
        .sum();
    assert_eq!(
        sum_refunds, 100,
        "cumulative partial refunds must equal line_total_minor exactly"
    );
    txn.rollback().await.ok();
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn return_window_uses_delivered_transition_timestamp_as_source_of_truth() {
    let db = Database::connect(&test_db_url()).await.expect("connect db");
    ensure_return_tables(&db).await;
    let txn = db.begin().await.expect("begin txn");
    let tag = Utc::now().timestamp_millis();
    let fx = place_order_fixture(&txn, tag, None, 1).await;
    transition_order_to_delivered(&txn, &fx).await;

    txn.execute(Statement::from_sql_and_values(
        DatabaseBackend::MySql,
        r#"UPDATE OrderEvents
           SET created_at = DATE_SUB(UTC_TIMESTAMP(), INTERVAL 8 DAY)
           WHERE order_id = ?
             AND LOWER(COALESCE(to_status, '')) = 'delivered'"#,
        [fx.order_id.into()],
    ))
    .await
    .expect("backdate delivered transition event");
    txn.execute(Statement::from_sql_and_values(
        DatabaseBackend::MySql,
        r#"UPDATE Shipments
           SET delivered_at = UTC_TIMESTAMP()
           WHERE order_id = ?"#,
        [fx.order_id.into()],
    ))
    .await
    .expect("set fresh shipment delivered_at");

    let err = core_operations::handlers::returns::request_return(
        &txn,
        Request::new(RequestReturnRequest {
            order_id: fx.order_id,
            user_id: fx.user_id,
            items: vec![ReturnRequestItemInput {
                order_detail_id: fx.order_detail_id,
                quantity: Some(1),
            }],
            reason: "Too late".to_string(),
        }),
    )
    .await
    .expect_err("backdated canonical delivered transition should close return window");
    assert_eq!(err.code(), Code::FailedPrecondition);
    assert!(err
        .message()
        .to_ascii_lowercase()
        .contains("window has closed"));
    txn.rollback().await.ok();
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn active_return_items_are_not_cancellable_even_with_stale_fulfillment_flags() {
    let db = Database::connect(&test_db_url()).await.expect("connect db");
    ensure_return_tables(&db).await;
    let txn = db.begin().await.expect("begin txn");
    let tag = Utc::now().timestamp_millis();
    let fx = place_order_fixture(&txn, tag, None, 1).await;
    transition_order_to_delivered(&txn, &fx).await;
    let _ = request_return_for_fixture(&txn, &fx, 1, "Do not cancel").await;
    let confirmed_id = ensure_order_status(&txn, "confirmed").await;

    txn.execute(Statement::from_sql_and_values(
        DatabaseBackend::MySql,
        r#"UPDATE Orders
           SET StatusID = ?,
               fulfillment_status = 'not_created',
               cancel_window_ends_at = DATE_ADD(UTC_TIMESTAMP(), INTERVAL 2 HOUR)
           WHERE OrderID = ?"#,
        [confirmed_id.into(), fx.order_id.into()],
    ))
    .await
    .expect("force stale fulfillment flags");

    let err = core_operations::handlers::orders::cancel_order_items(
        &txn,
        Request::new(CancelOrderItemsRequest {
            order_id: fx.order_id,
            order_detail_ids: vec![fx.order_detail_id],
            acting_user_id: Some(fx.user_id),
        }),
    )
    .await
    .expect_err("active return item cancellation must be rejected");
    assert_eq!(err.code(), Code::FailedPrecondition);
    assert!(
        err.message()
            .to_ascii_lowercase()
            .contains("active return request"),
        "unexpected message: {}",
        err.message()
    );
    txn.rollback().await.ok();
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn delivered_orders_remain_non_cancellable_even_if_fulfillment_flag_is_stale() {
    let db = Database::connect(&test_db_url()).await.expect("connect db");
    ensure_return_tables(&db).await;
    let txn = db.begin().await.expect("begin txn");
    let tag = Utc::now().timestamp_millis();
    let fx = place_order_fixture(&txn, tag, None, 1).await;
    transition_order_to_delivered(&txn, &fx).await;

    txn.execute(Statement::from_sql_and_values(
        DatabaseBackend::MySql,
        r#"UPDATE Orders
           SET fulfillment_status = 'not_created',
               cancel_window_ends_at = DATE_ADD(UTC_TIMESTAMP(), INTERVAL 2 HOUR)
           WHERE OrderID = ?"#,
        [fx.order_id.into()],
    ))
    .await
    .expect("force stale fulfillment flag");

    let err = core_operations::handlers::orders::cancel_order_items(
        &txn,
        Request::new(CancelOrderItemsRequest {
            order_id: fx.order_id,
            order_detail_ids: vec![fx.order_detail_id],
            acting_user_id: Some(fx.user_id),
        }),
    )
    .await
    .expect_err("delivered orders must not be cancellable");
    assert_eq!(err.code(), Code::FailedPrecondition);
    assert!(
        err.message()
            .to_ascii_lowercase()
            .contains("cancellation window closed"),
        "unexpected message: {}",
        err.message()
    );
    txn.rollback().await.ok();
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn admin_mark_received_is_idempotent_for_refund_pending_returns() {
    let db = Database::connect(&test_db_url()).await.expect("connect db");
    ensure_return_tables(&db).await;
    let txn = db.begin().await.expect("begin txn");
    let tag = Utc::now().timestamp_millis();
    let fx = place_order_fixture(&txn, tag, None, 1).await;
    transition_order_to_delivered(&txn, &fx).await;
    ensure_captured_payment_intent(&txn, &fx, tag).await;

    let requested = request_return_for_fixture(&txn, &fx, 1, "QC pending").await;
    let received = core_operations::handlers::returns::admin_mark_return_received(
        &txn,
        Request::new(AdminMarkReturnReceivedRequest {
            return_id: requested.return_id,
        }),
    )
    .await
    .expect("mark received")
    .into_inner()
    .items
    .into_iter()
    .next()
    .expect("received row");
    assert!(received.refund_attempt_id.is_some());

    core_operations::handlers::returns::set_return_status_and_items(
        &txn,
        requested.return_id,
        "refund_pending",
    )
    .await
    .expect("force refund_pending state");

    let repeated = core_operations::handlers::returns::admin_mark_return_received(
        &txn,
        Request::new(AdminMarkReturnReceivedRequest {
            return_id: requested.return_id,
        }),
    )
    .await
    .expect("repeat mark received should be no-op")
    .into_inner()
    .items
    .into_iter()
    .next()
    .expect("repeat row");
    assert_eq!(repeated.status.to_lowercase(), "refund_pending");
    assert!(
        repeated
            .items
            .iter()
            .all(|item| item.status.eq_ignore_ascii_case("refund_pending")),
        "return item state must remain refund_pending after repeat mark-received"
    );
    txn.rollback().await.ok();
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn full_return_reaches_refunded_state_without_cancellation_side_effects() {
    let db = Database::connect(&test_db_url()).await.expect("connect db");
    ensure_return_tables(&db).await;
    let setup_txn = db.begin().await.expect("begin setup txn");
    let tag = Utc::now().timestamp_millis();
    let fx = place_order_fixture(&setup_txn, tag, None, 1).await;
    transition_order_to_delivered(&setup_txn, &fx).await;
    ensure_captured_payment_intent(&setup_txn, &fx, tag).await;

    let requested = request_return_for_fixture(&setup_txn, &fx, 1, "Damaged").await;
    let received = core_operations::handlers::returns::admin_mark_return_received(
        &setup_txn,
        Request::new(AdminMarkReturnReceivedRequest {
            return_id: requested.return_id,
        }),
    )
    .await
    .expect("mark received")
    .into_inner()
    .items
    .into_iter()
    .next()
    .expect("received row");
    let attempt_id = received.refund_attempt_id.expect("attempt id");
    let gateway_refund_id = format!("gw_ret_final_{tag}");
    setup_txn
        .execute(Statement::from_sql_and_values(
            DatabaseBackend::MySql,
            r#"UPDATE RefundAttempts
               SET status = 'submitted',
                   gateway_refund_id = ?
               WHERE attempt_id = ?"#,
            [gateway_refund_id.into(), attempt_id.into()],
        ))
        .await
        .expect("prepare attempt");
    setup_txn.commit().await.expect("commit setup");

    let _ = cancellation_saga::process_pending_refund_attempts(&db, 25)
        .await
        .expect("worker run");

    let verify_txn = db.begin().await.expect("begin verify");
    let return_row = return_requests::Entity::find_by_id(requested.return_id)
        .one(&verify_txn)
        .await
        .expect("query return")
        .expect("return exists");
    assert_eq!(return_row.status.to_lowercase(), "refunded");

    let return_items = return_request_items::Entity::find()
        .filter(return_request_items::Column::ReturnId.eq(requested.return_id))
        .all(&verify_txn)
        .await
        .expect("query return items");
    assert!(!return_items.is_empty());
    assert!(return_items
        .iter()
        .all(|row| row.status.eq_ignore_ascii_case("refunded")));

    let order_row = orders::Entity::find_by_id(fx.order_id)
        .one(&verify_txn)
        .await
        .expect("query order")
        .expect("order exists");
    assert_eq!(
        order_row.payment_status,
        Some(PaymentStatus::Failed),
        "refund recording should follow existing refund handler behavior"
    );
    let detail_row = order_details::Entity::find_by_id(fx.order_detail_id)
        .one(&verify_txn)
        .await
        .expect("query detail")
        .expect("detail exists");
    assert!(
        !detail_row.item_status.eq_ignore_ascii_case("cancelled"),
        "return flow must not reuse cancellation item status"
    );

    let restored_rows = order_inventory_restore_items::Entity::find()
        .filter(order_inventory_restore_items::Column::OrderId.eq(fx.order_id))
        .count(&verify_txn)
        .await
        .expect("count restore rows");
    assert_eq!(
        restored_rows, 0,
        "return flow must not restore sellable inventory automatically"
    );
    verify_txn.rollback().await.ok();
}
