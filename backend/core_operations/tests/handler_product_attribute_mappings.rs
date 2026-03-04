//! Unit tests for product_attribute_mappings handlers.

use core_db_entities::entity::product_attribute_mapping;
use proto::proto::core::{
    CreateProductAttributeMappingRequest, DeleteProductAttributeMappingRequest,
    ProductAttributeMappingsResponse, SearchProductAttributeMappingRequest,
};
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, TransactionTrait};
use tonic::Request;

#[tokio::test]
async fn create_product_attribute_mapping_inserts_and_returns_row() {
    use core_operations::handlers::product_attribute_mappings::create_product_attribute_mapping;

    let model = product_attribute_mapping::Model {
        product_id: 10,
        attribute_id: 5,
    };
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(CreateProductAttributeMappingRequest {
        product_id: 10,
        attribute_id: 5,
    });
    let result = create_product_attribute_mapping(&txn, req).await;
    assert!(result.is_ok());
    let ProductAttributeMappingsResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].product_id, 10);
    assert_eq!(items[0].attribute_id, 5);
}

#[tokio::test]
async fn delete_product_attribute_mapping_not_found_yields_not_found_status() {
    use core_operations::handlers::product_attribute_mappings::delete_product_attribute_mapping;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<product_attribute_mapping::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(DeleteProductAttributeMappingRequest {
        product_id: 99,
        attribute_id: 77,
    });
    let result = delete_product_attribute_mapping(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn search_product_attribute_mapping_filters_by_product_and_attribute() {
    use core_operations::handlers::product_attribute_mappings::search_product_attribute_mapping;

    let model = product_attribute_mapping::Model {
        product_id: 20,
        attribute_id: 8,
    };
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(SearchProductAttributeMappingRequest {
        product_id: 20,
        attribute_id: 8,
    });
    let result = search_product_attribute_mapping(&txn, req).await;
    assert!(result.is_ok());
    let res = result.unwrap().into_inner();
    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].product_id, 20);
    assert_eq!(res.items[0].attribute_id, 8);
}

