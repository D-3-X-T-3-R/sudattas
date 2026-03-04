//! Integration tests for reviews: create, search, update, delete, admin_update_review_status.
//!
//! **Setup**
//! - Set `TEST_DATABASE_URL` or `DATABASE_URL`.
//! - Schema must be loaded (e.g. migrations or `backend/database/sql_dump/01_schema.sql`).
//!
//! **Run**
//! - `cargo test --test integration_reviews -- --ignored`

mod integration_common;

use chrono::Utc;
use core_db_entities::entity::product_categories;
use integration_common::test_db_url;
use proto::proto::core::{
    AdminUpdateReviewStatusRequest, CreateProductRequest, CreateReviewRequest, CreateUserRequest,
    DeleteReviewRequest, SearchReviewRequest, UpdateReviewRequest,
};
use sea_orm::{ActiveModelTrait, ActiveValue, Database, TransactionTrait};
use tonic::Request;

/// Create user + category + product; return (user_id, product_id).
async fn reviews_test_setup(txn: &sea_orm::DatabaseTransaction, now_tag: i64) -> (i64, i64) {
    let role = core_db_entities::entity::user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(format!("itest_rv_{}", now_tag)),
    }
    .insert(txn)
    .await
    .expect("insert UserRoles");

    let user_res = core_operations::handlers::users::create_user(
        txn,
        Request::new(CreateUserRequest {
            username: format!("itest_rv_{}", now_tag),
            email: format!("itest_rv+{}@example.com", now_tag),
            full_name: None,
            address: None,
            phone: None,
            password_plain: "StrongPass123!".to_string(),
            role_id: Some(role.role_id),
        }),
    )
    .await
    .expect("create_user");
    let user_id = user_res.into_inner().items[0].user_id;

    let cat = product_categories::ActiveModel {
        category_id: ActiveValue::NotSet,
        name: ActiveValue::Set(format!("itest_cat_rv_{}", now_tag)),
    }
    .insert(txn)
    .await
    .expect("insert ProductCategories");

    let create_res = core_operations::handlers::products::create_product(
        txn,
        Request::new(CreateProductRequest {
            name: format!("Review Product {}", now_tag),
            description: None,
            price_paise: 2_000,
            category_id: cat.category_id,
        }),
    )
    .await
    .expect("create_product");
    let product_id = create_res.into_inner().items[0].product_id;

    (user_id, product_id)
}

/// RV1 – create_review + search_review by product_id returns the review.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_create_review_search_by_product_id_returns_review() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let (user_id, product_id) = reviews_test_setup(&txn, now_tag).await;

    let create_res = core_operations::handlers::reviews::create_review(
        &txn,
        Request::new(CreateReviewRequest {
            product_id,
            user_id,
            rating: 4,
            comment: "Great product.".to_string(),
        }),
    )
    .await
    .expect("create_review should succeed");
    let created = create_res.into_inner().items[0].clone();
    assert_eq!(created.product_id, product_id);
    assert_eq!(created.user_id, user_id);
    assert_eq!(created.rating, 4);
    assert_eq!(created.comment, "Great product.");

    let search_res = core_operations::handlers::reviews::search_review(
        &txn,
        Request::new(SearchReviewRequest {
            review_id: 0,
            product_id: Some(product_id),
            user_id: None,
            limit: Some(10),
            offset: None,
            status_filter: None,
        }),
    )
    .await
    .expect("search_review should succeed");
    let items = search_res.into_inner().items;
    assert!(!items.is_empty());
    let found = items
        .iter()
        .find(|r| r.review_id == created.review_id)
        .expect("review in results");
    assert_eq!(found.product_id, product_id);
    assert_eq!(found.rating, 4);
    assert_eq!(found.comment, "Great product.");

    txn.rollback().await.ok();
}

/// RV2 – update_review modifies rating/body; search_review reflects the updated content.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_update_review_search_reflects_updated_content() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let (user_id, product_id) = reviews_test_setup(&txn, now_tag).await;

    let create_res = core_operations::handlers::reviews::create_review(
        &txn,
        Request::new(CreateReviewRequest {
            product_id,
            user_id,
            rating: 3,
            comment: "Original comment.".to_string(),
        }),
    )
    .await
    .expect("create_review should succeed");
    let review_id = create_res.into_inner().items[0].review_id;

    let _ = core_operations::handlers::reviews::update_review(
        &txn,
        Request::new(UpdateReviewRequest {
            review_id,
            product_id: None,
            user_id: None,
            rating: Some(5),
            comment: Some("Updated: excellent!".to_string()),
        }),
    )
    .await
    .expect("update_review should succeed");

    let search_res = core_operations::handlers::reviews::search_review(
        &txn,
        Request::new(SearchReviewRequest {
            review_id: 0,
            product_id: Some(product_id),
            user_id: None,
            limit: Some(10),
            offset: None,
            status_filter: None,
        }),
    )
    .await
    .expect("search_review should succeed");
    let items = search_res.into_inner().items;
    let found = items
        .iter()
        .find(|r| r.review_id == review_id)
        .expect("review in results");
    assert_eq!(found.rating, 5);
    assert_eq!(found.comment, "Updated: excellent!");

    txn.rollback().await.ok();
}

/// RV3 – delete_review removes review so search_review no longer returns it.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_delete_review_search_no_longer_returns_it() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let (user_id, product_id) = reviews_test_setup(&txn, now_tag).await;

    let create_res = core_operations::handlers::reviews::create_review(
        &txn,
        Request::new(CreateReviewRequest {
            product_id,
            user_id,
            rating: 2,
            comment: "Will be deleted.".to_string(),
        }),
    )
    .await
    .expect("create_review should succeed");
    let review_id = create_res.into_inner().items[0].review_id;

    let _ = core_operations::handlers::reviews::delete_review(
        &txn,
        Request::new(DeleteReviewRequest { review_id }),
    )
    .await
    .expect("delete_review should succeed");

    let search_res = core_operations::handlers::reviews::search_review(
        &txn,
        Request::new(SearchReviewRequest {
            review_id: 0,
            product_id: Some(product_id),
            user_id: None,
            limit: Some(10),
            offset: None,
            status_filter: None,
        }),
    )
    .await
    .expect("search_review should succeed");
    let items = search_res.into_inner().items;
    assert!(
        !items.iter().any(|r| r.review_id == review_id),
        "search_review should not return deleted review"
    );

    txn.rollback().await.ok();
}

/// RV4 – admin_update_review_status flips status (e.g. pending → approved) and persists correctly.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_admin_update_review_status_persists() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let (user_id, product_id) = reviews_test_setup(&txn, now_tag).await;

    let create_res = core_operations::handlers::reviews::create_review(
        &txn,
        Request::new(CreateReviewRequest {
            product_id,
            user_id,
            rating: 5,
            comment: "Moderation test.".to_string(),
        }),
    )
    .await
    .expect("create_review should succeed");
    let review_id = create_res.into_inner().items[0].review_id;

    let admin_res = core_operations::handlers::reviews::admin_update_review_status(
        &txn,
        Request::new(AdminUpdateReviewStatusRequest {
            review_id,
            status: "approved".to_string(),
        }),
    )
    .await
    .expect("admin_update_review_status should succeed");
    assert!(admin_res.into_inner().success);

    let search_res = core_operations::handlers::reviews::search_review(
        &txn,
        Request::new(SearchReviewRequest {
            review_id: 0,
            product_id: Some(product_id),
            user_id: None,
            limit: Some(10),
            offset: None,
            status_filter: Some("approved".to_string()),
        }),
    )
    .await
    .expect("search_review should succeed");
    let items = search_res.into_inner().items;
    let found = items.iter().find(|r| r.review_id == review_id);
    assert!(
        found.is_some(),
        "search with status_filter=approved should return the review after admin approval"
    );

    txn.rollback().await.ok();
}
