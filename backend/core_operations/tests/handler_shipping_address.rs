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
        is_default: 0,
        country: "IN".into(),
        state_region: "KA".into(),
        city: "BLR".into(),
        postal_code: "560001".into(),
        road: Some("MG Road".into()),
        apartment_no_or_name: Some("42".into()),
        recipient_name: Some("Test User".into()),
        phone_number: Some("+1 415 555 0101".into()),
        is_deleted: 0,
    }
}

#[tokio::test]
async fn create_shipping_address_inserts_and_returns_created_model() {
    use core_operations::handlers::shipping_address::create_shipping_address;

    let model = make_address(1);
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![
            MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            },
            MockExecResult {
                last_insert_id: 1,
                rows_affected: 1,
            },
        ])
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(CreateShippingAddressRequest {
        user_id: Some(1),
        is_default: true,
        country: "IN".into(),
        state_region: "KA".into(),
        city: "BLR".into(),
        postal_code: "560001".into(),
        road: Some("MG Road".into()),
        apartment_no_or_name: Some("42".into()),
        recipient_name: Some("Test User".into()),
        phone_number: Some("+1 415 555 0101".into()),
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
        .append_exec_results(vec![
            MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            },
            MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            },
        ])
        .append_query_results(vec![vec![updated]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(UpdateShippingAddressRequest {
        shipping_address_id: 2,
        user_id: Some(1),
        is_default: true,
        country: "IN".into(),
        state_region: "KA".into(),
        city: "BLR".into(),
        postal_code: "560001".into(),
        road: Some("MG Road".into()),
        apartment_no_or_name: Some("42".into()),
        recipient_name: Some("Updated User".into()),
        phone_number: Some("+44 20 7946 0958".into()),
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
async fn delete_shipping_address_soft_deletes_non_default_address() {
    use core_operations::handlers::shipping_address::delete_shipping_address;

    // Non-default address: find_by_id, then the soft-delete update (exec + MySQL fetch-back
    // query) — no reassignment branch since is_default was already 0.
    let existing = make_address(1);
    let mut after_delete = existing.clone();
    after_delete.is_deleted = 1;
    after_delete.is_default = 0;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![existing]])
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![after_delete]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(DeleteShippingAddressRequest {
        shipping_address_id: 1,
    });
    let result = delete_shipping_address(&txn, req).await;
    assert!(result.is_ok(), "delete should succeed: {:?}", result.err());
    let ShippingAddressesResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].shipping_address_id, 1);

    // The naive hard-delete this replaced would have failed here with a raw FK constraint
    // error (1451) the moment any real order referenced this address — this soft-delete path
    // never touches the Orders table at all, so it can't hit that constraint.
}

#[tokio::test]
async fn delete_shipping_address_reassigns_default_to_next_remaining_address() {
    use core_operations::handlers::shipping_address::delete_shipping_address;

    let mut default_addr = make_address(1);
    default_addr.is_default = 1;
    let mut after_delete = default_addr.clone();
    after_delete.is_deleted = 1;
    after_delete.is_default = 0;

    let mut next_default = make_address(2);
    next_default.is_default = 0;
    let mut promoted = next_default.clone();
    promoted.is_default = 1;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        // find_by_id(1) — the address being deleted.
        .append_query_results(vec![vec![default_addr]])
        // soft-delete update on address 1: exec + MySQL fetch-back query.
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![after_delete]])
        // find next remaining (non-deleted) address for this user to promote.
        .append_query_results(vec![vec![next_default.clone()]])
        // update_many: unset is_default for the user's remaining rows.
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        // promote address 2 to default: exec + MySQL fetch-back query.
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![promoted]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(DeleteShippingAddressRequest {
        shipping_address_id: 1,
    });
    let result = delete_shipping_address(&txn, req).await;
    assert!(result.is_ok(), "delete should succeed: {:?}", result.err());
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
