//! Unit tests for category handlers using SeaORM MockDatabase.

use core_db_entities::entity::product_categories;
use proto::proto::core::{
    CategoriesResponse, CreateCategoryRequest, DeleteCategoryRequest, SearchCategoryRequest,
    UpdateCategoryRequest,
};
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, TransactionTrait};
use tonic::Request;

#[tokio::test]
async fn create_category_inserts_and_returns_created_model() {
    use core_operations::handlers::categories::create_category;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![product_categories::Model {
            category_id: 1,
            name: "Shoes".to_string(),
        }]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(CreateCategoryRequest {
        name: "Shoes".to_string(),
    });
    let result = create_category(&txn, req).await;
    assert!(result.is_ok());
    let CategoriesResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].category_id, 1);
    assert_eq!(items[0].name, "Shoes");
}

#[tokio::test]
async fn update_category_updates_existing_row() {
    use core_operations::handlers::categories::update_category;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![product_categories::Model {
            category_id: 2,
            name: "Apparel".to_string(),
        }]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(UpdateCategoryRequest {
        category_id: 2,
        name: "Clothing".to_string(),
    });
    let result = update_category(&txn, req).await;
    assert!(result.is_ok());
    let res = result.unwrap().into_inner();
    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].category_id, 2);
}

#[tokio::test]
async fn delete_category_not_found_yields_not_found_status() {
    use core_operations::handlers::categories::delete_category;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<product_categories::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(DeleteCategoryRequest { category_id: 99 });
    let result = delete_category(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn search_category_filters_by_id_when_provided() {
    use core_operations::handlers::categories::search_category;

    let model = product_categories::Model {
        category_id: 5,
        name: "Accessories".to_string(),
    };
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(SearchCategoryRequest {
        category_id: Some(5),
        name: None,
    });
    let result = search_category(&txn, req).await;
    assert!(result.is_ok());
    let res = result.unwrap().into_inner();
    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].category_id, 5);
}
