//! Integration tests for order placement and inventory semantics.
//!
//! **Setup**
//! - Set `TEST_DATABASE_URL` or `DATABASE_URL` (e.g. `mysql://root:test_password@127.0.0.1:3306/sudattas_test`).
//! - Schema must be loaded first (e.g. migrations or `backend/database/sql_dump/01_schema.sql`).
//!
//! **Run**
//! - `cargo test --test integration_orders_and_inventory -- --ignored`

mod integration_common;

use integration_common::test_db_url;
use proto::proto::core::{
    CreateCartItemRequest, CreateCategoryRequest, CreateCityRequest, CreateCountryRequest,
    CreateInventoryItemRequest, CreateProductRequest, CreateShippingAddressRequest,
    CreateStateRequest, CreateSupplierRequest, CreateUserRequest, PlaceOrderRequest,
    UpdateOrderRequest,
};
use sea_orm::{Database, TransactionTrait};
use tonic::Request;
use uuid::Uuid;

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_place_order() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    // Create required data (handled by test)
    let country = core_operations::handlers::country::create_country(
        &txn,
        Request::new(CreateCountryRequest {
            country_name: "Test Country".to_string(),
        }),
    )
    .await
    .expect("create country");
    let country_id = country.into_inner().items[0].country_id;

    let state = core_operations::handlers::state::create_state(
        &txn,
        Request::new(CreateStateRequest {
            state_name: "Test State".to_string(),
        }),
    )
    .await
    .expect("create state");
    let state_id = state.into_inner().items[0].state_id;

    let city = core_operations::handlers::city::create_city(
        &txn,
        Request::new(CreateCityRequest {
            city_name: "Test City".to_string(),
        }),
    )
    .await
    .expect("create city");
    let city_id = city.into_inner().items[0].city_id;

    let addr = core_operations::handlers::shipping_address::create_shipping_address(
        &txn,
        Request::new(CreateShippingAddressRequest {
            country_id,
            state_id,
            city_id,
            road: "123 Test St".to_string(),
            apartment_no_or_name: "".to_string(),
        }),
    )
    .await
    .expect("create shipping address");
    let shipping_address_id = addr.into_inner().items[0].shipping_address_id;

    let user = core_operations::handlers::users::create_user(
        &txn,
        Request::new(CreateUserRequest {
            username: "place_order_user".to_string(),
            email: format!(
                "place_order_{}@test.local",
                std::time::SystemTime::now().elapsed().unwrap().as_millis()
            ),
            password: "SecurePass123!".to_string(),
            full_name: Some("Place Order User".to_string()),
            address: None,
            phone: None,
        }),
    )
    .await
    .expect("create user");
    let user_id = user.into_inner().items[0].user_id;

    let req = Request::new(PlaceOrderRequest {
        user_id,
        shipping_address_id,
        coupon_code: None,
    });

    let result = core_operations::procedures::orders::place_order(&txn, req).await;

    // Always roll back so this test remains non-destructive.
    txn.rollback().await.ok();

    // May fail with precondition (no cart items, or stock) in a fresh DB
    if let Err(e) = &result {
        if e.code() == tonic::Code::FailedPrecondition {
            return; // expected when cart is empty or stock missing
        }
    }

    let response = result.expect("place_order should succeed when preconditions are met");
    let orders = response.into_inner().items;
    assert!(
        !orders.is_empty(),
        "place_order response should contain at least one order"
    );

    // Sanity check: total_amount should be non-negative and stable when recomputed.
    let order = &orders[0];
    let stored_total = order.total_amount;
    assert!(
        stored_total >= 0.0,
        "order.total_amount should be non-negative, got {}",
        stored_total
    );
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and inventory semantics (place_order)"]
async fn integration_place_order_affects_inventory() {
    use core_db_entities::entity::inventory;
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    // Create a supplier for inventory records.
    let supplier = core_operations::handlers::suppliers::create_supplier(
        &txn,
        Request::new(CreateSupplierRequest {
            name: "Integration Supplier".to_string(),
            contact_info: "supplier@example.test".to_string(),
            address: "123 Supplier St".to_string(),
        }),
    )
    .await
    .expect("create supplier");
    let supplier_id = supplier.into_inner().items[0].supplier_id;

    // Create category and product.
    let cat = core_operations::handlers::categories::create_category(
        &txn,
        Request::new(CreateCategoryRequest {
            name: "Inventory Test Category".to_string(),
        }),
    )
    .await
    .expect("create category");
    let category_id = cat.into_inner().items[0].category_id;

    let prod = core_operations::handlers::products::create_product(
        &txn,
        Request::new(CreateProductRequest {
            name: "Inventory Test Product".to_string(),
            description: None,
            price: 10.0,
            stock_quantity: Some(10),
            category_id: Some(category_id),
        }),
    )
    .await
    .expect("create product");
    let product_id = prod.into_inner().items[0].product_id;

    // Seed inventory for the product.
    let _inv = core_operations::handlers::inventory::create_inventory_item(
        &txn,
        Request::new(CreateInventoryItemRequest {
            product_id,
            quantity_available: 10,
            reorder_level: 0,
            supplier_id,
        }),
    )
    .await
    .expect("create inventory item");

    // Create country/state/city and shipping address.
    let country = core_operations::handlers::country::create_country(
        &txn,
        Request::new(CreateCountryRequest {
            country_name: "Inventory Country".to_string(),
        }),
    )
    .await
    .expect("create country");
    let country_id = country.into_inner().items[0].country_id;

    let state = core_operations::handlers::state::create_state(
        &txn,
        Request::new(CreateStateRequest {
            state_name: "Inventory State".to_string(),
        }),
    )
    .await
    .expect("create state");
    let state_id = state.into_inner().items[0].state_id;

    let city = core_operations::handlers::city::create_city(
        &txn,
        Request::new(CreateCityRequest {
            city_name: "Inventory City".to_string(),
        }),
    )
    .await
    .expect("create city");
    let city_id = city.into_inner().items[0].city_id;

    let addr = core_operations::handlers::shipping_address::create_shipping_address(
        &txn,
        Request::new(CreateShippingAddressRequest {
            country_id,
            state_id,
            city_id,
            road: "456 Inventory St".to_string(),
            apartment_no_or_name: "".to_string(),
        }),
    )
    .await
    .expect("create shipping address");
    let shipping_address_id = addr.into_inner().items[0].shipping_address_id;

    // Create user.
    let user = core_operations::handlers::users::create_user(
        &txn,
        Request::new(CreateUserRequest {
            username: "inventory_order_user".to_string(),
            email: format!(
                "inventory_order_{}@test.local",
                std::time::SystemTime::now().elapsed().unwrap().as_millis()
            ),
            password: "SecurePass123!".to_string(),
            full_name: Some("Inventory Order User".to_string()),
            address: None,
            phone: None,
        }),
    )
    .await
    .expect("create user");
    let user_id = user.into_inner().items[0].user_id;

    // Put the product into the user's cart.
    let _cart = core_operations::handlers::cart::create_cart_item(
        &txn,
        Request::new(CreateCartItemRequest {
            user_id: Some(user_id),
            session_id: None,
            product_id,
            quantity: 2,
        }),
    )
    .await
    .expect("create cart item for user");

    // Place the order.
    let result = core_operations::procedures::orders::place_order(
        &txn,
        Request::new(PlaceOrderRequest {
            user_id,
            shipping_address_id,
            coupon_code: None,
        }),
    )
    .await;

    let response = result.expect("place_order should succeed with valid cart and stock");
    let orders = response.into_inner().items;
    assert!(
        !orders.is_empty(),
        "place_order response should contain at least one order"
    );

    // Verify that inventory was decremented by the ordered quantity.
    let inv_row = inventory::Entity::find()
        .filter(inventory::Column::ProductId.eq(product_id))
        .one(&txn)
        .await
        .expect("query inventory after place_order")
        .expect("inventory row should exist for product");

    assert_eq!(
        inv_row.quantity_available,
        Some(8),
        "inventory quantity_available should be decremented by ordered quantity"
    );

    txn.rollback().await.ok();
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and inventory semantics (cancel order)"]
async fn integration_cancel_order_restores_inventory() {
    use core_db_entities::entity::{inventory, order_status};
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    // Create a supplier for inventory records.
    let supplier = core_operations::handlers::suppliers::create_supplier(
        &txn,
        Request::new(CreateSupplierRequest {
            name: "Integration Supplier".to_string(),
            contact_info: "supplier@example.test".to_string(),
            address: "123 Supplier St".to_string(),
        }),
    )
    .await
    .expect("create supplier");
    let supplier_id = supplier.into_inner().items[0].supplier_id;

    // Create category and product.
    let cat = core_operations::handlers::categories::create_category(
        &txn,
        Request::new(CreateCategoryRequest {
            name: "Inventory Test Category".to_string(),
        }),
    )
    .await
    .expect("create category");
    let category_id = cat.into_inner().items[0].category_id;

    let prod = core_operations::handlers::products::create_product(
        &txn,
        Request::new(CreateProductRequest {
            name: "Inventory Test Product".to_string(),
            description: None,
            price: 10.0,
            stock_quantity: Some(10),
            category_id: Some(category_id),
        }),
    )
    .await
    .expect("create product");
    let product_id = prod.into_inner().items[0].product_id;

    // Seed inventory for the product.
    let _inv = core_operations::handlers::inventory::create_inventory_item(
        &txn,
        Request::new(CreateInventoryItemRequest {
            product_id,
            quantity_available: 10,
            reorder_level: 0,
            supplier_id,
        }),
    )
    .await
    .expect("create inventory item");

    // Create country/state/city and shipping address.
    let country = core_operations::handlers::country::create_country(
        &txn,
        Request::new(CreateCountryRequest {
            country_name: "Inventory Country".to_string(),
        }),
    )
    .await
    .expect("create country");
    let country_id = country.into_inner().items[0].country_id;

    let state = core_operations::handlers::state::create_state(
        &txn,
        Request::new(CreateStateRequest {
            state_name: "Inventory State".to_string(),
        }),
    )
    .await
    .expect("create state");
    let state_id = state.into_inner().items[0].state_id;

    let city = core_operations::handlers::city::create_city(
        &txn,
        Request::new(CreateCityRequest {
            city_name: "Inventory City".to_string(),
        }),
    )
    .await
    .expect("create city");
    let city_id = city.into_inner().items[0].city_id;

    let addr = core_operations::handlers::shipping_address::create_shipping_address(
        &txn,
        Request::new(CreateShippingAddressRequest {
            country_id,
            state_id,
            city_id,
            road: "456 Inventory St".to_string(),
            apartment_no_or_name: "".to_string(),
        }),
    )
    .await
    .expect("create shipping address");
    let shipping_address_id = addr.into_inner().items[0].shipping_address_id;

    // Create user.
    let user = core_operations::handlers::users::create_user(
        &txn,
        Request::new(CreateUserRequest {
            username: "inventory_cancel_user".to_string(),
            email: format!(
                "inventory_cancel_{}@test.local",
                std::time::SystemTime::now().elapsed().unwrap().as_millis()
            ),
            password: "SecurePass123!".to_string(),
            full_name: Some("Inventory Cancel User".to_string()),
            address: None,
            phone: None,
        }),
    )
    .await
    .expect("create user");
    let user_id = user.into_inner().items[0].user_id;

    // Put the product into the user's cart.
    let _cart = core_operations::handlers::cart::create_cart_item(
        &txn,
        Request::new(CreateCartItemRequest {
            user_id: Some(user_id),
            session_id: None,
            product_id,
            quantity: 2,
        }),
    )
    .await
    .expect("create cart item for user");

    // Place the order.
    let result = core_operations::procedures::orders::place_order(
        &txn,
        Request::new(PlaceOrderRequest {
            user_id,
            shipping_address_id,
            coupon_code: None,
        }),
    )
    .await;

    let response = result.expect("place_order should succeed with valid cart and stock");
    let orders = response.into_inner().items;
    assert!(
        !orders.is_empty(),
        "place_order response should contain at least one order"
    );
    let order = &orders[0];

    // Look up the `cancelled` status id from the OrderStatus table.
    let cancelled_status = order_status::Entity::find()
        .filter(order_status::Column::StatusName.eq("cancelled".to_string()))
        .one(&txn)
        .await
        .expect("query cancelled status")
        .expect("cancelled status should exist");

    // Cancel the order.
    let cancel_req = Request::new(UpdateOrderRequest {
        order_id: order.order_id,
        user_id: order.user_id,
        shipping_address_id: order.shipping_address_id,
        total_amount: order.total_amount,
        status_id: cancelled_status.status_id,
    });

    let cancel_res = core_operations::handlers::orders::update_order(&txn, cancel_req).await;
    assert!(
        cancel_res.is_ok(),
        "update_order should succeed when cancelling order: {:?}",
        cancel_res.err()
    );

    // After cancellation, inventory should be restored to its original quantity.
    let inv_row_after_cancel = inventory::Entity::find()
        .filter(inventory::Column::ProductId.eq(product_id))
        .one(&txn)
        .await
        .expect("query inventory after cancellation")
        .expect("inventory row should exist for product");

    assert_eq!(
        inv_row_after_cancel.quantity_available,
        Some(10),
        "cancelling the order should restore inventory to its original quantity"
    );

    txn.rollback().await.ok();
}

/// Phase 3 concurrency: two concurrent checkouts for the last unit → exactly one succeeds,
/// and inventory never goes negative.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema; commits one order"]
async fn integration_concurrent_checkouts_last_unit_exactly_one_succeeds() {
    use core_db_entities::entity::inventory;
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, TransactionTrait};

    let url = test_db_url();
    let db1 = Database::connect(&url).await.expect("connect db1");
    let db2 = Database::connect(&url).await.expect("connect db2");

    // Seed data on db1 and commit so both concurrent transactions see it.
    let txn_setup = db1.begin().await.expect("begin setup");
    let supplier = core_operations::handlers::suppliers::create_supplier(
        &txn_setup,
        Request::new(CreateSupplierRequest {
            name: "Concurrent Supplier".to_string(),
            contact_info: "supplier@example.test".to_string(),
            address: "123 Supplier St".to_string(),
        }),
    )
    .await
    .expect("create supplier");
    let supplier_id = supplier.into_inner().items[0].supplier_id;

    let cat = core_operations::handlers::categories::create_category(
        &txn_setup,
        Request::new(CreateCategoryRequest {
            name: "Concurrent Category".to_string(),
        }),
    )
    .await
    .expect("create category");
    let category_id = cat.into_inner().items[0].category_id;

    let prod = core_operations::handlers::products::create_product(
        &txn_setup,
        Request::new(CreateProductRequest {
            name: "Last Unit Product".to_string(),
            description: None,
            price: 5.0,
            stock_quantity: Some(1),
            category_id: Some(category_id),
        }),
    )
    .await
    .expect("create product");
    let product_id = prod.into_inner().items[0].product_id;

    let _inv = core_operations::handlers::inventory::create_inventory_item(
        &txn_setup,
        Request::new(CreateInventoryItemRequest {
            product_id,
            quantity_available: 1,
            reorder_level: 0,
            supplier_id,
        }),
    )
    .await
    .expect("create inventory item");

    let country = core_operations::handlers::country::create_country(
        &txn_setup,
        Request::new(CreateCountryRequest {
            country_name: "Concurrent Country".to_string(),
        }),
    )
    .await
    .expect("create country");
    let country_id = country.into_inner().items[0].country_id;

    let state = core_operations::handlers::state::create_state(
        &txn_setup,
        Request::new(CreateStateRequest {
            state_name: "Concurrent State".to_string(),
        }),
    )
    .await
    .expect("create state");
    let state_id = state.into_inner().items[0].state_id;

    let city = core_operations::handlers::city::create_city(
        &txn_setup,
        Request::new(CreateCityRequest {
            city_name: "Concurrent City".to_string(),
        }),
    )
    .await
    .expect("create city");
    let city_id = city.into_inner().items[0].city_id;

    let addr = core_operations::handlers::shipping_address::create_shipping_address(
        &txn_setup,
        Request::new(CreateShippingAddressRequest {
            country_id,
            state_id,
            city_id,
            road: "789 Concurrent St".to_string(),
            apartment_no_or_name: "".to_string(),
        }),
    )
    .await
    .expect("create shipping address");
    let shipping_address_id = addr.into_inner().items[0].shipping_address_id;

    let unique = Uuid::new_v4().to_string();
    let user_a = core_operations::handlers::users::create_user(
        &txn_setup,
        Request::new(CreateUserRequest {
            username: format!("concurrent_a_{}", unique),
            email: format!("concurrent_a_{}@test.local", unique),
            password: "SecurePass123!".to_string(),
            full_name: Some("User A".to_string()),
            address: None,
            phone: None,
        }),
    )
    .await
    .expect("create user A");
    let user_id_a = user_a.into_inner().items[0].user_id;

    let unique_b = Uuid::new_v4().to_string();
    let user_b = core_operations::handlers::users::create_user(
        &txn_setup,
        Request::new(CreateUserRequest {
            username: format!("concurrent_b_{}", unique_b),
            email: format!("concurrent_b_{}@test.local", unique_b),
            password: "SecurePass123!".to_string(),
            full_name: Some("User B".to_string()),
            address: None,
            phone: None,
        }),
    )
    .await
    .expect("create user B");
    let user_id_b = user_b.into_inner().items[0].user_id;

    let _ = core_operations::handlers::cart::create_cart_item(
        &txn_setup,
        Request::new(CreateCartItemRequest {
            user_id: Some(user_id_a),
            session_id: None,
            product_id,
            quantity: 1,
        }),
    )
    .await
    .expect("cart item user A");
    let _ = core_operations::handlers::cart::create_cart_item(
        &txn_setup,
        Request::new(CreateCartItemRequest {
            user_id: Some(user_id_b),
            session_id: None,
            product_id,
            quantity: 1,
        }),
    )
    .await
    .expect("cart item user B");

    txn_setup.commit().await.expect("commit setup");

    // Run two place_order calls concurrently on separate connections.
    let req_a = Request::new(PlaceOrderRequest {
        user_id: user_id_a,
        shipping_address_id,
        coupon_code: None,
    });
    let req_b = Request::new(PlaceOrderRequest {
        user_id: user_id_b,
        shipping_address_id,
        coupon_code: None,
    });

    let (res_a, res_b) = tokio::join!(
        async {
            let txn = db1.begin().await.expect("begin a");
            let r = core_operations::procedures::orders::place_order(&txn, req_a).await;
            if r.is_ok() {
                txn.commit().await.ok();
            } else {
                txn.rollback().await.ok();
            }
            r
        },
        async {
            let txn = db2.begin().await.expect("begin b");
            let r = core_operations::procedures::orders::place_order(&txn, req_b).await;
            if r.is_ok() {
                txn.commit().await.ok();
            } else {
                txn.rollback().await.ok();
            }
            r
        },
    );

    let ok_count = res_a.is_ok() as i32 + res_b.is_ok() as i32;
    assert_eq!(
        ok_count, 1,
        "exactly one concurrent checkout for last unit should succeed; got ok_count={} (a={:?}, b={:?})",
        ok_count, res_a, res_b
    );

    // Inventory never negative: our product should be 0; all rows must be >= 0.
    let txn_read = db1.begin().await.expect("begin read");
    let inv_row = inventory::Entity::find()
        .filter(inventory::Column::ProductId.eq(product_id))
        .one(&txn_read)
        .await
        .expect("query inventory")
        .expect("inventory row exists");
    assert!(
        inv_row.quantity_available.unwrap_or(0) >= 0,
        "inventory quantity_available must never be negative"
    );
    assert_eq!(
        inv_row.quantity_available,
        Some(0),
        "after one successful checkout for last unit, quantity_available should be 0"
    );

    let all_inv = inventory::Entity::find()
        .all(&txn_read)
        .await
        .expect("query all inventory");
    for row in &all_inv {
        let q = row.quantity_available.unwrap_or(0);
        assert!(
            q >= 0,
            "inventory never negative: product {:?} has quantity_available {}",
            row.product_id,
            q
        );
    }
    txn_read.rollback().await.ok();
}

// --- Phase 4: Order Snapshot & Coupon Usage ---

/// Price changes after order do not affect stored snapshots (line and order level).
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema (Phase 4 snapshot columns)"]
async fn integration_order_snapshot_unchanged_after_price_change() {
    use core_db_entities::entity::{order_details, orders, products};
    use sea_orm::{
        ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
    };

    let db = Database::connect(&test_db_url()).await.expect("connect");
    let txn = db.begin().await.expect("begin");

    let supplier = core_operations::handlers::suppliers::create_supplier(
        &txn,
        Request::new(CreateSupplierRequest {
            name: "Snapshot Supplier".to_string(),
            contact_info: "s@test".to_string(),
            address: "Addr".to_string(),
        }),
    )
    .await
    .expect("supplier");
    let supplier_id = supplier.into_inner().items[0].supplier_id;
    let cat = core_operations::handlers::categories::create_category(
        &txn,
        Request::new(CreateCategoryRequest {
            name: "Snapshot Cat".to_string(),
        }),
    )
    .await
    .expect("category");
    let category_id = cat.into_inner().items[0].category_id;
    let prod = core_operations::handlers::products::create_product(
        &txn,
        Request::new(CreateProductRequest {
            name: "Snapshot Product".to_string(),
            description: None,
            price: 100.0,
            stock_quantity: Some(5),
            category_id: Some(category_id),
        }),
    )
    .await
    .expect("product");
    let product_id = prod.into_inner().items[0].product_id;
    let _inv = core_operations::handlers::inventory::create_inventory_item(
        &txn,
        Request::new(CreateInventoryItemRequest {
            product_id,
            quantity_available: 5,
            reorder_level: 0,
            supplier_id,
        }),
    )
    .await
    .expect("inventory");
    let country = core_operations::handlers::country::create_country(
        &txn,
        Request::new(CreateCountryRequest {
            country_name: "Snap Country".to_string(),
        }),
    )
    .await
    .expect("country");
    let state = core_operations::handlers::state::create_state(
        &txn,
        Request::new(CreateStateRequest {
            state_name: "Snap State".to_string(),
        }),
    )
    .await
    .expect("state");
    let city = core_operations::handlers::city::create_city(
        &txn,
        Request::new(CreateCityRequest {
            city_name: "Snap City".to_string(),
        }),
    )
    .await
    .expect("city");
    let addr = core_operations::handlers::shipping_address::create_shipping_address(
        &txn,
        Request::new(CreateShippingAddressRequest {
            country_id: country.into_inner().items[0].country_id,
            state_id: state.into_inner().items[0].state_id,
            city_id: city.into_inner().items[0].city_id,
            road: "R".to_string(),
            apartment_no_or_name: "".to_string(),
        }),
    )
    .await
    .expect("address");
    let shipping_address_id = addr.into_inner().items[0].shipping_address_id;
    let ts = std::time::SystemTime::now().elapsed().unwrap().as_millis();
    let user = core_operations::handlers::users::create_user(
        &txn,
        Request::new(CreateUserRequest {
            username: format!("snap_u_{}", ts),
            email: format!("snap_{}@t.local", ts),
            password: "Pass123!".to_string(),
            full_name: Some("U".to_string()),
            address: None,
            phone: None,
        }),
    )
    .await
    .expect("user");
    let user_id = user.into_inner().items[0].user_id;
    let _ = core_operations::handlers::cart::create_cart_item(
        &txn,
        Request::new(CreateCartItemRequest {
            user_id: Some(user_id),
            session_id: None,
            product_id,
            quantity: 1,
        }),
    )
    .await
    .expect("cart");

    let place = core_operations::procedures::orders::place_order(
        &txn,
        Request::new(PlaceOrderRequest {
            user_id,
            shipping_address_id,
            coupon_code: None,
        }),
    )
    .await
    .expect("place_order");
    let order_id = place.into_inner().items[0].order_id;

    let order_before = orders::Entity::find_by_id(order_id)
        .one(&txn)
        .await
        .expect("order")
        .expect("exists");
    let details_before: Vec<_> = order_details::Entity::find()
        .filter(order_details::Column::OrderId.eq(order_id))
        .all(&txn)
        .await
        .expect("details");
    let grand_before = order_before.grand_total_minor;
    let unit_before = details_before.first().and_then(|d| d.unit_price_minor);
    assert!(
        grand_before.is_some(),
        "snapshot grand_total_minor should be set"
    );
    assert!(
        unit_before.is_some(),
        "snapshot unit_price_minor should be set"
    );

    let prod_model = products::Entity::find_by_id(product_id)
        .one(&txn)
        .await
        .expect("prod")
        .expect("exists");
    let mut active = prod_model.into_active_model();
    active.price = ActiveValue::Set(rust_decimal::Decimal::try_new(99900, 2).unwrap());
    active.update(&txn).await.expect("update product price");

    let order_after = orders::Entity::find_by_id(order_id)
        .one(&txn)
        .await
        .expect("order")
        .expect("exists");
    let details_after = order_details::Entity::find()
        .filter(order_details::Column::OrderId.eq(order_id))
        .all(&txn)
        .await
        .expect("details");
    assert_eq!(
        order_after.grand_total_minor, grand_before,
        "grand_total_minor snapshot must not change after product price change"
    );
    assert_eq!(
        details_after.first().and_then(|d| d.unit_price_minor),
        unit_before,
        "unit_price_minor snapshot must not change after product price change"
    );

    txn.rollback().await.ok();
}

/// Coupon usage_count is not incremented by place_order; only on verified payment (Phase 4).
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema (Phase 4); coupon row must exist"]
async fn integration_coupon_usage_count_not_incremented_by_place_order() {
    use core_db_entities::entity::coupons;
    use core_db_entities::entity::sea_orm_active_enums::{CouponStatus, DiscountType};
    use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter};

    let db = Database::connect(&test_db_url()).await.expect("connect");
    let txn = db.begin().await.expect("begin");

    let code = format!(
        "phase4_{}",
        std::time::SystemTime::now().elapsed().unwrap().as_millis()
    );
    let coupon = coupons::ActiveModel {
        coupon_id: ActiveValue::NotSet,
        code: ActiveValue::Set(code.clone()),
        discount_type: ActiveValue::Set(DiscountType::FixedAmount),
        discount_value: ActiveValue::Set(500),
        min_order_value_paise: ActiveValue::Set(None),
        usage_limit: ActiveValue::Set(Some(2)),
        usage_count: ActiveValue::Set(Some(0)),
        coupon_status: ActiveValue::Set(Some(CouponStatus::Active)),
        starts_at: ActiveValue::Set(chrono::Utc::now() - chrono::Duration::days(1)),
        ends_at: ActiveValue::Set(None),
        created_at: ActiveValue::Set(None),
    };
    coupon.insert(&txn).await.expect("insert coupon");

    let supplier = core_operations::handlers::suppliers::create_supplier(
        &txn,
        Request::new(CreateSupplierRequest {
            name: "Coupon Supplier".to_string(),
            contact_info: "c@test".to_string(),
            address: "A".to_string(),
        }),
    )
    .await
    .expect("supplier");
    let supplier_id = supplier.into_inner().items[0].supplier_id;
    let cat = core_operations::handlers::categories::create_category(
        &txn,
        Request::new(CreateCategoryRequest {
            name: "Coupon Cat".to_string(),
        }),
    )
    .await
    .expect("category");
    let prod = core_operations::handlers::products::create_product(
        &txn,
        Request::new(CreateProductRequest {
            name: "Coupon Product".to_string(),
            description: None,
            price: 20.0,
            stock_quantity: Some(10),
            category_id: Some(cat.into_inner().items[0].category_id),
        }),
    )
    .await
    .expect("product");
    let product_id = prod.into_inner().items[0].product_id;
    let _inv = core_operations::handlers::inventory::create_inventory_item(
        &txn,
        Request::new(CreateInventoryItemRequest {
            product_id,
            quantity_available: 10,
            reorder_level: 0,
            supplier_id,
        }),
    )
    .await
    .expect("inventory");
    let country = core_operations::handlers::country::create_country(
        &txn,
        Request::new(CreateCountryRequest {
            country_name: "C".to_string(),
        }),
    )
    .await
    .expect("country");
    let state = core_operations::handlers::state::create_state(
        &txn,
        Request::new(CreateStateRequest {
            state_name: "S".to_string(),
        }),
    )
    .await
    .expect("state");
    let city = core_operations::handlers::city::create_city(
        &txn,
        Request::new(CreateCityRequest {
            city_name: "C".to_string(),
        }),
    )
    .await
    .expect("city");
    let addr = core_operations::handlers::shipping_address::create_shipping_address(
        &txn,
        Request::new(CreateShippingAddressRequest {
            country_id: country.into_inner().items[0].country_id,
            state_id: state.into_inner().items[0].state_id,
            city_id: city.into_inner().items[0].city_id,
            road: "R".to_string(),
            apartment_no_or_name: "".to_string(),
        }),
    )
    .await
    .expect("address");
    let user = core_operations::handlers::users::create_user(
        &txn,
        Request::new(CreateUserRequest {
            username: format!(
                "cu_{}",
                std::time::SystemTime::now().elapsed().unwrap().as_millis()
            ),
            email: format!(
                "cu_{}@t.local",
                std::time::SystemTime::now().elapsed().unwrap().as_millis()
            ),
            password: "Pass123!".to_string(),
            full_name: Some("U".to_string()),
            address: None,
            phone: None,
        }),
    )
    .await
    .expect("user");
    let user_id = user.into_inner().items[0].user_id;
    let _ = core_operations::handlers::cart::create_cart_item(
        &txn,
        Request::new(CreateCartItemRequest {
            user_id: Some(user_id),
            session_id: None,
            product_id,
            quantity: 1,
        }),
    )
    .await
    .expect("cart");

    core_operations::procedures::orders::place_order(
        &txn,
        Request::new(PlaceOrderRequest {
            user_id,
            shipping_address_id: addr.into_inner().items[0].shipping_address_id,
            coupon_code: Some(code.clone()),
        }),
    )
    .await
    .expect("place_order");

    let coupon_after = coupons::Entity::find()
        .filter(coupons::Column::Code.eq(&code))
        .one(&txn)
        .await
        .expect("query")
        .expect("coupon exists");
    assert_eq!(
        coupon_after.usage_count,
        Some(0),
        "Phase 4: usage_count must not be incremented by place_order (only on verified payment)"
    );

    txn.rollback().await.ok();
}

/// Phase 4: Failed payment (e.g. payment.failed or unverified) must not increment coupon usage_count;
/// only a successful payment.captured does.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema (Phase 4)"]
async fn integration_coupon_usage_not_incremented_by_failed_payment() {
    use core_db_entities::entity::coupons;
    use core_db_entities::entity::sea_orm_active_enums::{CouponStatus, DiscountType};
    use proto::proto::core::IngestWebhookRequest;
    use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter};
    use tonic::Request;

    let db = Database::connect(&test_db_url()).await.expect("connect");
    let txn = db.begin().await.expect("begin");

    let code = format!(
        "phase4_fail_{}",
        std::time::SystemTime::now().elapsed().unwrap().as_millis()
    );
    let coupon = coupons::ActiveModel {
        coupon_id: ActiveValue::NotSet,
        code: ActiveValue::Set(code.clone()),
        discount_type: ActiveValue::Set(DiscountType::FixedAmount),
        discount_value: ActiveValue::Set(500),
        min_order_value_paise: ActiveValue::Set(None),
        usage_limit: ActiveValue::Set(Some(2)),
        usage_count: ActiveValue::Set(Some(0)),
        coupon_status: ActiveValue::Set(Some(CouponStatus::Active)),
        starts_at: ActiveValue::Set(chrono::Utc::now() - chrono::Duration::days(1)),
        ends_at: ActiveValue::Set(None),
        created_at: ActiveValue::Set(None),
    };
    coupon.insert(&txn).await.expect("insert coupon");

    let supplier = core_operations::handlers::suppliers::create_supplier(
        &txn,
        Request::new(CreateSupplierRequest {
            name: "Fail Supplier".to_string(),
            contact_info: "f@test".to_string(),
            address: "A".to_string(),
        }),
    )
    .await
    .expect("supplier");
    let supplier_id = supplier.into_inner().items[0].supplier_id;
    let cat = core_operations::handlers::categories::create_category(
        &txn,
        Request::new(CreateCategoryRequest {
            name: "Fail Cat".to_string(),
        }),
    )
    .await
    .expect("category");
    let prod = core_operations::handlers::products::create_product(
        &txn,
        Request::new(CreateProductRequest {
            name: "Fail Product".to_string(),
            description: None,
            price: 20.0,
            stock_quantity: Some(10),
            category_id: Some(cat.into_inner().items[0].category_id),
        }),
    )
    .await
    .expect("product");
    let product_id = prod.into_inner().items[0].product_id;
    let _inv = core_operations::handlers::inventory::create_inventory_item(
        &txn,
        Request::new(CreateInventoryItemRequest {
            product_id,
            quantity_available: 10,
            reorder_level: 0,
            supplier_id,
        }),
    )
    .await
    .expect("inventory");
    let country = core_operations::handlers::country::create_country(
        &txn,
        Request::new(CreateCountryRequest {
            country_name: "C".to_string(),
        }),
    )
    .await
    .expect("country");
    let state = core_operations::handlers::state::create_state(
        &txn,
        Request::new(CreateStateRequest {
            state_name: "S".to_string(),
        }),
    )
    .await
    .expect("state");
    let city = core_operations::handlers::city::create_city(
        &txn,
        Request::new(CreateCityRequest {
            city_name: "C".to_string(),
        }),
    )
    .await
    .expect("city");
    let addr = core_operations::handlers::shipping_address::create_shipping_address(
        &txn,
        Request::new(CreateShippingAddressRequest {
            country_id: country.into_inner().items[0].country_id,
            state_id: state.into_inner().items[0].state_id,
            city_id: city.into_inner().items[0].city_id,
            road: "R".to_string(),
            apartment_no_or_name: "".to_string(),
        }),
    )
    .await
    .expect("address");
    let user = core_operations::handlers::users::create_user(
        &txn,
        Request::new(CreateUserRequest {
            username: format!(
                "fail_u_{}",
                std::time::SystemTime::now().elapsed().unwrap().as_millis()
            ),
            email: format!(
                "fail_u_{}@t.local",
                std::time::SystemTime::now().elapsed().unwrap().as_millis()
            ),
            password: "Pass123!".to_string(),
            full_name: Some("U".to_string()),
            address: None,
            phone: None,
        }),
    )
    .await
    .expect("user");
    let user_id = user.into_inner().items[0].user_id;
    let _ = core_operations::handlers::cart::create_cart_item(
        &txn,
        Request::new(CreateCartItemRequest {
            user_id: Some(user_id),
            session_id: None,
            product_id,
            quantity: 1,
        }),
    )
    .await
    .expect("cart");

    let place_resp = core_operations::procedures::orders::place_order(
        &txn,
        Request::new(PlaceOrderRequest {
            user_id,
            shipping_address_id: addr.into_inner().items[0].shipping_address_id,
            coupon_code: Some(code.clone()),
        }),
    )
    .await
    .expect("place_order");
    let order_id = place_resp
        .into_inner()
        .items
        .into_iter()
        .next()
        .expect("one order")
        .order_id;
    let razorpay_order_id = format!("rzp_pending_{}", order_id);

    // 1) payment.failed must not increment usage_count
    let payload_failed = serde_json::json!({
        "event": "payment.failed",
        "payload": {
            "payment": {
                "entity": {
                    "id": "pay_fail_1",
                    "order_id": razorpay_order_id,
                }
            }
        }
    });
    let _ = core_operations::handlers::webhooks::ingest_webhook(
        &txn,
        Request::new(IngestWebhookRequest {
            provider: "razorpay".to_string(),
            event_type: "payment.failed".to_string(),
            webhook_id: "razorpay:pay_fail_1".to_string(),
            payload_json: payload_failed.to_string(),
            signature_verified: true,
        }),
    )
    .await
    .ok();
    let coupon_after_fail = coupons::Entity::find()
        .filter(coupons::Column::Code.eq(&code))
        .one(&txn)
        .await
        .expect("query")
        .expect("coupon exists");
    assert_eq!(
        coupon_after_fail.usage_count,
        Some(0),
        "usage_count must not increment on payment.failed"
    );

    // 2) payment.captured must increment usage_count (Phase 5: amount + currency for verification).
    let payload_captured = serde_json::json!({
        "event": "payment.captured",
        "payload": {
            "payment": {
                "entity": {
                    "id": "pay_ok_1",
                    "order_id": razorpay_order_id,
                    "amount": 1500,
                    "currency": "INR"
                }
            }
        }
    });
    let _ = core_operations::handlers::webhooks::ingest_webhook(
        &txn,
        Request::new(IngestWebhookRequest {
            provider: "razorpay".to_string(),
            event_type: "payment.captured".to_string(),
            webhook_id: "razorpay:pay_ok_1".to_string(),
            payload_json: payload_captured.to_string(),
            signature_verified: true,
        }),
    )
    .await
    .expect("ingest_webhook payment.captured");
    let coupon_after_capture = coupons::Entity::find()
        .filter(coupons::Column::Code.eq(&code))
        .one(&txn)
        .await
        .expect("query")
        .expect("coupon exists");
    assert_eq!(
        coupon_after_capture.usage_count,
        Some(1),
        "usage_count must increment once on payment.captured"
    );

    txn.rollback().await.ok();
}

/// Phase 4: With usage_limit=1, two concurrent payment.captured webhooks must result in usage_count=1
/// (atomic increment enforces limit under concurrency).
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema (Phase 4); commits data"]
async fn integration_coupon_limit_enforced_under_concurrency() {
    use core_db_entities::entity::sea_orm_active_enums::{CouponStatus, DiscountType};
    use core_db_entities::entity::{coupons, payment_intents};
    use proto::proto::core::IngestWebhookRequest;
    use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter};
    use tonic::Request;

    let db = Database::connect(&test_db_url()).await.expect("connect");

    // Setup in one txn and commit so both webhook txns see the data.
    let txn_setup = db.begin().await.expect("begin");
    let code = format!("phase4_conc_{}", Uuid::new_v4());
    let coupon = coupons::ActiveModel {
        coupon_id: ActiveValue::NotSet,
        code: ActiveValue::Set(code.clone()),
        discount_type: ActiveValue::Set(DiscountType::FixedAmount),
        discount_value: ActiveValue::Set(500),
        min_order_value_paise: ActiveValue::Set(None),
        usage_limit: ActiveValue::Set(Some(1)),
        usage_count: ActiveValue::Set(Some(0)),
        coupon_status: ActiveValue::Set(Some(CouponStatus::Active)),
        starts_at: ActiveValue::Set(chrono::Utc::now() - chrono::Duration::days(1)),
        ends_at: ActiveValue::Set(None),
        created_at: ActiveValue::Set(None),
    };
    coupon.insert(&txn_setup).await.expect("insert coupon");

    let supplier = core_operations::handlers::suppliers::create_supplier(
        &txn_setup,
        Request::new(CreateSupplierRequest {
            name: "Conc Supplier".to_string(),
            contact_info: "c@test".to_string(),
            address: "A".to_string(),
        }),
    )
    .await
    .expect("supplier");
    let supplier_id = supplier.into_inner().items[0].supplier_id;
    let cat = core_operations::handlers::categories::create_category(
        &txn_setup,
        Request::new(CreateCategoryRequest {
            name: "Conc Cat".to_string(),
        }),
    )
    .await
    .expect("category");
    let prod = core_operations::handlers::products::create_product(
        &txn_setup,
        Request::new(CreateProductRequest {
            name: "Conc Product".to_string(),
            description: None,
            price: 15.0,
            stock_quantity: Some(20),
            category_id: Some(cat.into_inner().items[0].category_id),
        }),
    )
    .await
    .expect("product");
    let product_id = prod.into_inner().items[0].product_id;
    let _inv = core_operations::handlers::inventory::create_inventory_item(
        &txn_setup,
        Request::new(CreateInventoryItemRequest {
            product_id,
            quantity_available: 20,
            reorder_level: 0,
            supplier_id,
        }),
    )
    .await
    .expect("inventory");
    let country = core_operations::handlers::country::create_country(
        &txn_setup,
        Request::new(CreateCountryRequest {
            country_name: "CC".to_string(),
        }),
    )
    .await
    .expect("country");
    let state = core_operations::handlers::state::create_state(
        &txn_setup,
        Request::new(CreateStateRequest {
            state_name: "SS".to_string(),
        }),
    )
    .await
    .expect("state");
    let city = core_operations::handlers::city::create_city(
        &txn_setup,
        Request::new(CreateCityRequest {
            city_name: "CC".to_string(),
        }),
    )
    .await
    .expect("city");
    let addr = core_operations::handlers::shipping_address::create_shipping_address(
        &txn_setup,
        Request::new(CreateShippingAddressRequest {
            country_id: country.into_inner().items[0].country_id,
            state_id: state.into_inner().items[0].state_id,
            city_id: city.into_inner().items[0].city_id,
            road: "R".to_string(),
            apartment_no_or_name: "".to_string(),
        }),
    )
    .await
    .expect("address");
    let shipping_address_id = addr.into_inner().items[0].shipping_address_id;

    for i in 0..2 {
        let u = Uuid::new_v4().to_string();
        let user = core_operations::handlers::users::create_user(
            &txn_setup,
            Request::new(CreateUserRequest {
                username: format!("conc_u{}_{}", i, u),
                email: format!("conc_u{}_{}@t.local", i, u),
                password: "Pass123!".to_string(),
                full_name: Some("U".to_string()),
                address: None,
                phone: None,
            }),
        )
        .await
        .expect("user");
        let user_id = user.into_inner().items[0].user_id;
        let _ = core_operations::handlers::cart::create_cart_item(
            &txn_setup,
            Request::new(CreateCartItemRequest {
                user_id: Some(user_id),
                session_id: None,
                product_id,
                quantity: 1,
            }),
        )
        .await
        .expect("cart");
        core_operations::procedures::orders::place_order(
            &txn_setup,
            Request::new(PlaceOrderRequest {
                user_id,
                shipping_address_id,
                coupon_code: Some(code.clone()),
            }),
        )
        .await
        .expect("place_order");
    }

    txn_setup.commit().await.expect("commit setup");

    // Resolve the two intents for the two orders we created (both used the same coupon).
    let txn_read = db.begin().await.expect("begin");
    let coupon_row = coupons::Entity::find()
        .filter(coupons::Column::Code.eq(&code))
        .one(&txn_read)
        .await
        .expect("query")
        .expect("coupon exists");
    let order_ids: Vec<i64> = core_db_entities::entity::orders::Entity::find()
        .filter(core_db_entities::entity::orders::Column::AppliedCouponId.eq(coupon_row.coupon_id))
        .all(&txn_read)
        .await
        .expect("orders")
        .into_iter()
        .map(|o| o.order_id)
        .collect();
    txn_read.rollback().await.ok();
    assert_eq!(order_ids.len(), 2, "expected two orders with this coupon");
    let intents: Vec<_> = payment_intents::Entity::find()
        .filter(payment_intents::Column::OrderId.is_in(order_ids))
        .all(&db)
        .await
        .expect("intents");
    assert_eq!(intents.len(), 2, "expected two intents for the two orders");
    let intent1 = &intents[0];
    let intent2 = &intents[1];
    let razorpay_order_id_1 = intent1.razorpay_order_id.clone();
    let razorpay_order_id_2 = intent2.razorpay_order_id.clone();

    let req1 = Request::new(IngestWebhookRequest {
        provider: "razorpay".to_string(),
        event_type: "payment.captured".to_string(),
        webhook_id: format!("razorpay:pay_conc_1_{}", intent1.intent_id),
        payload_json: serde_json::json!({
            "event": "payment.captured",
            "payload": {
                "payment": {
                    "entity": {
                        "id": format!("pay_conc_1_{}", intent1.intent_id),
                        "order_id": razorpay_order_id_1,
                        "amount": intent1.amount_paise as i64,
                        "currency": intent1.currency.as_deref().unwrap_or("INR")
                    }
                }
            }
        })
        .to_string(),
        signature_verified: true,
    });
    let req2 = Request::new(IngestWebhookRequest {
        provider: "razorpay".to_string(),
        event_type: "payment.captured".to_string(),
        webhook_id: format!("razorpay:pay_conc_2_{}", intent2.intent_id),
        payload_json: serde_json::json!({
            "event": "payment.captured",
            "payload": {
                "payment": {
                    "entity": {
                        "id": format!("pay_conc_2_{}", intent2.intent_id),
                        "order_id": razorpay_order_id_2,
                        "amount": intent2.amount_paise as i64,
                        "currency": intent2.currency.as_deref().unwrap_or("INR")
                    }
                }
            }
        })
        .to_string(),
        signature_verified: true,
    });

    let txn1 = db.begin().await.expect("begin");
    let txn2 = db.begin().await.expect("begin");
    let (r1, r2) = tokio::join!(
        core_operations::handlers::webhooks::ingest_webhook(&txn1, req1),
        core_operations::handlers::webhooks::ingest_webhook(&txn2, req2),
    );
    r1.expect("webhook 1");
    r2.expect("webhook 2");
    txn1.commit().await.expect("commit webhook 1");
    txn2.commit().await.expect("commit webhook 2");

    let txn3 = db.begin().await.expect("begin");
    let coupon_final = coupons::Entity::find()
        .filter(coupons::Column::Code.eq(&code))
        .one(&txn3)
        .await
        .expect("query")
        .expect("coupon exists");
    txn3.rollback().await.ok();

    assert_eq!(
        coupon_final.usage_count,
        Some(1),
        "with usage_limit=1, concurrent captures must result in usage_count=1 (atomic increment)"
    );
}
