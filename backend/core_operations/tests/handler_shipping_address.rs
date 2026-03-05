//! Unit tests for shipping_address handlers.

use core_db_entities::entity::shipping_addresses;
use proto::proto::core::{
    CreateShippingAddressRequest, DeleteShippingAddressRequest, GetShippingAddressRequest,
    ShippingAddressesResponse, UpdateShippingAddressRequest,
};
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, TransactionTrait};
use tonic::Request;

fn make_address(id: i64) -> shipping_addresses::Model {
    shipping_addresses::Model {
        shipping_address_id: id,
        user_id: Some(1),
        country: "IN".into(),
        state_region: "KA".into(),
        city: "BLR".into(),
        postal_code: "560001".into(),
        road: Some("MG Road".into()),
        apartment_no_or_name: Some("42".into()),
    }
}

#[tokio::test]
async fn create_shipping_address_inserts_and_returns_created_model() {
    use core_operations::handlers::shipping_address::create_shipping_address;

    let model = make_address(1);
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(CreateShippingAddressRequest {
        user_id: Some(1),
        country: "IN".into(),
        state_region: "KA".into(),
        city: "BLR".into(),
        postal_code: "560001".into(),
        road: Some("MG Road".into()),
        apartment_no_or_name: Some("42".into()),
    });
    let result = create_shipping_address(&txn, req).await;
    assert!(result.is_ok());
    let ShippingAddressesResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].shipping_address_id, 1);
}

#[tokio::test]
async fn update_shipping_address_updates_all_fields() {
    use core_operations::handlers::shipping_address::update_shipping_address;

    let updated = make_address(2);
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![updated]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(UpdateShippingAddressRequest {
        shipping_address_id: 2,
        user_id: Some(1),
        country: "IN".into(),
        state_region: "KA".into(),
        city: "BLR".into(),
        postal_code: "560001".into(),
        road: Some("MG Road".into()),
        apartment_no_or_name: Some("42".into()),
    });
    let result = update_shipping_address(&txn, req).await;
    assert!(result.is_ok());
    let res = result.unwrap().into_inner();
    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].shipping_address_id, 2);
}

#[tokio::test]
async fn get_shipping_address_returns_all_rows() {
    use core_operations::handlers::shipping_address::get_shipping_address;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![make_address(1), make_address(2)]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(GetShippingAddressRequest {});
    let result = get_shipping_address(&txn, req).await;
    assert!(result.is_ok());
    let ShippingAddressesResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 2);
}

#[tokio::test]
async fn delete_shipping_address_not_found_yields_not_found_status() {
    use core_operations::handlers::shipping_address::delete_shipping_address;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<shipping_addresses::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(DeleteShippingAddressRequest {
        shipping_address_id: 99,
    });
    let result = delete_shipping_address(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}
