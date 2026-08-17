//! Integration tests for coupons and promotions (create, place_order with coupon, apply_coupon, limits).
//!
//! **Setup**
//! - Set `TEST_DATABASE_URL` or `DATABASE_URL`.
//! - Schema must be loaded (e.g. migrations or `backend/database/sql_dump/01_schema.sql`).
//!
//! **Run**
//! - `cargo test --test integration_coupons -- --ignored`

mod integration_common;
mod provider_test_gate;

use chrono::{Duration, Utc};
use integration_common::test_db_url;

use core_db_entities::entity::{
    coupons, inventory, order_status, orders, product_categories, product_variants, products,
    shipping_addresses, user_roles,
};
use core_operations::procedures::orders::place_order;
use proto::proto::core::{
    ApplyCouponRequest, CreateCartItemRequest, CreateCouponRequest, CreateUserRequest,
    PlaceOrderRequest,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Database, EntityTrait, QueryFilter,
    TransactionTrait,
};
use tonic::Request;

/// Fixture ids created by `ensure_pending_and_place_order_setup`, needed both to build the
/// `PlaceOrderRequest` and (now that setup must be committed for place_order to see it — see
/// the comment above each test's `txn.commit()` call) to clean the rows up afterward.
struct CouponScenarioFixtures {
    user_id: i64,
    shipping_id: i64,
    cart_id: i64,
    role_id: i64,
    category_id: i64,
    variant_id: i64,
}

async fn ensure_pending_and_place_order_setup(
    txn: &sea_orm::DatabaseTransaction,
    now_tag: i64,
    cart_total_paise: i64,
) -> CouponScenarioFixtures {
    let pending = order_status::Entity::find()
        .filter(order_status::Column::StatusName.eq("pending"))
        .one(txn)
        .await
        .expect("query OrderStatus");
    if pending.is_none() {
        let status = order_status::ActiveModel {
            status_id: ActiveValue::NotSet,
            status_name: ActiveValue::Set("pending".to_string()),
        };
        let _ = status
            .insert(txn)
            .await
            .expect("insert pending OrderStatus");
    }

    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(format!("itest_cp_{}", now_tag)),
    }
    .insert(txn)
    .await
    .expect("insert UserRoles");

    let user_res = core_operations::handlers::users::create_user(
        txn,
        Request::new(CreateUserRequest {
            username: format!("itest_cp_{}", now_tag),
            email: format!("itest_cp+{}@example.com", now_tag),
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
    .insert(txn)
    .await
    .expect("insert ShippingAddresses");
    let shipping_id = ship.shipping_address_id;

    let cat = product_categories::ActiveModel {
        category_id: ActiveValue::NotSet,
        name: ActiveValue::Set(format!("itest_cat_cp_{}", now_tag)),
    }
    .insert(txn)
    .await
    .expect("insert ProductCategories");

    let prod = products::ActiveModel {
        product_id: ActiveValue::NotSet,
        sku: ActiveValue::Set(None),
        name: ActiveValue::Set("Coupon Test Product".to_string()),
        slug: ActiveValue::Set(None),
        description: ActiveValue::Set(None),
        price_paise: ActiveValue::Set(cart_total_paise as i32),
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
    .insert(txn)
    .await
    .expect("insert Products");

    let variant = product_variants::ActiveModel {
        variant_id: ActiveValue::NotSet,
        product_id: ActiveValue::Set(prod.product_id),
        size_id: ActiveValue::Set(None),
        color_id: ActiveValue::Set(None),
        additional_price: ActiveValue::Set(Some(0)),
    }
    .insert(txn)
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
    .insert(txn)
    .await
    .expect("insert Inventory");

    let cart_res = core_operations::handlers::cart::create_cart_item(
        txn,
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

    CouponScenarioFixtures {
        user_id,
        shipping_id,
        cart_id,
        role_id: role.role_id,
        category_id: cat.category_id,
        variant_id: variant.variant_id,
    }
}

/// Best-effort teardown of everything `ensure_pending_and_place_order_setup` committed, plus
/// anything place_order itself persisted. Setup used to live inside the same uncommitted
/// transaction as the place_order call under test, so a `txn.rollback()` at the end undid it
/// automatically; now that setup must be committed before place_order's own (separately-opened)
/// transactions can see it (see the comment above each test's `txn.commit()` call), it needs
/// explicit cleanup instead. Deletes in FK-safe order (children before parents). Errors here are
/// logged, not fatal — they must never mask the actual test assertions.
async fn cleanup_coupon_scenario_rows(
    db: &sea_orm::DatabaseConnection,
    fixtures: &CouponScenarioFixtures,
    coupon_code: Option<&str>,
    observed_order_id: Option<i64>,
) -> Result<(), String> {
    use sea_orm::{ConnectionTrait, DbBackend, Statement};

    if let Some(order_id) = observed_order_id {
        for (table, column) in [
            ("PaymentIntents", "order_id"),
            ("Shipments", "order_id"),
            ("OrderDetails", "OrderID"),
            ("OrderEvents", "OrderID"),
            ("Orders", "OrderID"),
        ] {
            db.execute(Statement::from_sql_and_values(
                DbBackend::MySql,
                format!("DELETE FROM `{table}` WHERE `{column}` = ?"),
                [order_id.into()],
            ))
            .await
            .map_err(|e| format!("cleanup {table} for order_id={order_id}: {e}"))?;
        }
    }
    db.execute(Statement::from_sql_and_values(
        DbBackend::MySql,
        "DELETE FROM `Cart` WHERE `UserID` = ?",
        [fixtures.user_id.into()],
    ))
    .await
    .map_err(|e| format!("cleanup Cart for user_id={}: {e}", fixtures.user_id))?;
    db.execute(Statement::from_sql_and_values(
        DbBackend::MySql,
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
        DbBackend::MySql,
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
        DbBackend::MySql,
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
        DbBackend::MySql,
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
        DbBackend::MySql,
        "DELETE FROM `ShippingAddresses` WHERE `ShippingAddressID` = ?",
        [fixtures.shipping_id.into()],
    ))
    .await
    .map_err(|e| format!("cleanup ShippingAddresses id={}: {e}", fixtures.shipping_id))?;
    if let Some(code) = coupon_code {
        db.execute(Statement::from_sql_and_values(
            DbBackend::MySql,
            "DELETE FROM `Coupons` WHERE `code` = ?",
            [code.into()],
        ))
        .await
        .map_err(|e| format!("cleanup Coupons for code={code}: {e}"))?;
    }
    db.execute(Statement::from_sql_and_values(
        DbBackend::MySql,
        "DELETE FROM `Users` WHERE `UserID` = ?",
        [fixtures.user_id.into()],
    ))
    .await
    .map_err(|e| format!("cleanup Users for user_id={}: {e}", fixtures.user_id))?;
    db.execute(Statement::from_sql_and_values(
        DbBackend::MySql,
        "DELETE FROM `UserRoles` WHERE `RoleID` = ?",
        [fixtures.role_id.into()],
    ))
    .await
    .map_err(|e| format!("cleanup UserRoles for role_id={}: {e}", fixtures.role_id))?;
    Ok(())
}

/// CP1 – create_coupon + place_order with valid coupon applies discount to grand_total_minor and order snapshot.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_coupon_applied_at_checkout_reduces_total() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_coupon_applied_at_checkout_reduces_total",
    ) {
        return;
    }

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    // Keep subtotal above FREE_SHIPPING_THRESHOLD_MINOR so checkout does not depend on live shipping quote.
    let cart_total = 200_000_i64;
    let fixtures = ensure_pending_and_place_order_setup(&txn, now_tag, cart_total).await;

    let code = format!("CP1_{}", now_tag);
    let _ = core_operations::handlers::coupons::create_coupon(
        &txn,
        Request::new(CreateCouponRequest {
            code: code.clone(),
            discount_type: "fixed_amount".to_string(),
            discount_value: 500,
            min_order_value_paise: Some(1000),
            usage_limit: Some(10),
            max_uses_per_customer: None,
            starts_at: (Utc::now() - Duration::hours(1)).to_rfc3339(),
            ends_at: Some((Utc::now() + Duration::days(7)).to_rfc3339()),
        }),
    )
    .await
    .expect("create_coupon");

    // Commit setup so it's visible to place_order's own (separately-opened) transactions —
    // place_order no longer runs as a nested savepoint inside our transaction, so anything it
    // needs to read (user, cart, shipping address, coupon) must already be committed.
    txn.commit().await.expect("commit setup txn");

    let place_res = place_order(
        &db,
        Request::new(PlaceOrderRequest {
            shipping_address_id: fixtures.shipping_id,
            user_id: fixtures.user_id,
            coupon_code: Some(code.clone()),
            selected_cart_ids: vec![fixtures.cart_id],
            payment_mode: None,
        }),
    )
    .await
    .expect("place_order should succeed");
    let order = place_res.into_inner().items[0].clone();
    assert_eq!(
        order.total_amount_paise, 199_500,
        "200000 - 500 discount = 199500"
    );

    let db_order = orders::Entity::find_by_id(order.order_id)
        .one(&db)
        .await
        .expect("query order")
        .expect("order exists");
    assert_eq!(db_order.grand_total_minor, 199_500);
    assert!(db_order.applied_coupon_id.is_some());
    assert_eq!(db_order.applied_coupon_code.as_deref(), Some(code.as_str()));
    assert_eq!(db_order.applied_discount_paise, Some(500));

    // Best-effort cleanup of the now-committed setup fixtures and the order place_order
    // persisted (see the comment on `cleanup_coupon_scenario_rows`).
    if let Err(e) =
        cleanup_coupon_scenario_rows(&db, &fixtures, Some(code.as_str()), Some(order.order_id))
            .await
    {
        eprintln!("warning: scenario cleanup failed (non-fatal): {e}");
    }
}

/// CP2 – Expired coupon (past ends_at) is ignored at checkout; full price charged and no coupon snapshot stored.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_expired_coupon_ignored_at_checkout() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_expired_coupon_ignored_at_checkout",
    ) {
        return;
    }

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let cart_total = 200_000_i64;
    let fixtures = ensure_pending_and_place_order_setup(&txn, now_tag, cart_total).await;

    let code = format!("CP2_EXP_{}", now_tag);
    let _ = core_operations::handlers::coupons::create_coupon(
        &txn,
        Request::new(CreateCouponRequest {
            code: code.clone(),
            discount_type: "percentage".to_string(),
            discount_value: 20,
            min_order_value_paise: Some(0),
            usage_limit: Some(100),
            max_uses_per_customer: None,
            starts_at: (Utc::now() - Duration::days(1)).to_rfc3339(),
            ends_at: Some((Utc::now() - Duration::hours(1)).to_rfc3339()),
        }),
    )
    .await
    .expect("create_coupon");

    // Commit setup so it's visible to place_order's own (separately-opened) transactions —
    // see the comment on the equivalent commit in `integration_coupon_applied_at_checkout_reduces_total`.
    txn.commit().await.expect("commit setup txn");

    let place_res = place_order(
        &db,
        Request::new(PlaceOrderRequest {
            shipping_address_id: fixtures.shipping_id,
            user_id: fixtures.user_id,
            coupon_code: Some(code.clone()),
            selected_cart_ids: vec![fixtures.cart_id],
            payment_mode: None,
        }),
    )
    .await
    .expect("place_order should succeed");
    let order = place_res.into_inner().items[0].clone();
    assert_eq!(
        order.total_amount_paise, cart_total,
        "full price when coupon expired"
    );

    let db_order = orders::Entity::find_by_id(order.order_id)
        .one(&db)
        .await
        .expect("query order")
        .expect("order exists");
    assert_eq!(db_order.grand_total_minor, cart_total);
    assert!(db_order.applied_coupon_id.is_none());
    assert!(db_order.applied_coupon_code.is_none());

    if let Err(e) =
        cleanup_coupon_scenario_rows(&db, &fixtures, Some(code.as_str()), Some(order.order_id))
            .await
    {
        eprintln!("warning: scenario cleanup failed (non-fatal): {e}");
    }
}

/// CP3 – apply_coupon is preview-only and does not increment usage_count.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_apply_coupon_is_preview_only() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let create_res = core_operations::handlers::coupons::create_coupon(
        &txn,
        Request::new(CreateCouponRequest {
            code: format!("CP3_{}", now_tag),
            discount_type: "fixed_amount".to_string(),
            discount_value: 100,
            min_order_value_paise: Some(500),
            usage_limit: Some(5),
            max_uses_per_customer: None,
            starts_at: (Utc::now() - Duration::seconds(1)).to_rfc3339(),
            ends_at: Some((Utc::now() + Duration::days(1)).to_rfc3339()),
        }),
    )
    .await
    .expect("create_coupon");
    let code = create_res.into_inner().items[0].code.clone();

    let apply_res = core_operations::handlers::coupons::apply_coupon(
        &txn,
        Request::new(ApplyCouponRequest {
            code: code.clone(),
            order_amount_paise: 1_000,
        }),
    )
    .await
    .expect("apply_coupon should succeed");
    assert!(apply_res.into_inner().items[0].is_valid);

    let coupon = coupons::Entity::find()
        .filter(coupons::Column::Code.eq(&code))
        .one(&txn)
        .await
        .expect("query coupon")
        .expect("coupon exists");
    assert_eq!(coupon.usage_count, Some(0));

    txn.rollback().await.ok();
}

/// CP4 – Coupon with usage_limit = 1: repeated apply_coupon previews stay valid until payment finalization.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_coupon_usage_limit_preview_does_not_burn_limit() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let create_res = core_operations::handlers::coupons::create_coupon(
        &txn,
        Request::new(CreateCouponRequest {
            code: format!("CP4_{}", now_tag),
            discount_type: "percentage".to_string(),
            discount_value: 10,
            min_order_value_paise: Some(0),
            usage_limit: Some(1),
            max_uses_per_customer: None,
            starts_at: (Utc::now() - Duration::seconds(1)).to_rfc3339(),
            ends_at: None,
        }),
    )
    .await
    .expect("create_coupon");
    let code = create_res.into_inner().items[0].code.clone();

    let first = core_operations::handlers::coupons::apply_coupon(
        &txn,
        Request::new(ApplyCouponRequest {
            code: code.clone(),
            order_amount_paise: 1_000,
        }),
    )
    .await
    .expect("first apply_coupon should not error");
    assert!(first.into_inner().items[0].is_valid);

    let second = core_operations::handlers::coupons::apply_coupon(
        &txn,
        Request::new(ApplyCouponRequest {
            code: code.clone(),
            order_amount_paise: 1_000,
        }),
    )
    .await
    .expect("second apply_coupon returns Ok (handler does not error)");
    let result = second.into_inner().items[0].clone();
    assert!(
        result.is_valid,
        "preview should not consume coupon capacity"
    );

    let coupon = coupons::Entity::find()
        .filter(coupons::Column::Code.eq(&code))
        .one(&txn)
        .await
        .expect("query coupon")
        .expect("coupon exists");
    assert_eq!(coupon.usage_count, Some(0));

    txn.rollback().await.ok();
}

/// CP5 – Min order value not met: checkout rejects coupon and leaves order total undiscounted.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_coupon_min_order_not_met_not_applied() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_coupon_min_order_not_met_not_applied",
    ) {
        return;
    }

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let cart_total = 150_000_i64;
    let fixtures = ensure_pending_and_place_order_setup(&txn, now_tag, cart_total).await;

    let code = format!("CP5_MIN_{}", now_tag);
    let _ = core_operations::handlers::coupons::create_coupon(
        &txn,
        Request::new(CreateCouponRequest {
            code: code.clone(),
            discount_type: "fixed_amount".to_string(),
            discount_value: 200,
            min_order_value_paise: Some(300_000),
            usage_limit: Some(10),
            max_uses_per_customer: None,
            starts_at: Utc::now().to_rfc3339(),
            ends_at: None,
        }),
    )
    .await
    .expect("create_coupon");

    // Commit setup so it's visible to place_order's own (separately-opened) transactions —
    // see the comment on the equivalent commit in `integration_coupon_applied_at_checkout_reduces_total`.
    txn.commit().await.expect("commit setup txn");

    let place_res = place_order(
        &db,
        Request::new(PlaceOrderRequest {
            shipping_address_id: fixtures.shipping_id,
            user_id: fixtures.user_id,
            coupon_code: Some(code.clone()),
            selected_cart_ids: vec![fixtures.cart_id],
            payment_mode: None,
        }),
    )
    .await
    .expect("place_order should succeed");
    let order = place_res.into_inner().items[0].clone();
    assert_eq!(
        order.total_amount_paise, cart_total,
        "total undiscounted when min not met"
    );

    let db_order = orders::Entity::find_by_id(order.order_id)
        .one(&db)
        .await
        .expect("query order")
        .expect("order exists");
    assert_eq!(db_order.grand_total_minor, cart_total);
    assert!(db_order.applied_coupon_id.is_none());
    assert!(db_order.applied_coupon_code.is_none());

    if let Err(e) =
        cleanup_coupon_scenario_rows(&db, &fixtures, Some(code.as_str()), Some(order.order_id))
            .await
    {
        eprintln!("warning: scenario cleanup failed (non-fatal): {e}");
    }
}

/// CP6 – Free-shipping threshold is evaluated on post-discount items total.
/// In test env, shipping quote is unavailable; crossing below threshold after coupon should fail checkout.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_free_shipping_threshold_uses_post_discount_total() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_free_shipping_threshold_uses_post_discount_total",
    ) {
        return;
    }

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let cart_total = 120_000_i64;
    let fixtures = ensure_pending_and_place_order_setup(&txn, now_tag, cart_total).await;

    let code = format!("CP6_THRESH_{}", now_tag);
    let _ = core_operations::handlers::coupons::create_coupon(
        &txn,
        Request::new(CreateCouponRequest {
            code: code.clone(),
            discount_type: "fixed_amount".to_string(),
            discount_value: 30_001,
            min_order_value_paise: Some(1),
            usage_limit: Some(10),
            max_uses_per_customer: None,
            starts_at: (Utc::now() - Duration::hours(1)).to_rfc3339(),
            ends_at: Some((Utc::now() + Duration::days(1)).to_rfc3339()),
        }),
    )
    .await
    .expect("create_coupon");

    // Commit setup so it's visible to place_order's own (separately-opened) transactions —
    // see the comment on the equivalent commit in `integration_coupon_applied_at_checkout_reduces_total`.
    txn.commit().await.expect("commit setup txn");

    let err = place_order(
        &db,
        Request::new(PlaceOrderRequest {
            shipping_address_id: fixtures.shipping_id,
            user_id: fixtures.user_id,
            coupon_code: Some(code.clone()),
            selected_cart_ids: vec![fixtures.cart_id],
            payment_mode: None,
        }),
    )
    .await
    .expect_err("post-discount subtotal should require live shipping quote in this test env");

    assert_eq!(err.code(), tonic::Code::Unavailable);
    assert!(
        err.message()
            .contains("Live shipping quote is unavailable for this checkout"),
        "unexpected error message: {}",
        err.message()
    );

    // place_order failed during the external-call phase (no write transaction ever opened), so
    // no Orders/OrderDetails row exists to clean up here — only the setup fixtures themselves.
    if let Err(e) = cleanup_coupon_scenario_rows(&db, &fixtures, Some(code.as_str()), None).await {
        eprintln!("warning: scenario cleanup failed (non-fatal): {e}");
    }
}
