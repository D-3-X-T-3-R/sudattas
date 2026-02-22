//! gRPC handler tests for user-facing flows: categories, country, state, wishlist.
//! Uses SeaORM MockDatabase; same pattern as handler_cart_orders.

use chrono::Utc;
use core_db_entities::entity::{categories, countries, states, wishlist};
use proto::proto::core::{
    AddWishlistItemRequest, CreateCategoryRequest, CreateCountryRequest, CreateStateRequest,
    SearchCategoryRequest, SearchCountryRequest, SearchStateRequest, SearchWishlistItemRequest,
};
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, TransactionTrait};
use tonic::Request;

// ---------- Categories ----------

#[tokio::test]
async fn test_create_category_success() {
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![categories::Model {
            category_id: 1,
            name: "Electronics".to_string(),
        }]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(CreateCategoryRequest {
        name: "Electronics".to_string(),
    });
    let result = core_operations::handlers::categories::create_category(&txn, req).await;
    assert!(result.is_ok());
    let res = result.unwrap().into_inner();
    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].category_id, 1);
    assert_eq!(res.items[0].name, "Electronics");
}

#[tokio::test]
async fn test_search_category_empty() {
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<categories::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(SearchCategoryRequest {
        category_id: None,
        name: None,
    });
    let result = core_operations::handlers::categories::search_category(&txn, req).await;
    assert!(result.is_ok());
    assert!(result.unwrap().into_inner().items.is_empty());
}

#[tokio::test]
async fn test_search_category_by_name() {
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![
            categories::Model {
                category_id: 1,
                name: "Books".to_string(),
            },
            categories::Model {
                category_id: 2,
                name: "E-Books".to_string(),
            },
        ]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(SearchCategoryRequest {
        category_id: None,
        name: Some("Book".to_string()),
    });
    let result = core_operations::handlers::categories::search_category(&txn, req).await;
    assert!(result.is_ok());
    let res = result.unwrap().into_inner();
    assert_eq!(res.items.len(), 2);
    assert_eq!(res.items[0].name, "Books");
    assert_eq!(res.items[1].name, "E-Books");
}

// ---------- Country ----------

#[tokio::test]
async fn test_create_country_success() {
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![countries::Model {
            country_id: 1,
            country_name: Some("India".to_string()),
        }]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(CreateCountryRequest {
        country_name: "India".to_string(),
    });
    let result = core_operations::handlers::country::create_country(&txn, req).await;
    assert!(result.is_ok());
    let res = result.unwrap().into_inner();
    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].country_id, 1);
    assert_eq!(res.items[0].country_name, "India");
}

#[tokio::test]
async fn test_search_country_empty() {
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<countries::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(SearchCountryRequest {
        country_id: None,
        country_name: None,
    });
    let result = core_operations::handlers::country::search_country(&txn, req).await;
    assert!(result.is_ok());
    assert!(result.unwrap().into_inner().items.is_empty());
}

#[tokio::test]
async fn test_search_country_by_id() {
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![countries::Model {
            country_id: 1,
            country_name: Some("India".to_string()),
        }]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(SearchCountryRequest {
        country_id: Some(1),
        country_name: None,
    });
    let result = core_operations::handlers::country::search_country(&txn, req).await;
    assert!(result.is_ok());
    let res = result.unwrap().into_inner();
    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].country_id, 1);
    assert_eq!(res.items[0].country_name, "India");
}

// ---------- State ----------

#[tokio::test]
async fn test_create_state_success() {
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![states::Model {
            state_id: 1,
            state_name: Some("Maharashtra".to_string()),
        }]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(CreateStateRequest {
        state_name: "Maharashtra".to_string(),
    });
    let result = core_operations::handlers::state::create_state(&txn, req).await;
    assert!(result.is_ok());
    let res = result.unwrap().into_inner();
    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].state_id, 1);
    assert_eq!(res.items[0].state_name, "Maharashtra");
}

#[tokio::test]
async fn test_search_state_empty() {
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<states::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(SearchStateRequest {
        state_id: None,
        state_name: None,
    });
    let result = core_operations::handlers::state::search_state(&txn, req).await;
    assert!(result.is_ok());
    assert!(result.unwrap().into_inner().items.is_empty());
}

#[tokio::test]
async fn test_search_state_by_name() {
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![states::Model {
            state_id: 1,
            state_name: Some("Karnataka".to_string()),
        }]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(SearchStateRequest {
        state_id: None,
        state_name: Some("Karna".to_string()),
    });
    let result = core_operations::handlers::state::search_state(&txn, req).await;
    assert!(result.is_ok());
    let res = result.unwrap().into_inner();
    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].state_name, "Karnataka");
}

// ---------- Wishlist ----------

#[tokio::test]
async fn test_add_wishlist_item_success() {
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![wishlist::Model {
            wishlist_id: 1,
            user_id: Some(10),
            product_id: Some(5),
            date_added: Utc::now(),
        }]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(AddWishlistItemRequest {
        user_id: 10,
        product_id: 5,
    });
    let result = core_operations::handlers::wishlist::add_wishlist_item(&txn, req).await;
    assert!(result.is_ok());
    let res = result.unwrap().into_inner();
    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].wishlist_id, 1);
    assert_eq!(res.items[0].user_id, 10);
    assert_eq!(res.items[0].product_id, 5);
}

#[tokio::test]
async fn test_search_wishlist_item_empty() {
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<wishlist::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(SearchWishlistItemRequest {
        user_id: 1,
        wishlist_id: None,
        product_id: None,
    });
    let result = core_operations::handlers::wishlist::search_wishlist_item(&txn, req).await;
    assert!(result.is_ok());
    assert!(result.unwrap().into_inner().items.is_empty());
}

#[tokio::test]
async fn test_search_wishlist_item_returns_items() {
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![
            wishlist::Model {
                wishlist_id: 1,
                user_id: Some(1),
                product_id: Some(10),
                date_added: Utc::now(),
            },
            wishlist::Model {
                wishlist_id: 2,
                user_id: Some(1),
                product_id: Some(20),
                date_added: Utc::now(),
            },
        ]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(SearchWishlistItemRequest {
        user_id: 1,
        wishlist_id: None,
        product_id: None,
    });
    let result = core_operations::handlers::wishlist::search_wishlist_item(&txn, req).await;
    assert!(result.is_ok());
    let res = result.unwrap().into_inner();
    assert_eq!(res.items.len(), 2);
    assert_eq!(res.items[0].product_id, 10);
    assert_eq!(res.items[1].product_id, 20);
}
