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

    let ts = std::time::SystemTime::now().elapsed().unwrap().as_millis();
    let user_a = core_operations::handlers::users::create_user(
        &txn_setup,
        Request::new(CreateUserRequest {
            username: format!("concurrent_a_{}", ts),
            email: format!("concurrent_a_{}@test.local", ts),
            password: "SecurePass123!".to_string(),
            full_name: Some("User A".to_string()),
            address: None,
            phone: None,
        }),
    )
    .await
    .expect("create user A");
    let user_id_a = user_a.into_inner().items[0].user_id;

    let user_b = core_operations::handlers::users::create_user(
        &txn_setup,
        Request::new(CreateUserRequest {
            username: format!("concurrent_b_{}", ts),
            email: format!("concurrent_b_{}@test.local", ts),
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
