//! Unit tests for shipping_methods handlers.

use core_db_entities::entity::shipping_methods;
use proto::proto::core::{
    CreateShippingMethodRequest, DeleteShippingMethodRequest, SearchShippingMethodRequest,
    ShippingMethodsResponse, UpdateShippingMethodRequest,
};
use rust_decimal::Decimal;
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, TransactionTrait};
use tonic::Request;

fn make_method(id: i64) -> shipping_methods::Model {
    shipping_methods::Model {
        method_id: id,
        method_name: Some("Standard".into()),
        cost: Some(Decimal::new(500, 2)),
        estimated_delivery_time: Some("3-5 days".into()),
    }
}

#[tokio::test]
async fn create_shipping_method_inserts_and_returns_created_model() {
    use core_operations::handlers::shipping_methods::create_shipping_method;

    let model = make_method(1);
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(CreateShippingMethodRequest {
        method_name: "Standard".into(),
        cost_paise: 500,
        estimated_delivery_time: "3-5 days".into(),
    });
    let result = create_shipping_method(&txn, req).await;
    assert!(result.is_ok());
    let ShippingMethodsResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].method_id, 1);
}

#[tokio::test]
async fn update_shipping_method_not_found_yields_not_found_status() {
    use core_operations::handlers::shipping_methods::update_shipping_method;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<shipping_methods::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(UpdateShippingMethodRequest {
        method_id: 99,
        method_name: None,
        cost_paise: None,
        estimated_delivery_time: None,
    });
    let result = update_shipping_method(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn delete_shipping_method_not_found_yields_not_found_status() {
    use core_operations::handlers::shipping_methods::delete_shipping_method;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<shipping_methods::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(DeleteShippingMethodRequest { method_id: 77 });
    let result = delete_shipping_method(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn search_shipping_method_filters_by_id_when_nonzero() {
    use core_operations::handlers::shipping_methods::search_shipping_method;

    let model = make_method(3);
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(SearchShippingMethodRequest { method_id: 3 });
    let result = search_shipping_method(&txn, req).await;
    assert!(result.is_ok());
    let res = result.unwrap().into_inner();
    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].method_id, 3);
}
