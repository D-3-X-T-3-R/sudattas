//! Unit tests for order_events handlers using SeaORM MockDatabase.

use core_db_entities::entity::order_events;
use core_db_entities::entity::sea_orm_active_enums::ActorType;
use proto::proto::core::{CreateOrderEventRequest, GetOrderEventsRequest, OrderEventsResponse};
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, TransactionTrait};
use tonic::Request;

#[tokio::test]
async fn create_order_event_with_admin_actor_inserts_and_returns_response() {
    use core_operations::handlers::order_events::create_order_event;

    let now = chrono::Utc::now();
    let model = order_events::Model {
        event_id: 1,
        order_id: 42,
        event_type: "status_change".to_string(),
        from_status: Some("pending".to_string()),
        to_status: Some("processing".to_string()),
        actor_type: ActorType::Admin,
        message: Some("admin updated status".to_string()),
        created_at: Some(now),
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(CreateOrderEventRequest {
        order_id: 42,
        event_type: "status_change".to_string(),
        from_status: Some("pending".to_string()),
        to_status: Some("processing".to_string()),
        actor_type: "admin".to_string(),
        message: Some("admin updated status".to_string()),
    });

    let result = create_order_event(&txn, req).await;
    assert!(result.is_ok());
    let OrderEventsResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    let ev = &items[0];
    assert_eq!(ev.event_id, 1);
    assert_eq!(ev.order_id, 42);
    assert_eq!(ev.event_type, "status_change");
    assert_eq!(ev.from_status, "pending");
    assert_eq!(ev.to_status, "processing");
    assert_eq!(ev.actor_type, "admin");
}

#[tokio::test]
async fn create_order_event_with_unknown_actor_defaults_to_system() {
    use core_operations::handlers::order_events::create_order_event;

    let now = chrono::Utc::now();
    let model = order_events::Model {
        event_id: 2,
        order_id: 100,
        event_type: "note".to_string(),
        from_status: None,
        to_status: None,
        actor_type: ActorType::System,
        message: Some("system note".to_string()),
        created_at: Some(now),
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 2,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(CreateOrderEventRequest {
        order_id: 100,
        event_type: "note".to_string(),
        from_status: None,
        to_status: None,
        actor_type: "unexpected".to_string(),
        message: Some("system note".to_string()),
    });

    let result = create_order_event(&txn, req).await;
    assert!(result.is_ok());
    let OrderEventsResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    let ev = &items[0];
    assert_eq!(ev.order_id, 100);
    assert_eq!(ev.actor_type, "system");
}

#[tokio::test]
async fn get_order_events_returns_events_for_order_id() {
    use core_operations::handlers::order_events::get_order_events;

    let now = chrono::Utc::now();
    let m1 = order_events::Model {
        event_id: 10,
        order_id: 200,
        event_type: "created".to_string(),
        from_status: None,
        to_status: Some("pending".to_string()),
        actor_type: ActorType::System,
        message: Some("order created".to_string()),
        created_at: Some(now - chrono::Duration::minutes(5)),
    };
    let m2 = order_events::Model {
        event_id: 11,
        order_id: 200,
        event_type: "status_change".to_string(),
        from_status: Some("pending".to_string()),
        to_status: Some("processing".to_string()),
        actor_type: ActorType::Admin,
        message: Some("admin moved to processing".to_string()),
        created_at: Some(now),
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![m1, m2]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(GetOrderEventsRequest { order_id: 200 });
    let result = get_order_events(&txn, req).await;
    assert!(result.is_ok());
    let OrderEventsResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 2);
    assert!(items.iter().all(|ev| ev.order_id == 200));
}

