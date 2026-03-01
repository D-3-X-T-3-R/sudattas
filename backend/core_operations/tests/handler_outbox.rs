//! P1 Tests for outbox enqueue and worker (idempotent publish).
//! Unit tests use mock; integration test requires TEST_DATABASE_URL.
//!
//! Tests that use `OUTBOX_DELIVER_FAIL` share process env. To avoid flakiness when running
//! the full suite, run this file with one thread:
//! `cargo test -p core_operations --test handler_outbox -- --test-threads=1`

mod integration_common;

use serde_json::json;

/// Ensures OUTBOX_DELIVER_FAIL is unset so tests don't affect each other when run in parallel.
struct OutboxDeliverFailGuard;
impl OutboxDeliverFailGuard {
    /// Clear now and again on drop (for success test).
    fn clear() -> Self {
        std::env::remove_var("OUTBOX_DELIVER_FAIL");
        Self
    }
    /// Only clear on drop (for delivery_fail test: var is set during test, cleaned up after).
    fn restore_on_drop() -> Self {
        Self
    }
}
impl Drop for OutboxDeliverFailGuard {
    fn drop(&mut self) {
        std::env::remove_var("OUTBOX_DELIVER_FAIL");
    }
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_outbox_enqueue_and_worker() {
    use core_operations::handlers::outbox::enqueue_outbox_event;
    use core_operations::procedures::outbox_worker::process_pending_outbox_events;
    use sea_orm::{Database, TransactionTrait};

    let db = Database::connect(&integration_common::test_db_url())
        .await
        .expect("connect");
    let txn = db.begin().await.expect("begin");
    enqueue_outbox_event(
        &txn,
        "OrderPlaced",
        "order",
        "999998",
        json!({ "order_id": 999998, "user_id": 1 }),
    )
    .await
    .expect("enqueue");
    txn.commit().await.expect("commit");

    let count = process_pending_outbox_events(&db, 5)
        .await
        .expect("process");
    assert!(count >= 1);
}

/// Worker with no pending events returns 0 (retry path: failed events stay Pending and are retried next run).
#[tokio::test]
async fn process_pending_outbox_events_empty_returns_zero() {
    use core_operations::procedures::outbox_worker::process_pending_outbox_events;
    use sea_orm::{DatabaseBackend, MockDatabase};

    use core_db_entities::entity::outbox_events;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<outbox_events::Model>::new()])
        .into_connection();
    let count = process_pending_outbox_events(&db, 5)
        .await
        .expect("process");
    assert_eq!(count, 0);
}

/// Worker with one pending event and successful delivery marks it processed and returns 1.
#[tokio::test]
async fn process_pending_outbox_events_one_success_returns_one() {
    use core_db_entities::entity::outbox_events;
    use core_db_entities::entity::sea_orm_active_enums::Status as OutboxStatus;
    use core_operations::procedures::outbox_worker::process_pending_outbox_events;
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};

    // Clear so deliver_event() succeeds; required when tests run in parallel with delivery_fail test.
    let _guard = OutboxDeliverFailGuard::clear();
    let now = chrono::Utc::now();
    let row = outbox_events::Model {
        event_id: 1,
        event_type: "OrderPlaced".to_string(),
        aggregate_type: "order".to_string(),
        aggregate_id: "42".to_string(),
        payload: serde_json::json!({ "order_id": 42 }),
        status: OutboxStatus::Pending,
        created_at: now,
        published_at: None,
    };
    // MySQL: update() does exec (UPDATE) then find_updated_model_by_id (SELECT by pk) = 1 exec + 1 query.
    // So total: 1 query (find pending), 1 exec (UPDATE), 1 query (find_by_id for updated row).
    let row_updated = outbox_events::Model {
        status: OutboxStatus::Processed,
        published_at: Some(now),
        ..row.clone()
    };
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![row], vec![row_updated]])
        .append_exec_results(vec![
            MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            },
            MockExecResult {
                last_insert_id: 0,
                rows_affected: 0,
            },
        ])
        .into_connection();
    std::env::remove_var("OUTBOX_DELIVER_FAIL");
    let result = process_pending_outbox_events(&db, 5).await;
    let count = result.expect("process should not return error");
    assert_eq!(
        count, 1,
        "one event processed (deliver Ok, update Ok, commit Ok)"
    );
}

/// When delivery fails (OUTBOX_DELIVER_FAIL=1), event is left Pending and processed_count is 0.
#[tokio::test]
async fn process_pending_outbox_events_delivery_fail_leaves_pending_returns_zero() {
    use core_db_entities::entity::outbox_events;
    use core_db_entities::entity::sea_orm_active_enums::Status as OutboxStatus;
    use core_operations::procedures::outbox_worker::process_pending_outbox_events;
    use sea_orm::{DatabaseBackend, MockDatabase};

    std::env::set_var("OUTBOX_DELIVER_FAIL", "1");
    let _guard = OutboxDeliverFailGuard::restore_on_drop();
    let now = chrono::Utc::now();
    let row = outbox_events::Model {
        event_id: 1,
        event_type: "OrderPlaced".to_string(),
        aggregate_type: "order".to_string(),
        aggregate_id: "42".to_string(),
        payload: serde_json::json!({ "order_id": 42 }),
        status: OutboxStatus::Pending,
        created_at: now,
        published_at: None,
    };
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![row]])
        .into_connection();
    let count = process_pending_outbox_events(&db, 5)
        .await
        .expect("process");
    assert_eq!(
        count, 0,
        "on delivery failure event stays Pending, nothing processed"
    );
}

/// Sanity check: outbox event type constants are non-empty and distinct.
#[test]
fn outbox_event_types_are_defined() {
    use core_operations::handlers::outbox::{
        ABANDONED_CART, DELIVERED, ORDER_PLACED, PAYMENT_CAPTURED, REFUNDED, SHIPPED,
    };
    let types = [
        ORDER_PLACED,
        PAYMENT_CAPTURED,
        SHIPPED,
        DELIVERED,
        REFUNDED,
        ABANDONED_CART,
    ];
    for t in &types {
        assert!(!t.is_empty(), "event type must not be empty");
    }
    for i in 0..types.len() {
        for j in (i + 1)..types.len() {
            assert_ne!(types[i], types[j], "event types must be distinct");
        }
    }
}
