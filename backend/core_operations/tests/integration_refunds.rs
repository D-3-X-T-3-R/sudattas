//! Integration tests for refunds: create_refund (full/partial, idempotent, reject pending).
//!
//! **Setup**
//! - Set `TEST_DATABASE_URL` or `DATABASE_URL`.
//! - Schema must be loaded (e.g. migrations or `backend/database/sql_dump/01_schema.sql`).
//!
//! **Run**
//! - `cargo test --test integration_refunds -- --ignored`

mod integration_common;
mod provider_test_gate;

use chrono::Utc;
use core_db_entities::entity::sea_orm_active_enums::PaymentStatus;
use core_db_entities::entity::{
    inventory, order_status, orders, product_categories, product_variants, products, refunds,
    shipping_addresses, user_roles,
};
use core_operations::order_state_machine;
use core_operations::procedures::orders::place_order;
use integration_common::test_db_url;
use proto::proto::core::{
    AdminMarkOrderDeliveredRequest, AdminMarkOrderShippedRequest, CreateCartItemRequest,
    CreateRefundRequest, CreateUserRequest, PlaceOrderRequest, UpdateOrderRequest,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, Database, DatabaseBackend,
    DatabaseConnection, EntityTrait, QueryFilter, Statement, TransactionTrait,
};
use tonic::{Code, Request};

async fn ensure_order_status(txn: &sea_orm::DatabaseTransaction, name: &str) -> i64 {
    if let Ok(Some(id)) = order_state_machine::get_status_id(txn, name).await {
        return id;
    }
    let m = order_status::ActiveModel {
        status_id: ActiveValue::NotSet,
        status_name: ActiveValue::Set(name.to_string()),
    }
    .insert(txn)
    .await
    .expect("insert OrderStatus");
    m.status_id
}

async fn make_order_booking_eligible(txn: &sea_orm::DatabaseTransaction, order_id: i64) {
    let _ = txn
        .execute(Statement::from_sql_and_values(
            DatabaseBackend::MySql,
            r#"UPDATE Orders
               SET earliest_booking_at = DATE_SUB(UTC_TIMESTAMP(), INTERVAL 1 HOUR),
                   cancel_window_ends_at = DATE_SUB(UTC_TIMESTAMP(), INTERVAL 1 HOUR),
                   payment_status = 'captured'
               WHERE OrderID = ?"#,
            [order_id.into()],
        ))
        .await
        .expect("make order booking-eligible");
}

/// Fixtures `place_order_minimal` created, so callers can both drive the rest of their scenario
/// (`order_id`/`user_id`/`shipping_id`/`total_paise`, as before) and clean up everything
/// afterward (`role_id`/`category_id`/`variant_id`), since those rows are now committed rather
/// than left inside a transaction the caller can roll back — see the doc comment on
/// `place_order_minimal` for why.
struct PlaceOrderMinimalFixture {
    order_id: i64,
    user_id: i64,
    shipping_id: i64,
    total_paise: i64,
    role_id: i64,
    category_id: i64,
    variant_id: i64,
}

/// Place order; return the created fixture ids (order_id, user_id, shipping_id, total_paise,
/// plus the role/category/variant ids needed for cleanup).
///
/// Setup runs in its own transaction, committed right before calling `place_order`: place_order
/// now manages its own transactions internally (a short claim/prep transaction, then the
/// Shiprocket/Razorpay calls with no DB connection held, then a write transaction) so it can no
/// longer run as a nested savepoint inside a caller-supplied, still-open transaction — see
/// `integration_place_order_atomicity.rs` for the full rationale. Callers therefore get a plain
/// `&DatabaseConnection` here (not a `&DatabaseTransaction`), and must begin their own fresh
/// transaction for whatever they do next.
async fn place_order_minimal(db: &DatabaseConnection, now_tag: i64) -> PlaceOrderMinimalFixture {
    let txn = db.begin().await.expect("begin setup transaction");

    let _ = ensure_order_status(&txn, "pending").await;
    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(format!("itest_ref_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert UserRoles");

    let user_res = core_operations::handlers::users::create_user(
        &txn,
        Request::new(CreateUserRequest {
            username: format!("itest_ref_{}", now_tag),
            email: format!("itest_ref+{}@example.com", now_tag),
            full_name: None,
            address: None,
            phone: None,
            auth_provider: "email".to_string(),
            password_plain: Some("StrongPass123!".to_string()),
            google_sub: None,
            role_id: Some(role.role_id),
        }),
    )
    .await
    .expect("create_user");
    let user_id = user_res.into_inner().items[0].user_id;

    let ship = shipping_addresses::ActiveModel {
        shipping_address_id: ActiveValue::NotSet,
        user_id: ActiveValue::Set(Some(user_id)),
        recipient_name: ActiveValue::Set(Some("Test User".to_string())),
        phone_number: ActiveValue::Set(Some("+919876543210".to_string())),
        is_default: ActiveValue::Set(0),
        country: ActiveValue::Set("IN".to_string()),
        state_region: ActiveValue::Set("KA".to_string()),
        city: ActiveValue::Set("City".to_string()),
        postal_code: ActiveValue::Set("100001".to_string()),
        road: ActiveValue::Set(None),
        apartment_no_or_name: ActiveValue::Set(None),
    }
    .insert(&txn)
    .await
    .expect("insert ShippingAddresses");
    let shipping_id = ship.shipping_address_id;

    let cat = product_categories::ActiveModel {
        category_id: ActiveValue::NotSet,
        name: ActiveValue::Set(format!("itest_cat_ref_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductCategories");

    let prod = products::ActiveModel {
        product_id: ActiveValue::NotSet,
        sku: ActiveValue::Set(None),
        name: ActiveValue::Set("Refund Test Product".to_string()),
        slug: ActiveValue::Set(None),
        description: ActiveValue::Set(None),
        // Keep subtotal above FREE_SHIPPING_THRESHOLD_MINOR to avoid live quote dependency.
        price_paise: ActiveValue::Set(150_000),
        category_id: ActiveValue::Set(cat.category_id),
        fabric: ActiveValue::Set(None),
        weave: ActiveValue::Set(None),
        occasion: ActiveValue::Set(None),
        has_blouse_piece: ActiveValue::Set(None),
        care_instructions: ActiveValue::Set(None),
        product_status_id: ActiveValue::Set(None),
        created_at: ActiveValue::Set(Some(Utc::now())),
        updated_at: ActiveValue::Set(None),
    }
    .insert(&txn)
    .await
    .expect("insert Products");

    let variant = product_variants::ActiveModel {
        variant_id: ActiveValue::NotSet,
        product_id: ActiveValue::Set(prod.product_id),
        size_id: ActiveValue::Set(None),
        color_id: ActiveValue::Set(None),
        additional_price: ActiveValue::Set(Some(0)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductVariants");

    let _ = inventory::ActiveModel {
        inventory_id: ActiveValue::NotSet,
        variant_id: ActiveValue::Set(Some(variant.variant_id)),
        quantity_available: ActiveValue::Set(Some(10)),
        quantity_reserved: ActiveValue::Set(Some(0)),
        reorder_level: ActiveValue::Set(None),
        updated_at: ActiveValue::Set(Some(Utc::now())),
    }
    .insert(&txn)
    .await
    .expect("insert Inventory");

    let cart_res = core_operations::handlers::cart::create_cart_item(
        &txn,
        Request::new(CreateCartItemRequest {
            user_id: Some(user_id),
            session_id: None,
            variant_id: variant.variant_id,
            quantity: 1,
        }),
    )
    .await
    .expect("create_cart_item");
    let cart_id = cart_res.into_inner().items[0].cart_id;

    // Commit setup so it's visible to place_order's own (separately-opened) transactions.
    txn.commit().await.expect("commit setup txn");

    let place_res = place_order(
        db,
        Request::new(PlaceOrderRequest {
            shipping_address_id: shipping_id,
            user_id,
            coupon_code: None,
            selected_cart_ids: vec![cart_id],
            payment_mode: None,
        }),
    )
    .await
    .expect("place_order");
    let order = place_res.into_inner().items[0].clone();

    PlaceOrderMinimalFixture {
        order_id: order.order_id,
        user_id,
        shipping_id,
        total_paise: order.total_amount_paise,
        role_id: role.role_id,
        category_id: cat.category_id,
        variant_id: variant.variant_id,
    }
}

/// Best-effort cleanup of everything `place_order_minimal` committed. Setup used to live inside
/// the same uncommitted transaction as the place_order call under test and simply never got
/// committed on rollback; now that it must be committed (see the doc comment on
/// `place_order_minimal`), it needs explicit cleanup instead. Deletes in FK-safe order (children
/// before parents). Errors are logged, not fatal — by the time this runs, the caller's own
/// assertions (against their separately-rolled-back transaction) have already completed, so a
/// cleanup failure here must never mask them.
async fn cleanup_place_order_minimal_fixture(
    db: &DatabaseConnection,
    fixture: &PlaceOrderMinimalFixture,
) {
    for (table, column) in [
        ("Refunds", "order_id"),
        ("PaymentIntents", "order_id"),
        ("Shipments", "order_id"),
        ("OrderDetails", "OrderID"),
        ("OrderEvents", "order_id"),
        ("Orders", "OrderID"),
    ] {
        if let Err(e) = db
            .execute(Statement::from_sql_and_values(
                DatabaseBackend::MySql,
                format!("DELETE FROM `{table}` WHERE `{column}` = ?"),
                [fixture.order_id.into()],
            ))
            .await
        {
            eprintln!(
                "warning: cleanup {table} for order_id={} failed (non-fatal): {e}",
                fixture.order_id
            );
        }
    }
    if let Err(e) = db
        .execute(Statement::from_sql_and_values(
            DatabaseBackend::MySql,
            "DELETE FROM `Cart` WHERE `UserID` = ?",
            [fixture.user_id.into()],
        ))
        .await
    {
        eprintln!(
            "warning: cleanup Cart for user_id={} failed (non-fatal): {e}",
            fixture.user_id
        );
    }
    if let Err(e) = db
        .execute(Statement::from_sql_and_values(
            DatabaseBackend::MySql,
            "DELETE FROM `Inventory` WHERE `VariantID` = ?",
            [fixture.variant_id.into()],
        ))
        .await
    {
        eprintln!(
            "warning: cleanup Inventory for variant_id={} failed (non-fatal): {e}",
            fixture.variant_id
        );
    }
    if let Err(e) = db
        .execute(Statement::from_sql_and_values(
            DatabaseBackend::MySql,
            "DELETE FROM `ProductVariants` WHERE `VariantID` = ?",
            [fixture.variant_id.into()],
        ))
        .await
    {
        eprintln!(
            "warning: cleanup ProductVariants for variant_id={} failed (non-fatal): {e}",
            fixture.variant_id
        );
    }
    if let Err(e) = db
        .execute(Statement::from_sql_and_values(
            DatabaseBackend::MySql,
            "DELETE FROM `Products` WHERE `CategoryID` = ?",
            [fixture.category_id.into()],
        ))
        .await
    {
        eprintln!(
            "warning: cleanup Products for category_id={} failed (non-fatal): {e}",
            fixture.category_id
        );
    }
    if let Err(e) = db
        .execute(Statement::from_sql_and_values(
            DatabaseBackend::MySql,
            "DELETE FROM `ProductCategories` WHERE `CategoryID` = ?",
            [fixture.category_id.into()],
        ))
        .await
    {
        eprintln!(
            "warning: cleanup ProductCategories for category_id={} failed (non-fatal): {e}",
            fixture.category_id
        );
    }
    if let Err(e) = db
        .execute(Statement::from_sql_and_values(
            DatabaseBackend::MySql,
            "DELETE FROM `ShippingAddresses` WHERE `ShippingAddressID` = ?",
            [fixture.shipping_id.into()],
        ))
        .await
    {
        eprintln!(
            "warning: cleanup ShippingAddresses id={} failed (non-fatal): {e}",
            fixture.shipping_id
        );
    }
    if let Err(e) = db
        .execute(Statement::from_sql_and_values(
            DatabaseBackend::MySql,
            "DELETE FROM `Users` WHERE `UserID` = ?",
            [fixture.user_id.into()],
        ))
        .await
    {
        eprintln!(
            "warning: cleanup Users for user_id={} failed (non-fatal): {e}",
            fixture.user_id
        );
    }
    if let Err(e) = db
        .execute(Statement::from_sql_and_values(
            DatabaseBackend::MySql,
            "DELETE FROM `UserRoles` WHERE `RoleID` = ?",
            [fixture.role_id.into()],
        ))
        .await
    {
        eprintln!(
            "warning: cleanup UserRoles for role_id={} failed (non-fatal): {e}",
            fixture.role_id
        );
    }
}

/// Transition order: pending → confirmed → processing → shipped → delivered.
///
/// `db` is unrelated to the place_order restructuring this file was otherwise adapted for — it's
/// required because `admin_mark_order_shipped` separately takes a `&DatabaseConnection` (for the
/// same "don't hold a transaction open across a Shiprocket call" reason), so it's threaded
/// through here too.
async fn transition_order_to_delivered(
    txn: &sea_orm::DatabaseTransaction,
    db: &DatabaseConnection,
    order_id: i64,
    user_id: i64,
    shipping_id: i64,
    total_paise: i64,
) {
    let confirmed_id = ensure_order_status(txn, "confirmed").await;
    let processing_id = ensure_order_status(txn, "processing").await;

    let _ = core_operations::handlers::orders::update_order(
        txn,
        Request::new(UpdateOrderRequest {
            order_id,
            user_id,
            shipping_address_id: shipping_id,
            total_amount_paise: total_paise,
            status_id: confirmed_id,
        }),
    )
    .await
    .expect("update to confirmed");

    let _ = core_operations::handlers::orders::update_order(
        txn,
        Request::new(UpdateOrderRequest {
            order_id,
            user_id,
            shipping_address_id: shipping_id,
            total_amount_paise: total_paise,
            status_id: processing_id,
        }),
    )
    .await
    .expect("update to processing");

    make_order_booking_eligible(txn, order_id).await;

    let _ = core_operations::handlers::orders::admin_mark_order_shipped(
        txn,
        db,
        Request::new(AdminMarkOrderShippedRequest {
            order_id,
            awb_code: Some("AWBREF".to_string()),
            carrier: Some("Carrier".to_string()),
            shiprocket_book: None,
            shiprocket_order_id: None,
            shiprocket_status_id: None,
            shiprocket_status_label: None,
        }),
    )
    .await
    .expect("admin_mark_order_shipped");

    let _ = core_operations::handlers::orders::admin_mark_order_delivered(
        txn,
        Request::new(AdminMarkOrderDeliveredRequest { order_id }),
    )
    .await
    .expect("admin_mark_order_delivered");
}

/// Transition order: pending → confirmed → processing → shipped.
///
/// `db` is unrelated to the place_order restructuring this file was otherwise adapted for — see
/// the doc comment on `transition_order_to_delivered`.
async fn transition_order_to_shipped(
    txn: &sea_orm::DatabaseTransaction,
    db: &DatabaseConnection,
    order_id: i64,
    user_id: i64,
    shipping_id: i64,
    total_paise: i64,
) {
    let confirmed_id = ensure_order_status(txn, "confirmed").await;
    let processing_id = ensure_order_status(txn, "processing").await;

    let _ = core_operations::handlers::orders::update_order(
        txn,
        Request::new(UpdateOrderRequest {
            order_id,
            user_id,
            shipping_address_id: shipping_id,
            total_amount_paise: total_paise,
            status_id: confirmed_id,
        }),
    )
    .await
    .expect("update to confirmed");

    let _ = core_operations::handlers::orders::update_order(
        txn,
        Request::new(UpdateOrderRequest {
            order_id,
            user_id,
            shipping_address_id: shipping_id,
            total_amount_paise: total_paise,
            status_id: processing_id,
        }),
    )
    .await
    .expect("update to processing");

    make_order_booking_eligible(txn, order_id).await;

    let _ = core_operations::handlers::orders::admin_mark_order_shipped(
        txn,
        db,
        Request::new(AdminMarkOrderShippedRequest {
            order_id,
            awb_code: Some("AWBREF-SHIPPED".to_string()),
            carrier: Some("Carrier".to_string()),
            shiprocket_book: None,
            shiprocket_order_id: None,
            shiprocket_status_id: None,
            shiprocket_status_label: None,
        }),
    )
    .await
    .expect("admin_mark_order_shipped");
}

/// R1 – Full refund on a delivered order via create_refund transitions order to Refunded and updates payment_status.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_full_refund_delivered_transitions_to_refunded() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_full_refund_delivered_transitions_to_refunded",
    ) {
        return;
    }

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");

    let now_tag = Utc::now().timestamp_millis();
    let fixture = place_order_minimal(&db, now_tag).await;
    let (order_id, user_id, shipping_id, total_paise) = (
        fixture.order_id,
        fixture.user_id,
        fixture.shipping_id,
        fixture.total_paise,
    );

    // place_order_minimal's own writes are already committed by this point (see its doc
    // comment), so this fresh transaction only covers the rest of the scenario and is rolled
    // back at the end, same as before.
    let txn = db.begin().await.expect("begin transaction");
    transition_order_to_delivered(&txn, &db, order_id, user_id, shipping_id, total_paise).await;

    let _ = core_operations::handlers::refunds::create_refund(
        &txn,
        Request::new(CreateRefundRequest {
            order_id,
            gateway_refund_id: format!("gw_full_{}", now_tag),
            amount_paise: total_paise,
            currency: None,
            line_items_refunded_json: None,
        }),
    )
    .await
    .expect("create_refund full should succeed");

    let refunded_id = ensure_order_status(&txn, "refunded").await;
    let order = orders::Entity::find_by_id(order_id)
        .one(&txn)
        .await
        .expect("query order")
        .expect("order exists");
    assert_eq!(
        order.status_id, refunded_id,
        "order should transition to refunded"
    );
    assert_eq!(
        order.payment_status,
        Some(PaymentStatus::Failed),
        "payment_status should be updated to Failed"
    );

    txn.rollback().await.ok();

    cleanup_place_order_minimal_fixture(&db, &fixture).await;
}

/// R2 – Partial refund on a shipped order creates refund row but leaves order status shipped.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_partial_refund_shipped_leaves_status_shipped() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_partial_refund_shipped_leaves_status_shipped",
    ) {
        return;
    }

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");

    let now_tag = Utc::now().timestamp_millis();
    let fixture = place_order_minimal(&db, now_tag).await;
    let (order_id, user_id, shipping_id, total_paise) = (
        fixture.order_id,
        fixture.user_id,
        fixture.shipping_id,
        fixture.total_paise,
    );

    // place_order_minimal's own writes are already committed by this point (see its doc
    // comment), so this fresh transaction only covers the rest of the scenario and is rolled
    // back at the end, same as before.
    let txn = db.begin().await.expect("begin transaction");
    transition_order_to_shipped(&txn, &db, order_id, user_id, shipping_id, total_paise).await;

    let partial_amount = total_paise / 2;
    let create_res = core_operations::handlers::refunds::create_refund(
        &txn,
        Request::new(CreateRefundRequest {
            order_id,
            gateway_refund_id: format!("gw_partial_{}", now_tag),
            amount_paise: partial_amount,
            currency: None,
            line_items_refunded_json: None,
        }),
    )
    .await
    .expect("create_refund partial should succeed");
    assert_eq!(create_res.into_inner().items.len(), 1);

    let refund_rows = refunds::Entity::find()
        .filter(refunds::Column::OrderId.eq(order_id))
        .all(&txn)
        .await
        .expect("query refunds");
    assert_eq!(refund_rows.len(), 1);
    assert_eq!(refund_rows[0].amount_paise, partial_amount as i32);

    let shipped_id = ensure_order_status(&txn, "shipped").await;
    let order = orders::Entity::find_by_id(order_id)
        .one(&txn)
        .await
        .expect("query order")
        .expect("order exists");
    assert_eq!(
        order.status_id, shipped_id,
        "order should remain shipped after partial refund"
    );

    txn.rollback().await.ok();

    cleanup_place_order_minimal_fixture(&db, &fixture).await;
}

/// R3 – Duplicate gateway_refund_id for create_refund is idempotent (second call returns existing refund).
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_create_refund_duplicate_gateway_id_idempotent() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_create_refund_duplicate_gateway_id_idempotent",
    ) {
        return;
    }

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");

    let now_tag = Utc::now().timestamp_millis();
    let fixture = place_order_minimal(&db, now_tag).await;
    let (order_id, user_id, shipping_id, total_paise) = (
        fixture.order_id,
        fixture.user_id,
        fixture.shipping_id,
        fixture.total_paise,
    );

    // place_order_minimal's own writes are already committed by this point (see its doc
    // comment), so this fresh transaction only covers the rest of the scenario and is rolled
    // back at the end, same as before.
    let txn = db.begin().await.expect("begin transaction");
    let confirmed_id = ensure_order_status(&txn, "confirmed").await;
    let _ = core_operations::handlers::orders::update_order(
        &txn,
        Request::new(UpdateOrderRequest {
            order_id,
            user_id,
            shipping_address_id: shipping_id,
            total_amount_paise: total_paise,
            status_id: confirmed_id,
        }),
    )
    .await
    .expect("update to confirmed");

    let gateway_id = format!("gw_idem_{}", now_tag);
    let first = core_operations::handlers::refunds::create_refund(
        &txn,
        Request::new(CreateRefundRequest {
            order_id,
            gateway_refund_id: gateway_id.clone(),
            amount_paise: 1_000,
            currency: None,
            line_items_refunded_json: None,
        }),
    )
    .await
    .expect("first create_refund should succeed");
    let first_refund_id = first.into_inner().items[0].refund_id;

    let second = core_operations::handlers::refunds::create_refund(
        &txn,
        Request::new(CreateRefundRequest {
            order_id,
            gateway_refund_id: gateway_id.clone(),
            amount_paise: 1_000,
            currency: None,
            line_items_refunded_json: None,
        }),
    )
    .await
    .expect("second create_refund should succeed (idempotent)");
    let second_refund_id = second.into_inner().items[0].refund_id;
    assert_eq!(
        first_refund_id, second_refund_id,
        "idempotent call should return same refund"
    );

    let count = refunds::Entity::find()
        .filter(refunds::Column::GatewayRefundId.eq(&gateway_id))
        .all(&txn)
        .await
        .expect("query refunds")
        .len();
    assert_eq!(count, 1, "only one refund row for same gateway_refund_id");

    txn.rollback().await.ok();

    cleanup_place_order_minimal_fixture(&db, &fixture).await;
}

/// R4 – create_refund on a pending order returns FailedPrecondition and does not create a refund row.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_create_refund_pending_returns_failed_precondition() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_create_refund_pending_returns_failed_precondition",
    ) {
        return;
    }

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");

    let now_tag = Utc::now().timestamp_millis();
    let fixture = place_order_minimal(&db, now_tag).await;
    let (order_id, total_paise) = (fixture.order_id, fixture.total_paise);

    // place_order_minimal's own writes are already committed by this point (see its doc
    // comment), so this fresh transaction only covers the rest of the scenario and is rolled
    // back at the end, same as before.
    let txn = db.begin().await.expect("begin transaction");

    let result = core_operations::handlers::refunds::create_refund(
        &txn,
        Request::new(CreateRefundRequest {
            order_id,
            gateway_refund_id: format!("gw_pending_{}", now_tag),
            amount_paise: total_paise,
            currency: None,
            line_items_refunded_json: None,
        }),
    )
    .await;

    let err = result.expect_err("create_refund on pending order should fail");
    assert_eq!(err.code(), Code::FailedPrecondition);

    let refund_count = refunds::Entity::find()
        .filter(refunds::Column::OrderId.eq(order_id))
        .all(&txn)
        .await
        .expect("query refunds")
        .len();
    assert_eq!(refund_count, 0, "no refund row should be created");

    txn.rollback().await.ok();

    cleanup_place_order_minimal_fixture(&db, &fixture).await;
}

/// R5 – create_refund on a cancelled order succeeds (needed for async courier cancellation → refund flow).
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_create_refund_cancelled_order_succeeds() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_create_refund_cancelled_order_succeeds",
    ) {
        return;
    }

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");

    let now_tag = Utc::now().timestamp_millis();
    let fixture = place_order_minimal(&db, now_tag).await;
    let (order_id, user_id, shipping_id, total_paise) = (
        fixture.order_id,
        fixture.user_id,
        fixture.shipping_id,
        fixture.total_paise,
    );

    // place_order_minimal's own writes are already committed by this point (see its doc
    // comment), so this fresh transaction only covers the rest of the scenario and is rolled
    // back at the end, same as before.
    let txn = db.begin().await.expect("begin transaction");

    let confirmed_id = ensure_order_status(&txn, "confirmed").await;
    let cancelled_id = ensure_order_status(&txn, "cancelled").await;
    let _ = core_operations::handlers::orders::update_order(
        &txn,
        Request::new(UpdateOrderRequest {
            order_id,
            user_id,
            shipping_address_id: shipping_id,
            total_amount_paise: total_paise,
            status_id: confirmed_id,
        }),
    )
    .await
    .expect("update to confirmed");

    let _ = core_operations::handlers::orders::update_order(
        &txn,
        Request::new(UpdateOrderRequest {
            order_id,
            user_id,
            shipping_address_id: shipping_id,
            total_amount_paise: total_paise,
            status_id: cancelled_id,
        }),
    )
    .await
    .expect("update to cancelled");

    let create_res = core_operations::handlers::refunds::create_refund(
        &txn,
        Request::new(CreateRefundRequest {
            order_id,
            gateway_refund_id: format!("gw_cancelled_{}", now_tag),
            amount_paise: total_paise,
            currency: None,
            line_items_refunded_json: None,
        }),
    )
    .await
    .expect("create_refund on cancelled should succeed");
    assert_eq!(create_res.into_inner().items.len(), 1);

    let refunded_id = ensure_order_status(&txn, "refunded").await;
    let order = orders::Entity::find_by_id(order_id)
        .one(&txn)
        .await
        .expect("query order")
        .expect("order exists");
    assert_eq!(
        order.status_id, refunded_id,
        "cancelled order should transition to refunded"
    );

    txn.rollback().await.ok();

    cleanup_place_order_minimal_fixture(&db, &fixture).await;
}
