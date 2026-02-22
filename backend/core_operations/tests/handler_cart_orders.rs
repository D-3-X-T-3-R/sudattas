//! gRPC handler tests for cart and orders using SeaORM MockDatabase.

use core_db_entities::entity::{cart, products, users};
use proto::proto::core::{
    CreateCartItemRequest, CreateUserRequest, GetCartItemsRequest, SearchProductRequest,
};
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, TransactionTrait};
use tonic::Request;

#[tokio::test]
async fn test_create_cart_item_requires_user_or_session() {
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![cart::Model {
            cart_id: 1,
            user_id: None,
            session_id: Some("sess_123".to_string()),
            product_id: 10,
            quantity: 2,
            created_at: None,
            updated_at: None,
        }]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(CreateCartItemRequest {
        user_id: None,
        session_id: None,
        product_id: 10,
        quantity: 2,
    });
    let result = core_operations::handlers::cart::create_cart_item(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn test_create_cart_item_with_session_id() {
    // create_cart_item: INSERT then SELECT — exec then query
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![cart::Model {
            cart_id: 1,
            user_id: None,
            session_id: Some("guest_abc".to_string()),
            product_id: 5,
            quantity: 1,
            created_at: None,
            updated_at: None,
        }]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(CreateCartItemRequest {
        user_id: None,
        session_id: Some("guest_abc".to_string()),
        product_id: 5,
        quantity: 1,
    });
    let result = core_operations::handlers::cart::create_cart_item(&txn, req).await;
    assert!(result.is_ok());
    let res = result.unwrap().into_inner();
    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].product_id, 5);
    assert_eq!(res.items[0].quantity, 1);
}

#[tokio::test]
async fn test_get_cart_items_empty() {
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<cart::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(GetCartItemsRequest {
        user_id: Some(1),
        session_id: None,
    });
    let result = core_operations::handlers::cart::get_cart_items(&txn, req).await;
    assert!(result.is_ok());
    assert!(result.unwrap().into_inner().items.is_empty());
}

#[tokio::test]
async fn test_search_product_empty() {
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<products::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(SearchProductRequest {
        name: Some("nonexistent".to_string()),
        description: None,
        starting_price: None,
        ending_price: None,
        stock_quantity: None,
        category_id: None,
        product_id: None,
        limit: None,
        offset: None,
    });
    let result = core_operations::handlers::products::search_product(&txn, req).await;
    assert!(result.is_ok());
    assert!(result.unwrap().into_inner().items.is_empty());
}

#[tokio::test]
async fn test_create_user_success() {
    // create_user: INSERT then SELECT — exec then query (after hash_password)
    use chrono::Utc;
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![users::Model {
            user_id: 1,
            username: "newuser".to_string(),
            password: "".to_string(),
            password_hash: Some("$argon2id$v=19$...".to_string()),
            email: "u@test.local".to_string(),
            email_verified: None,
            email_verified_at: None,
            full_name: Some("New User".to_string()),
            address: None,
            phone: None,
            status: None,
            last_login_at: None,
            create_date: Utc::now(),
            updated_at: None,
        }]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(CreateUserRequest {
        username: "newuser".to_string(),
        email: "u@test.local".to_string(),
        password: "ValidPass123!".to_string(),
        full_name: Some("New User".to_string()),
        address: None,
        phone: None,
    });
    let result = core_operations::handlers::users::create_user(&txn, req).await;
    assert!(result.is_ok());
    let res = result.unwrap().into_inner();
    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].username, "newuser");
}

#[tokio::test]
async fn test_create_user_weak_password() {
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![] as Vec<Vec<users::Model>>)
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(CreateUserRequest {
        username: "u".to_string(),
        email: "u@test.local".to_string(),
        password: "short".to_string(),
        full_name: None,
        address: None,
        phone: None,
    });
    let result = core_operations::handlers::users::create_user(&txn, req).await;
    assert!(result.is_err());
}
