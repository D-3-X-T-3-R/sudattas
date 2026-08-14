//! Integration tests for selected-cart-line checkout.
//!
//! **Setup**
//! - Set `TEST_DATABASE_URL` or `DATABASE_URL`.
//! - Schema must be loaded (e.g. migrations or `backend/database/sql_dump/01_schema.sql`).
//!
//! **Run**
//! - `cargo test --test integration_selected_checkout -- --ignored`

mod integration_common;
mod provider_test_gate;

use chrono::{Duration, Utc};
use integration_common::test_db_url;

use core_db_entities::entity::{
    cart, coupons, inventory, order_details, order_status, orders, payment_intents,
    product_categories, product_variants, products, shipping_addresses, user_roles,
};
use core_operations::handlers::orders::{cancel_order_items, delete_order};
use core_operations::procedures::orders::place_order;
use proto::proto::core::{
    CancelOrderItemsRequest, CreateCartItemRequest, CreateCouponRequest, CreateUserRequest,
    DeleteOrderRequest, PlaceOrderRequest,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, Database, DatabaseConnection,
    DbBackend, EntityTrait, QueryFilter, QueryOrder, Statement, TransactionTrait,
};
use tonic::{Code, Request};

// Generalized to `&impl ConnectionTrait` (rather than the previous `&DatabaseTransaction`)
// because place_order no longer runs as a nested savepoint inside a caller-supplied, still-open
// transaction (see integration_place_order_atomicity.rs for the full rationale), so this helper
// now gets called both during setup (still inside the setup transaction) and after place_order
// returns (against the plain `DatabaseConnection`). The lookup is inlined directly against the
// `order_status` entity rather than delegating to `order_state_machine::get_status_id`, since
// that library helper's signature is still pinned to `&DatabaseTransaction` specifically and
// can't accept a generic connection; the query itself is unchanged.
async fn ensure_order_status(conn: &impl ConnectionTrait, name: &str) -> i64 {
    if let Ok(Some(existing)) = order_status::Entity::find()
        .filter(order_status::Column::StatusName.eq(name))
        .one(conn)
        .await
    {
        return existing.status_id;
    }
    let m = order_status::ActiveModel {
        status_id: ActiveValue::NotSet,
        status_name: ActiveValue::Set(name.to_string()),
    }
    .insert(conn)
    .await
    .expect("insert OrderStatus");
    m.status_id
}

/// Returns `(user_id, shipping_address_id, role_id)`. `role_id` is only needed for best-effort
/// cleanup now that setup fixtures must be committed for place_order to see them (see
/// `cleanup_checkout_rows`), so it's threaded back out even though the original callers of this
/// helper (pre-restructuring) had no need for it.
async fn create_checkout_user(txn: &sea_orm::DatabaseTransaction, now_tag: i64) -> (i64, i64, i64) {
    let _ = ensure_order_status(txn, "pending").await;

    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(format!("itest_sel_role_{}", now_tag)),
    }
    .insert(txn)
    .await
    .expect("insert UserRoles");

    let user_res = core_operations::handlers::users::create_user(
        txn,
        Request::new(CreateUserRequest {
            username: format!("itest_sel_{}", now_tag),
            email: format!("itest_sel+{}@example.com", now_tag),
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

    let shipping = shipping_addresses::ActiveModel {
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

    (user_id, shipping.shipping_address_id, role.role_id)
}

/// Returns `(category_id, variant_id, cart_id)`. `category_id` is only needed for best-effort
/// cleanup (see `cleanup_checkout_rows`) — each call creates its own uniquely-named category, so
/// deleting `Products`/`ProductCategories` by this id exactly targets the product this call made.
async fn create_variant_with_cart_item(
    txn: &sea_orm::DatabaseTransaction,
    now_tag: i64,
    user_id: i64,
    name_suffix: &str,
    price_paise: i32,
    inventory_qty: i32,
    cart_qty: i64,
) -> (i64, i64, i64) {
    let category = product_categories::ActiveModel {
        category_id: ActiveValue::NotSet,
        name: ActiveValue::Set(format!("itest_sel_cat_{}_{}", name_suffix, now_tag)),
    }
    .insert(txn)
    .await
    .expect("insert ProductCategories");

    let product = products::ActiveModel {
        product_id: ActiveValue::NotSet,
        sku: ActiveValue::Set(None),
        name: ActiveValue::Set(format!("Selected Checkout {}", name_suffix)),
        slug: ActiveValue::Set(None),
        description: ActiveValue::Set(None),
        price_paise: ActiveValue::Set(price_paise),
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
    .expect("insert Products");

    let variant = product_variants::ActiveModel {
        variant_id: ActiveValue::NotSet,
        product_id: ActiveValue::Set(product.product_id),
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
        quantity_available: ActiveValue::Set(Some(i64::from(inventory_qty))),
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
            quantity: cart_qty,
        }),
    )
    .await
    .expect("create_cart_item");

    (
        category.category_id,
        variant.variant_id,
        cart_res.into_inner().items[0].cart_id,
    )
}

/// Best-effort cleanup of fixtures committed by these tests. Setup used to live inside the same
/// uncommitted transaction as the place_order call under test and simply never got committed on
/// rollback; now that setup must be committed for place_order's own (separately-opened)
/// transactions to see it (see the `txn.commit()` calls below), it needs explicit cleanup
/// instead. Deletes in FK-safe order (children before parents), mirroring
/// `cleanup_scenario_rows` in integration_place_order_atomicity.rs. Errors are logged, not
/// fatal — they must never mask the actual test assertions.
#[allow(clippy::too_many_arguments)]
async fn cleanup_checkout_rows(
    db: &DatabaseConnection,
    order_ids: &[i64],
    user_ids: &[i64],
    role_ids: &[i64],
    category_ids: &[i64],
    variant_ids: &[i64],
    shipping_address_ids: &[i64],
    coupon_codes: &[&str],
) {
    for order_id in order_ids {
        for (table, column) in [
            ("PaymentIntents", "order_id"),
            ("Shipments", "order_id"),
            ("OrderDetails", "OrderID"),
            ("OrderEvents", "OrderID"),
            ("Orders", "OrderID"),
        ] {
            if let Err(e) = db
                .execute(Statement::from_sql_and_values(
                    DbBackend::MySql,
                    format!("DELETE FROM `{table}` WHERE `{column}` = ?"),
                    [(*order_id).into()],
                ))
                .await
            {
                eprintln!(
                    "warning: cleanup {table} for order_id={order_id} failed (non-fatal): {e}"
                );
            }
        }
    }
    for user_id in user_ids {
        if let Err(e) = db
            .execute(Statement::from_sql_and_values(
                DbBackend::MySql,
                "DELETE FROM `Cart` WHERE `UserID` = ?",
                [(*user_id).into()],
            ))
            .await
        {
            eprintln!("warning: cleanup Cart for user_id={user_id} failed (non-fatal): {e}");
        }
    }
    for coupon_code in coupon_codes {
        if let Err(e) = db
            .execute(Statement::from_sql_and_values(
                DbBackend::MySql,
                "DELETE FROM `Coupons` WHERE `code` = ?",
                [(*coupon_code).into()],
            ))
            .await
        {
            eprintln!("warning: cleanup Coupons for code={coupon_code} failed (non-fatal): {e}");
        }
    }
    for variant_id in variant_ids {
        for table in ["Inventory", "ProductVariants"] {
            if let Err(e) = db
                .execute(Statement::from_sql_and_values(
                    DbBackend::MySql,
                    format!("DELETE FROM `{table}` WHERE `VariantID` = ?"),
                    [(*variant_id).into()],
                ))
                .await
            {
                eprintln!(
                    "warning: cleanup {table} for variant_id={variant_id} failed (non-fatal): {e}"
                );
            }
        }
    }
    for category_id in category_ids {
        for table in ["Products", "ProductCategories"] {
            if let Err(e) = db
                .execute(Statement::from_sql_and_values(
                    DbBackend::MySql,
                    format!("DELETE FROM `{table}` WHERE `CategoryID` = ?"),
                    [(*category_id).into()],
                ))
                .await
            {
                eprintln!(
                    "warning: cleanup {table} for category_id={category_id} failed (non-fatal): {e}"
                );
            }
        }
    }
    for shipping_address_id in shipping_address_ids {
        if let Err(e) = db
            .execute(Statement::from_sql_and_values(
                DbBackend::MySql,
                "DELETE FROM `ShippingAddresses` WHERE `ShippingAddressID` = ?",
                [(*shipping_address_id).into()],
            ))
            .await
        {
            eprintln!(
                "warning: cleanup ShippingAddresses id={shipping_address_id} failed (non-fatal): {e}"
            );
        }
    }
    for user_id in user_ids {
        if let Err(e) = db
            .execute(Statement::from_sql_and_values(
                DbBackend::MySql,
                "DELETE FROM `Users` WHERE `UserID` = ?",
                [(*user_id).into()],
            ))
            .await
        {
            eprintln!("warning: cleanup Users for user_id={user_id} failed (non-fatal): {e}");
        }
    }
    for role_id in role_ids {
        if let Err(e) = db
            .execute(Statement::from_sql_and_values(
                DbBackend::MySql,
                "DELETE FROM `UserRoles` WHERE `RoleID` = ?",
                [(*role_id).into()],
            ))
            .await
        {
            eprintln!("warning: cleanup UserRoles for role_id={role_id} failed (non-fatal): {e}");
        }
    }
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_selected_subset_creates_order_for_only_selected_lines() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_selected_subset_creates_order_for_only_selected_lines",
    ) {
        return;
    }

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let (user_id, shipping_id, role_id) = create_checkout_user(&txn, now_tag).await;
    let (selected_category_id, selected_variant_id, selected_cart_id) =
        // Keep selected subtotal above FREE_SHIPPING_THRESHOLD_MINOR to avoid live quote coupling.
        create_variant_with_cart_item(&txn, now_tag, user_id, "A", 60_000, 10, 2).await;
    let (unselected_category_id, unselected_variant_id, unselected_cart_id) =
        create_variant_with_cart_item(&txn, now_tag, user_id, "B", 20_000, 10, 1).await;

    // Commit setup so it's visible to place_order's own (separately-opened) transactions —
    // place_order no longer runs as a nested savepoint inside our transaction; see
    // integration_place_order_atomicity.rs for the full rationale.
    txn.commit().await.expect("commit setup txn");

    let place_res = place_order(
        &db,
        Request::new(PlaceOrderRequest {
            shipping_address_id: shipping_id,
            user_id,
            coupon_code: None,
            selected_cart_ids: vec![selected_cart_id],
            payment_mode: None,
        }),
    )
    .await
    .expect("place_order should succeed");
    let order = place_res.into_inner().items[0].clone();

    assert_eq!(
        order.total_amount_paise, 120_000,
        "only the selected 2 x 60000 line is charged"
    );

    let detail_rows = order_details::Entity::find()
        .filter(order_details::Column::OrderId.eq(order.order_id))
        .all(&db)
        .await
        .expect("query order_details");

    let payment_intent = payment_intents::Entity::find()
        .filter(payment_intents::Column::OrderId.eq(order.order_id))
        .one(&db)
        .await
        .expect("query payment_intents")
        .expect("payment intent exists");

    let remaining_cart = cart::Entity::find()
        .filter(cart::Column::UserId.eq(user_id))
        .all(&db)
        .await
        .expect("query cart");

    cleanup_checkout_rows(
        &db,
        &[order.order_id],
        &[user_id],
        &[role_id],
        &[selected_category_id, unselected_category_id],
        &[selected_variant_id, unselected_variant_id],
        &[shipping_id],
        &[],
    )
    .await;

    assert_eq!(
        detail_rows.len(),
        1,
        "only selected line should become an order detail"
    );
    assert_eq!(detail_rows[0].variant_id, selected_variant_id);
    assert_eq!(detail_rows[0].quantity, 2);

    assert_eq!(payment_intent.amount_paise, 120_000);

    assert_eq!(
        remaining_cart.len(),
        1,
        "unselected line should remain in cart"
    );
    assert_eq!(remaining_cart[0].cart_id, unselected_cart_id);
    assert_eq!(remaining_cart[0].variant_id, unselected_variant_id);
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_selected_subset_coupon_logic_uses_only_selected_lines() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_selected_subset_coupon_logic_uses_only_selected_lines",
    ) {
        return;
    }

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let (user_id, shipping_id, role_id) = create_checkout_user(&txn, now_tag).await;
    let (selected_category_id, _selected_variant_id, selected_cart_id) =
        create_variant_with_cart_item(&txn, now_tag, user_id, "CouponA", 120_000, 10, 1).await;
    let (unselected_category_id, unselected_variant_id, _unselected_cart_id) =
        create_variant_with_cart_item(&txn, now_tag, user_id, "CouponB", 120_000, 10, 1).await;

    let code = format!("SELCP_{}", now_tag);
    let _ = core_operations::handlers::coupons::create_coupon(
        &txn,
        Request::new(CreateCouponRequest {
            code: code.clone(),
            discount_type: "fixed_amount".to_string(),
            discount_value: 50_000,
            min_order_value_paise: Some(200_000),
            usage_limit: Some(10),
            max_uses_per_customer: None,
            starts_at: (Utc::now() - Duration::hours(1)).to_rfc3339(),
            ends_at: Some((Utc::now() + Duration::days(1)).to_rfc3339()),
        }),
    )
    .await
    .expect("create_coupon");

    // Commit setup so it's visible to place_order's own (separately-opened) transactions.
    txn.commit().await.expect("commit setup txn");

    let place_res = place_order(
        &db,
        Request::new(PlaceOrderRequest {
            shipping_address_id: shipping_id,
            user_id,
            coupon_code: Some(code.clone()),
            selected_cart_ids: vec![selected_cart_id],
            payment_mode: None,
        }),
    )
    .await
    .expect("place_order should succeed");
    let order = place_res.into_inner().items[0].clone();

    assert_eq!(
        order.total_amount_paise, 120_000,
        "coupon should not apply because selected subset is below min order"
    );

    let db_order = orders::Entity::find_by_id(order.order_id)
        .one(&db)
        .await
        .expect("query order")
        .expect("order exists");

    let coupon_row = coupons::Entity::find()
        .filter(coupons::Column::Code.eq(&code))
        .one(&db)
        .await
        .expect("query coupon")
        .expect("coupon exists");

    cleanup_checkout_rows(
        &db,
        &[order.order_id],
        &[user_id],
        &[role_id],
        &[selected_category_id, unselected_category_id],
        &[_selected_variant_id, unselected_variant_id],
        &[shipping_id],
        &[code.as_str()],
    )
    .await;

    assert!(
        db_order.applied_coupon_id.is_none(),
        "coupon snapshot should not be stored"
    );
    assert!(db_order.applied_coupon_code.is_none());

    assert_eq!(
        coupon_row.usage_count,
        Some(0),
        "usage should not increment before payment or on non-applicable subset"
    );
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_selected_subset_coupon_allocation_rounding_is_deterministic() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_selected_subset_coupon_allocation_rounding_is_deterministic",
    ) {
        return;
    }

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let (user_id, shipping_id, role_id) = create_checkout_user(&txn, now_tag).await;
    let (category_a, variant_a, cart_a) =
        create_variant_with_cart_item(&txn, now_tag, user_id, "RoundA", 120_003, 10, 1).await;
    let (category_b, variant_b, cart_b) =
        create_variant_with_cart_item(&txn, now_tag, user_id, "RoundB", 80_002, 10, 1).await;
    let (category_c, variant_c, cart_c) =
        create_variant_with_cart_item(&txn, now_tag, user_id, "RoundC", 60_001, 10, 1).await;

    let code = format!("SELROUND_{}", now_tag);
    let _ = core_operations::handlers::coupons::create_coupon(
        &txn,
        Request::new(CreateCouponRequest {
            code: code.clone(),
            discount_type: "fixed_amount".to_string(),
            discount_value: 10_001,
            min_order_value_paise: Some(1),
            usage_limit: Some(20),
            max_uses_per_customer: None,
            starts_at: (Utc::now() - Duration::hours(1)).to_rfc3339(),
            ends_at: Some((Utc::now() + Duration::days(1)).to_rfc3339()),
        }),
    )
    .await
    .expect("create_coupon");

    // Commit setup so it's visible to place_order's own (separately-opened) transactions. This
    // test calls place_order twice (to check that discount-rounding allocation is deterministic
    // for identical inputs replayed later); each call now independently commits real state that
    // the next call's own claim/write phases observe, which is a more realistic test of replay
    // behavior than the old shared-uncommitted-transaction version.
    txn.commit().await.expect("commit setup txn");

    let first = place_order(
        &db,
        Request::new(PlaceOrderRequest {
            shipping_address_id: shipping_id,
            user_id,
            coupon_code: Some(code.clone()),
            selected_cart_ids: vec![cart_a, cart_b, cart_c],
            payment_mode: None,
        }),
    )
    .await
    .expect("first place_order should succeed")
    .into_inner()
    .items[0]
        .clone();

    let first_order = orders::Entity::find_by_id(first.order_id)
        .one(&db)
        .await
        .expect("query first order")
        .expect("first order exists");
    let first_details = order_details::Entity::find()
        .filter(order_details::Column::OrderId.eq(first.order_id))
        .order_by_asc(order_details::Column::VariantId)
        .all(&db)
        .await
        .expect("query first order details");
    assert_eq!(
        first_details.len(),
        3,
        "all selected lines should be present"
    );

    let first_sum_line_totals: i64 = first_details.iter().map(|row| row.line_total_minor).sum();
    let first_shipping = first_order.shipping_charge_minor.unwrap_or(0);
    assert_eq!(
        first_order.items_total_minor_before_discount,
        Some(260_006),
        "before-discount snapshot must match selected lines"
    );
    assert_eq!(first_order.discount_total_minor, Some(10_001));
    assert_eq!(
        first_order.items_total_minor_after_discount,
        Some(first_sum_line_totals),
        "after-discount snapshot must equal persisted line totals"
    );
    assert_eq!(
        first_sum_line_totals + first_shipping,
        first_order.grand_total_minor,
        "sum(line_total_minor) + shipping_charge_minor must equal grand_total_minor"
    );

    // Recreating the cart rows for the replay uses the `create_cart_item` handler, which (unlike
    // place_order) still takes a caller-supplied `&DatabaseTransaction`. Open and commit a small
    // transaction for just these inserts so they're visible (committed) before the second
    // place_order call's own claim phase reads them.
    let replay_txn = db.begin().await.expect("begin cart replay transaction");
    let cart_a_replay = core_operations::handlers::cart::create_cart_item(
        &replay_txn,
        Request::new(CreateCartItemRequest {
            user_id: Some(user_id),
            session_id: None,
            variant_id: variant_a,
            quantity: 1,
        }),
    )
    .await
    .expect("recreate cart A")
    .into_inner()
    .items[0]
        .cart_id;
    let cart_b_replay = core_operations::handlers::cart::create_cart_item(
        &replay_txn,
        Request::new(CreateCartItemRequest {
            user_id: Some(user_id),
            session_id: None,
            variant_id: variant_b,
            quantity: 1,
        }),
    )
    .await
    .expect("recreate cart B")
    .into_inner()
    .items[0]
        .cart_id;
    let cart_c_replay = core_operations::handlers::cart::create_cart_item(
        &replay_txn,
        Request::new(CreateCartItemRequest {
            user_id: Some(user_id),
            session_id: None,
            variant_id: variant_c,
            quantity: 1,
        }),
    )
    .await
    .expect("recreate cart C")
    .into_inner()
    .items[0]
        .cart_id;
    replay_txn
        .commit()
        .await
        .expect("commit cart replay txn");

    let second = place_order(
        &db,
        Request::new(PlaceOrderRequest {
            shipping_address_id: shipping_id,
            user_id,
            coupon_code: Some(code.clone()),
            selected_cart_ids: vec![cart_a_replay, cart_b_replay, cart_c_replay],
            payment_mode: None,
        }),
    )
    .await
    .expect("second place_order should succeed")
    .into_inner()
    .items[0]
        .clone();

    let second_details = order_details::Entity::find()
        .filter(order_details::Column::OrderId.eq(second.order_id))
        .order_by_asc(order_details::Column::VariantId)
        .all(&db)
        .await
        .expect("query second order details");
    assert_eq!(second_details.len(), 3);

    let first_allocation: Vec<(i64, i64)> = first_details
        .iter()
        .map(|row| (row.variant_id, row.line_total_minor))
        .collect();
    let second_allocation: Vec<(i64, i64)> = second_details
        .iter()
        .map(|row| (row.variant_id, row.line_total_minor))
        .collect();

    cleanup_checkout_rows(
        &db,
        &[first.order_id, second.order_id],
        &[user_id],
        &[role_id],
        &[category_a, category_b, category_c],
        &[variant_a, variant_b, variant_c],
        &[shipping_id],
        &[code.as_str()],
    )
    .await;

    assert_eq!(
        first_allocation, second_allocation,
        "discount allocation with rounding remainder must be deterministic for identical inputs"
    );
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_selected_checkout_rejects_empty_or_invalid_selection() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let (user_id, shipping_id, role_id) = create_checkout_user(&txn, now_tag).await;
    let (category_id, variant_id, cart_id) =
        create_variant_with_cart_item(&txn, now_tag, user_id, "Invalid", 900, 10, 1).await;

    // Commit setup so it's visible to place_order's own (separately-opened) transactions. Both
    // calls below are expected to fail validation before any order is created, so there is
    // nothing for place_order itself to persist here regardless.
    txn.commit().await.expect("commit setup txn");

    let empty = place_order(
        &db,
        Request::new(PlaceOrderRequest {
            shipping_address_id: shipping_id,
            user_id,
            coupon_code: None,
            selected_cart_ids: vec![],
            payment_mode: None,
        }),
    )
    .await
    .expect_err("empty selection should fail");
    assert_eq!(empty.code(), Code::FailedPrecondition);
    assert!(empty.message().to_lowercase().contains("selected"));

    let invalid = place_order(
        &db,
        Request::new(PlaceOrderRequest {
            shipping_address_id: shipping_id,
            user_id,
            coupon_code: None,
            selected_cart_ids: vec![cart_id, cart_id + 999_999],
            payment_mode: None,
        }),
    )
    .await
    .expect_err("invalid selection should fail");
    assert_eq!(invalid.code(), Code::InvalidArgument);
    assert!(invalid
        .message()
        .to_lowercase()
        .contains("selected cart item"));

    let remaining_cart = cart::Entity::find()
        .filter(cart::Column::UserId.eq(user_id))
        .all(&db)
        .await
        .expect("query cart");

    // No order is ever created in this test (both calls fail validation), so cleanup only needs
    // to remove the setup fixtures.
    cleanup_checkout_rows(
        &db,
        &[],
        &[user_id],
        &[role_id],
        &[category_id],
        &[variant_id],
        &[shipping_id],
        &[],
    )
    .await;

    assert_eq!(
        remaining_cart.len(),
        1,
        "cart should remain untouched on selection validation failure"
    );
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_selected_checkout_rejects_out_of_stock_selected_line() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_selected_checkout_rejects_out_of_stock_selected_line",
    ) {
        return;
    }

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let (user_id, shipping_id, role_id) = create_checkout_user(&txn, now_tag).await;
    let (low_stock_category_id, low_stock_variant_id, selected_cart_id) =
        create_variant_with_cart_item(&txn, now_tag, user_id, "LowStock", 120_000, 1, 2).await;
    let (safe_category_id, safe_variant_id, _safe_cart_id) =
        create_variant_with_cart_item(&txn, now_tag, user_id, "Safe", 90_000, 10, 1).await;

    // Commit setup so it's visible to place_order's own (separately-opened) transactions.
    txn.commit().await.expect("commit setup txn");

    let result = place_order(
        &db,
        Request::new(PlaceOrderRequest {
            shipping_address_id: shipping_id,
            user_id,
            coupon_code: None,
            selected_cart_ids: vec![selected_cart_id],
            payment_mode: None,
        }),
    )
    .await
    .expect_err("out-of-stock selected line should fail");

    assert_eq!(result.code(), Code::FailedPrecondition);
    assert!(result.message().contains("Insufficient stock for variant"));

    let order_count = orders::Entity::find()
        .filter(orders::Column::UserId.eq(user_id))
        .all(&db)
        .await
        .expect("query orders")
        .len();

    let remaining_cart = cart::Entity::find()
        .filter(cart::Column::UserId.eq(user_id))
        .all(&db)
        .await
        .expect("query cart");

    // No order is created (place_order failed on the stock check), so cleanup only needs to
    // remove the setup fixtures.
    cleanup_checkout_rows(
        &db,
        &[],
        &[user_id],
        &[role_id],
        &[low_stock_category_id, safe_category_id],
        &[low_stock_variant_id, safe_variant_id],
        &[shipping_id],
        &[],
    )
    .await;

    assert_eq!(order_count, 0, "no order should be created");

    assert_eq!(
        remaining_cart.len(),
        2,
        "all cart rows should remain for retry"
    );
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_selected_checkout_rejects_cross_user_cancellation_attempts() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_selected_checkout_rejects_cross_user_cancellation_attempts",
    ) {
        return;
    }

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let (owner_user_id, owner_shipping_id, owner_role_id) =
        create_checkout_user(&txn, now_tag).await;
    let (owner_category_id, owner_variant_id, owner_cart_id) =
        create_variant_with_cart_item(&txn, now_tag, owner_user_id, "OwnerItem", 150_000, 10, 1)
            .await;
    // Created here (still inside the setup transaction) rather than after place_order, so it's
    // committed together with the owner fixtures and visible to the cancel_order_items/
    // delete_order calls below, which — unlike place_order — still run as nested handlers
    // against a caller-supplied `&DatabaseTransaction`.
    let (other_user_id, other_shipping_id, other_role_id) =
        create_checkout_user(&txn, now_tag + 1).await;

    // Commit setup so it's visible to place_order's own (separately-opened) transactions.
    txn.commit().await.expect("commit setup txn");

    let place_res = place_order(
        &db,
        Request::new(PlaceOrderRequest {
            shipping_address_id: owner_shipping_id,
            user_id: owner_user_id,
            coupon_code: None,
            selected_cart_ids: vec![owner_cart_id],
            payment_mode: None,
        }),
    )
    .await
    .expect("owner place_order should succeed");
    let order_id = place_res.into_inner().items[0].order_id;

    let detail_id = order_details::Entity::find()
        .filter(order_details::Column::OrderId.eq(order_id))
        .one(&db)
        .await
        .expect("query order detail")
        .expect("order detail exists")
        .order_detail_id;

    // cancel_order_items/delete_order are unrelated handlers whose own signatures didn't change
    // (only place_order's did) — they still take a caller-supplied `&DatabaseTransaction`, so we
    // open a fresh transaction here to drive them against the now-committed order/users.
    let post_txn = db
        .begin()
        .await
        .expect("begin post-place_order transaction");

    let partial_err = cancel_order_items(
        &post_txn,
        Request::new(CancelOrderItemsRequest {
            order_id,
            acting_user_id: Some(other_user_id),
            order_detail_ids: vec![detail_id],
        }),
    )
    .await
    .expect_err("cross-user partial cancel should fail");
    assert!(
        partial_err.code() == tonic::Code::PermissionDenied
            || partial_err.code() == tonic::Code::NotFound,
        "expected permission/not-found error, got {:?}",
        partial_err.code()
    );

    let full_err = delete_order(
        &post_txn,
        Request::new(DeleteOrderRequest {
            order_id,
            acting_user_id: Some(other_user_id),
        }),
    )
    .await
    .expect_err("cross-user full cancel should fail");
    assert!(
        full_err.code() == tonic::Code::PermissionDenied
            || full_err.code() == tonic::Code::NotFound,
        "expected permission/not-found error, got {:?}",
        full_err.code()
    );

    // Both calls above were expected (and asserted) to fail, so nothing was mutated in
    // post_txn — roll it back rather than committing no-op reads/failed writes.
    post_txn.rollback().await.ok();

    let order_row = orders::Entity::find_by_id(order_id)
        .one(&db)
        .await
        .expect("query order")
        .expect("order exists");
    let active_sale_id = ensure_order_status(&db, "active_sale").await;

    cleanup_checkout_rows(
        &db,
        &[order_id],
        &[owner_user_id, other_user_id],
        &[owner_role_id, other_role_id],
        &[owner_category_id],
        &[owner_variant_id],
        &[owner_shipping_id, other_shipping_id],
        &[],
    )
    .await;

    assert_eq!(
        order_row.status_id, active_sale_id,
        "unauthorized cancellation attempts must not mutate order state"
    );
}
