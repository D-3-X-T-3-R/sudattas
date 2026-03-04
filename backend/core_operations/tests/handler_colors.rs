//! Unit tests for color handlers using SeaORM MockDatabase.

use core_db_entities::entity::colors;
use proto::proto::core::{
    ColorResponse, ColorsResponse, CreateColorRequest, DeleteColorRequest, SearchColorRequest,
    UpdateColorRequest,
};
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, TransactionTrait};
use tonic::Request;

#[tokio::test]
async fn create_color_inserts_and_returns_created_model() {
    use core_operations::handlers::colors::create_color;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![colors::Model {
            color_id: 1,
            color_name: "Red".to_string(),
        }]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(CreateColorRequest {
        color_name: "Red".to_string(),
    });
    let result = create_color(&txn, req).await;
    assert!(result.is_ok());
    let ColorsResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].color_id, 1);
    assert_eq!(items[0].color_name, "Red");
}

#[tokio::test]
async fn update_color_updates_existing_row() {
    use core_operations::handlers::colors::update_color;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![colors::Model {
            color_id: 2,
            color_name: "Blue".to_string(),
        }]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(UpdateColorRequest {
        color_id: 2,
        color_name: "Navy".to_string(),
    });
    let result = update_color(&txn, req).await;
    assert!(result.is_ok());
    let res = result.unwrap().into_inner();
    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].color_id, 2);
}

#[tokio::test]
async fn delete_color_not_found_yields_not_found_status() {
    use core_operations::handlers::colors::delete_color;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<colors::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(DeleteColorRequest { color_id: 99 });
    let result = delete_color(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn search_color_filters_by_id_when_nonzero() {
    use core_operations::handlers::colors::search_color;

    let model = colors::Model {
        color_id: 7,
        color_name: "Green".to_string(),
    };
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(SearchColorRequest { color_id: 7 });
    let result = search_color(&txn, req).await;
    assert!(result.is_ok());
    let ColorsResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    let ColorResponse {
        color_id,
        color_name,
    } = &items[0];
    assert_eq!(*color_id, 7);
    assert_eq!(color_name, "Green");
}

