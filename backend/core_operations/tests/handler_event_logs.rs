//! Unit tests for event_logs handlers using SeaORM MockDatabase.

use core_db_entities::entity::event_logs;
use proto::proto::core::{
    CreateEventLogRequest, DeleteEventLogRequest, EventLogsResponse, SearchEventLogRequest,
    UpdateEventLogRequest,
};
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, TransactionTrait};
use tonic::Request;

#[tokio::test]
async fn create_event_log_inserts_and_returns_created_model() {
    use core_operations::handlers::event_logs::create_event_log;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![event_logs::Model {
            log_id: 1,
            event_type: "login".to_string(),
            event_description: Some("user logged in".to_string()),
            user_id: Some(10),
            event_time: chrono::Utc::now(),
        }]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(CreateEventLogRequest {
        event_type: "login".to_string(),
        event_description: "user logged in".to_string(),
        user_id: 10,
    });
    let result = create_event_log(&txn, req).await;
    assert!(result.is_ok());
    let EventLogsResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].log_id, 1);
    assert_eq!(items[0].event_type, "login");
}

#[tokio::test]
async fn search_event_log_filters_by_id_when_nonzero() {
    use core_operations::handlers::event_logs::search_event_log;

    let model = event_logs::Model {
        log_id: 5,
        event_type: "order_created".to_string(),
        event_description: Some("order #5 created".to_string()),
        user_id: Some(20),
        event_time: chrono::Utc::now(),
    };
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(SearchEventLogRequest { log_id: 5 });
    let result = search_event_log(&txn, req).await;
    assert!(result.is_ok());
    let res = result.unwrap().into_inner();
    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].log_id, 5);
}

#[tokio::test]
async fn delete_event_log_deletes_existing_and_returns_response() {
    use core_operations::handlers::event_logs::delete_event_log;

    let model = event_logs::Model {
        log_id: 7,
        event_type: "password_reset".to_string(),
        event_description: Some("user requested password reset".to_string()),
        user_id: Some(42),
        event_time: chrono::Utc::now(),
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![model]])
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(DeleteEventLogRequest { log_id: 7 });
    let result = delete_event_log(&txn, req).await;
    assert!(result.is_ok());
    let EventLogsResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].log_id, 7);
    assert_eq!(items[0].event_type, "password_reset");
    assert_eq!(items[0].user_id, 42);
}

#[tokio::test]
async fn delete_event_log_not_found_yields_not_found_status() {
    use core_operations::handlers::event_logs::delete_event_log;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<event_logs::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(DeleteEventLogRequest { log_id: 999 });
    let result = delete_event_log(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn update_event_log_not_found_yields_not_found_status() {
    use core_operations::handlers::event_logs::update_event_log;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<event_logs::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(UpdateEventLogRequest {
        log_id: 123,
        event_type: Some("login".to_string()),
        event_description: Some("updated".to_string()),
        user_id: Some(1),
    });
    let result = update_event_log(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

// Additional happy-path update scenarios (field-specific and multi-field) are exercised via higher-level flows.

