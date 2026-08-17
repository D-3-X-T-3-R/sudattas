//! Integration tests for abandoned cart (enqueue_abandoned_cart_events) and outbox (place_order, shipped, delivered).
//!
//! **Setup**
//! - Set `TEST_DATABASE_URL` or `DATABASE_URL`.
//! - Schema must be loaded (e.g. migrations or `backend/database/sql_dump/01_schema.sql`).
//!
//! **Run**
//! - `cargo test --test integration_abandoned_cart_outbox -- --ignored`
//!
//! **Note:** AC1 and AC2 commit data (enqueue_abandoned_cart_events commits) and clean up
//! explicitly. OB1 and OB2 place an order via `place_order_setup`, which now must commit its
//! setup and the placed order itself (place_order manages its own transactions and can no
//! longer run as a nested savepoint — see the comment on `place_order_setup`'s `txn.commit()`
//! call), so both also clean up their place_order fixtures explicitly; OB2's own
//! update_order/admin_mark_* calls under test still run inside a rollback-able transaction, same
//! as before.

mod integration_common;

use chrono::{Duration, Utc};
use core_db_entities::entity::{
    inventory, order_status, outbox_events, product_categories, product_variants, products,
    shipping_addresses, user_roles, users,
};
use core_operations::handlers::outbox::{DELIVERED, ORDER_PLACED, SHIPPED};
use core_operations::order_state_machine;
use core_operations::procedures::abandoned_cart::enqueue_abandoned_cart_events;
use core_operations::procedures::orders::place_order;
use integration_common::test_db_url;
use proto::proto::core::{
    AdminMarkOrderDeliveredRequest, AdminMarkOrderShippedRequest, CreateCartItemRequest,
    CreateUserRequest, PlaceOrderRequest, UpdateOrderRequest,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, Database, EntityTrait,
    IntoActiveModel, QueryFilter, Statement, TransactionTrait,
};
use tonic::Request;

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

fn should_run_live_logistics_coupled_test() -> bool {
    let flag = std::env::var("RUN_LIVE_LOGISTICS_TESTS").ok();
    if flag.as_deref() == Some("1") {
        return true;
    }
    let current = flag.unwrap_or_else(|| "<unset>".to_string());
    eprintln!(
        "skipping provider-coupled test: RUN_LIVE_LOGISTICS_TESTS must be exactly '1' (current: {current})"
    );
    false
}

/// AC1 – Stale user cart with marketing_opt_out = 0 triggers enqueue_abandoned_cart_events and enqueues one outbox event.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_abandoned_cart_opt_in_enqueues_one_event() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");

    let now_tag = Utc::now().timestamp_millis();
    let txn = db.begin().await.expect("begin");
    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(format!("itest_ac1_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert UserRoles");

    let user_res = core_operations::handlers::users::create_user(
        &txn,
        Request::new(CreateUserRequest {
            username: format!("itest_ac1_{}", now_tag),
            email: format!("itest_ac1+{}@example.com", now_tag),
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

    let _ = txn
        .execute(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::MySql,
            "UPDATE Users SET marketing_opt_out = 0 WHERE UserID = ?",
            [user_id.into()],
        ))
        .await
        .expect("set marketing_opt_out = 0");

    let cat = product_categories::ActiveModel {
        category_id: ActiveValue::NotSet,
        name: ActiveValue::Set(format!("itest_cat_ac1_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductCategories");

    let prod = products::ActiveModel {
        product_id: ActiveValue::NotSet,
        sku: ActiveValue::Set(None),
        name: ActiveValue::Set("AC Product".to_string()),
        slug: ActiveValue::Set(None),
        description: ActiveValue::Set(None),
        price_paise: ActiveValue::Set(1_000),
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

    let _ = core_operations::handlers::cart::create_cart_item(
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

    let stale_at = Utc::now() - Duration::hours(25);
    let _ = txn
        .execute(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::MySql,
            "UPDATE Cart SET updated_at = ? WHERE UserID = ?",
            [stale_at.into(), user_id.into()],
        ))
        .await
        .expect("set cart updated_at to stale");

    txn.commit().await.expect("commit setup");

    let count = enqueue_abandoned_cart_events(&db, 24)
        .await
        .expect("enqueue_abandoned_cart_events");
    assert!(
        count >= 1,
        "at least one user with stale cart and opt-in should enqueue (count may be >1 from prior runs)"
    );

    let txn2 = db.begin().await.expect("begin");
    let events = outbox_events::Entity::find()
        .filter(outbox_events::Column::EventType.eq("AbandonedCart"))
        .filter(outbox_events::Column::AggregateId.eq(user_id.to_string()))
        .all(&txn2)
        .await
        .expect("query outbox_events");
    txn2.rollback().await.ok();
    assert!(
        !events.is_empty(),
        "our user should have at least one AbandonedCart event (may be >1 from prior runs)"
    );
}

/// AC2 – Stale cart but marketing_opt_out = 1 results in no abandoned-cart events enqueued.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_abandoned_cart_opt_out_no_events() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");

    let now_tag = Utc::now().timestamp_millis();
    let txn = db.begin().await.expect("begin");
    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(format!("itest_ac2_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert UserRoles");

    let user_res = core_operations::handlers::users::create_user(
        &txn,
        Request::new(CreateUserRequest {
            username: format!("itest_ac2_{}", now_tag),
            email: format!("itest_ac2+{}@example.com", now_tag),
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

    let user = users::Entity::find_by_id(user_id)
        .one(&txn)
        .await
        .expect("find user")
        .expect("user exists");
    let mut active = user.into_active_model();
    active.marketing_opt_out = ActiveValue::Set(Some(1));
    active
        .update(&txn)
        .await
        .expect("set marketing_opt_out = 1");

    let cat = product_categories::ActiveModel {
        category_id: ActiveValue::NotSet,
        name: ActiveValue::Set(format!("itest_cat_ac2_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductCategories");

    let prod = products::ActiveModel {
        product_id: ActiveValue::NotSet,
        sku: ActiveValue::Set(None),
        name: ActiveValue::Set("AC2 Product".to_string()),
        slug: ActiveValue::Set(None),
        description: ActiveValue::Set(None),
        price_paise: ActiveValue::Set(1_000),
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

    let _ = core_operations::handlers::cart::create_cart_item(
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

    let stale_at = Utc::now() - Duration::hours(25);
    let _ = txn
        .execute(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::MySql,
            "UPDATE Cart SET updated_at = ? WHERE UserID = ?",
            [stale_at.into(), user_id.into()],
        ))
        .await
        .expect("set cart updated_at to stale");

    txn.commit().await.expect("commit setup");

    let _count = enqueue_abandoned_cart_events(&db, 24)
        .await
        .expect("enqueue_abandoned_cart_events");

    let txn2 = db.begin().await.expect("begin");
    let events = outbox_events::Entity::find()
        .filter(outbox_events::Column::EventType.eq("AbandonedCart"))
        .filter(outbox_events::Column::AggregateId.eq(user_id.to_string()))
        .all(&txn2)
        .await
        .expect("query outbox_events");
    txn2.rollback().await.ok();
    assert!(
        events.is_empty(),
        "opted-out user should have no AbandonedCart event (found {}); total enqueued may be >0 from other users",
        events.len()
    );
}

/// Fixture ids from `place_order_setup`: the placed order plus everything created to place it,
/// needed both by the OB tests that build on top of it and (now that setup/place_order must be
/// committed rather than left inside a rollback-able transaction — see the comment on the
/// `txn.commit()` call below) to clean the rows up afterward.
struct OutboxOrderFixtures {
    order_id: i64,
    user_id: i64,
    shipping_id: i64,
    total_paise: i64,
    role_id: i64,
    category_id: i64,
    variant_id: i64,
}

/// Place order minimal setup.
///
/// Setup runs in its own transaction, committed before calling `place_order`: `place_order` now
/// manages its own transactions internally (a short claim/prep transaction, then external calls
/// with no DB connection held, then a write transaction) so it can no longer run as a nested
/// savepoint inside a caller-supplied, still-open transaction. Setup fixtures must therefore be
/// committed and visible before place_order's own prep phase reads them, rather than sharing one
/// uncommitted transaction with it as before. OB2 (the only caller with further mutations to
/// test) opens its own fresh transaction after this returns to wrap the update_order/
/// admin_mark_* calls it is actually testing (those handlers are unaffected by the place_order
/// restructuring and still take `&DatabaseTransaction`).
async fn place_order_setup(db: &sea_orm::DatabaseConnection, now_tag: i64) -> OutboxOrderFixtures {
    // Make checkout deterministic in CI/local by ensuring this test order
    // always qualifies for free shipping and never needs a live quote.
    std::env::set_var("FREE_SHIPPING_THRESHOLD_MINOR", "100000");

    let txn = db.begin().await.expect("begin transaction");
    let _ = ensure_order_status(&txn, "pending").await;
    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(format!("itest_ob_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert UserRoles");

    let user_res = core_operations::handlers::users::create_user(
        &txn,
        Request::new(CreateUserRequest {
            username: format!("itest_ob_{}", now_tag),
            email: format!("itest_ob+{}@example.com", now_tag),
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
        name: ActiveValue::Set(format!("itest_cat_ob_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductCategories");

    let prod = products::ActiveModel {
        product_id: ActiveValue::NotSet,
        sku: ActiveValue::Set(None),
        name: ActiveValue::Set("Outbox Product".to_string()),
        slug: ActiveValue::Set(None),
        description: ActiveValue::Set(None),
        // Keep subtotal above FREE_SHIPPING_THRESHOLD_MINOR to avoid live quote dependency in order placement.
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

    OutboxOrderFixtures {
        order_id: order.order_id,
        user_id,
        shipping_id,
        total_paise: order.total_amount_paise,
        role_id: role.role_id,
        category_id: cat.category_id,
        variant_id: variant.variant_id,
    }
}

/// Best-effort teardown of everything `place_order_setup` committed (fixtures + the placed order
/// and its dependents). Previously this all lived inside the same uncommitted transaction as the
/// calling test (rolled back at the end); now that place_order_setup's work is committed
/// independently (see the comment on its `txn.commit()` call), it needs explicit cleanup
/// instead. Deletes in FK-safe order (children before parents). No `OutboxEvents` cleanup is
/// needed here: OB1 asserts none exist for this order, and OB2's Shipped/Delivered events are
/// created inside its own rollback-able transaction, never committed. Errors here are logged,
/// not fatal — they must never mask the actual test assertions.
async fn cleanup_outbox_order_fixtures(
    db: &sea_orm::DatabaseConnection,
    fixtures: &OutboxOrderFixtures,
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

/// OB1 – place_order does not enqueue OrderPlaced; confirmation email is enqueued on PaymentCaptured (Paid).
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_place_order_does_not_enqueue_order_placed_outbox() {
    if !should_run_live_logistics_coupled_test() {
        return;
    }

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");

    let now_tag = Utc::now().timestamp_millis();
    let fixtures = place_order_setup(&db, now_tag).await;

    // No further mutation is under test here, so read the committed state directly via `db`
    // rather than opening a transaction — place_order_setup already committed everything (see
    // the comment on its `txn.commit()` call).
    let events = outbox_events::Entity::find()
        .filter(outbox_events::Column::EventType.eq(ORDER_PLACED))
        .filter(outbox_events::Column::AggregateId.eq(fixtures.order_id.to_string()))
        .all(&db)
        .await
        .expect("query outbox_events");
    assert!(
        events.is_empty(),
        "OrderPlaced outbox removed; customer email fires on payment success (PaymentCaptured)"
    );

    if let Err(e) = cleanup_outbox_order_fixtures(&db, &fixtures).await {
        eprintln!("warning: scenario cleanup failed (non-fatal): {e}");
    }
}

/// OB2 – admin_mark_order_shipped enqueues Shipped; admin_mark_order_delivered enqueues Delivered.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_shipped_delivered_enqueue_outbox_events() {
    if !should_run_live_logistics_coupled_test() {
        return;
    }

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");

    let now_tag = Utc::now().timestamp_millis();
    let fixtures = place_order_setup(&db, now_tag).await;
    let (order_id, user_id, shipping_id, total_paise) = (
        fixtures.order_id,
        fixtures.user_id,
        fixtures.shipping_id,
        fixtures.total_paise,
    );

    // Fresh transaction for the update_order/admin_mark_* calls under test; place_order_setup
    // already committed its own work (see the comment on its `txn.commit()` call), so this only
    // wraps the state-machine mutations and their assertions, same as before.
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

    // Shipping handlers now enforce shared booking eligibility:
    // booking window must be open and prepaid orders must be captured.
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
        .expect("make order booking-eligible for shipped event test");

    let _ = core_operations::handlers::orders::admin_mark_order_shipped(
        &txn,
        &db, // pre-existing signature drift, unrelated to the place_order adaptation — see final report
        Request::new(AdminMarkOrderShippedRequest {
            order_id,
            awb_code: Some("OB2AWB".to_string()),
            carrier: Some("Carrier".to_string()),
            shiprocket_book: None,
            shiprocket_order_id: None,
            shiprocket_status_id: None,
            shiprocket_status_label: None,
        }),
    )
    .await
    .expect("admin_mark_order_shipped");

    let shipped = outbox_events::Entity::find()
        .filter(outbox_events::Column::EventType.eq(SHIPPED))
        .filter(outbox_events::Column::AggregateId.eq(order_id.to_string()))
        .all(&txn)
        .await
        .expect("query outbox_events");
    assert_eq!(shipped.len(), 1);
    assert_eq!(
        shipped[0]
            .payload
            .get("order_id")
            .and_then(|v: &serde_json::Value| v.as_i64()),
        Some(order_id)
    );

    let _ = core_operations::handlers::orders::admin_mark_order_delivered(
        &txn,
        Request::new(AdminMarkOrderDeliveredRequest { order_id }),
    )
    .await
    .expect("admin_mark_order_delivered");

    let delivered = outbox_events::Entity::find()
        .filter(outbox_events::Column::EventType.eq(DELIVERED))
        .filter(outbox_events::Column::AggregateId.eq(order_id.to_string()))
        .all(&txn)
        .await
        .expect("query outbox_events");
    assert_eq!(delivered.len(), 1);
    assert_eq!(
        delivered[0]
            .payload
            .get("order_id")
            .and_then(|v: &serde_json::Value| v.as_i64()),
        Some(order_id)
    );

    txn.rollback().await.ok();

    if let Err(e) = cleanup_outbox_order_fixtures(&db, &fixtures).await {
        eprintln!("warning: scenario cleanup failed (non-fatal): {e}");
    }
}
