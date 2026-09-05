//! Integration tests for order state machine (update_order, admin_mark_shipped/delivered, inventory restore).
//!
//! **Setup**
//! - Set `TEST_DATABASE_URL` or `DATABASE_URL`.
//! - Schema must be loaded (e.g. migrations or `backend/database/sql_dump/01_schema.sql`).
//!
//! **Run**
//! - `cargo test --test integration_order_state -- --ignored`

mod integration_common;
mod provider_test_gate;

use chrono::Utc;
use integration_common::test_db_url;

use core_db_entities::entity::{
    inventory, order_events, order_status, orders, product_categories, product_variants, products,
    shipments, shipping_addresses, user_roles,
};
use core_operations::procedures::orders::place_order;
use proto::proto::core::{
    AdminMarkOrderDeliveredRequest, AdminMarkOrderShippedRequest, CreateCartItemRequest,
    CreateUserRequest, DeleteOrderRequest, PlaceOrderRequest, UpdateOrderRequest,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, Database, EntityTrait,
    QueryFilter, Statement, TransactionTrait,
};
use tonic::{Code, Request};

async fn ensure_order_status(txn: &sea_orm::DatabaseTransaction, name: &str) -> i64 {
    if let Ok(Some(id)) = core_operations::order_state_machine::get_status_id(txn, name).await {
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
            sea_orm::DatabaseBackend::MySql,
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

/// Fixture ids from `place_order_minimal`: the placed order plus everything created to place
/// it, needed both by the tests that build on top of it and (now that setup/place_order must be
/// committed rather than left inside a rollback-able transaction — see the comment on the
/// `txn.commit()` call below) to clean the rows up afterward.
struct OrderStateFixtures {
    order_id: i64,
    user_id: i64,
    shipping_id: i64,
    variant_id: i64,
    total_paise: i64,
    role_id: i64,
    category_id: i64,
}

/// Build user + shipping + one product/variant/inventory + one cart item, place order.
///
/// Setup runs in its own transaction, committed before calling `place_order`: `place_order` now
/// manages its own transactions internally (a short claim/prep transaction, then the
/// Shiprocket/Razorpay calls with no DB connection held, then a write transaction) so it can no
/// longer run as a nested savepoint inside a caller-supplied, still-open transaction. Setup
/// fixtures must therefore be committed and visible before place_order's own prep phase reads
/// them, rather than sharing one uncommitted transaction with it as before. Each O-test below
/// opens its own fresh transaction after this returns, to wrap the order-state-machine calls it
/// is actually testing (those handlers are unaffected by the place_order restructuring and still
/// take `&DatabaseTransaction`).
async fn place_order_minimal(db: &sea_orm::DatabaseConnection, now_tag: i64) -> OrderStateFixtures {
    let txn = db.begin().await.expect("begin transaction");
    let _ = ensure_order_status(&txn, "pending").await;
    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(format!("itest_ord_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert UserRoles");

    let user_res = core_operations::handlers::users::create_user(
        &txn,
        Request::new(CreateUserRequest {
            username: format!("itest_ord_{}", now_tag),
            email: format!("itest_ord+{}@example.com", now_tag),
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
        exchange_eligible: sea_orm::ActiveValue::Set(0),
        name: ActiveValue::Set(format!("itest_cat_ord_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductCategories");

    let prod = products::ActiveModel {
        product_id: ActiveValue::NotSet,
        sku: ActiveValue::Set(None),
        name: ActiveValue::Set("Order State Product".to_string()),
        slug: ActiveValue::Set(None),
        description: ActiveValue::Set(None),
        // Keep subtotal above FREE_SHIPPING_THRESHOLD_MINOR to avoid live shipping quote dependency.
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

    // Commit setup so it's visible to place_order's own (separately-opened) transactions —
    // place_order no longer runs as a nested savepoint inside our transaction, so anything it
    // needs to read (user, cart, shipping address) must already be committed.
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

    OrderStateFixtures {
        order_id: order.order_id,
        user_id,
        shipping_id,
        variant_id: variant.variant_id,
        total_paise: order.total_amount_paise,
        role_id: role.role_id,
        category_id: cat.category_id,
    }
}

/// Best-effort teardown of everything `place_order_minimal` committed (fixtures + the placed
/// order and its dependents). Previously this all lived inside the same uncommitted transaction
/// as the test's own state-machine assertions, so one final `txn.rollback()` undid everything;
/// now that place_order_minimal's work is committed independently (see the comment on its
/// `txn.commit()` call), it needs explicit cleanup instead. Each O-test's own per-test
/// transaction (wrapping the update_order/admin_mark_* calls under test) is still rolled back as
/// before, so this only needs to clean up what place_order_minimal itself persisted. Deletes in
/// FK-safe order (children before parents). Errors here are logged, not fatal — they must never
/// mask the actual test assertions.
async fn cleanup_order_state_fixtures(
    db: &sea_orm::DatabaseConnection,
    fixtures: &OrderStateFixtures,
) -> Result<(), String> {
    for (table, column) in [
        ("PaymentIntents", "order_id"),
        ("Shipments", "order_id"),
        ("OrderDetails", "OrderID"),
        ("OrderEvents", "OrderID"),
        ("Orders", "OrderID"),
    ] {
        db.execute(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::MySql,
            format!("DELETE FROM `{table}` WHERE `{column}` = ?"),
            [fixtures.order_id.into()],
        ))
        .await
        .map_err(|e| format!("cleanup {table} for order_id={}: {e}", fixtures.order_id))?;
    }
    db.execute(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::MySql,
        "DELETE FROM `Cart` WHERE `UserID` = ?",
        [fixtures.user_id.into()],
    ))
    .await
    .map_err(|e| format!("cleanup Cart for user_id={}: {e}", fixtures.user_id))?;
    db.execute(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::MySql,
        "DELETE FROM `Inventory` WHERE `VariantID` = ?",
        [fixtures.variant_id.into()],
    ))
    .await
    .map_err(|e| {
        format!(
            "cleanup Inventory for variant_id={}: {e}",
            fixtures.variant_id
        )
    })?;
    db.execute(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::MySql,
        "DELETE FROM `ProductVariants` WHERE `VariantID` = ?",
        [fixtures.variant_id.into()],
    ))
    .await
    .map_err(|e| {
        format!(
            "cleanup ProductVariants for variant_id={}: {e}",
            fixtures.variant_id
        )
    })?;
    db.execute(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::MySql,
        "DELETE FROM `Products` WHERE `CategoryID` = ?",
        [fixtures.category_id.into()],
    ))
    .await
    .map_err(|e| {
        format!(
            "cleanup Products for category_id={}: {e}",
            fixtures.category_id
        )
    })?;
    db.execute(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::MySql,
        "DELETE FROM `ProductCategories` WHERE `CategoryID` = ?",
        [fixtures.category_id.into()],
    ))
    .await
    .map_err(|e| {
        format!(
            "cleanup ProductCategories for category_id={}: {e}",
            fixtures.category_id
        )
    })?;
    db.execute(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::MySql,
        "DELETE FROM `ShippingAddresses` WHERE `ShippingAddressID` = ?",
        [fixtures.shipping_id.into()],
    ))
    .await
    .map_err(|e| format!("cleanup ShippingAddresses id={}: {e}", fixtures.shipping_id))?;
    db.execute(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::MySql,
        "DELETE FROM `Users` WHERE `UserID` = ?",
        [fixtures.user_id.into()],
    ))
    .await
    .map_err(|e| format!("cleanup Users for user_id={}: {e}", fixtures.user_id))?;
    db.execute(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::MySql,
        "DELETE FROM `UserRoles` WHERE `RoleID` = ?",
        [fixtures.role_id.into()],
    ))
    .await
    .map_err(|e| format!("cleanup UserRoles for role_id={}: {e}", fixtures.role_id))?;
    Ok(())
}

/// O1 – update_order transitions pending → confirmed; order row updated and order_events entry created.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_order_update_pending_to_confirmed_and_order_event() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_order_update_pending_to_confirmed_and_order_event",
    ) {
        return;
    }

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");

    let now_tag = Utc::now().timestamp_millis();
    let fixtures = place_order_minimal(&db, now_tag).await;
    let (order_id, user_id, shipping_id, total_paise) = (
        fixtures.order_id,
        fixtures.user_id,
        fixtures.shipping_id,
        fixtures.total_paise,
    );

    // Fresh transaction for the update_order call under test; place_order_minimal already
    // committed its own work (see the comment on its `txn.commit()` call), so this only wraps
    // the state-machine mutation and its assertions, same as before.
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
    .expect("update_order to confirmed should succeed");

    let order = orders::Entity::find_by_id(order_id)
        .one(&txn)
        .await
        .expect("query order")
        .expect("order exists");
    assert_eq!(order.status_id, confirmed_id);

    let events = order_events::Entity::find()
        .filter(order_events::Column::OrderId.eq(order_id))
        .all(&txn)
        .await
        .expect("query order_events");
    assert!(
        events
            .iter()
            .any(|e| e.to_status.as_deref() == Some("confirmed")),
        "order_events should contain transition to confirmed"
    );

    txn.rollback().await.ok();

    if let Err(e) = cleanup_order_state_fixtures(&db, &fixtures).await {
        eprintln!("warning: scenario cleanup failed (non-fatal): {e}");
    }
}

/// O2 – Cancelling an order via update_order restores inventory quantities from order_details.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_order_cancel_restores_inventory() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_order_cancel_restores_inventory",
    ) {
        return;
    }

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");

    let now_tag = Utc::now().timestamp_millis();
    let fixtures = place_order_minimal(&db, now_tag).await;
    let (order_id, user_id, variant_id) =
        (fixtures.order_id, fixtures.user_id, fixtures.variant_id);

    // Fresh transaction for the delete_order call under test; see the comment on the equivalent
    // transaction in `integration_order_update_pending_to_confirmed_and_order_event`.
    let txn = db.begin().await.expect("begin transaction");

    let inv_before = inventory::Entity::find()
        .filter(inventory::Column::VariantId.eq(Some(variant_id)))
        .one(&txn)
        .await
        .expect("query inventory")
        .expect("inventory exists");
    let qty_before = inv_before.quantity_available.unwrap_or(0);

    let _ = core_operations::handlers::orders::delete_order(
        &txn,
        Request::new(DeleteOrderRequest {
            order_id,
            acting_user_id: Some(user_id),
        }),
    )
    .await
    .expect("delete_order should cancel and restore inventory");

    let inv_after = inventory::Entity::find()
        .filter(inventory::Column::VariantId.eq(Some(variant_id)))
        .one(&txn)
        .await
        .expect("query inventory")
        .expect("inventory exists");
    let qty_after = inv_after.quantity_available.unwrap_or(0);
    assert_eq!(
        qty_after,
        qty_before + 1,
        "inventory should be restored by 1 (one line item quantity)"
    );

    txn.rollback().await.ok();

    if let Err(e) = cleanup_order_state_fixtures(&db, &fixtures).await {
        eprintln!("warning: scenario cleanup failed (non-fatal): {e}");
    }
}

/// O3 – Illegal transition (pending → delivered) via update_order returns InvalidArgument.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_order_illegal_transition_returns_invalid_argument() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_order_illegal_transition_returns_invalid_argument",
    ) {
        return;
    }

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");

    let now_tag = Utc::now().timestamp_millis();
    let fixtures = place_order_minimal(&db, now_tag).await;
    let (order_id, user_id, shipping_id, total_paise) = (
        fixtures.order_id,
        fixtures.user_id,
        fixtures.shipping_id,
        fixtures.total_paise,
    );

    // Fresh transaction for the update_order call under test; see the comment on the equivalent
    // transaction in `integration_order_update_pending_to_confirmed_and_order_event`.
    let txn = db.begin().await.expect("begin transaction");

    let delivered_id = ensure_order_status(&txn, "delivered").await;

    let result = core_operations::handlers::orders::update_order(
        &txn,
        Request::new(UpdateOrderRequest {
            order_id,
            user_id,
            shipping_address_id: shipping_id,
            total_amount_paise: total_paise,
            status_id: delivered_id,
        }),
    )
    .await;

    let err = result.expect_err("update_order pending→delivered should fail");
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(
        err.message().to_lowercase().contains("illegal") || err.message().contains("transition"),
        "error should mention illegal transition, got: {}",
        err.message()
    );

    txn.rollback().await.ok();

    if let Err(e) = cleanup_order_state_fixtures(&db, &fixtures).await {
        eprintln!("warning: scenario cleanup failed (non-fatal): {e}");
    }
}

/// O4 – admin_mark_order_shipped transitions order to shipped and creates a shipment row when tracking is provided.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_admin_mark_shipped_creates_shipment() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_admin_mark_shipped_creates_shipment",
    ) {
        return;
    }

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");

    let now_tag = Utc::now().timestamp_millis();
    let fixtures = place_order_minimal(&db, now_tag).await;
    let (order_id, user_id, shipping_id, total_paise) = (
        fixtures.order_id,
        fixtures.user_id,
        fixtures.shipping_id,
        fixtures.total_paise,
    );

    // Fresh transaction for the state-machine calls under test; see the comment on the
    // equivalent transaction in `integration_order_update_pending_to_confirmed_and_order_event`.
    let txn = db.begin().await.expect("begin transaction");

    let confirmed_id = ensure_order_status(&txn, "confirmed").await;
    let processing_id = ensure_order_status(&txn, "processing").await;

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
            status_id: processing_id,
        }),
    )
    .await
    .expect("update to processing");

    make_order_booking_eligible(&txn, order_id).await;

    let ship_res = core_operations::handlers::orders::admin_mark_order_shipped(
        &txn,
        &db, // pre-existing signature drift, unrelated to the place_order adaptation — see final report
        Request::new(AdminMarkOrderShippedRequest {
            order_id,
            awb_code: Some("AWB123".to_string()),
            carrier: Some("DHL".to_string()),
            shiprocket_book: None,
            shiprocket_order_id: None,
            shiprocket_status_id: None,
            shiprocket_status_label: None,
        }),
    )
    .await
    .expect("admin_mark_order_shipped should succeed");
    assert!(
        ship_res.into_inner().shipment_id > 0,
        "shipment should be created"
    );

    let order = orders::Entity::find_by_id(order_id)
        .one(&txn)
        .await
        .expect("query order")
        .expect("order exists");
    let shipped_id = ensure_order_status(&txn, "shipped").await;
    assert_eq!(order.status_id, shipped_id);

    let ship_rows = shipments::Entity::find()
        .filter(shipments::Column::OrderId.eq(order_id))
        .all(&txn)
        .await
        .expect("query Shipments");
    assert_eq!(ship_rows.len(), 1);
    assert_eq!(ship_rows[0].awb_code.as_deref(), Some("AWB123"));
    assert_eq!(ship_rows[0].carrier.as_deref(), Some("DHL"));

    txn.rollback().await.ok();

    if let Err(e) = cleanup_order_state_fixtures(&db, &fixtures).await {
        eprintln!("warning: scenario cleanup failed (non-fatal): {e}");
    }
}

/// O5 – admin_mark_order_shipped called twice updates existing shipment (awb/carrier) instead of creating a new one.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_admin_mark_shipped_twice_updates_shipment() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_admin_mark_shipped_twice_updates_shipment",
    ) {
        return;
    }

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");

    let now_tag = Utc::now().timestamp_millis();
    let fixtures = place_order_minimal(&db, now_tag).await;
    let (order_id, user_id, shipping_id, total_paise) = (
        fixtures.order_id,
        fixtures.user_id,
        fixtures.shipping_id,
        fixtures.total_paise,
    );

    // Fresh transaction for the state-machine calls under test; see the comment on the
    // equivalent transaction in `integration_order_update_pending_to_confirmed_and_order_event`.
    let txn = db.begin().await.expect("begin transaction");

    let confirmed_id = ensure_order_status(&txn, "confirmed").await;
    let processing_id = ensure_order_status(&txn, "processing").await;

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
            status_id: processing_id,
        }),
    )
    .await
    .expect("update to processing");

    make_order_booking_eligible(&txn, order_id).await;

    let _ = core_operations::handlers::orders::admin_mark_order_shipped(
        &txn,
        &db, // pre-existing signature drift, unrelated to the place_order adaptation — see final report
        Request::new(AdminMarkOrderShippedRequest {
            order_id,
            awb_code: Some("AWB1".to_string()),
            carrier: Some("Carrier1".to_string()),
            shiprocket_book: None,
            shiprocket_order_id: None,
            shiprocket_status_id: None,
            shiprocket_status_label: None,
        }),
    )
    .await
    .expect("first admin_mark_order_shipped");

    let _ = core_operations::handlers::orders::admin_mark_order_shipped(
        &txn,
        &db, // pre-existing signature drift, unrelated to the place_order adaptation — see final report
        Request::new(AdminMarkOrderShippedRequest {
            order_id,
            awb_code: Some("AWB2".to_string()),
            carrier: Some("Carrier2".to_string()),
            shiprocket_book: None,
            shiprocket_order_id: None,
            shiprocket_status_id: None,
            shiprocket_status_label: None,
        }),
    )
    .await
    .expect("second admin_mark_order_shipped");

    let ship_rows = shipments::Entity::find()
        .filter(shipments::Column::OrderId.eq(order_id))
        .all(&txn)
        .await
        .expect("query Shipments");
    assert_eq!(ship_rows.len(), 1, "should still be one shipment row");
    assert_eq!(ship_rows[0].awb_code.as_deref(), Some("AWB2"));
    assert_eq!(ship_rows[0].carrier.as_deref(), Some("Carrier2"));

    txn.rollback().await.ok();

    if let Err(e) = cleanup_order_state_fixtures(&db, &fixtures).await {
        eprintln!("warning: scenario cleanup failed (non-fatal): {e}");
    }
}

/// O6 – admin_mark_order_delivered transitions shipped → delivered and records the change.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_admin_mark_delivered_transitions_to_delivered() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_admin_mark_delivered_transitions_to_delivered",
    ) {
        return;
    }

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");

    let now_tag = Utc::now().timestamp_millis();
    let fixtures = place_order_minimal(&db, now_tag).await;
    let (order_id, user_id, shipping_id, total_paise) = (
        fixtures.order_id,
        fixtures.user_id,
        fixtures.shipping_id,
        fixtures.total_paise,
    );

    // Fresh transaction for the state-machine calls under test; see the comment on the
    // equivalent transaction in `integration_order_update_pending_to_confirmed_and_order_event`.
    let txn = db.begin().await.expect("begin transaction");

    let confirmed_id = ensure_order_status(&txn, "confirmed").await;
    let processing_id = ensure_order_status(&txn, "processing").await;

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
            status_id: processing_id,
        }),
    )
    .await
    .expect("update to processing");

    make_order_booking_eligible(&txn, order_id).await;

    let _ = core_operations::handlers::orders::admin_mark_order_shipped(
        &txn,
        &db, // pre-existing signature drift, unrelated to the place_order adaptation — see final report
        Request::new(AdminMarkOrderShippedRequest {
            order_id,
            awb_code: Some("AWB".to_string()),
            carrier: Some("Carrier".to_string()),
            shiprocket_book: None,
            shiprocket_order_id: None,
            shiprocket_status_id: None,
            shiprocket_status_label: None,
        }),
    )
    .await
    .expect("mark shipped");

    let _ = core_operations::handlers::orders::admin_mark_order_delivered(
        &txn,
        Request::new(AdminMarkOrderDeliveredRequest { order_id }),
    )
    .await
    .expect("admin_mark_order_delivered should succeed");

    let order = orders::Entity::find_by_id(order_id)
        .one(&txn)
        .await
        .expect("query order")
        .expect("order exists");
    let delivered_id = ensure_order_status(&txn, "delivered").await;
    assert_eq!(order.status_id, delivered_id);

    txn.rollback().await.ok();

    if let Err(e) = cleanup_order_state_fixtures(&db, &fixtures).await {
        eprintln!("warning: scenario cleanup failed (non-fatal): {e}");
    }
}

/// O7 – Full lifecycle: pending → confirmed → processing → shipped → delivered using allowed transitions only.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_order_full_lifecycle_pending_to_delivered() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_order_full_lifecycle_pending_to_delivered",
    ) {
        return;
    }

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");

    let now_tag = Utc::now().timestamp_millis();
    let fixtures = place_order_minimal(&db, now_tag).await;
    let (order_id, user_id, shipping_id, total_paise) = (
        fixtures.order_id,
        fixtures.user_id,
        fixtures.shipping_id,
        fixtures.total_paise,
    );

    // Fresh transaction for the state-machine calls under test; see the comment on the
    // equivalent transaction in `integration_order_update_pending_to_confirmed_and_order_event`.
    let txn = db.begin().await.expect("begin transaction");

    let active_sale_id = ensure_order_status(&txn, "active_sale").await;
    let confirmed_id = ensure_order_status(&txn, "confirmed").await;
    let processing_id = ensure_order_status(&txn, "processing").await;
    let _shipped_id = ensure_order_status(&txn, "shipped").await;
    let delivered_id = ensure_order_status(&txn, "delivered").await;

    let mut order = orders::Entity::find_by_id(order_id)
        .one(&txn)
        .await
        .expect("query order")
        .expect("order exists");
    assert_eq!(order.status_id, active_sale_id);

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
    .expect("pending → confirmed");

    let _ = core_operations::handlers::orders::update_order(
        &txn,
        Request::new(UpdateOrderRequest {
            order_id,
            user_id,
            shipping_address_id: shipping_id,
            total_amount_paise: total_paise,
            status_id: processing_id,
        }),
    )
    .await
    .expect("confirmed → processing");

    make_order_booking_eligible(&txn, order_id).await;

    let _ = core_operations::handlers::orders::admin_mark_order_shipped(
        &txn,
        &db, // pre-existing signature drift, unrelated to the place_order adaptation — see final report
        Request::new(AdminMarkOrderShippedRequest {
            order_id,
            awb_code: Some("FULL-AWB".to_string()),
            carrier: Some("FullCarrier".to_string()),
            shiprocket_book: None,
            shiprocket_order_id: None,
            shiprocket_status_id: None,
            shiprocket_status_label: None,
        }),
    )
    .await
    .expect("processing → shipped");

    let _ = core_operations::handlers::orders::admin_mark_order_delivered(
        &txn,
        Request::new(AdminMarkOrderDeliveredRequest { order_id }),
    )
    .await
    .expect("shipped → delivered");

    order = orders::Entity::find_by_id(order_id)
        .one(&txn)
        .await
        .expect("query order")
        .expect("order exists");
    assert_eq!(order.status_id, delivered_id);

    let events = order_events::Entity::find()
        .filter(order_events::Column::OrderId.eq(order_id))
        .all(&txn)
        .await
        .expect("query order_events");
    let to_statuses: Vec<Option<&str>> = events.iter().map(|e| e.to_status.as_deref()).collect();
    assert!(to_statuses.contains(&Some("confirmed")));
    assert!(to_statuses.contains(&Some("processing")));
    assert!(to_statuses.contains(&Some("shipped")));
    assert!(to_statuses.contains(&Some("delivered")));

    txn.rollback().await.ok();

    if let Err(e) = cleanup_order_state_fixtures(&db, &fixtures).await {
        eprintln!("warning: scenario cleanup failed (non-fatal): {e}");
    }
}
