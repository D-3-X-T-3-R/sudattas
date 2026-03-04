//! Unit tests for cart handlers (create and delete) using SeaORM MockDatabase.

use core_db_entities::entity::cart;
use proto::proto::core::{
    CartItemsResponse, CreateCartItemRequest, DeleteCartItemRequest, GetCartItemsRequest,
    UpdateCartItemRequest,
};
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, TransactionTrait};
use tonic::Request;

#[tokio::test]
async fn create_cart_item_requires_user_or_session() {
    use core_operations::handlers::cart::create_cart_item;

    let db = MockDatabase::new(DatabaseBackend::MySql).into_connection();
    let txn = db.begin().await.expect("begin transaction");

    let req = Request::new(CreateCartItemRequest {
        user_id: None,
        session_id: None,
        variant_id: 10,
        quantity: 2,
    });

    let result = create_cart_item(&txn, req).await;
    assert!(result.is_err(), "expected invalid_argument when both user_id and session_id are missing");
    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn create_cart_item_with_user_id_succeeds() {
    use core_operations::handlers::cart::create_cart_item;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![cart::Model {
            cart_id: 1,
            user_id: Some(42),
            session_id: None,
            variant_id: 10,
            quantity: 3,
            created_at: None,
            updated_at: None,
            abandoned_email_sent_at: None,
        }]])
        .into_connection();
    let txn = db.begin().await.expect("begin transaction");

    let req = Request::new(CreateCartItemRequest {
        user_id: Some(42),
        session_id: None,
        variant_id: 10,
        quantity: 3,
    });

    let result = create_cart_item(&txn, req).await;
    assert!(result.is_ok(), "create_cart_item should succeed for valid user_id");
    let resp = result.unwrap().into_inner();
    assert_eq!(resp.items.len(), 1);
    let item = &resp.items[0];
    assert_eq!(item.cart_id, 1);
    assert_eq!(item.variant_id, 10);
    assert_eq!(item.quantity, 3);
    // Handler maps None user_id to 0, so ensure the stored Some(42) comes back.
    assert_eq!(item.user_id, 42);
}

#[tokio::test]
async fn create_cart_item_with_session_only_succeeds_and_sets_user_id_zero() {
    use core_operations::handlers::cart::create_cart_item;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 2,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![cart::Model {
            cart_id: 2,
            user_id: None,
            session_id: Some("sess-123".to_string()),
            variant_id: 11,
            quantity: 1,
            created_at: None,
            updated_at: None,
            abandoned_email_sent_at: None,
        }]])
        .into_connection();
    let txn = db.begin().await.expect("begin transaction");

    let req = Request::new(CreateCartItemRequest {
        user_id: None,
        session_id: Some("sess-123".to_string()),
        variant_id: 11,
        quantity: 1,
    });

    let result = create_cart_item(&txn, req).await;
    assert!(result.is_ok(), "create_cart_item should succeed for session-only cart");
    let resp = result.unwrap().into_inner();
    assert_eq!(resp.items.len(), 1);
    let item = &resp.items[0];
    assert_eq!(item.cart_id, 2);
    assert_eq!(item.variant_id, 11);
    assert_eq!(item.quantity, 1);
    // Handler uses unwrap_or(0) for user_id; ensure guest carts report 0.
    assert_eq!(item.user_id, 0);
}

#[tokio::test]
async fn delete_cart_item_requires_user_or_session() {
    use core_operations::handlers::cart::delete_cart_item;

    let db = MockDatabase::new(DatabaseBackend::MySql).into_connection();
    let txn = db.begin().await.expect("begin transaction");

    let req = Request::new(DeleteCartItemRequest {
        cart_id: Some(1),
        user_id: None,
        session_id: None,
    });

    let result = delete_cart_item(&txn, req).await;
    assert!(
        result.is_err(),
        "expected invalid_argument when both user_id and session_id are missing"
    );
    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn delete_cart_item_with_user_and_cart_id_deletes_and_returns_remaining_items() {
    use core_operations::handlers::cart::delete_cart_item;

    let before_models = vec![
        cart::Model {
            cart_id: 1,
            user_id: Some(42),
            session_id: None,
            variant_id: 10,
            quantity: 3,
            created_at: None,
            updated_at: None,
            abandoned_email_sent_at: None,
        },
        cart::Model {
            cart_id: 2,
            user_id: Some(42),
            session_id: None,
            variant_id: 11,
            quantity: 1,
            created_at: None,
            updated_at: None,
            abandoned_email_sent_at: None,
        },
    ];

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![before_models])
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .into_connection();
    let txn = db.begin().await.expect("begin transaction");

    let req = Request::new(DeleteCartItemRequest {
        cart_id: Some(1),
        user_id: Some(42),
        session_id: None,
    });

    let result = delete_cart_item(&txn, req).await;
    assert!(
        result.is_ok(),
        "delete_cart_item should succeed when a row is deleted"
    );
    let CartItemsResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1, "exactly one cart item should remain");
    let remaining = &items[0];
    assert_eq!(remaining.cart_id, 2);
    assert_eq!(remaining.user_id, 42);
    assert_eq!(remaining.variant_id, 11);
    assert_eq!(remaining.quantity, 1);
}

#[tokio::test]
async fn delete_cart_item_not_found_yields_not_found_status() {
    use core_operations::handlers::cart::delete_cart_item;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<cart::Model>::new()])
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 0,
            rows_affected: 0,
        }])
        .into_connection();
    let txn = db.begin().await.expect("begin transaction");

    let req = Request::new(DeleteCartItemRequest {
        cart_id: Some(999),
        user_id: Some(1),
        session_id: None,
    });

    let result = delete_cart_item(&txn, req).await;
    assert!(result.is_err());
    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn get_cart_items_requires_user_or_session() {
    use core_operations::handlers::cart::get_cart_items;

    let db = MockDatabase::new(DatabaseBackend::MySql).into_connection();
    let txn = db.begin().await.expect("begin transaction");

    let req = Request::new(GetCartItemsRequest {
        user_id: None,
        session_id: None,
    });

    let result = get_cart_items(&txn, req).await;
    assert!(
        result.is_err(),
        "expected invalid_argument when both user_id and session_id are missing"
    );
    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn get_cart_items_filters_by_user_id() {
    use core_operations::handlers::cart::get_cart_items;

    let models = vec![
        cart::Model {
            cart_id: 1,
            user_id: Some(100),
            session_id: None,
            variant_id: 10,
            quantity: 2,
            created_at: None,
            updated_at: None,
            abandoned_email_sent_at: None,
        },
        cart::Model {
            cart_id: 2,
            user_id: Some(100),
            session_id: None,
            variant_id: 11,
            quantity: 1,
            created_at: None,
            updated_at: None,
            abandoned_email_sent_at: None,
        },
    ];

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![models])
        .into_connection();
    let txn = db.begin().await.expect("begin transaction");

    let req = Request::new(GetCartItemsRequest {
        user_id: Some(100),
        session_id: None,
    });

    let result = get_cart_items(&txn, req).await;
    assert!(result.is_ok());
    let CartItemsResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].user_id, 100);
    assert_eq!(items[1].user_id, 100);
}

#[tokio::test]
async fn get_cart_items_filters_by_session_id_and_maps_guest_user_to_zero() {
    use core_operations::handlers::cart::get_cart_items;

    let models = vec![
        cart::Model {
            cart_id: 3,
            user_id: None,
            session_id: Some("sess-abc".to_string()),
            variant_id: 20,
            quantity: 5,
            created_at: None,
            updated_at: None,
            abandoned_email_sent_at: None,
        },
        cart::Model {
            cart_id: 4,
            user_id: None,
            session_id: Some("sess-abc".to_string()),
            variant_id: 21,
            quantity: 1,
            created_at: None,
            updated_at: None,
            abandoned_email_sent_at: None,
        },
    ];

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![models])
        .into_connection();
    let txn = db.begin().await.expect("begin transaction");

    let req = Request::new(GetCartItemsRequest {
        user_id: None,
        session_id: Some("sess-abc".to_string()),
    });

    let result = get_cart_items(&txn, req).await;
    assert!(result.is_ok());
    let CartItemsResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].user_id, 0);
    assert_eq!(items[1].user_id, 0);
    assert_eq!(items[0].cart_id, 3);
    assert_eq!(items[1].cart_id, 4);
}

#[tokio::test]
async fn update_cart_item_requires_user_or_session() {
    use core_operations::handlers::cart::update_cart_item;

    let db = MockDatabase::new(DatabaseBackend::MySql).into_connection();
    let txn = db.begin().await.expect("begin transaction");

    let req = Request::new(UpdateCartItemRequest {
        cart_id: 1,
        user_id: None,
        session_id: None,
        variant_id: 10,
        quantity: 5,
    });

    let result = update_cart_item(&txn, req).await;
    assert!(
        result.is_err(),
        "expected invalid_argument when both user_id and session_id are missing"
    );
    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn update_cart_item_not_found_yields_not_found_status() {
    use core_operations::handlers::cart::update_cart_item;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<cart::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin transaction");

    let req = Request::new(UpdateCartItemRequest {
        cart_id: 99,
        user_id: Some(1),
        session_id: None,
        variant_id: 10,
        quantity: 1,
    });

    let result = update_cart_item(&txn, req).await;
    assert!(result.is_err());
    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn update_cart_item_updates_variant_and_quantity_for_user() {
    use core_operations::handlers::cart::update_cart_item;

    let existing = cart::Model {
        cart_id: 5,
        user_id: Some(7),
        session_id: None,
        variant_id: 10,
        quantity: 2,
        created_at: None,
        updated_at: None,
        abandoned_email_sent_at: None,
    };

    let updated = cart::Model {
        cart_id: 5,
        user_id: Some(7),
        session_id: None,
        variant_id: 99,
        quantity: 10,
        created_at: None,
        updated_at: None,
        abandoned_email_sent_at: None,
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![existing]])
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![updated]])
        .into_connection();
    let txn = db.begin().await.expect("begin transaction");

    let req = Request::new(UpdateCartItemRequest {
        cart_id: 5,
        user_id: Some(7),
        session_id: None,
        variant_id: 99,
        quantity: 10,
    });

    let result = update_cart_item(&txn, req).await;
    assert!(result.is_ok());
    let CartItemsResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    let item = &items[0];
    assert_eq!(item.cart_id, 5);
    assert_eq!(item.user_id, 7);
    assert_eq!(item.variant_id, 99);
    assert_eq!(item.quantity, 10);
}

