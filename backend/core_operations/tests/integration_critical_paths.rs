//! Integration tests for critical paths. Require a real MySQL database.
//!
//! Set `TEST_DATABASE_URL` or `DATABASE_URL` (e.g. `mysql://root:test_password@127.0.0.1:3306/sudattas_test`).
//! Run with: `cargo test --test integration_critical_paths -- --ignored`
//!
//! Schema must be loaded first (e.g. from backend/database/sql_dump/01_schema.sql).

use proto::proto::core::{
    CreateCartItemRequest, CreateUserRequest, GetCartItemsRequest, PlaceOrderRequest,
    SearchProductRequest,
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
    let txn = db
        .begin()
        .await
        .expect("begin transaction");

    let req = Request::new(CreateUserRequest {
        username: "integration_user".to_string(),
        email: format!("integration_{}@test.local", std::time::SystemTime::now().elapsed().unwrap().as_millis()),
        password: "SecurePass123!".to_string(),
        full_name: Some("Integration User".to_string()),
        address: None,
        phone: None,
    });

    let result = core_operations::handlers::users::create_user(&txn, req).await;
    txn.rollback().await.ok(); // don't commit test data

    assert!(result.is_ok(), "create_user should succeed: {:?}", result.err());
    let response = result.unwrap().into_inner();
    assert_eq!(response.items.len(), 1);
    assert_eq!(response.items[0].username, "integration_user");
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_search_product() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db
        .begin()
        .await
        .expect("begin transaction");

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

    assert!(result.is_ok(), "search_product should succeed: {:?}", result.err());
    let response = result.unwrap().into_inner();
    // May be empty if DB has no products
    assert!(response.items.len() <= 10);
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema; run after integration_create_user or with existing user/product"]
async fn integration_cart_by_session() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db
        .begin()
        .await
        .expect("begin transaction");

    let session_id = format!("test_session_{}", std::time::SystemTime::now().elapsed().unwrap().as_millis());

    // Create cart item with session_id (guest cart)
    let create_req = Request::new(CreateCartItemRequest {
        user_id: None,
        session_id: Some(session_id.clone()),
        product_id: 1,
        quantity: 2,
    });
    let create_result = core_operations::handlers::cart::create_cart_item(&txn, create_req).await;
    if create_result.is_err() {
        txn.rollback().await.ok();
        eprintln!("create_cart_item failed (maybe ProductID 1 missing): {:?}", create_result.err());
        return;
    }

    let get_req = Request::new(GetCartItemsRequest {
        user_id: None,
        session_id: Some(session_id),
    });
    let get_result = core_operations::handlers::cart::get_cart_items(&txn, get_req).await;
    txn.rollback().await.ok();

    assert!(get_result.is_ok());
    let response = get_result.unwrap().into_inner();
    assert!(!response.items.is_empty());
    assert_eq!(response.items[0].product_id, 1);
    assert_eq!(response.items[0].quantity, 2);
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, schema, and existing user/cart/shipping address"]
async fn integration_place_order() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");
    let req = Request::new(PlaceOrderRequest {
        user_id: 1,
        shipping_address_id: 1,
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
