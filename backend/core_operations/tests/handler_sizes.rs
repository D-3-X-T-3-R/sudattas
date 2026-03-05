//! Unit tests for sizes handlers.

use core_db_entities::entity::sizes;
use proto::proto::core::{
    CreateSizeRequest, DeleteSizeRequest, SearchSizeRequest, SizesResponse, UpdateSizeRequest,
};
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, TransactionTrait};
use tonic::Request;

#[tokio::test]
async fn create_size_inserts_and_returns_created_model() {
    use core_operations::handlers::sizes::create_size;

    let model = sizes::Model {
        size_id: 1,
        size_name: "M".into(),
    };
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(CreateSizeRequest {
        size_name: "M".into(),
    });
    let result = create_size(&txn, req).await;
    assert!(result.is_ok());
    let SizesResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].size_id, 1);
}

#[tokio::test]
async fn update_size_updates_existing_row() {
    use core_operations::handlers::sizes::update_size;

    let model = sizes::Model {
        size_id: 2,
        size_name: "L".into(),
    };
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(UpdateSizeRequest {
        size_id: 2,
        size_name: "XL".into(),
    });
    let result = update_size(&txn, req).await;
    assert!(result.is_ok());
    let res = result.unwrap().into_inner();
    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].size_id, 2);
}

#[tokio::test]
async fn delete_size_not_found_yields_not_found_status() {
    use core_operations::handlers::sizes::delete_size;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<sizes::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(DeleteSizeRequest { size_id: 99 });
    let result = delete_size(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn search_size_filters_by_id_when_nonzero() {
    use core_operations::handlers::sizes::search_size;

    let model = sizes::Model {
        size_id: 3,
        size_name: "S".into(),
    };
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(SearchSizeRequest { size_id: 3 });
    let result = search_size(&txn, req).await;
    assert!(result.is_ok());
    let SizesResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].size_id, 3);
}
