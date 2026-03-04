//! Unit tests for inventory_logs handlers using SeaORM MockDatabase.

use core_db_entities::entity::inventory_log;
use proto::proto::core::{
    CreateInventoryLogRequest, DeleteInventoryLogRequest, InventoryLogsResponse,
    SearchInventoryLogRequest, UpdateInventoryLogRequest,
};
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, TransactionTrait};
use tonic::Request;

#[tokio::test]
async fn create_inventory_log_inserts_and_returns_created_model() {
    use core_operations::handlers::inventory_logs::create_inventory_log;

    let model = inventory_log::Model {
        log_id: 1,
        variant_id: 10,
        change_quantity: 5,
        log_time: chrono::Utc::now(),
        reason: Some("adjustment".to_string()),
        actor_id: Some("7".to_string()),
        quantity_before: Some(10),
        quantity_after: Some(15),
    };
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(CreateInventoryLogRequest {
        variant_id: 10,
        change_quantity: 5,
        reason: "adjustment".into(),
        actor_id: Some("7".to_string()),
        quantity_before: Some(10),
        quantity_after: Some(15),
    });
    let result = create_inventory_log(&txn, req).await;
    assert!(result.is_ok());
    let InventoryLogsResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].variant_id, 10);
    assert_eq!(items[0].change_quantity, 5);
}

#[tokio::test]
async fn update_inventory_log_not_found_yields_not_found_status() {
    use core_operations::handlers::inventory_logs::update_inventory_log;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<inventory_log::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(UpdateInventoryLogRequest {
        log_id: 99,
        variant_id: None,
        change_quantity: None,
        reason: None,
    });
    let result = update_inventory_log(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn delete_inventory_log_not_found_yields_not_found_status() {
    use core_operations::handlers::inventory_logs::delete_inventory_log;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<inventory_log::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(DeleteInventoryLogRequest { log_id: 123 });
    let result = delete_inventory_log(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn search_inventory_log_filters_by_log_id_and_variant() {
    use core_operations::handlers::inventory_logs::search_inventory_log;

    let model = inventory_log::Model {
        log_id: 2,
        variant_id: 11,
        change_quantity: -1,
        log_time: chrono::Utc::now(),
        reason: Some("sale".into()),
        actor_id: Some("5".to_string()),
        quantity_before: Some(4),
        quantity_after: Some(3),
    };
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(SearchInventoryLogRequest {
        log_id: Some(2),
        variant_id: Some(11),
    });
    let result = search_inventory_log(&txn, req).await;
    assert!(result.is_ok());
    let res = result.unwrap().into_inner();
    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].log_id, 2);
    assert_eq!(res.items[0].variant_id, 11);
}

