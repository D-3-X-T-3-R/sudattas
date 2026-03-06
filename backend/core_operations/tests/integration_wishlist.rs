//! Integration tests for wishlist: add_wishlist_item, search_wishlist_item, delete_wishlist_item.
//!
//! **Setup**
//! - Set `TEST_DATABASE_URL` or `DATABASE_URL`.
//! - Schema must be loaded (e.g. migrations or `backend/database/sql_dump/01_schema.sql`).
//!
//! **Run**
//! - `cargo test --test integration_wishlist -- --ignored`

mod integration_common;

use chrono::Utc;
use core_db_entities::entity::product_categories;
use integration_common::test_db_url;
use proto::proto::core::{
    AddWishlistItemRequest, CreateProductRequest, CreateUserRequest, DeleteWishlistItemRequest,
    SearchWishlistItemRequest,
};
use sea_orm::{ActiveModelTrait, ActiveValue, Database, TransactionTrait};
use tonic::Request;

/// Create user + category + product; return (user_id, product_id).
async fn wishlist_test_setup(txn: &sea_orm::DatabaseTransaction, now_tag: i64) -> (i64, i64) {
    let role = core_db_entities::entity::user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(format!("itest_wl_{}", now_tag)),
    }
    .insert(txn)
    .await
    .expect("insert UserRoles");

    let user_res = core_operations::handlers::users::create_user(
        txn,
        Request::new(CreateUserRequest {
            username: format!("itest_wl_{}", now_tag),
            email: format!("itest_wl+{}@example.com", now_tag),
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
    .expect("create_user");
    let user_id = user_res.into_inner().items[0].user_id;

    let cat = product_categories::ActiveModel {
        category_id: ActiveValue::NotSet,
        name: ActiveValue::Set(format!("itest_cat_wl_{}", now_tag)),
    }
    .insert(txn)
    .await
    .expect("insert ProductCategories");

    let create_res = core_operations::handlers::products::create_product(
        txn,
        Request::new(CreateProductRequest {
            name: format!("Wishlist Product {}", now_tag),
            description: None,
            price_paise: 4_000,
            category_id: cat.category_id,
            sku: None,
            slug: None,
            fabric: None,
            weave: None,
            occasion: None,
            has_blouse_piece: None,
            care_instructions: None,
            product_status_id: None,
        }),
    )
    .await
    .expect("create_product");
    let product_id = create_res.into_inner().items[0].product_id;

    (user_id, product_id)
}

/// W1 – add_wishlist_item + search_wishlist (search_wishlist_item) returns expected items for a user.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_add_wishlist_item_search_returns_expected_items() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let (user_id, product_id) = wishlist_test_setup(&txn, now_tag).await;

    let add_res = core_operations::handlers::wishlist::add_wishlist_item(
        &txn,
        Request::new(AddWishlistItemRequest {
            user_id,
            product_id,
        }),
    )
    .await
    .expect("add_wishlist_item should succeed");
    let added = add_res.into_inner().items[0].clone();
    assert_eq!(added.user_id, user_id);
    assert_eq!(added.product_id, product_id);
    assert!(added.wishlist_id > 0);

    let search_res = core_operations::handlers::wishlist::search_wishlist_item(
        &txn,
        Request::new(SearchWishlistItemRequest {
            wishlist_id: None,
            user_id,
            product_id: None,
        }),
    )
    .await
    .expect("search_wishlist_item should succeed");
    let items = search_res.into_inner().items;
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].wishlist_id, added.wishlist_id);
    assert_eq!(items[0].user_id, user_id);
    assert_eq!(items[0].product_id, product_id);

    txn.rollback().await.ok();
}

/// W2 – delete_wishlist_item removes item so subsequent search_wishlist is empty.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_delete_wishlist_item_search_empty() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let (user_id, product_id) = wishlist_test_setup(&txn, now_tag).await;

    let add_res = core_operations::handlers::wishlist::add_wishlist_item(
        &txn,
        Request::new(AddWishlistItemRequest {
            user_id,
            product_id,
        }),
    )
    .await
    .expect("add_wishlist_item should succeed");
    let wishlist_id = add_res.into_inner().items[0].wishlist_id;

    let _ = core_operations::handlers::wishlist::delete_wishlist_item(
        &txn,
        Request::new(DeleteWishlistItemRequest {
            wishlist_id,
            user_id,
        }),
    )
    .await
    .expect("delete_wishlist_item should succeed");

    let search_res = core_operations::handlers::wishlist::search_wishlist_item(
        &txn,
        Request::new(SearchWishlistItemRequest {
            wishlist_id: None,
            user_id,
            product_id: None,
        }),
    )
    .await
    .expect("search_wishlist_item should succeed");
    let items = search_res.into_inner().items;
    assert!(
        items.is_empty(),
        "search_wishlist after delete should return no items"
    );

    txn.rollback().await.ok();
}
