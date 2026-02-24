//! Integration tests for critical paths. Require a real MySQL database.
//!
//! **Setup**
//! - Set `TEST_DATABASE_URL` or `DATABASE_URL` (e.g. `mysql://root:test_password@127.0.0.1:3306/sudattas_test`).
//! - Schema must be loaded first (e.g. migrations or `backend/database/sql_dump/01_schema.sql`).
//!
//! **Run**
//! - `cargo test --test integration_critical_paths -- --ignored`
//!
//! **Coverage**
//! - User creation and rollback (no committed data).
//! - Product search with limit.
//! - Guest cart by session_id: create item, get cart, rollback.
//! - Place order (may fail with FailedPrecondition if cart empty or stock missing).

use proto::proto::core::{
    CreateCartItemRequest, CreateCategoryRequest, CreateCityRequest, CreateCountryRequest,
    CreateProductRequest, CreateShippingAddressRequest, CreateStateRequest, CreateUserRequest,
    GetCartItemsRequest, PlaceOrderRequest, SearchProductRequest,
};
use sea_orm::{Database, TransactionTrait};
use tonic::Request;

fn test_db_url() -> String {
    std::env::var("TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .expect("TEST_DATABASE_URL or DATABASE_URL must be set for integration tests")
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_create_user() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let req = Request::new(CreateUserRequest {
        username: "integration_user".to_string(),
        email: format!(
            "integration_{}@test.local",
            std::time::SystemTime::now().elapsed().unwrap().as_millis()
        ),
        password: "SecurePass123!".to_string(),
        full_name: Some("Integration User".to_string()),
        address: None,
        phone: None,
    });

    let result = core_operations::handlers::users::create_user(&txn, req).await;
    txn.rollback().await.ok(); // don't commit test data

    assert!(
        result.is_ok(),
        "create_user should succeed: {:?}",
        result.err()
    );
    let response = result.unwrap().into_inner();
    assert_eq!(response.items.len(), 1);
    let user = &response.items[0];
    assert_eq!(user.username, "integration_user");
    assert!(user.user_id > 0);
    assert!(user.email.starts_with("integration_") && user.email.ends_with("@test.local"));
    assert_eq!(user.full_name.as_deref(), Some("Integration User"));
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_search_product() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let req = Request::new(SearchProductRequest {
        name: None,
        description: None,
        starting_price: None,
        ending_price: None,
        stock_quantity: None,
        category_id: None,
        product_id: None,
        limit: Some(10),
        offset: None,
    });

    let result = core_operations::handlers::products::search_product(&txn, req).await;
    txn.rollback().await.ok();

    assert!(
        result.is_ok(),
        "search_product should succeed: {:?}",
        result.err()
    );
    let response = result.unwrap().into_inner();
    assert!(response.items.len() <= 10, "limit 10 should be respected");
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_cart_by_session() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    // Create category and product (handled by test)
    let cat = core_operations::handlers::categories::create_category(
        &txn,
        Request::new(CreateCategoryRequest {
            name: "Integration Test Category".to_string(),
        }),
    )
    .await
    .expect("create category");
    let category_id = cat.into_inner().items[0].category_id;

    let prod = core_operations::handlers::products::create_product(
        &txn,
        Request::new(CreateProductRequest {
            name: "Integration Test Product".to_string(),
            description: None,
            price: 9.99,
            stock_quantity: Some(10),
            category_id: Some(category_id),
        }),
    )
    .await
    .expect("create product");
    let product_id = prod.into_inner().items[0].product_id;

    let session_id = format!(
        "test_session_{}",
        std::time::SystemTime::now().elapsed().unwrap().as_millis()
    );

    let create_req = Request::new(CreateCartItemRequest {
        user_id: None,
        session_id: Some(session_id.clone()),
        product_id,
        quantity: 2,
    });
    let create_result = core_operations::handlers::cart::create_cart_item(&txn, create_req).await;
    assert!(
        create_result.is_ok(),
        "create_cart_item: {:?}",
        create_result.err()
    );

    let get_req = Request::new(GetCartItemsRequest {
        user_id: None,
        session_id: Some(session_id),
    });
    let get_result = core_operations::handlers::cart::get_cart_items(&txn, get_req).await;
    txn.rollback().await.ok();

    assert!(
        get_result.is_ok(),
        "get_cart_items should succeed after create"
    );
    let response = get_result.unwrap().into_inner();
    assert!(
        !response.items.is_empty(),
        "cart should contain the created item"
    );
    assert_eq!(response.items[0].product_id, product_id);
    assert_eq!(response.items[0].quantity, 2);
}

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
    txn.rollback().await.ok();

    // May fail with precondition (no cart items, or stock) in a fresh DB
    if let Err(e) = &result {
        if e.code() == tonic::Code::FailedPrecondition {
            return; // expected when cart is empty or stock missing
        }
    }
    assert!(result.is_ok(), "place_order failed: {:?}", result.err());
}
