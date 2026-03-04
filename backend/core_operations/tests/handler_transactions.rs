//! Unit tests for transactions handlers.

use core_db_entities::entity::transactions;
use proto::proto::core::{
    CreateTransactionRequest, DeleteTransactionRequest, SearchTransactionRequest,
    TransactionsResponse, UpdateTransactionRequest,
};
use rust_decimal::Decimal;
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, TransactionTrait};
use tonic::Request;

fn make_tx(id: i64) -> transactions::Model {
    transactions::Model {
        transaction_id: id,
        user_id: 1,
        amount: Decimal::new(1000, 2),
        transaction_date: chrono::Utc::now(),
        r#type: "credit".into(),
    }
}

#[tokio::test]
async fn create_transaction_inserts_and_returns_created_model() {
    use core_operations::handlers::transactions::create_transaction;

    let model = make_tx(1);
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(CreateTransactionRequest {
        user_id: 1,
        amount_paise: 1000,
        r#type: "credit".into(),
    });
    let result = create_transaction(&txn, req).await;
    assert!(result.is_ok());
    let TransactionsResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].transaction_id, 1);
}

#[tokio::test]
async fn update_transaction_not_found_yields_not_found_status() {
    use core_operations::handlers::transactions::update_transaction;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<transactions::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(UpdateTransactionRequest {
        transaction_id: 99,
        user_id: None,
        amount_paise: None,
        r#type: None,
    });
    let result = update_transaction(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn delete_transaction_not_found_yields_not_found_status() {
    use core_operations::handlers::transactions::delete_transaction;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<transactions::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(DeleteTransactionRequest {
        transaction_id: 77,
    });
    let result = delete_transaction(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn search_transaction_filters_by_id_when_nonzero() {
    use core_operations::handlers::transactions::search_transaction;

    let model = make_tx(3);
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(SearchTransactionRequest {
        transaction_id: 3,
    });
    let result = search_transaction(&txn, req).await;
    assert!(result.is_ok());
    let TransactionsResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].transaction_id, 3);
}

