//! Unit tests for order_details handlers using SeaORM MockDatabase.

use core_db_entities::entity::order_details;
use proto::proto::core::{
    CreateOrderDetailsRequest, OrderDetailsResponse, SearchOrderDetailRequest,
    UpdateOrderDetailRequest,
};
use rust_decimal::Decimal;
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, TransactionTrait};
use tonic::Request;

#[tokio::test]
async fn create_order_details_inserts_multiple_lines_and_returns_responses() {
    use core_operations::handlers::order_details::create_order_details;

    let model1 = order_details::Model {
        order_detail_id: 1,
        order_id: 10,
        variant_id: 100,
        quantity: 2,
        price: Some(Decimal::new(5000, 2)),
        unit_price_minor: 5000,
        discount_minor: None,
        tax_minor: None,
        sku: None,
        title: None,
        line_attrs: None,
    };
    let model2 = order_details::Model {
        order_detail_id: 2,
        ..model1.clone()
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![
            MockExecResult {
                last_insert_id: 1,
                rows_affected: 1,
            },
            MockExecResult {
                last_insert_id: 2,
                rows_affected: 1,
            },
        ])
        .append_query_results(vec![vec![model1], vec![model2]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(CreateOrderDetailsRequest {
        order_details: vec![
            proto::proto::core::CreateOrderDetailRequest {
                order_id: 10,
                variant_id: 100,
                quantity: 2,
                price_paise: 5000,
                unit_price_minor: Some(5000),
                discount_minor: None,
                tax_minor: None,
                sku: None,
                title: None,
            },
            proto::proto::core::CreateOrderDetailRequest {
                order_id: 10,
                variant_id: 101,
                quantity: 1,
                price_paise: 2500,
                unit_price_minor: Some(2500),
                discount_minor: None,
                tax_minor: None,
                sku: None,
                title: None,
            },
        ],
    });
    let result = create_order_details(&txn, req).await;
    assert!(result.is_ok());
    let OrderDetailsResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 2);
}

#[tokio::test]
async fn update_order_detail_updates_single_line() {
    use core_operations::handlers::order_details::update_order_detail;

    let existing = order_details::Model {
        order_detail_id: 3,
        order_id: 11,
        variant_id: 200,
        quantity: 1,
        price: Some(Decimal::new(1000, 2)),
        unit_price_minor: 1000,
        discount_minor: None,
        tax_minor: None,
        sku: None,
        title: None,
        line_attrs: None,
    };
    let updated = order_details::Model {
        quantity: 2,
        ..existing.clone()
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![updated]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(UpdateOrderDetailRequest {
        order_detail_id: 3,
        order_id: 11,
        variant_id: 200,
        quantity: 2,
        price_paise: 1000,
    });
    let result = update_order_detail(&txn, req).await;
    assert!(result.is_ok());
    let res = result.unwrap().into_inner();
    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].order_detail_id, 3);
}

#[tokio::test]
async fn search_order_detail_filters_by_order_and_price_range() {
    use core_operations::handlers::order_details::search_order_detail;

    let model = order_details::Model {
        order_detail_id: 4,
        order_id: 20,
        variant_id: 300,
        quantity: 1,
        price: Some(Decimal::new(2500, 2)),
        unit_price_minor: 2500,
        discount_minor: None,
        tax_minor: None,
        sku: None,
        title: None,
        line_attrs: None,
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(SearchOrderDetailRequest {
        order_detail_id: Some(4),
        order_id: Some(20),
        variant_id: Some(300),
        quantity: Some(1),
        price_start_paise: Some(2_000),
        price_end_paise: Some(3_000),
    });
    let result = search_order_detail(&txn, req).await;
    assert!(result.is_ok());
    let OrderDetailsResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    let od = &items[0];
    assert_eq!(od.order_detail_id, 4);
    assert_eq!(od.order_id, 20);
    assert_eq!(od.variant_id, 300);
    assert_eq!(od.price_paise, 2_500);
}
