//! Integration tests for users, carts, orders, and products flows.
//!
//! **Setup**
//! - Set `TEST_DATABASE_URL` or `DATABASE_URL` (e.g. `mysql://root:test_password@127.0.0.1:3306/sudattas_test`).
//! - Schema must be loaded first (e.g. migrations or `backend/database/sql_dump/01_schema.sql`).
//!
//! **Run**
//! - `cargo test --test integration_users_carts_orders_products -- --ignored`

mod integration_common;
mod provider_test_gate;

use chrono::Utc;
use integration_common::test_db_url;

use core_db_entities::entity::{
    cart, inventory, order_details, order_status, product_categories, product_variants, products,
    shipping_addresses, user_roles,
};
use core_operations::procedures::orders::place_order;
use proto::proto::core::{
    CreateCartItemRequest, CreateUserRequest, GetCartItemsRequest, PlaceOrderRequest,
    SearchOrderRequest, UpdateCartItemRequest,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, Database, DatabaseConnection,
    DbBackend, EntityTrait, QueryFilter, Statement, TransactionTrait,
};
use tonic::{Code, Request};

#[allow(dead_code)]
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
async fn integration_place_order_happy_path_removes_only_selected_items_after_creation() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_place_order_happy_path_removes_only_selected_items_after_creation",
    ) {
        return;
    }

    use core_db_entities::entity::{inventory as inventory_entity, orders};

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    // Ensure OrderStatus 'pending' exists so place_order can resolve it.
    let pending = order_status::Entity::find()
        .filter(order_status::Column::StatusName.eq("pending"))
        .one(&txn)
        .await
        .expect("query OrderStatus");
    if pending.is_none() {
        let status = order_status::ActiveModel {
            status_id: ActiveValue::NotSet,
            status_name: ActiveValue::Set("pending".to_string()),
        };
        let _ = status
            .insert(&txn)
            .await
            .expect("insert pending OrderStatus");
    }

    // Create a test role to satisfy FK for users.role_id.
    let now_tag = Utc::now().timestamp_millis();
    let role_name = format!("itest_role_{}", now_tag);
    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(role_name),
    }
    .insert(&txn)
    .await
    .expect("insert UserRoles");

    // Create user via handler (covers auth + basic user creation).
    let user_res = core_operations::handlers::users::create_user(
        &txn,
        Request::new(CreateUserRequest {
            username: format!("itest_user_{}", now_tag),
            email: format!("itest+{}@example.com", now_tag),
            full_name: Some("Integration User".to_string()),
            address: Some("123 Test Street".to_string()),
            phone: Some("1234567890".to_string()),
            auth_provider: "email".to_string(),
            password_plain: Some("StrongPass123!".to_string()),
            google_sub: None,
            role_id: Some(role.role_id),
        }),
    )
    .await
    .expect("create_user should succeed");
    let user = user_res
        .into_inner()
        .items
        .into_iter()
        .next()
        .expect("user in response");
    let user_id = user.user_id;

    // Shipping address for the user.
    let shipping = shipping_addresses::ActiveModel {
        shipping_address_id: ActiveValue::NotSet,
        user_id: ActiveValue::Set(Some(user_id)),
        recipient_name: ActiveValue::Set(Some("Test User".to_string())),
        phone_number: ActiveValue::Set(Some("+919876543210".to_string())),
        is_default: ActiveValue::Set(0),
        country: ActiveValue::Set("IN".to_string()),
        state_region: ActiveValue::Set("KA".to_string()),
        city: ActiveValue::Set("Bengaluru".to_string()),
        postal_code: ActiveValue::Set("560001".to_string()),
        road: ActiveValue::Set(Some("Integration Road".to_string())),
        apartment_no_or_name: ActiveValue::Set(Some("1A".to_string())),
    }
    .insert(&txn)
    .await
    .expect("insert ShippingAddresses");

    // Minimal product category and product.
    let category = product_categories::ActiveModel {
        category_id: ActiveValue::NotSet,
        name: ActiveValue::Set(format!("itest_category_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductCategories");

    let product = products::ActiveModel {
        product_id: ActiveValue::NotSet,
        sku: ActiveValue::Set(None),
        name: ActiveValue::Set("Integration Product".to_string()),
        slug: ActiveValue::Set(None),
        description: ActiveValue::Set(None),
        // Keep subtotal above FREE_SHIPPING_THRESHOLD_MINOR to avoid live shipping quote dependency.
        price_paise: ActiveValue::Set(60_000),
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
    .insert(&txn)
    .await
    .expect("insert Products");

    // Variant and inventory for the product.
    let variant = product_variants::ActiveModel {
        variant_id: ActiveValue::NotSet,
        product_id: ActiveValue::Set(product.product_id),
        size_id: ActiveValue::Set(None),
        color_id: ActiveValue::Set(None),
        additional_price: ActiveValue::Set(Some(0)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductVariants");

    let _inventory = inventory::ActiveModel {
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

    // Add item to cart via handler.
    let cart_res = core_operations::handlers::cart::create_cart_item(
        &txn,
        Request::new(CreateCartItemRequest {
            user_id: Some(user_id),
            session_id: None,
            variant_id: variant.variant_id,
            quantity: 2,
        }),
    )
    .await
    .expect("create_cart_item should succeed");
    let selected_cart_id = cart_res.into_inner().items[0].cart_id;

    // Commit setup so it's visible to place_order's own (separately-opened) transactions —
    // place_order no longer runs as a nested savepoint inside our transaction; see
    // integration_place_order_atomicity.rs for the full rationale.
    txn.commit().await.expect("commit setup txn");

    // Place order – integrates users, cart, products, orders, inventory, and order_events/outbox.
    let result = place_order(
        &db,
        Request::new(PlaceOrderRequest {
            shipping_address_id: shipping.shipping_address_id,
            user_id,
            coupon_code: None,
            selected_cart_ids: vec![selected_cart_id],
            payment_mode: None,
        }),
    )
    .await;

    let resp = result.expect("place_order should succeed for happy path");
    let body = resp.into_inner();
    assert_eq!(body.items.len(), 1, "exactly one order returned");
    let order = &body.items[0];
    assert_eq!(order.user_id, user_id, "order belongs to user");
    assert_eq!(
        order.shipping_address_id, shipping.shipping_address_id,
        "order uses expected shipping address"
    );
    assert_eq!(
        order.total_amount_paise, 120_000,
        "2 items * 60000 paise each"
    );
    let order_id = order.order_id;

    // Order persisted with matching totals.
    let db_order = orders::Entity::find()
        .filter(orders::Column::OrderId.eq(order_id))
        .one(&db)
        .await
        .expect("query Orders")
        .expect("order row should exist");

    // OrderDetails created for the cart item.
    let details = order_details::Entity::find()
        .filter(order_details::Column::OrderId.eq(order_id))
        .all(&db)
        .await
        .expect("query OrderDetails");

    // Inventory decremented atomically.
    let inv = inventory_entity::Entity::find()
        .filter(inventory_entity::Column::VariantId.eq(Some(variant.variant_id)))
        .one(&db)
        .await
        .expect("query Inventory")
        .expect("inventory row should exist");

    // Purchased selected rows are removed immediately after successful order creation.
    let remaining_cart = cart::Entity::find()
        .filter(cart::Column::UserId.eq(user_id))
        .all(&db)
        .await
        .expect("query Cart for user");

    cleanup_checkout_rows(
        &db,
        &[order_id],
        &[user_id],
        &[role.role_id],
        &[category.category_id],
        &[variant.variant_id],
        &[shipping.shipping_address_id],
    )
    .await;

    assert_eq!(db_order.user_id, user_id);
    assert_eq!(
        db_order.grand_total_minor, 120_000,
        "grand_total_minor should match computed total"
    );

    assert_eq!(details.len(), 1, "one order detail row expected");
    assert_eq!(details[0].variant_id, variant.variant_id);
    assert_eq!(details[0].quantity, 2);

    assert_eq!(
        inv.quantity_available,
        Some(8),
        "quantity_available should be decremented by ordered quantity"
    );

    assert!(
        remaining_cart.is_empty(),
        "selected cart rows should be removed after successful order creation"
    );
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_place_order_insufficient_inventory_fails_and_preserves_cart() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_place_order_insufficient_inventory_fails_and_preserves_cart",
    ) {
        return;
    }

    use core_db_entities::entity::orders;

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    // Ensure OrderStatus 'pending' exists so place_order can resolve it.
    let pending = order_status::Entity::find()
        .filter(order_status::Column::StatusName.eq("pending"))
        .one(&txn)
        .await
        .expect("query OrderStatus");
    if pending.is_none() {
        let status = order_status::ActiveModel {
            status_id: ActiveValue::NotSet,
            status_name: ActiveValue::Set("pending".to_string()),
        };
        let _ = status
            .insert(&txn)
            .await
            .expect("insert pending OrderStatus");
    }

    // Create role and user as before.
    let role_name = format!("itest_role_{}", Utc::now().timestamp_millis());
    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(role_name),
    }
    .insert(&txn)
    .await
    .expect("insert UserRoles");

    let now_tag = Utc::now().timestamp_millis();
    let user_res = core_operations::handlers::users::create_user(
        &txn,
        Request::new(CreateUserRequest {
            username: format!("itest_user_insufficient_{}", now_tag),
            email: format!("itest_insufficient+{}@example.com", now_tag),
            full_name: Some("Integration User Insufficient".to_string()),
            address: Some("456 Test Street".to_string()),
            phone: Some("0987654321".to_string()),
            auth_provider: "email".to_string(),
            password_plain: Some("StrongPass123!".to_string()),
            google_sub: None,
            role_id: Some(role.role_id),
        }),
    )
    .await
    .expect("create_user should succeed");
    let user = user_res
        .into_inner()
        .items
        .into_iter()
        .next()
        .expect("user in response");
    let user_id = user.user_id;

    let shipping = shipping_addresses::ActiveModel {
        shipping_address_id: ActiveValue::NotSet,
        user_id: ActiveValue::Set(Some(user_id)),
        recipient_name: ActiveValue::Set(Some("Test User".to_string())),
        phone_number: ActiveValue::Set(Some("+919876543210".to_string())),
        is_default: ActiveValue::Set(0),
        country: ActiveValue::Set("IN".to_string()),
        state_region: ActiveValue::Set("KA".to_string()),
        city: ActiveValue::Set("Bengaluru".to_string()),
        postal_code: ActiveValue::Set("560001".to_string()),
        road: ActiveValue::Set(Some("Integration Road".to_string())),
        apartment_no_or_name: ActiveValue::Set(Some("1B".to_string())),
    }
    .insert(&txn)
    .await
    .expect("insert ShippingAddresses");

    let category = product_categories::ActiveModel {
        category_id: ActiveValue::NotSet,
        name: ActiveValue::Set(format!("itest_category_insufficient_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductCategories");

    let product = products::ActiveModel {
        product_id: ActiveValue::NotSet,
        sku: ActiveValue::Set(None),
        name: ActiveValue::Set("Integration Product Insufficient".to_string()),
        slug: ActiveValue::Set(None),
        description: ActiveValue::Set(None),
        price_paise: ActiveValue::Set(50_000),
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
    .insert(&txn)
    .await
    .expect("insert Products");

    let variant = product_variants::ActiveModel {
        variant_id: ActiveValue::NotSet,
        product_id: ActiveValue::Set(product.product_id),
        size_id: ActiveValue::Set(None),
        color_id: ActiveValue::Set(None),
        additional_price: ActiveValue::Set(Some(0)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductVariants");

    // Inventory has less stock than requested quantity.
    let _inventory = inventory::ActiveModel {
        inventory_id: ActiveValue::NotSet,
        variant_id: ActiveValue::Set(Some(variant.variant_id)),
        quantity_available: ActiveValue::Set(Some(1)),
        quantity_reserved: ActiveValue::Set(Some(0)),
        reorder_level: ActiveValue::Set(None),
        updated_at: ActiveValue::Set(Some(Utc::now())),
    }
    .insert(&txn)
    .await
    .expect("insert Inventory");

    // Add cart item with quantity > available stock.
    let cart_res = core_operations::handlers::cart::create_cart_item(
        &txn,
        Request::new(CreateCartItemRequest {
            user_id: Some(user_id),
            session_id: None,
            variant_id: variant.variant_id,
            quantity: 2,
        }),
    )
    .await
    .expect("create_cart_item should succeed");
    let selected_cart_id = cart_res.into_inner().items[0].cart_id;

    // Commit setup so it's visible to place_order's own (separately-opened) transactions.
    txn.commit().await.expect("commit setup txn");

    let result = place_order(
        &db,
        Request::new(PlaceOrderRequest {
            shipping_address_id: shipping.shipping_address_id,
            user_id,
            coupon_code: None,
            selected_cart_ids: vec![selected_cart_id],
            payment_mode: None,
        }),
    )
    .await;

    let err = result.expect_err("place_order should fail due to insufficient stock");
    assert_eq!(err.code(), Code::FailedPrecondition);
    assert!(
        err.message().contains("Insufficient stock for variant"),
        "error message should indicate insufficient stock, got: {}",
        err.message()
    );

    // No order should be created for the user.
    let orders_for_user = orders::Entity::find()
        .filter(orders::Column::UserId.eq(user_id))
        .all(&db)
        .await
        .expect("query Orders for user");

    // Cart should remain intact so the user can retry later.
    let cart_rows = cart::Entity::find()
        .filter(cart::Column::UserId.eq(user_id))
        .all(&db)
        .await
        .expect("query Cart for user");

    // No order is created (place_order failed on the stock check), so cleanup only needs to
    // remove the setup fixtures.
    cleanup_checkout_rows(
        &db,
        &[],
        &[user_id],
        &[role.role_id],
        &[category.category_id],
        &[variant.variant_id],
        &[shipping.shipping_address_id],
    )
    .await;

    assert!(
        orders_for_user.is_empty(),
        "no orders should be created on insufficient stock"
    );

    assert_eq!(
        cart_rows.len(),
        1,
        "cart row should remain when place_order fails"
    );
    assert_eq!(cart_rows[0].variant_id, variant.variant_id);
    assert_eq!(cart_rows[0].quantity, 2);
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_place_order_empty_cart_fails() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let pending = order_status::Entity::find()
        .filter(order_status::Column::StatusName.eq("pending"))
        .one(&txn)
        .await
        .expect("query OrderStatus");
    if pending.is_none() {
        let status = order_status::ActiveModel {
            status_id: ActiveValue::NotSet,
            status_name: ActiveValue::Set("pending".to_string()),
        };
        let _ = status
            .insert(&txn)
            .await
            .expect("insert pending OrderStatus");
    }

    let role_name = format!("itest_role_empty_{}", Utc::now().timestamp_millis());
    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(role_name),
    }
    .insert(&txn)
    .await
    .expect("insert UserRoles");

    let now_tag = Utc::now().timestamp_millis();
    let user_res = core_operations::handlers::users::create_user(
        &txn,
        Request::new(CreateUserRequest {
            username: format!("itest_user_empty_{}", now_tag),
            email: format!("itest_empty+{}@example.com", now_tag),
            full_name: Some("Empty Cart User".to_string()),
            address: None,
            phone: None,
            auth_provider: "email".to_string(),
            password_plain: Some("StrongPass123!".to_string()),
            google_sub: None,
            role_id: Some(role.role_id),
        }),
    )
    .await
    .expect("create_user should succeed");
    let user_id = user_res.into_inner().items[0].user_id;

    let shipping = shipping_addresses::ActiveModel {
        shipping_address_id: ActiveValue::NotSet,
        user_id: ActiveValue::Set(Some(user_id)),
        recipient_name: ActiveValue::Set(Some("Test User".to_string())),
        phone_number: ActiveValue::Set(Some("+919876543210".to_string())),
        is_default: ActiveValue::Set(0),
        country: ActiveValue::Set("IN".to_string()),
        state_region: ActiveValue::Set("KA".to_string()),
        city: ActiveValue::Set("Bengaluru".to_string()),
        postal_code: ActiveValue::Set("560001".to_string()),
        road: ActiveValue::Set(None),
        apartment_no_or_name: ActiveValue::Set(None),
    }
    .insert(&txn)
    .await
    .expect("insert ShippingAddresses");

    // Commit setup so it's visible to place_order's own (separately-opened) transactions. No
    // cart items are ever created here, so place_order is expected to fail regardless.
    txn.commit().await.expect("commit setup txn");

    // No cart items; place_order should fail.
    let result = place_order(
        &db,
        Request::new(PlaceOrderRequest {
            shipping_address_id: shipping.shipping_address_id,
            user_id,
            coupon_code: None,
            selected_cart_ids: vec![],
            payment_mode: None,
        }),
    )
    .await;

    let err = result.expect_err("place_order should fail when nothing is selected");

    // No order/category/variant fixtures were created in this test; cleanup only needs the
    // user/shipping/role rows.
    cleanup_checkout_rows(
        &db,
        &[],
        &[user_id],
        &[role.role_id],
        &[],
        &[],
        &[shipping.shipping_address_id],
    )
    .await;

    assert_eq!(err.code(), Code::FailedPrecondition);
    assert!(
        err.message().to_lowercase().contains("selected"),
        "error should mention missing selected cart items, got: {}",
        err.message()
    );
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_cart_add_get_update_then_place_order() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_cart_add_get_update_then_place_order",
    ) {
        return;
    }

    use core_db_entities::entity::{inventory as inventory_entity, orders};

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let pending = order_status::Entity::find()
        .filter(order_status::Column::StatusName.eq("pending"))
        .one(&txn)
        .await
        .expect("query OrderStatus");
    if pending.is_none() {
        let status = order_status::ActiveModel {
            status_id: ActiveValue::NotSet,
            status_name: ActiveValue::Set("pending".to_string()),
        };
        let _ = status
            .insert(&txn)
            .await
            .expect("insert pending OrderStatus");
    }

    let role_name = format!("itest_role_cartup_{}", Utc::now().timestamp_millis());
    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(role_name),
    }
    .insert(&txn)
    .await
    .expect("insert UserRoles");

    let now_tag = Utc::now().timestamp_millis();
    let user_res = core_operations::handlers::users::create_user(
        &txn,
        Request::new(CreateUserRequest {
            username: format!("itest_cartup_{}", now_tag),
            email: format!("itest_cartup+{}@example.com", now_tag),
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
    .expect("create_user should succeed");
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
    .insert(&txn)
    .await
    .expect("insert ShippingAddresses");

    let category = product_categories::ActiveModel {
        category_id: ActiveValue::NotSet,
        name: ActiveValue::Set(format!("itest_cat_cartup_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductCategories");

    let product = products::ActiveModel {
        product_id: ActiveValue::NotSet,
        sku: ActiveValue::Set(None),
        name: ActiveValue::Set("Cart Update Product".to_string()),
        slug: ActiveValue::Set(None),
        description: ActiveValue::Set(None),
        price_paise: ActiveValue::Set(50_000),
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
    .insert(&txn)
    .await
    .expect("insert Products");

    let variant = product_variants::ActiveModel {
        variant_id: ActiveValue::NotSet,
        product_id: ActiveValue::Set(product.product_id),
        size_id: ActiveValue::Set(None),
        color_id: ActiveValue::Set(None),
        additional_price: ActiveValue::Set(Some(0)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductVariants");

    let _inv = inventory::ActiveModel {
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
    .expect("create_cart_item should succeed");
    let cart_id = cart_res.into_inner().items[0].cart_id;

    let get_res = core_operations::handlers::cart::get_cart_items(
        &txn,
        Request::new(GetCartItemsRequest {
            user_id: Some(user_id),
            session_id: None,
        }),
    )
    .await
    .expect("get_cart_items should succeed");
    assert_eq!(get_res.get_ref().items.len(), 1);
    assert_eq!(get_res.get_ref().items[0].quantity, 1);

    let _ = core_operations::handlers::cart::update_cart_item(
        &txn,
        Request::new(UpdateCartItemRequest {
            cart_id,
            user_id: Some(user_id),
            variant_id: variant.variant_id,
            quantity: 3,
            session_id: None,
        }),
    )
    .await
    .expect("update_cart_item should succeed");

    let get_after = core_operations::handlers::cart::get_cart_items(
        &txn,
        Request::new(GetCartItemsRequest {
            user_id: Some(user_id),
            session_id: None,
        }),
    )
    .await
    .expect("get_cart_items should succeed");
    assert_eq!(get_after.get_ref().items.len(), 1);
    assert_eq!(get_after.get_ref().items[0].quantity, 3);

    // Commit setup (including the cart add/get/update dance above, which all still use handlers
    // pinned to `&DatabaseTransaction`) so it's visible to place_order's own (separately-opened)
    // transactions.
    txn.commit().await.expect("commit setup txn");

    let place_res = place_order(
        &db,
        Request::new(PlaceOrderRequest {
            shipping_address_id: shipping.shipping_address_id,
            user_id,
            coupon_code: None,
            selected_cart_ids: vec![cart_id],
            payment_mode: None,
        }),
    )
    .await
    .expect("place_order should succeed");
    let order = &place_res.into_inner().items[0];
    let order_id = order.order_id;
    assert_eq!(order.total_amount_paise, 150_000, "3 * 50000 paise");

    let db_order = orders::Entity::find_by_id(order_id)
        .one(&db)
        .await
        .expect("query order")
        .expect("order exists");

    let inv_after = inventory_entity::Entity::find()
        .filter(inventory_entity::Column::VariantId.eq(Some(variant.variant_id)))
        .one(&db)
        .await
        .expect("query inventory")
        .expect("inventory exists");

    cleanup_checkout_rows(
        &db,
        &[order_id],
        &[user_id],
        &[role.role_id],
        &[category.category_id],
        &[variant.variant_id],
        &[shipping.shipping_address_id],
    )
    .await;

    assert_eq!(db_order.grand_total_minor, 150_000);
    assert_eq!(inv_after.quantity_available, Some(7));
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_place_order_multiple_items_two_variants() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_place_order_multiple_items_two_variants",
    ) {
        return;
    }

    use core_db_entities::entity::{inventory as inventory_entity, orders};

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let pending = order_status::Entity::find()
        .filter(order_status::Column::StatusName.eq("pending"))
        .one(&txn)
        .await
        .expect("query OrderStatus");
    if pending.is_none() {
        let status = order_status::ActiveModel {
            status_id: ActiveValue::NotSet,
            status_name: ActiveValue::Set("pending".to_string()),
        };
        let _ = status
            .insert(&txn)
            .await
            .expect("insert pending OrderStatus");
    }

    let role_name = format!("itest_role_multi_{}", Utc::now().timestamp_millis());
    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(role_name),
    }
    .insert(&txn)
    .await
    .expect("insert UserRoles");

    let now_tag = Utc::now().timestamp_millis();
    let user_res = core_operations::handlers::users::create_user(
        &txn,
        Request::new(CreateUserRequest {
            username: format!("itest_multi_{}", now_tag),
            email: format!("itest_multi+{}@example.com", now_tag),
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
    .expect("create_user should succeed");
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
    .insert(&txn)
    .await
    .expect("insert ShippingAddresses");

    let category = product_categories::ActiveModel {
        category_id: ActiveValue::NotSet,
        name: ActiveValue::Set(format!("itest_cat_multi_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductCategories");

    let product_a = products::ActiveModel {
        product_id: ActiveValue::NotSet,
        sku: ActiveValue::Set(None),
        name: ActiveValue::Set("Product A".to_string()),
        slug: ActiveValue::Set(None),
        description: ActiveValue::Set(None),
        price_paise: ActiveValue::Set(60_000),
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
    .insert(&txn)
    .await
    .expect("insert Products");
    let product_b = products::ActiveModel {
        product_id: ActiveValue::NotSet,
        sku: ActiveValue::Set(None),
        name: ActiveValue::Set("Product B".to_string()),
        slug: ActiveValue::Set(None),
        description: ActiveValue::Set(None),
        price_paise: ActiveValue::Set(40_000),
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
    .insert(&txn)
    .await
    .expect("insert Products");

    let variant_a = product_variants::ActiveModel {
        variant_id: ActiveValue::NotSet,
        product_id: ActiveValue::Set(product_a.product_id),
        size_id: ActiveValue::Set(None),
        color_id: ActiveValue::Set(None),
        additional_price: ActiveValue::Set(Some(0)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductVariants");
    let variant_b = product_variants::ActiveModel {
        variant_id: ActiveValue::NotSet,
        product_id: ActiveValue::Set(product_b.product_id),
        size_id: ActiveValue::Set(None),
        color_id: ActiveValue::Set(None),
        additional_price: ActiveValue::Set(Some(0)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductVariants");

    let _inv_a = inventory::ActiveModel {
        inventory_id: ActiveValue::NotSet,
        variant_id: ActiveValue::Set(Some(variant_a.variant_id)),
        quantity_available: ActiveValue::Set(Some(5)),
        quantity_reserved: ActiveValue::Set(Some(0)),
        reorder_level: ActiveValue::Set(None),
        updated_at: ActiveValue::Set(Some(Utc::now())),
    }
    .insert(&txn)
    .await
    .expect("insert Inventory");
    let _inv_b = inventory::ActiveModel {
        inventory_id: ActiveValue::NotSet,
        variant_id: ActiveValue::Set(Some(variant_b.variant_id)),
        quantity_available: ActiveValue::Set(Some(5)),
        quantity_reserved: ActiveValue::Set(Some(0)),
        reorder_level: ActiveValue::Set(None),
        updated_at: ActiveValue::Set(Some(Utc::now())),
    }
    .insert(&txn)
    .await
    .expect("insert Inventory");

    let cart_a = core_operations::handlers::cart::create_cart_item(
        &txn,
        Request::new(CreateCartItemRequest {
            user_id: Some(user_id),
            session_id: None,
            variant_id: variant_a.variant_id,
            quantity: 2,
        }),
    )
    .await
    .expect("create_cart_item A");
    let cart_b = core_operations::handlers::cart::create_cart_item(
        &txn,
        Request::new(CreateCartItemRequest {
            user_id: Some(user_id),
            session_id: None,
            variant_id: variant_b.variant_id,
            quantity: 1,
        }),
    )
    .await
    .expect("create_cart_item B");
    let selected_cart_ids = vec![
        cart_a.into_inner().items[0].cart_id,
        cart_b.into_inner().items[0].cart_id,
    ];

    // Commit setup so it's visible to place_order's own (separately-opened) transactions.
    txn.commit().await.expect("commit setup txn");

    let place_res = place_order(
        &db,
        Request::new(PlaceOrderRequest {
            shipping_address_id: shipping.shipping_address_id,
            user_id,
            coupon_code: None,
            selected_cart_ids,
            payment_mode: None,
        }),
    )
    .await
    .expect("place_order should succeed");
    let order = &place_res.into_inner().items[0];
    let order_id = order.order_id;
    let expected_total = 2 * 60_000 + 40_000;
    assert_eq!(
        order.total_amount_paise, expected_total,
        "2*A + 1*B = 160000"
    );

    let db_order = orders::Entity::find_by_id(order_id)
        .one(&db)
        .await
        .expect("query order")
        .expect("order exists");

    let details = order_details::Entity::find()
        .filter(order_details::Column::OrderId.eq(order_id))
        .all(&db)
        .await
        .expect("query OrderDetails");

    let inv_a = inventory_entity::Entity::find()
        .filter(inventory_entity::Column::VariantId.eq(Some(variant_a.variant_id)))
        .one(&db)
        .await
        .expect("query inventory A")
        .expect("exists");
    let inv_b = inventory_entity::Entity::find()
        .filter(inventory_entity::Column::VariantId.eq(Some(variant_b.variant_id)))
        .one(&db)
        .await
        .expect("query inventory B")
        .expect("exists");

    cleanup_checkout_rows(
        &db,
        &[order_id],
        &[user_id],
        &[role.role_id],
        &[category.category_id],
        &[variant_a.variant_id, variant_b.variant_id],
        &[shipping.shipping_address_id],
    )
    .await;

    assert_eq!(db_order.grand_total_minor, expected_total);
    assert_eq!(details.len(), 2, "two line items");
    assert_eq!(inv_a.quantity_available, Some(3));
    assert_eq!(inv_b.quantity_available, Some(4));
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_place_order_then_search_order() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_place_order_then_search_order",
    ) {
        return;
    }

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let pending = order_status::Entity::find()
        .filter(order_status::Column::StatusName.eq("pending"))
        .one(&txn)
        .await
        .expect("query OrderStatus");
    if pending.is_none() {
        let status = order_status::ActiveModel {
            status_id: ActiveValue::NotSet,
            status_name: ActiveValue::Set("pending".to_string()),
        };
        let _ = status
            .insert(&txn)
            .await
            .expect("insert pending OrderStatus");
    }

    let role_name = format!("itest_role_srch_{}", Utc::now().timestamp_millis());
    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(role_name),
    }
    .insert(&txn)
    .await
    .expect("insert UserRoles");

    let now_tag = Utc::now().timestamp_millis();
    let user_res = core_operations::handlers::users::create_user(
        &txn,
        Request::new(CreateUserRequest {
            username: format!("itest_srch_{}", now_tag),
            email: format!("itest_srch+{}@example.com", now_tag),
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
    .expect("create_user should succeed");
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
    .insert(&txn)
    .await
    .expect("insert ShippingAddresses");

    let category = product_categories::ActiveModel {
        category_id: ActiveValue::NotSet,
        name: ActiveValue::Set(format!("itest_cat_srch_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductCategories");

    let product = products::ActiveModel {
        product_id: ActiveValue::NotSet,
        sku: ActiveValue::Set(None),
        name: ActiveValue::Set("Search Order Product".to_string()),
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
    .insert(&txn)
    .await
    .expect("insert Products");

    let variant = product_variants::ActiveModel {
        variant_id: ActiveValue::NotSet,
        product_id: ActiveValue::Set(product.product_id),
        size_id: ActiveValue::Set(None),
        color_id: ActiveValue::Set(None),
        additional_price: ActiveValue::Set(Some(0)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductVariants");

    let _inv = inventory::ActiveModel {
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
    .expect("create_cart_item should succeed");
    let selected_cart_id = cart_res.into_inner().items[0].cart_id;

    // Commit setup so it's visible to place_order's own (separately-opened) transactions.
    txn.commit().await.expect("commit setup txn");

    let place_res = place_order(
        &db,
        Request::new(PlaceOrderRequest {
            shipping_address_id: shipping.shipping_address_id,
            user_id,
            coupon_code: None,
            selected_cart_ids: vec![selected_cart_id],
            payment_mode: None,
        }),
    )
    .await
    .expect("place_order should succeed");
    let order_id = place_res.into_inner().items[0].order_id;

    // search_order is an unrelated handler whose own signature didn't change (only place_order's
    // did) — it still takes a caller-supplied `&DatabaseTransaction`, so we open a fresh
    // (read-only) transaction here to drive it against the now-committed order.
    let search_txn = db.begin().await.expect("begin search transaction");
    let search_res = core_operations::handlers::orders::search_order(
        &search_txn,
        Request::new(SearchOrderRequest {
            order_id: Some(order_id),
            user_id: Some(user_id),
            order_date_start: None,
            order_date_end: None,
            status_id: None,
            limit: Some(10),
            offset: Some(0),
        }),
    )
    .await
    .expect("search_order should succeed");
    let orders = search_res.into_inner().items;
    // Read-only lookup; nothing to commit.
    search_txn.rollback().await.ok();

    cleanup_checkout_rows(
        &db,
        &[order_id],
        &[user_id],
        &[role.role_id],
        &[category.category_id],
        &[variant.variant_id],
        &[shipping.shipping_address_id],
    )
    .await;

    assert_eq!(orders.len(), 1);
    assert_eq!(orders[0].order_id, order_id);
    assert_eq!(orders[0].user_id, user_id);
}
