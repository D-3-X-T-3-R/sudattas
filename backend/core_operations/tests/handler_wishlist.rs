//! Unit tests for wishlist handlers.

use core_db_entities::entity::wishlist;
use proto::proto::core::{
    AddWishlistItemRequest, DeleteWishlistItemRequest, SearchWishlistItemRequest,
    WishlistItemsResponse,
};
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, TransactionTrait};
use tonic::Request;

fn make_wishlist(id: i64) -> wishlist::Model {
    wishlist::Model {
        wishlist_id: id,
        user_id: Some(1),
        product_id: Some(10),
        date_added: chrono::Utc::now(),
    }
}

#[tokio::test]
async fn add_wishlist_item_inserts_and_returns_created_model() {
    use core_operations::handlers::wishlist::add_wishlist_item;

    let model = make_wishlist(1);
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(AddWishlistItemRequest {
        user_id: 1,
        product_id: 10,
    });
    let result = add_wishlist_item(&txn, req).await;
    assert!(result.is_ok());
    let WishlistItemsResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].wishlist_id, 1);
}

#[tokio::test]
async fn delete_wishlist_item_not_found_yields_not_found_status() {
    use core_operations::handlers::wishlist::delete_wishlist_item;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<wishlist::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(DeleteWishlistItemRequest {
        wishlist_id: 99,
        user_id: 1,
    });
    let result = delete_wishlist_item(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn search_wishlist_item_filters_by_user_and_optional_ids() {
    use core_operations::handlers::wishlist::search_wishlist_item;

    let model = make_wishlist(3);
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(SearchWishlistItemRequest {
        user_id: 1,
        wishlist_id: Some(3),
        product_id: Some(10),
    });
    let result = search_wishlist_item(&txn, req).await;
    assert!(result.is_ok());
    let WishlistItemsResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].wishlist_id, 3);
}
