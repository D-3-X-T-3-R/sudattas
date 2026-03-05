//! Unit tests for product_images handlers (confirm/delete). R2 client is not exercised here.

use core_db_entities::entity::product_images;
use proto::proto::core::{
    ConfirmImageUploadRequest, DeleteProductImageRequest, ProductImagesResponse,
    SearchProductImageRequest, UpdateProductImageRequest,
};
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, TransactionTrait};
use serde_json::json;
use tonic::Request;

#[tokio::test]
async fn confirm_image_upload_inserts_row_and_maps_urls() {
    use core_operations::handlers::product_images::confirm_image_upload;

    // Simulate DB returning a row with urls map containing key "1".
    let urls = json!({ "1": "https://cdn.example.com/products/1/img.png" });
    let model = product_images::Model {
        image_id: 10,
        product_id: 5,
        urls,
        created_at: None,
    };
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 10,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    std::env::set_var("R2_PUBLIC_URL", "https://cdn.example.com");

    let req = Request::new(ConfirmImageUploadRequest {
        product_id: 5,
        key: "products/5/uuid/file.png".into(),
        display_order: Some(1),
        alt_text: Some("alt".into()),
    });
    let result = confirm_image_upload(&txn, req).await;
    assert!(result.is_ok());
    let ProductImagesResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].product_id, 5);
    assert!(items[0].url.is_some());
}

#[tokio::test]
async fn delete_product_image_not_found_yields_not_found_status() {
    use core_operations::handlers::product_images::delete_product_image;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<product_images::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(DeleteProductImageRequest { image_id: 999 });
    let result = delete_product_image(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn update_product_image_not_found_yields_not_found_status() {
    use core_operations::handlers::product_images::update_product_image;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<product_images::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(UpdateProductImageRequest {
        image_id: 123,
        product_id: 10,
        image_base64: String::new(),
        alt_text: None,
    });
    let result = update_product_image(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn update_product_image_updates_product_id_and_preserves_url() {
    use core_operations::handlers::product_images::update_product_image;

    let urls = serde_json::json!({ "1": "https://cdn.example.com/products/1/img.png" });
    let existing = product_images::Model {
        image_id: 20,
        product_id: 5,
        urls: urls.clone(),
        created_at: None,
    };
    let updated = product_images::Model {
        image_id: 20,
        product_id: 7,
        urls,
        created_at: None,
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![existing]])
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![updated]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(UpdateProductImageRequest {
        image_id: 20,
        product_id: 7,
        image_base64: String::new(),
        alt_text: None,
    });
    let result = update_product_image(&txn, req).await;
    assert!(result.is_ok());
    let ProductImagesResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    let img = &items[0];
    assert_eq!(img.image_id, 20);
    assert_eq!(img.product_id, 7);
    assert!(img.url.as_deref().unwrap().contains("img.png"));
}

#[tokio::test]
async fn search_product_image_filters_by_image_and_product_id() {
    use core_operations::handlers::product_images::search_product_image;

    let urls = serde_json::json!({ "1": "https://cdn.example.com/products/1/img.png" });
    let model = product_images::Model {
        image_id: 30,
        product_id: 9,
        urls,
        created_at: None,
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(SearchProductImageRequest {
        image_id: Some(30),
        product_id: Some(9),
        alt_text: None,
    });
    let result = search_product_image(&txn, req).await;
    assert!(result.is_ok());
    let ProductImagesResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    let img = &items[0];
    assert_eq!(img.image_id, 30);
    assert_eq!(img.product_id, 9);
    assert!(img.url.is_some());
}
