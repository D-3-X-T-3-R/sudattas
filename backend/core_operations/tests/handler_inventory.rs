//! Unit tests for inventory handlers using SeaORM MockDatabase.

use core_db_entities::entity::inventory;
use proto::proto::core::{
    CreateInventoryItemRequest, DeleteInventoryItemRequest, InventoryItemsResponse,
    SearchInventoryItemRequest, UpdateInventoryItemRequest,
};
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, TransactionTrait};
use tonic::Request;

#[tokio::test]
async fn create_inventory_item_inserts_and_returns_created_model() {
    use core_operations::handlers::inventory::create_inventory_item;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![inventory::Model {
            inventory_id: 1,
            variant_id: Some(10),
            quantity_available: Some(100),
            reorder_level: Some(5),
            quantity_reserved: Some(0),
            updated_at: None,
        }]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(CreateInventoryItemRequest {
        variant_id: 10,
        quantity_available: 100,
        reorder_level: 5,
    });
    let result = create_inventory_item(&txn, req).await;
    assert!(result.is_ok());
    let InventoryItemsResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].inventory_id, 1);
    assert_eq!(items[0].variant_id, 10);
}

#[tokio::test]
async fn update_inventory_item_not_found_yields_not_found_status() {
    use core_operations::handlers::inventory::update_inventory_item;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<inventory::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(UpdateInventoryItemRequest {
        inventory_id: 99,
        variant_id: None,
        quantity_available: None,
        reorder_level: None,
    });
    let result = update_inventory_item(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn update_inventory_item_merges_existing_and_new_values() {
    use core_operations::handlers::inventory::update_inventory_item;

    let existing = inventory::Model {
        inventory_id: 2,
        variant_id: Some(11),
        quantity_available: Some(5),
        reorder_level: Some(2),
        quantity_reserved: Some(0),
        updated_at: None,
    };
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![existing]])
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![inventory::Model {
            inventory_id: 2,
            variant_id: Some(12),
            quantity_available: Some(10),
            reorder_level: Some(3),
            quantity_reserved: Some(0),
            updated_at: None,
        }]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(UpdateInventoryItemRequest {
        inventory_id: 2,
        variant_id: Some(12),
        quantity_available: Some(10),
        reorder_level: Some(3),
    });
    let result = update_inventory_item(&txn, req).await;
    assert!(result.is_ok());
    let res = result.unwrap().into_inner();
    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].inventory_id, 2);
    assert_eq!(res.items[0].variant_id, 12);
}

#[tokio::test]
async fn search_inventory_item_filters_by_inventory_and_variant() {
    use core_operations::handlers::inventory::search_inventory_item;

    let model = inventory::Model {
        inventory_id: 10,
        variant_id: Some(200),
        quantity_available: Some(50),
        reorder_level: Some(5),
        quantity_reserved: Some(0),
        updated_at: None,
    };
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(SearchInventoryItemRequest {
        inventory_id: Some(10),
        variant_id: Some(200),
    });
    let result = search_inventory_item(&txn, req).await;
    assert!(result.is_ok());
    let InventoryItemsResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].inventory_id, 10);
    assert_eq!(items[0].variant_id, 200);
}

#[tokio::test]
async fn delete_inventory_item_deletes_existing_and_returns_response() {
    use core_operations::handlers::inventory::delete_inventory_item;

    let model = inventory::Model {
        inventory_id: 3,
        variant_id: Some(30),
        quantity_available: Some(5),
        reorder_level: Some(1),
        quantity_reserved: Some(0),
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

    let req = Request::new(DeleteInventoryItemRequest { inventory_id: 3 });
    let result = delete_inventory_item(&txn, req).await;
    assert!(result.is_ok());
    let InventoryItemsResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].inventory_id, 3);
    assert_eq!(items[0].variant_id, 30);
}

#[tokio::test]
async fn delete_inventory_item_not_found_yields_not_found_status() {
    use core_operations::handlers::inventory::delete_inventory_item;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<inventory::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(DeleteInventoryItemRequest { inventory_id: 999 });
    let result = delete_inventory_item(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

