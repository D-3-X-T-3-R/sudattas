//! P1 Tests for outbox enqueue and worker (idempotent publish).
//! Unit tests use mock; integration test requires TEST_DATABASE_URL.

mod integration_common;

use serde_json::json;

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
