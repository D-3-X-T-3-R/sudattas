//! gRPC handler tests for cart and orders using SeaORM MockDatabase.
//!
//! Covers: create_cart_item (validation, session, user), get_cart_items,
//! update_cart_item, delete_cart_item, search_product, create_user (success,
//! password validation, optional fields).

use core_db_entities::entity::{cart, products, users};
use proto::proto::core::{
    CreateCartItemRequest, CreateUserRequest, DeleteCartItemRequest, GetCartItemsRequest,
    SearchProductRequest, UpdateCartItemRequest,
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
    assert!(result.is_err(), "must provide user_id or session_id");
    let err = result.unwrap_err();
    assert_eq!(err.code(), tonic::Code::InvalidArgument);
    assert!(
        err.message().to_lowercase().contains("user_id")
            || err.message().to_lowercase().contains("session")
    );
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
    assert_eq!(res.items[0].cart_id, 1);
    assert_eq!(res.items[0].product_id, 5);
    assert_eq!(res.items[0].quantity, 1);
}

#[tokio::test]
async fn test_create_cart_item_with_user_id() {
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![cart::Model {
            cart_id: 1,
            user_id: Some(100),
            session_id: None,
            product_id: 3,
            quantity: 2,
            created_at: None,
            updated_at: None,
        }]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(CreateCartItemRequest {
        user_id: Some(100),
        session_id: None,
        product_id: 3,
        quantity: 2,
    });
    let result = core_operations::handlers::cart::create_cart_item(&txn, req).await;
    assert!(result.is_ok());
    let res = result.unwrap().into_inner();
    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].user_id, 100);
    assert_eq!(res.items[0].product_id, 3);
    assert_eq!(res.items[0].quantity, 2);
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
async fn test_get_cart_items_returns_items_for_session() {
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![
            cart::Model {
                cart_id: 1,
                user_id: None,
                session_id: Some("sess_xyz".to_string()),
                product_id: 10,
                quantity: 1,
                created_at: None,
                updated_at: None,
            },
            cart::Model {
                cart_id: 2,
                user_id: None,
                session_id: Some("sess_xyz".to_string()),
                product_id: 20,
                quantity: 3,
                created_at: None,
                updated_at: None,
            },
        ]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(GetCartItemsRequest {
        user_id: None,
        session_id: Some("sess_xyz".to_string()),
    });
    let result = core_operations::handlers::cart::get_cart_items(&txn, req).await;
    assert!(result.is_ok());
    let res = result.unwrap().into_inner();
    assert_eq!(res.items.len(), 2);
    assert_eq!(res.items[0].product_id, 10);
    assert_eq!(res.items[0].quantity, 1);
    assert_eq!(res.items[1].product_id, 20);
    assert_eq!(res.items[1].quantity, 3);
}

#[tokio::test]
async fn test_update_cart_item_requires_user_or_session() {
    let db = MockDatabase::new(DatabaseBackend::MySql).into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(UpdateCartItemRequest {
        cart_id: 1,
        user_id: None,
        session_id: None,
        product_id: 1,
        quantity: 1,
    });
    let result = core_operations::handlers::cart::update_cart_item(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn test_update_cart_item_success() {
    // update_cart_item: find (query) then update (exec)
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![cart::Model {
            cart_id: 1,
            user_id: Some(1),
            session_id: None,
            product_id: 5,
            quantity: 1,
            created_at: None,
            updated_at: None,
        }]])
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![cart::Model {
            cart_id: 1,
            user_id: Some(1),
            session_id: None,
            product_id: 10,
            quantity: 5,
            created_at: None,
            updated_at: None,
        }]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(UpdateCartItemRequest {
        cart_id: 1,
        user_id: Some(1),
        session_id: None,
        product_id: 10,
        quantity: 5,
    });
    let result = core_operations::handlers::cart::update_cart_item(&txn, req).await;
    assert!(result.is_ok());
    let res = result.unwrap().into_inner();
    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].product_id, 10);
    assert_eq!(res.items[0].quantity, 5);
}

#[tokio::test]
async fn test_delete_cart_item_requires_user_or_session() {
    let db = MockDatabase::new(DatabaseBackend::MySql).into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(DeleteCartItemRequest {
        user_id: None,
        cart_id: None,
        session_id: None,
    });
    let result = core_operations::handlers::cart::delete_cart_item(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::InvalidArgument);
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
async fn test_search_product_with_limit_offset() {
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<products::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(SearchProductRequest {
        name: None,
        description: None,
        starting_price: None,
        ending_price: None,
        stock_quantity: None,
        category_id: None,
        product_id: None,
        limit: Some(5),
        offset: Some(10),
    });
    let result = core_operations::handlers::products::search_product(&txn, req).await;
    assert!(result.is_ok());
    let res = result.unwrap().into_inner();
    assert!(res.items.is_empty());
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
            user_status_id: None,
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
    assert_eq!(res.items[0].email, "u@test.local");
    assert_eq!(res.items[0].full_name.as_deref(), Some("New User"));
}

#[tokio::test]
async fn test_create_user_with_optional_fields() {
    use chrono::Utc;
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 2,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![users::Model {
            user_id: 2,
            username: "fulluser".to_string(),
            password: "".to_string(),
            password_hash: Some("$argon2id$v=19$...".to_string()),
            email: "full@test.local".to_string(),
            email_verified: None,
            email_verified_at: None,
            full_name: Some("Full User".to_string()),
            address: Some("123 Main St".to_string()),
            phone: Some("+1234567890".to_string()),
            user_status_id: None,
            last_login_at: None,
            create_date: Utc::now(),
            updated_at: None,
        }]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(CreateUserRequest {
        username: "fulluser".to_string(),
        email: "full@test.local".to_string(),
        password: "SecurePass99!".to_string(),
        full_name: Some("Full User".to_string()),
        address: Some("123 Main St".to_string()),
        phone: Some("+1234567890".to_string()),
    });
    let result = core_operations::handlers::users::create_user(&txn, req).await;
    assert!(result.is_ok());
    let res = result.unwrap().into_inner();
    assert_eq!(res.items[0].user_id, 2);
    assert_eq!(res.items[0].address.as_deref(), Some("123 Main St"));
    assert_eq!(res.items[0].phone.as_deref(), Some("+1234567890"));
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
    let code = result.unwrap_err().code();
    assert!(
        code == tonic::Code::InvalidArgument || code == tonic::Code::Internal,
        "weak password should be rejected (InvalidArgument or Internal from auth)"
    );
}

#[tokio::test]
async fn test_create_user_password_boundary_seven_chars_fails() {
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![] as Vec<Vec<users::Model>>)
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(CreateUserRequest {
        username: "u".to_string(),
        email: "u@test.local".to_string(),
        password: "seven77".to_string(),
        full_name: None,
        address: None,
        phone: None,
    });
    let result = core_operations::handlers::users::create_user(&txn, req).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_user_password_eight_chars_succeeds() {
    use chrono::Utc;
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![users::Model {
            user_id: 1,
            username: "eight".to_string(),
            password: "".to_string(),
            password_hash: Some("$argon2id$v=19$...".to_string()),
            email: "eight@test.local".to_string(),
            email_verified: None,
            email_verified_at: None,
            full_name: None,
            address: None,
            phone: None,
            user_status_id: None,
            last_login_at: None,
            create_date: Utc::now(),
            updated_at: None,
        }]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(CreateUserRequest {
        username: "eight".to_string(),
        email: "eight@test.local".to_string(),
        password: "eight888".to_string(),
        full_name: None,
        address: None,
        phone: None,
    });
    let result = core_operations::handlers::users::create_user(&txn, req).await;
    assert!(result.is_ok(), "password of length 8 is accepted");
}
