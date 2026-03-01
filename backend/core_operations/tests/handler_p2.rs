//! P2 Unit and integration tests: abandoned cart, reviews moderation, related products, sitemap.

mod integration_common;

use core_db_entities::entity::{cart, product_related, products, reviews};
use proto::proto::core::{
    AdminUpdateReviewStatusRequest, GetRelatedProductsRequest, GetSitemapProductUrlsRequest,
    SearchReviewRequest,
};
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, TransactionTrait};
use tonic::Request;

// ---------- Abandoned cart ----------

#[tokio::test]
async fn enqueue_abandoned_cart_events_no_stale_carts_returns_zero() {
    use core_operations::procedures::abandoned_cart::enqueue_abandoned_cart_events;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<cart::Model>::new()])
        .into_connection();
    let result = enqueue_abandoned_cart_events(&db, 24).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
}

// ---------- Reviews moderation ----------

#[tokio::test]
async fn admin_update_review_status_invalid_status_returns_invalid_argument() {
    use core_operations::handlers::reviews::admin_update_review_status;

    let db = MockDatabase::new(DatabaseBackend::MySql).into_connection();
    let txn = db.begin().await.expect("begin");
    let req = Request::new(AdminUpdateReviewStatusRequest {
        review_id: 1,
        status: "invalid".to_string(),
    });
    let result = admin_update_review_status(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn admin_update_review_status_approved_success() {
    use core_operations::handlers::reviews::admin_update_review_status;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .into_connection();
    let txn = db.begin().await.expect("begin");
    let req = Request::new(AdminUpdateReviewStatusRequest {
        review_id: 1,
        status: "approved".to_string(),
    });
    let result = admin_update_review_status(&txn, req).await;
    assert!(result.is_ok());
    assert!(result.unwrap().into_inner().success);
}

#[tokio::test]
async fn search_review_with_status_filter_returns_ok() {
    use core_operations::handlers::reviews::search_review;

    let empty: Vec<reviews::Model> = vec![];
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![empty])
        .into_connection();
    let txn = db.begin().await.expect("begin");
    let req = Request::new(SearchReviewRequest {
        review_id: 0,
        product_id: None,
        user_id: None,
        status_filter: Some("approved".to_string()),
        limit: Some(10),
        offset: None,
    });
    let result = search_review(&txn, req).await;
    assert!(result.is_ok());
    assert!(result.unwrap().into_inner().items.is_empty());
}

// ---------- Related products ----------

#[tokio::test]
async fn get_related_products_empty_returns_empty_items() {
    use core_operations::handlers::products::get_related_products;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<product_related::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");
    let req = Request::new(GetRelatedProductsRequest {
        product_id: 1,
        limit: None,
    });
    let result = get_related_products(&txn, req).await;
    assert!(result.is_ok());
    assert!(result.unwrap().into_inner().items.is_empty());
}

#[tokio::test]
async fn get_related_products_returns_products_in_display_order() {
    use core_operations::handlers::products::get_related_products;

    let related_row = product_related::Model {
        id: 1,
        product_id: 100,
        related_product_id: 200,
        display_order: 0,
    };
    let product_row = products::Model {
        product_id: 200,
        sku: Some("SKU200".to_string()),
        name: "Related Saree".to_string(),
        slug: Some("related-saree".to_string()),
        description: None,
        price: rust_decimal::Decimal::try_new(1999, 2).unwrap(),
        price_paise: None,
        stock_quantity: Some(5),
        category_id: Some(1),
        fabric: None,
        weave: None,
        occasion: None,
        length_meters: None,
        has_blouse_piece: None,
        care_instructions: None,
        product_status_id: None,
        created_at: None,
        updated_at: None,
    };
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![related_row]])
        .append_query_results(vec![vec![product_row]])
        .into_connection();
    let txn = db.begin().await.expect("begin");
    let req = Request::new(GetRelatedProductsRequest {
        product_id: 100,
        limit: None,
    });
    let result = get_related_products(&txn, req).await;
    assert!(result.is_ok());
    let items = result.unwrap().into_inner().items;
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].product_id, 200);
    assert_eq!(items[0].name, "Related Saree");
}

// ---------- Sitemap ----------

#[tokio::test]
async fn get_sitemap_product_urls_empty_returns_empty_entries() {
    use core_operations::handlers::products::get_sitemap_product_urls;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<products::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");
    let req = Request::new(GetSitemapProductUrlsRequest { limit: None });
    let result = get_sitemap_product_urls(&txn, req).await;
    assert!(result.is_ok());
    assert!(result.unwrap().into_inner().entries.is_empty());
}

#[tokio::test]
async fn get_sitemap_product_urls_returns_slug_and_lastmod() {
    use core_operations::handlers::products::get_sitemap_product_urls;

    let now = chrono::Utc::now();
    let product_row = products::Model {
        product_id: 1,
        sku: None,
        name: "Saree".to_string(),
        slug: Some("saree-one".to_string()),
        description: None,
        price: rust_decimal::Decimal::try_new(999, 2).unwrap(),
        price_paise: None,
        stock_quantity: None,
        category_id: None,
        fabric: None,
        weave: None,
        occasion: None,
        length_meters: None,
        has_blouse_piece: None,
        care_instructions: None,
        product_status_id: None,
        created_at: Some(now),
        updated_at: Some(now),
    };
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![product_row]])
        .into_connection();
    let txn = db.begin().await.expect("begin");
    let req = Request::new(GetSitemapProductUrlsRequest { limit: None });
    let result = get_sitemap_product_urls(&txn, req).await;
    assert!(result.is_ok());
    let entries = result.unwrap().into_inner().entries;
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].slug, "saree-one");
    assert!(!entries[0].lastmod.is_empty());
}

// ---------- Integration (requires TEST_DATABASE_URL) ----------

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_enqueue_abandoned_cart_events() {
    use core_operations::procedures::abandoned_cart::enqueue_abandoned_cart_events;
    use sea_orm::Database;

    let db = Database::connect(&integration_common::test_db_url())
        .await
        .expect("connect");
    let result = enqueue_abandoned_cart_events(&db, 24).await;
    assert!(result.is_ok());
    let _count = result.unwrap();
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_get_related_products() {
    use core_operations::handlers::products::get_related_products;
    use sea_orm::{Database, TransactionTrait};

    let db = Database::connect(&integration_common::test_db_url())
        .await
        .expect("connect");
    let txn = db.begin().await.expect("begin");

    let req = Request::new(GetRelatedProductsRequest {
        product_id: 1,
        limit: Some(10),
    });
    let result = get_related_products(&txn, req).await;
    assert!(result.is_ok());
    let items = result.unwrap().into_inner().items;
    // May be empty if product_related has no rows for product_id 1
    let _ = items;
}
