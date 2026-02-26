//! Integration tests for webhook ingestion and payment capture.
//!
//! **Setup**
//! - Set `TEST_DATABASE_URL` or `DATABASE_URL` (e.g. `mysql://root:test_password@127.0.0.1:3306/sudattas_test`).
//! - Schema must be loaded first (e.g. migrations or `backend/database/sql_dump/01_schema.sql`).
//!
//! **Run**
//! - `cargo test --test integration_webhooks -- --ignored`

mod integration_common;

use integration_common::test_db_url;
use proto::proto::core::IngestWebhookRequest;
use sea_orm::{Database, TransactionTrait};
use tonic::Request;

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_webhook_triggers_capture_payment() {
    use core_db_entities::entity::payment_intents;
    use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait};

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    // Seed a minimal payment_intent row that ingest_webhook can resolve by razorpay_order_id.
    let razorpay_order_id = format!(
        "order_{}",
        std::time::SystemTime::now().elapsed().unwrap().as_millis()
    );
    let intent = payment_intents::ActiveModel {
        intent_id: ActiveValue::NotSet,
        razorpay_order_id: ActiveValue::Set(razorpay_order_id.clone()),
        order_id: ActiveValue::Set(None),
        user_id: ActiveValue::Set(None),
        amount_paise: ActiveValue::Set(10_000), // ₹100.00
        currency: ActiveValue::Set(Some("INR".to_string())),
        status: ActiveValue::Set(Some(
            core_db_entities::entity::sea_orm_active_enums::Status::Pending,
        )),
        razorpay_payment_id: ActiveValue::Set(None),
        metadata: ActiveValue::Set(None),
        created_at: ActiveValue::Set(None),
        expires_at: ActiveValue::Set(chrono::Utc::now()),
    };
    let inserted_intent = intent
        .insert(&txn)
        .await
        .expect("insert payment_intent should succeed");

    // Craft a minimal Razorpay-like webhook payload that matches the intent.
    let payment_id = format!(
        "pay_{}",
        std::time::SystemTime::now().elapsed().unwrap().as_millis()
    );
    let payload = serde_json::json!({
        "event": "payment.captured",
        "payload": {
            "payment": {
                "entity": {
                    "id": payment_id,
                    "order_id": razorpay_order_id,
                }
            }
        }
    });

    let req = Request::new(IngestWebhookRequest {
        provider: "razorpay".to_string(),
        event_type: "payment.captured".to_string(),
        webhook_id: format!("razorpay:{}", payment_id),
        payload_json: payload.to_string(),
        signature_verified: true,
    });

    let result = core_operations::handlers::webhooks::ingest_webhook(&txn, req).await;

    // Always roll back so this test remains non-destructive.
    txn.rollback().await.ok();

    assert!(
        result.is_ok(),
        "ingest_webhook should succeed for valid payment.captured: {:?}",
        result.err()
    );

    // Verify that capture_payment was triggered by checking that the intent now
    // has the gateway payment id attached and status processed.
    let updated_intent = payment_intents::Entity::find_by_id(inserted_intent.intent_id)
        .one(&db)
        .await
        .expect("re-query payment_intent")
        .expect("payment_intent should exist");

    assert_eq!(
        updated_intent.razorpay_payment_id.as_deref(),
        Some(payment_id.as_str()),
        "capture_payment should set razorpay_payment_id on the intent"
    );
    assert_eq!(
        updated_intent.status,
        Some(core_db_entities::entity::sea_orm_active_enums::Status::Processed),
        "capture_payment should mark intent as processed"
    );
}

