//! P1 Tests for outbox enqueue and worker (idempotent publish).
//! Unit tests use mock; integration test requires TEST_DATABASE_URL.
//!
//! Tests that use `OUTBOX_DELIVER_FAIL` share process env. To avoid flakiness when running
//! the full suite, run this file with one thread:
//! `cargo test -p core_operations --test handler_outbox -- --test-threads=1`

mod integration_common;

use serde_json::json;
use std::time::Duration;

async fn connect_test_db() -> sea_orm::DatabaseConnection {
    use sea_orm::{ConnectOptions, Database};

    let db_url = integration_common::test_db_url();
    let mut last_err: Option<sea_orm::DbErr> = None;
    for _attempt in 0..8 {
        let mut opts = ConnectOptions::new(db_url.clone());
        // This test file includes a deliberate two-worker concurrency assertion, so the
        // integration pool must allow >1 concurrent connection.
        opts.max_connections(4)
            .min_connections(0)
            .acquire_timeout(Duration::from_secs(20))
            .connect_timeout(Duration::from_secs(15))
            .idle_timeout(Duration::from_secs(10))
            .max_lifetime(Duration::from_secs(60))
            .sqlx_logging(false);
        match Database::connect(opts).await {
            Ok(db) => return db,
            Err(err) => {
                last_err = Some(err);
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }
    panic!(
        "failed to connect to test DB after retries: {:?}",
        last_err.expect("last error")
    );
}

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
    use sea_orm::TransactionTrait;

    let db = connect_test_db().await;
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
    db.close().await.ok();
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_outbox_two_workers_cannot_claim_same_row_concurrently() {
    use core_operations::handlers::outbox::enqueue_outbox_event;
    use core_operations::procedures::outbox_worker::process_pending_outbox_events;
    use sea_orm::{ConnectionTrait, DbBackend, Statement, TransactionTrait};

    let db = connect_test_db().await;
    db.execute_unprepared(
        r#"UPDATE OutboxEvents
           SET status = 'processed',
               published_at = UTC_TIMESTAMP()
           WHERE status = 'pending'"#,
    )
    .await
    .expect("clear pre-existing pending outbox rows for deterministic concurrency assertion");
    let aggregate_id = format!(
        "itest_outbox_concurrency_{}",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
    );

    let insert_txn = db.begin().await.expect("begin insert txn");
    enqueue_outbox_event(
        &insert_txn,
        "OrderPlaced",
        "order",
        aggregate_id.as_str(),
        json!({ "order_id": aggregate_id }),
    )
    .await
    .expect("enqueue");
    insert_txn.commit().await.expect("commit insert");

    let verify_txn = db.begin().await.expect("begin verify txn");
    let event_row = verify_txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::MySql,
            r#"SELECT event_id
               FROM OutboxEvents
               WHERE aggregate_id = ?
               ORDER BY event_id DESC
               LIMIT 1"#,
            [aggregate_id.clone().into()],
        ))
        .await
        .expect("query inserted outbox row")
        .expect("inserted outbox row");
    let event_id: i64 = event_row.try_get("", "event_id").expect("event_id");
    verify_txn.rollback().await.ok();

    let audit_table = "itest_outbox_claim_audit";
    let trigger_name = format!("trg_itest_outbox_claim_audit_{event_id}");
    db.execute_unprepared(&format!("DROP TRIGGER IF EXISTS `{trigger_name}`"))
        .await
        .expect("drop trigger");
    db.execute_unprepared(&format!("DROP TABLE IF EXISTS `{audit_table}`"))
        .await
        .expect("drop audit table");
    db.execute_unprepared(&format!(
        r#"CREATE TABLE `{audit_table}` (
               id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
               event_id BIGINT NOT NULL
           ) ENGINE=InnoDB"#
    ))
    .await
    .expect("create audit table");
    db.execute_unprepared(&format!(
        r#"CREATE TRIGGER `{trigger_name}`
BEFORE UPDATE ON `OutboxEvents`
FOR EACH ROW
BEGIN
    IF NEW.event_id = {event_id}
       AND OLD.status = 'pending'
       AND NEW.status = 'client_verified' THEN
        INSERT INTO `{audit_table}` (event_id) VALUES (NEW.event_id);
    END IF;
END"#
    ))
    .await
    .expect("create audit trigger");

    let (r1, r2) = tokio::join!(
        process_pending_outbox_events(&db, 1),
        process_pending_outbox_events(&db, 1)
    );
    r1.expect("worker A");
    r2.expect("worker B");

    let assert_txn = db.begin().await.expect("begin assert txn");
    let claim_count_row = assert_txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::MySql,
            format!("SELECT COUNT(*) AS count FROM `{audit_table}` WHERE event_id = ?"),
            [event_id.into()],
        ))
        .await
        .expect("query claim count")
        .expect("claim count row");
    let claim_count: i64 = claim_count_row.try_get("", "count").expect("claim count");
    assert_eq!(
        claim_count, 1,
        "exactly one worker must claim a given outbox row"
    );

    let status_row = assert_txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::MySql,
            "SELECT status FROM OutboxEvents WHERE event_id = ?",
            [event_id.into()],
        ))
        .await
        .expect("query outbox status")
        .expect("status row");
    let status: String = status_row.try_get("", "status").expect("status");
    assert_eq!(status, "processed");
    assert_txn.rollback().await.ok();

    db.execute_unprepared(&format!("DROP TRIGGER IF EXISTS `{trigger_name}`"))
        .await
        .expect("drop trigger");
    db.execute_unprepared(&format!("DROP TABLE IF EXISTS `{audit_table}`"))
        .await
        .expect("drop audit table");
    db.close().await.ok();
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
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![row.clone()]])
        .append_exec_results(vec![
            // claim pending -> client_verified
            MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            },
            // mark client_verified -> processed
            MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            },
        ])
        .into_connection();
    std::env::remove_var("OUTBOX_DELIVER_FAIL");
    let result = process_pending_outbox_events(&db, 5).await;
    let count = result.expect("process should not return error");
    assert_eq!(
        count, 1,
        "claimed + delivered event must be processed exactly once"
    );
}

/// When delivery fails (OUTBOX_DELIVER_FAIL=1), event is left Pending and processed_count is 0.
#[tokio::test]
async fn process_pending_outbox_events_delivery_fail_leaves_pending_returns_zero() {
    use core_db_entities::entity::outbox_events;
    use core_db_entities::entity::sea_orm_active_enums::Status as OutboxStatus;
    use core_operations::procedures::outbox_worker::process_pending_outbox_events;
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};

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
        .append_exec_results(vec![
            // claim pending -> client_verified
            MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            },
            // revert client_verified -> pending after delivery failure
            MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            },
        ])
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
