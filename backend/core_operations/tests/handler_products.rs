//! Unit tests for product handlers using SeaORM MockDatabase.

use core_db_entities::entity::products;
use proto::proto::core::{
    CreateProductRequest, DeleteProductRequest, GetProductsByIdRequest, ProductsResponse,
    SearchProductRequest, UpdateProductRequest,
};
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, TransactionTrait};
use tonic::Request;

#[tokio::test]
async fn create_product_inserts_and_returns_created_model() {
    use core_operations::handlers::products::create_product;

    let model = products::Model {
        product_id: 1,
        sku: Some("SKU-1".to_string()),
        name: "Saree".to_string(),
        slug: Some("saree-1".to_string()),
        description: Some("A nice saree".to_string()),
        price_paise: 99_90,
        category_id: 5,
        fabric: None,
        weave: None,
        occasion: None,
        has_blouse_piece: None,
        care_instructions: None,
        product_status_id: None,
        created_at: None,
        updated_at: None,
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(CreateProductRequest {
        name: "Saree".to_string(),
        description: Some("A nice saree".to_string()),
        price_paise: 99_90,
        category_id: 5,
    });
    let result = create_product(&txn, req).await;
    assert!(result.is_ok());
    let ProductsResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    let p = &items[0];
    assert_eq!(p.product_id, 1);
    assert_eq!(p.name, "Saree");
    assert_eq!(p.category_id, 5);
    assert_eq!(p.price_paise, 99_90);
}

#[tokio::test]
async fn update_product_updates_fields_and_returns_response() {
    use core_operations::handlers::products::update_product;

    let updated = products::Model {
        product_id: 2,
        sku: Some("SKU-2".to_string()),
        name: "Updated".to_string(),
        slug: Some("updated-2".to_string()),
        description: Some("Updated desc".to_string()),
        price_paise: 12_34,
        category_id: 10,
        fabric: None,
        weave: None,
        occasion: None,
        has_blouse_piece: None,
        care_instructions: None,
        product_status_id: None,
        created_at: None,
        updated_at: None,
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![updated]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(UpdateProductRequest {
        product_id: 2,
        name: "Updated".to_string(),
        description: Some("Updated desc".to_string()),
        price_paise: 12_34,
        category_id: 10,
    });
    let result = update_product(&txn, req).await;
    assert!(result.is_ok());
    let ProductsResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    let p = &items[0];
    assert_eq!(p.product_id, 2);
    assert_eq!(p.name, "Updated");
    assert_eq!(p.price_paise, 12_34);
}

#[tokio::test]
async fn delete_product_deletes_existing_and_returns_response() {
    use core_operations::handlers::products::delete_product;

    let model = products::Model {
        product_id: 3,
        sku: Some("SKU-3".to_string()),
        name: "DeleteMe".to_string(),
        slug: Some("delete-me".to_string()),
        description: Some("To be deleted".to_string()),
        price_paise: 5_000,
        category_id: 7,
        fabric: None,
        weave: None,
        occasion: None,
        has_blouse_piece: None,
        care_instructions: None,
        product_status_id: None,
        created_at: None,
        updated_at: None,
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![model]])
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(DeleteProductRequest { product_id: 3 });
    let result = delete_product(&txn, req).await;
    assert!(result.is_ok());
    let ProductsResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    let p = &items[0];
    assert_eq!(p.product_id, 3);
    assert_eq!(p.name, "DeleteMe");
}

#[tokio::test]
async fn delete_product_not_found_yields_not_found_status() {
    use core_operations::handlers::products::delete_product;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<products::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(DeleteProductRequest { product_id: 999 });
    let result = delete_product(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn search_product_filters_by_multiple_fields_and_pagination() {
    use core_operations::handlers::products::search_product;

    let model = products::Model {
        product_id: 4,
        sku: Some("SKU-4".to_string()),
        name: "Festive Saree".to_string(),
        slug: Some("festive-saree".to_string()),
        description: Some("Red festive saree".to_string()),
        price_paise: 15_000,
        category_id: 9,
        fabric: None,
        weave: None,
        occasion: Some("festive".to_string()),
        has_blouse_piece: None,
        care_instructions: None,
        product_status_id: None,
        created_at: None,
        updated_at: None,
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(SearchProductRequest {
        product_id: Some(4),
        name: Some("Festive".to_string()),
        description: Some("Red".to_string()),
        category_id: Some(9),
        starting_price_paise: Some(10_000),
        ending_price_paise: Some(20_000),
        limit: Some(10),
        offset: Some(0),
    });

    let result = search_product(&txn, req).await;
    assert!(result.is_ok());
    let ProductsResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    let p = &items[0];
    assert_eq!(p.product_id, 4);
    assert_eq!(p.category_id, 9);
    assert_eq!(p.price_paise, 15_000);
}

#[tokio::test]
async fn search_product_filters_by_product_id_only() {
    use core_operations::handlers::products::search_product;

    let model = products::Model {
        product_id: 20,
        sku: Some("SKU-20".to_string()),
        name: "Single".to_string(),
        slug: Some("single".to_string()),
        description: Some("Only product".to_string()),
        price_paise: 2_000,
        category_id: 2,
        fabric: None,
        weave: None,
        occasion: None,
        has_blouse_piece: None,
        care_instructions: None,
        product_status_id: None,
        created_at: None,
        updated_at: None,
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(SearchProductRequest {
        product_id: Some(20),
        name: None,
        description: None,
        category_id: None,
        starting_price_paise: None,
        ending_price_paise: None,
        limit: None,
        offset: None,
    });

    let result = search_product(&txn, req).await;
    assert!(result.is_ok());
    let ProductsResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    let p = &items[0];
    assert_eq!(p.product_id, 20);
}

#[tokio::test]
async fn search_product_filters_by_name_only() {
    use core_operations::handlers::products::search_product;

    let model = products::Model {
        product_id: 21,
        sku: Some("SKU-21".to_string()),
        name: "Cotton Dress".to_string(),
        slug: Some("cotton-dress".to_string()),
        description: Some("Light cotton dress".to_string()),
        price_paise: 3_000,
        category_id: 3,
        fabric: None,
        weave: None,
        occasion: None,
        has_blouse_piece: None,
        care_instructions: None,
        product_status_id: None,
        created_at: None,
        updated_at: None,
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(SearchProductRequest {
        product_id: None,
        name: Some("Dress".to_string()),
        description: None,
        category_id: None,
        starting_price_paise: None,
        ending_price_paise: None,
        limit: None,
        offset: None,
    });

    let result = search_product(&txn, req).await;
    assert!(result.is_ok());
    let ProductsResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    assert!(items[0].name.contains("Dress"));
}

#[tokio::test]
async fn search_product_filters_by_category_only() {
    use core_operations::handlers::products::search_product;

    let model = products::Model {
        product_id: 22,
        sku: Some("SKU-22".to_string()),
        name: "CategoryOnly".to_string(),
        slug: Some("category-only".to_string()),
        description: Some("Belongs to category 4".to_string()),
        price_paise: 4_000,
        category_id: 4,
        fabric: None,
        weave: None,
        occasion: None,
        has_blouse_piece: None,
        care_instructions: None,
        product_status_id: None,
        created_at: None,
        updated_at: None,
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(SearchProductRequest {
        product_id: None,
        name: None,
        description: None,
        category_id: Some(4),
        starting_price_paise: None,
        ending_price_paise: None,
        limit: None,
        offset: None,
    });

    let result = search_product(&txn, req).await;
    assert!(result.is_ok());
    let ProductsResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].category_id, 4);
}

#[tokio::test]
async fn search_product_filters_by_price_range_only() {
    use core_operations::handlers::products::search_product;

    let model = products::Model {
        product_id: 23,
        sku: Some("SKU-23".to_string()),
        name: "PriceRange".to_string(),
        slug: Some("price-range".to_string()),
        description: Some("Within range".to_string()),
        price_paise: 7_500,
        category_id: 6,
        fabric: None,
        weave: None,
        occasion: None,
        has_blouse_piece: None,
        care_instructions: None,
        product_status_id: None,
        created_at: None,
        updated_at: None,
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(SearchProductRequest {
        product_id: None,
        name: None,
        description: None,
        category_id: None,
        starting_price_paise: Some(5_000),
        ending_price_paise: Some(10_000),
        limit: None,
        offset: None,
    });

    let result = search_product(&txn, req).await;
    assert!(result.is_ok());
    let ProductsResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].price_paise, 7_500);
}

#[tokio::test]
async fn get_products_by_id_returns_matching_products() {
    use core_operations::handlers::products::get_product_by_ids::get_products_by_id;

    let p1 = products::Model {
        product_id: 10,
        sku: Some("SKU-10".to_string()),
        name: "P1".to_string(),
        slug: Some("p1".to_string()),
        description: Some("First".to_string()),
        price_paise: 1_000,
        category_id: 1,
        fabric: None,
        weave: None,
        occasion: None,
        has_blouse_piece: None,
        care_instructions: None,
        product_status_id: None,
        created_at: None,
        updated_at: None,
    };
    let p2 = products::Model {
        product_id: 11,
        ..p1.clone()
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![p1, p2]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(GetProductsByIdRequest {
        product_ids: vec![10, 11, 999],
    });
    let result = get_products_by_id(&txn, req).await;
    assert!(result.is_ok());
    let ProductsResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 2);
    let ids: Vec<i64> = items.into_iter().map(|p| p.product_id).collect();
    assert!(ids.contains(&10));
    assert!(ids.contains(&11));
}
