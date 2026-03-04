//! Integration tests for product catalog: create, search, get_by_id, update, delete.
//!
//! **Setup**
//! - Set `TEST_DATABASE_URL` or `DATABASE_URL`.
//! - Schema must be loaded (e.g. migrations or `backend/database/sql_dump/01_schema.sql`).
//!
//! **Run**
//! - `cargo test --test integration_products -- --ignored`

mod integration_common;

use chrono::Utc;
use core_db_entities::entity::{product_categories, product_variants, products};
use integration_common::test_db_url;
use proto::proto::core::{
    CreateProductRequest, CreateProductVariantRequest, DeleteProductRequest, GetProductsByIdRequest,
    SearchProductRequest, UpdateProductRequest,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Database, EntityTrait, QueryFilter, TransactionTrait,
};
use tonic::Request;

/// PR1 – create_product + search_product by name and category returns the created product.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_create_product_search_by_name_and_category_returns_product() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let cat = product_categories::ActiveModel {
        category_id: ActiveValue::NotSet,
        name: ActiveValue::Set(format!("itest_cat_pr1_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductCategories");

    let product_name = format!("Saree PR1 {}", now_tag);
    let create_res = core_operations::handlers::products::create_product(
        &txn,
        Request::new(CreateProductRequest {
            name: product_name.clone(),
            description: Some("Integration test product".to_string()),
            price_paise: 7_500,
            category_id: cat.category_id,
        }),
    )
    .await
    .expect("create_product should succeed");
    let created = create_res.into_inner().items[0].clone();
    assert_eq!(created.name, product_name);
    assert_eq!(created.category_id, cat.category_id);
    assert_eq!(created.price_paise, 7_500);

    let search_res = core_operations::handlers::products::search_product(
        &txn,
        Request::new(SearchProductRequest {
            name: Some(product_name.clone()),
            description: None,
            starting_price_paise: None,
            category_id: Some(cat.category_id),
            ending_price_paise: None,
            product_id: None,
            limit: Some(10),
            offset: Some(0),
        }),
    )
    .await
    .expect("search_product should succeed");
    let items = search_res.into_inner().items;
    assert!(!items.is_empty(), "search by name and category should return the created product");
    let found = items.iter().find(|p| p.product_id == created.product_id).expect("product in results");
    assert_eq!(found.name, product_name);
    assert_eq!(found.category_id, cat.category_id);

    txn.rollback().await.ok();
}

/// PR2 – create_product + create_product_variant + get_products_by_id returns product with associated variants.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_create_product_and_variant_get_by_id_returns_product_and_variants_exist() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let cat = product_categories::ActiveModel {
        category_id: ActiveValue::NotSet,
        name: ActiveValue::Set(format!("itest_cat_pr2_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductCategories");

    let create_res = core_operations::handlers::products::create_product(
        &txn,
        Request::new(CreateProductRequest {
            name: format!("Product PR2 {}", now_tag),
            description: None,
            price_paise: 12_000,
            category_id: cat.category_id,
        }),
    )
    .await
    .expect("create_product should succeed");
    let product_id = create_res.into_inner().items[0].product_id;

    let variant_res = core_operations::handlers::product_variants::create_product_variant(
        &txn,
        Request::new(CreateProductVariantRequest {
            product_id,
            size_id: None,
            color_id: None,
            additional_price_paise: Some(500),
        }),
    )
    .await
    .expect("create_product_variant should succeed");
    let variant_id = variant_res.into_inner().items[0].variant_id;

    let get_res = core_operations::handlers::products::get_product_by_ids::get_products_by_id(
        &txn,
        Request::new(GetProductsByIdRequest {
            product_ids: vec![product_id],
        }),
    )
    .await
    .expect("get_products_by_id should succeed");
    let items = get_res.into_inner().items;
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].product_id, product_id);
    assert_eq!(items[0].price_paise, 12_000);

    let variants = product_variants::Entity::find()
        .filter(product_variants::Column::ProductId.eq(product_id))
        .all(&txn)
        .await
        .expect("query product_variants");
    assert_eq!(variants.len(), 1);
    assert_eq!(variants[0].variant_id, variant_id);
    assert_eq!(variants[0].product_id, product_id);

    txn.rollback().await.ok();
}

/// PR3 – update_product changes price; get_products_by_id reflects the new price in responses.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_update_product_price_get_by_id_reflects_new_price() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let cat = product_categories::ActiveModel {
        category_id: ActiveValue::NotSet,
        name: ActiveValue::Set(format!("itest_cat_pr3_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductCategories");

    let create_res = core_operations::handlers::products::create_product(
        &txn,
        Request::new(CreateProductRequest {
            name: format!("Product PR3 {}", now_tag),
            description: None,
            price_paise: 9_000,
            category_id: cat.category_id,
        }),
    )
    .await
    .expect("create_product should succeed");
    let product_id = create_res.into_inner().items[0].product_id;

    let new_price = 11_000_i64;
    let _ = core_operations::handlers::products::update_product(
        &txn,
        Request::new(UpdateProductRequest {
            product_id,
            name: format!("Product PR3 {}", now_tag),
            description: None,
            price_paise: new_price,
            category_id: cat.category_id,
        }),
    )
    .await
    .expect("update_product should succeed");

    let get_res = core_operations::handlers::products::get_product_by_ids::get_products_by_id(
        &txn,
        Request::new(GetProductsByIdRequest {
            product_ids: vec![product_id],
        }),
    )
    .await
    .expect("get_products_by_id should succeed");
    let items = get_res.into_inner().items;
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].price_paise, new_price, "get_products_by_id should reflect updated price");

    txn.rollback().await.ok();
}

/// PR4 – delete_product removes it so a later get_products_by_id returns empty for that ID.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_delete_product_get_by_id_returns_empty() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let cat = product_categories::ActiveModel {
        category_id: ActiveValue::NotSet,
        name: ActiveValue::Set(format!("itest_cat_pr4_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductCategories");

    let create_res = core_operations::handlers::products::create_product(
        &txn,
        Request::new(CreateProductRequest {
            name: format!("Product PR4 {}", now_tag),
            description: None,
            price_paise: 3_000,
            category_id: cat.category_id,
        }),
    )
    .await
    .expect("create_product should succeed");
    let product_id = create_res.into_inner().items[0].product_id;

    let _ = core_operations::handlers::products::delete_product(
        &txn,
        Request::new(DeleteProductRequest { product_id }),
    )
    .await
    .expect("delete_product should succeed");

    let get_res = core_operations::handlers::products::get_product_by_ids::get_products_by_id(
        &txn,
        Request::new(GetProductsByIdRequest {
            product_ids: vec![product_id],
        }),
    )
    .await
    .expect("get_products_by_id should succeed");
    let items = get_res.into_inner().items;
    assert!(items.is_empty(), "get_products_by_id should return empty for deleted product ID");

    let db_product = products::Entity::find_by_id(product_id)
        .one(&txn)
        .await
        .expect("query product");
    assert!(db_product.is_none(), "product row should be removed from DB");

    txn.rollback().await.ok();
}
