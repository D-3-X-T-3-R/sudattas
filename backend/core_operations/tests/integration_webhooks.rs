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
use sea_orm::{ColumnTrait, ConnectionTrait, Database, QueryFilter, TransactionTrait};
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

    // Craft a minimal Razorpay-like webhook payload that matches the intent (Phase 5: amount + currency).
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
                    "amount": 10_000,
                    "currency": "INR"
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

    assert!(
        result.is_ok(),
        "ingest_webhook should succeed for valid payment.captured: {:?}",
        result.err()
    );

    // Verify within the transaction that capture_payment was triggered (intent updated).
    let updated_intent = payment_intents::Entity::find_by_id(inserted_intent.intent_id)
        .one(&txn)
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

    // Roll back so this test remains non-destructive.
    txn.rollback().await.ok();
}

/// Phase 5: Duplicate webhooks – same webhook_id delivered twice returns same result and does not double-apply.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_webhook_duplicate_same_webhook_id_idempotent() {
    use core_db_entities::entity::payment_intents;
    use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait};

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let razorpay_order_id = format!(
        "order_dup_{}",
        std::time::SystemTime::now().elapsed().unwrap().as_millis()
    );
    let payment_id = format!(
        "pay_dup_{}",
        std::time::SystemTime::now().elapsed().unwrap().as_millis()
    );
    let webhook_id = format!("razorpay:{}", payment_id);

    let intent = payment_intents::ActiveModel {
        intent_id: ActiveValue::NotSet,
        razorpay_order_id: ActiveValue::Set(razorpay_order_id.clone()),
        order_id: ActiveValue::Set(None),
        user_id: ActiveValue::Set(None),
        amount_paise: ActiveValue::Set(15_000),
        currency: ActiveValue::Set(Some("INR".to_string())),
        status: ActiveValue::Set(Some(
            core_db_entities::entity::sea_orm_active_enums::Status::Pending,
        )),
        razorpay_payment_id: ActiveValue::Set(None),
        metadata: ActiveValue::Set(None),
        created_at: ActiveValue::Set(None),
        expires_at: ActiveValue::Set(chrono::Utc::now()),
    };
    let inserted = intent.insert(&txn).await.expect("insert intent");

    let payload = serde_json::json!({
        "event": "payment.captured",
        "payload": {
            "payment": {
                "entity": {
                    "id": payment_id,
                    "order_id": razorpay_order_id,
                    "amount": 15_000,
                    "currency": "INR"
                }
            }
        }
    });

    let req = Request::new(IngestWebhookRequest {
        provider: "razorpay".to_string(),
        event_type: "payment.captured".to_string(),
        webhook_id: webhook_id.clone(),
        payload_json: payload.to_string(),
        signature_verified: true,
    });

    let r1 = core_operations::handlers::webhooks::ingest_webhook(&txn, req).await;
    assert!(r1.is_ok(), "first ingest should succeed: {:?}", r1.err());

    let req2 = Request::new(IngestWebhookRequest {
        provider: "razorpay".to_string(),
        event_type: "payment.captured".to_string(),
        webhook_id: webhook_id.clone(),
        payload_json: payload.to_string(),
        signature_verified: true,
    });
    let r2 = core_operations::handlers::webhooks::ingest_webhook(&txn, req2).await;
    assert!(
        r2.is_ok(),
        "second ingest (duplicate webhook_id) should succeed: {:?}",
        r2.err()
    );

    let updated = payment_intents::Entity::find_by_id(inserted.intent_id)
        .one(&txn)
        .await
        .expect("query")
        .expect("intent exists");
    assert_eq!(
        updated.razorpay_payment_id.as_deref(),
        Some(payment_id.as_str()),
        "intent should have been captured once"
    );
    assert_eq!(
        updated.status,
        Some(core_db_entities::entity::sea_orm_active_enums::Status::Processed),
        "intent should be processed"
    );

    txn.rollback().await.ok();
}

/// Phase 5: Out-of-order – same payment reported by two different webhook events (e.g. retries); second is idempotent.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_webhook_out_of_order_same_payment_second_idempotent() {
    use core_db_entities::entity::payment_intents;
    use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait};

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let razorpay_order_id = format!(
        "order_oo_{}",
        std::time::SystemTime::now().elapsed().unwrap().as_millis()
    );
    let payment_id = "pay_oo_same";

    let intent = payment_intents::ActiveModel {
        intent_id: ActiveValue::NotSet,
        razorpay_order_id: ActiveValue::Set(razorpay_order_id.clone()),
        order_id: ActiveValue::Set(None),
        user_id: ActiveValue::Set(None),
        amount_paise: ActiveValue::Set(20_000),
        currency: ActiveValue::Set(Some("INR".to_string())),
        status: ActiveValue::Set(Some(
            core_db_entities::entity::sea_orm_active_enums::Status::Pending,
        )),
        razorpay_payment_id: ActiveValue::Set(None),
        metadata: ActiveValue::Set(None),
        created_at: ActiveValue::Set(None),
        expires_at: ActiveValue::Set(chrono::Utc::now()),
    };
    let _ = intent.insert(&txn).await.expect("insert intent");

    let payload = serde_json::json!({
        "event": "payment.captured",
        "payload": {
            "payment": {
                "entity": {
                    "id": payment_id,
                    "order_id": razorpay_order_id,
                    "amount": 20_000,
                    "currency": "INR"
                }
            }
        }
    });

    let r1 = core_operations::handlers::webhooks::ingest_webhook(
        &txn,
        Request::new(IngestWebhookRequest {
            provider: "razorpay".to_string(),
            event_type: "payment.captured".to_string(),
            webhook_id: "razorpay:pay_oo_first".to_string(),
            payload_json: payload.to_string(),
            signature_verified: true,
        }),
    )
    .await;
    assert!(r1.is_ok(), "first webhook should succeed: {:?}", r1.err());

    let r2 = core_operations::handlers::webhooks::ingest_webhook(
        &txn,
        Request::new(IngestWebhookRequest {
            provider: "razorpay".to_string(),
            event_type: "payment.captured".to_string(),
            webhook_id: "razorpay:pay_oo_second".to_string(),
            payload_json: payload.to_string(),
            signature_verified: true,
        }),
    )
    .await;
    assert!(
        r2.is_ok(),
        "second webhook (same payment, different event id) should succeed: {:?}",
        r2.err()
    );

    let intents_with_payment: Vec<core_db_entities::entity::payment_intents::Model> =
        payment_intents::Entity::find()
            .filter(payment_intents::Column::RazorpayPaymentId.eq(payment_id))
            .all(&txn)
            .await
            .expect("query");
    assert_eq!(
        intents_with_payment.len(),
        1,
        "only one intent should be linked to this payment id"
    );

    txn.rollback().await.ok();
}

/// Phase 5: Amount or currency mismatch → intent and order marked NeedsReview, not paid.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_webhook_amount_mismatch_marked_needs_review_not_paid() {
    use core_db_entities::entity::payment_intents;
    use sea_orm::DbBackend;
    use sea_orm::{ActiveModelTrait, ActiveValue};

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let razorpay_order_id = format!(
        "order_mm_{}",
        std::time::SystemTime::now().elapsed().unwrap().as_millis()
    );
    let payment_id = format!(
        "pay_mm_{}",
        std::time::SystemTime::now().elapsed().unwrap().as_millis()
    );

    let intent = payment_intents::ActiveModel {
        intent_id: ActiveValue::NotSet,
        razorpay_order_id: ActiveValue::Set(razorpay_order_id.clone()),
        order_id: ActiveValue::Set(None),
        user_id: ActiveValue::Set(None),
        amount_paise: ActiveValue::Set(25_000),
        currency: ActiveValue::Set(Some("INR".to_string())),
        status: ActiveValue::Set(Some(
            core_db_entities::entity::sea_orm_active_enums::Status::Pending,
        )),
        razorpay_payment_id: ActiveValue::Set(None),
        metadata: ActiveValue::Set(None),
        created_at: ActiveValue::Set(None),
        expires_at: ActiveValue::Set(chrono::Utc::now()),
    };
    let inserted = intent.insert(&txn).await.expect("insert intent");

    let payload_wrong_amount = serde_json::json!({
        "event": "payment.captured",
        "payload": {
            "payment": {
                "entity": {
                    "id": payment_id,
                    "order_id": razorpay_order_id,
                    "amount": 10_000,
                    "currency": "INR"
                }
            }
        }
    });

    let r = core_operations::handlers::webhooks::ingest_webhook(
        &txn,
        Request::new(IngestWebhookRequest {
            provider: "razorpay".to_string(),
            event_type: "payment.captured".to_string(),
            webhook_id: format!("razorpay:{}", payment_id),
            payload_json: payload_wrong_amount.to_string(),
            signature_verified: true,
        }),
    )
    .await;
    assert!(
        r.is_ok(),
        "ingest should succeed (we mark needs_review, not fail): {:?}",
        r.err()
    );

    let row = txn
        .query_one(sea_orm::Statement::from_sql_and_values(
            DbBackend::MySql,
            "SELECT status FROM payment_intents WHERE intent_id = ?",
            [inserted.intent_id.into()],
        ))
        .await
        .expect("query");
    let status: Option<String> = row.and_then(|r| r.try_get::<String>("", "status").ok());
    assert_eq!(
        status.as_deref(),
        Some("needs_review"),
        "intent should be marked needs_review on amount mismatch"
    );

    let row2 = txn
        .query_one(sea_orm::Statement::from_sql_and_values(
            DbBackend::MySql,
            "SELECT razorpay_payment_id FROM payment_intents WHERE intent_id = ?",
            [inserted.intent_id.into()],
        ))
        .await
        .expect("query");
    let payment_id_value: Option<String> =
        row2.and_then(|r| r.try_get::<String>("", "razorpay_payment_id").ok());
    assert!(
        payment_id_value.is_none() || payment_id_value.as_deref() == Some(""),
        "payment id should not be set when we mark needs_review"
    );

    txn.rollback().await.ok();
}

/// Phase 5: Currency mismatch → NeedsReview, not paid.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_webhook_currency_mismatch_marked_needs_review_not_paid() {
    use core_db_entities::entity::payment_intents;
    use sea_orm::DbBackend;
    use sea_orm::{ActiveModelTrait, ActiveValue};

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let razorpay_order_id = format!(
        "order_cur_{}",
        std::time::SystemTime::now().elapsed().unwrap().as_millis()
    );
    let payment_id = format!(
        "pay_cur_{}",
        std::time::SystemTime::now().elapsed().unwrap().as_millis()
    );

    let intent = payment_intents::ActiveModel {
        intent_id: ActiveValue::NotSet,
        razorpay_order_id: ActiveValue::Set(razorpay_order_id.clone()),
        order_id: ActiveValue::Set(None),
        user_id: ActiveValue::Set(None),
        amount_paise: ActiveValue::Set(30_000),
        currency: ActiveValue::Set(Some("INR".to_string())),
        status: ActiveValue::Set(Some(
            core_db_entities::entity::sea_orm_active_enums::Status::Pending,
        )),
        razorpay_payment_id: ActiveValue::Set(None),
        metadata: ActiveValue::Set(None),
        created_at: ActiveValue::Set(None),
        expires_at: ActiveValue::Set(chrono::Utc::now()),
    };
    let inserted = intent.insert(&txn).await.expect("insert intent");

    let payload_wrong_currency = serde_json::json!({
        "event": "payment.captured",
        "payload": {
            "payment": {
                "entity": {
                    "id": payment_id,
                    "order_id": razorpay_order_id,
                    "amount": 30_000,
                    "currency": "USD"
                }
            }
        }
    });

    let r = core_operations::handlers::webhooks::ingest_webhook(
        &txn,
        Request::new(IngestWebhookRequest {
            provider: "razorpay".to_string(),
            event_type: "payment.captured".to_string(),
            webhook_id: format!("razorpay:{}", payment_id),
            payload_json: payload_wrong_currency.to_string(),
            signature_verified: true,
        }),
    )
    .await;
    assert!(
        r.is_ok(),
        "ingest should succeed (we mark needs_review): {:?}",
        r.err()
    );

    let row = txn
        .query_one(sea_orm::Statement::from_sql_and_values(
            DbBackend::MySql,
            "SELECT status FROM payment_intents WHERE intent_id = ?",
            [inserted.intent_id.into()],
        ))
        .await
        .expect("query");
    let status: Option<String> = row.and_then(|r| r.try_get::<String>("", "status").ok());
    assert_eq!(
        status.as_deref(),
        Some("needs_review"),
        "intent should be marked needs_review on currency mismatch"
    );

    txn.rollback().await.ok();
}
