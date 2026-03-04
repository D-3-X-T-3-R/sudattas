//! Unit tests for product_attributes handlers.

use core_db_entities::entity::product_attributes;
use proto::proto::core::{
    CreateProductAttributeRequest, DeleteProductAttributeRequest, ProductAttributesResponse,
    SearchProductAttributeRequest, UpdateProductAttributeRequest,
};
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, TransactionTrait};
use tonic::Request;

#[tokio::test]
async fn create_product_attribute_inserts_and_returns_created_model() {
    use core_operations::handlers::product_attributes::create_product_attribute;

    let model = product_attributes::Model {
        attribute_id: 1,
        attribute_name: "material".into(),
        attribute_value: "cotton".into(),
    };
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(CreateProductAttributeRequest {
        attribute_name: "material".into(),
        attribute_value: "cotton".into(),
    });
    let result = create_product_attribute(&txn, req).await;
    assert!(result.is_ok());
    let ProductAttributesResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].attribute_name, "material");
}

#[tokio::test]
async fn update_product_attribute_not_found_yields_not_found_status() {
    use core_operations::handlers::product_attributes::update_product_attribute;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<product_attributes::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(UpdateProductAttributeRequest {
        attribute_id: 99,
        attribute_name: Some("x".into()),
        attribute_value: Some("y".into()),
    });
    let result = update_product_attribute(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn delete_product_attribute_not_found_yields_not_found_status() {
    use core_operations::handlers::product_attributes::delete_product_attribute;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<product_attributes::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(DeleteProductAttributeRequest { attribute_id: 77 });
    let result = delete_product_attribute(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn search_product_attribute_filters_by_id_and_name() {
    use core_operations::handlers::product_attributes::search_product_attribute;

    let model = product_attributes::Model {
        attribute_id: 3,
        attribute_name: "color".into(),
        attribute_value: "red".into(),
    };
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(SearchProductAttributeRequest {
        attribute_id: Some(3),
        attribute_name: Some("color".into()),
        attribute_value: None,
    });
    let result = search_product_attribute(&txn, req).await;
    assert!(result.is_ok());
    let res = result.unwrap().into_inner();
    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].attribute_id, 3);
}

