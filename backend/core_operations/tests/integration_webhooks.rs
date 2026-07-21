//! Integration tests for webhook ingestion and payment capture.
//!
//! **Setup**
//! - Set `TEST_DATABASE_URL` or `DATABASE_URL` (e.g. `mysql://root:test_password@127.0.0.1:3306/sudattas_test`).
//! - Schema must be loaded first (e.g. migrations or `backend/database/sql_dump/01_schema.sql`).
//!
//! **Run**
//! - `cargo test --test integration_webhooks -- --ignored`

mod integration_common;
mod provider_test_gate;

use integration_common::test_db_url;
use proto::proto::core::IngestWebhookRequest;
use sea_orm::{ColumnTrait, ConnectionTrait, Database, QueryFilter, TransactionTrait};
use tonic::Request;

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_webhook_triggers_capture_payment() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_webhook_triggers_capture_payment",
    ) {
        return;
    }

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
        active_order_id: ActiveValue::NotSet,
        user_id: ActiveValue::Set(None),
        amount_paise: ActiveValue::Set(10_000), // ₹100.00
        currency: ActiveValue::Set(Some("INR".to_string())),
        status: ActiveValue::Set(core_db_entities::entity::sea_orm_active_enums::Status::Pending),
        razorpay_payment_id: ActiveValue::Set(None),
        metadata: ActiveValue::Set(None),
        gateway_fee_paise: ActiveValue::Set(None),
        gateway_tax_paise: ActiveValue::Set(None),
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
        provider_event_id: None,
        raw_signature: None,
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
        core_db_entities::entity::sea_orm_active_enums::Status::Processed,
        "capture_payment should mark intent as processed"
    );

    // Roll back so this test remains non-destructive.
    txn.rollback().await.ok();
}

/// Phase 5: Duplicate webhooks – same webhook_id delivered twice returns same result and does not double-apply.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_webhook_duplicate_same_webhook_id_idempotent() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_webhook_duplicate_same_webhook_id_idempotent",
    ) {
        return;
    }

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
        active_order_id: ActiveValue::NotSet,
        user_id: ActiveValue::Set(None),
        amount_paise: ActiveValue::Set(15_000),
        currency: ActiveValue::Set(Some("INR".to_string())),
        status: ActiveValue::Set(core_db_entities::entity::sea_orm_active_enums::Status::Pending),
        razorpay_payment_id: ActiveValue::Set(None),
        metadata: ActiveValue::Set(None),
        gateway_fee_paise: ActiveValue::Set(None),
        gateway_tax_paise: ActiveValue::Set(None),
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
        provider_event_id: None,
        raw_signature: None,
    });

    let r1 = core_operations::handlers::webhooks::ingest_webhook(&txn, req).await;
    assert!(r1.is_ok(), "first ingest should succeed: {:?}", r1.err());
    let first_event = r1
        .expect("already checked is_ok")
        .into_inner()
        .items
        .into_iter()
        .next()
        .expect("first webhook response item");
    assert_eq!(first_event.status, "processed");

    let req2 = Request::new(IngestWebhookRequest {
        provider: "razorpay".to_string(),
        event_type: "payment.captured".to_string(),
        webhook_id: webhook_id.clone(),
        payload_json: payload.to_string(),
        signature_verified: true,
        provider_event_id: None,
        raw_signature: None,
    });
    let r2 = core_operations::handlers::webhooks::ingest_webhook(&txn, req2).await;
    assert!(
        r2.is_ok(),
        "second ingest (duplicate webhook_id) should succeed: {:?}",
        r2.err()
    );
    let second_event = r2
        .expect("already checked is_ok")
        .into_inner()
        .items
        .into_iter()
        .next()
        .expect("second webhook response item");
    assert_eq!(second_event.status, "processed");
    assert_eq!(
        second_event.event_id, first_event.event_id,
        "duplicate processed webhook should replay idempotently"
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
        core_db_entities::entity::sea_orm_active_enums::Status::Processed,
        "intent should be processed"
    );

    txn.rollback().await.ok();
}

/// Phase 5: Out-of-order – same payment reported by two different webhook events (e.g. retries); second is idempotent.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_webhook_out_of_order_same_payment_second_idempotent() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_webhook_out_of_order_same_payment_second_idempotent",
    ) {
        return;
    }

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
        active_order_id: ActiveValue::NotSet,
        user_id: ActiveValue::Set(None),
        amount_paise: ActiveValue::Set(20_000),
        currency: ActiveValue::Set(Some("INR".to_string())),
        status: ActiveValue::Set(core_db_entities::entity::sea_orm_active_enums::Status::Pending),
        razorpay_payment_id: ActiveValue::Set(None),
        metadata: ActiveValue::Set(None),
        gateway_fee_paise: ActiveValue::Set(None),
        gateway_tax_paise: ActiveValue::Set(None),
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
            provider_event_id: None,
            raw_signature: None,
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
            provider_event_id: None,
            raw_signature: None,
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
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_webhook_amount_mismatch_marked_needs_review_not_paid",
    ) {
        return;
    }

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
        active_order_id: ActiveValue::NotSet,
        user_id: ActiveValue::Set(None),
        amount_paise: ActiveValue::Set(25_000),
        currency: ActiveValue::Set(Some("INR".to_string())),
        status: ActiveValue::Set(core_db_entities::entity::sea_orm_active_enums::Status::Pending),
        razorpay_payment_id: ActiveValue::Set(None),
        metadata: ActiveValue::Set(None),
        gateway_fee_paise: ActiveValue::Set(None),
        gateway_tax_paise: ActiveValue::Set(None),
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
            provider_event_id: None,
            raw_signature: None,
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
            "SELECT status FROM PaymentIntents WHERE intent_id = ?",
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
            "SELECT razorpay_payment_id FROM PaymentIntents WHERE intent_id = ?",
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
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_webhook_currency_mismatch_marked_needs_review_not_paid",
    ) {
        return;
    }

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
        active_order_id: ActiveValue::NotSet,
        user_id: ActiveValue::Set(None),
        amount_paise: ActiveValue::Set(30_000),
        currency: ActiveValue::Set(Some("INR".to_string())),
        status: ActiveValue::Set(core_db_entities::entity::sea_orm_active_enums::Status::Pending),
        razorpay_payment_id: ActiveValue::Set(None),
        metadata: ActiveValue::Set(None),
        gateway_fee_paise: ActiveValue::Set(None),
        gateway_tax_paise: ActiveValue::Set(None),
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
            provider_event_id: None,
            raw_signature: None,
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
            "SELECT status FROM PaymentIntents WHERE intent_id = ?",
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

/// Phase 6: Replay dedupe by provider_event_id – duplicate processed event is idempotent no-op.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and schema with provider_event_id (run generate.ps1 after schema)"]
async fn integration_webhook_duplicate_provider_event_id_processed_noop() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_webhook_duplicate_provider_event_id_processed_noop",
    ) {
        return;
    }

    use core_db_entities::entity::webhook_events;
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let provider_event_id = format!(
        "evt_replay_{}",
        std::time::SystemTime::now().elapsed().unwrap().as_millis()
    );
    let webhook_id = format!(
        "razorpay:pay_replay_{}",
        std::time::SystemTime::now().elapsed().unwrap().as_millis()
    );
    let payload = serde_json::json!({
        "event": "payment.authorized",
        "payload": { "payment": { "entity": { "id": "pay_replay_1" } } }
    });

    let r1 = core_operations::handlers::webhooks::ingest_webhook(
        &txn,
        Request::new(IngestWebhookRequest {
            provider: "razorpay".to_string(),
            event_type: "payment.authorized".to_string(),
            webhook_id: webhook_id.clone(),
            payload_json: payload.to_string(),
            signature_verified: true,
            provider_event_id: Some(provider_event_id.clone()),
            raw_signature: None,
        }),
    )
    .await;
    assert!(
        r1.is_ok(),
        "first ingest with provider_event_id should succeed: {:?}",
        r1.err()
    );

    let r2 = core_operations::handlers::webhooks::ingest_webhook(
        &txn,
        Request::new(IngestWebhookRequest {
            provider: "razorpay".to_string(),
            event_type: "payment.authorized".to_string(),
            webhook_id: format!(
                "razorpay:pay_replay_2_{}",
                std::time::SystemTime::now().elapsed().unwrap().as_millis()
            ),
            payload_json: payload.to_string(),
            signature_verified: true,
            provider_event_id: Some(provider_event_id.clone()),
            raw_signature: None,
        }),
    )
    .await;
    assert!(
        r2.is_ok(),
        "duplicate provider_event_id should return no-op success"
    );
    let first_event = r1
        .expect("first ok")
        .into_inner()
        .items
        .into_iter()
        .next()
        .expect("first response item");
    let second_event = r2
        .expect("second ok")
        .into_inner()
        .items
        .into_iter()
        .next()
        .expect("second response item");
    assert_eq!(second_event.event_id, first_event.event_id);
    assert_eq!(second_event.status, "processed");

    let count = webhook_events::Entity::find()
        .filter(webhook_events::Column::ProviderEventId.eq(provider_event_id.as_str()))
        .all(&txn)
        .await
        .expect("query");
    assert_eq!(
        count.len(),
        1,
        "only one row should have this provider_event_id"
    );

    txn.rollback().await.ok();
}

/// Failed webhook replay with the same webhook_id should reprocess successfully once prerequisites exist.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_webhook_failed_same_webhook_id_can_replay_to_success() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_webhook_failed_same_webhook_id_can_replay_to_success",
    ) {
        return;
    }

    use core_db_entities::entity::payment_intents;
    use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait};

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let razorpay_order_id = format!(
        "order_replay_{}",
        std::time::SystemTime::now().elapsed().unwrap().as_millis()
    );
    let payment_id = format!(
        "pay_replay_{}",
        std::time::SystemTime::now().elapsed().unwrap().as_millis()
    );
    let webhook_id = format!(
        "razorpay:failed-replay:{}",
        std::time::SystemTime::now().elapsed().unwrap().as_millis()
    );
    let payload = serde_json::json!({
        "event": "payment.captured",
        "payload": {
            "payment": {
                "entity": {
                    "id": payment_id,
                    "order_id": razorpay_order_id,
                    "amount": 9_900,
                    "currency": "INR"
                }
            }
        }
    });

    let first = core_operations::handlers::webhooks::ingest_webhook(
        &txn,
        Request::new(IngestWebhookRequest {
            provider: "razorpay".to_string(),
            event_type: "payment.captured".to_string(),
            webhook_id: webhook_id.clone(),
            payload_json: payload.to_string(),
            signature_verified: true,
            provider_event_id: None,
            raw_signature: None,
        }),
    )
    .await
    .expect("first ingest should store failed event, not error")
    .into_inner()
    .items
    .into_iter()
    .next()
    .expect("first webhook response item");
    assert_eq!(first.status, "failed");

    let intent = payment_intents::ActiveModel {
        intent_id: ActiveValue::NotSet,
        razorpay_order_id: ActiveValue::Set(razorpay_order_id.clone()),
        order_id: ActiveValue::Set(None),
        active_order_id: ActiveValue::NotSet,
        user_id: ActiveValue::Set(None),
        amount_paise: ActiveValue::Set(9_900),
        currency: ActiveValue::Set(Some("INR".to_string())),
        status: ActiveValue::Set(core_db_entities::entity::sea_orm_active_enums::Status::Pending),
        razorpay_payment_id: ActiveValue::Set(None),
        metadata: ActiveValue::Set(None),
        gateway_fee_paise: ActiveValue::Set(None),
        gateway_tax_paise: ActiveValue::Set(None),
        created_at: ActiveValue::Set(None),
        expires_at: ActiveValue::Set(chrono::Utc::now()),
    };
    let inserted = intent.insert(&txn).await.expect("insert replay intent");

    let second = core_operations::handlers::webhooks::ingest_webhook(
        &txn,
        Request::new(IngestWebhookRequest {
            provider: "razorpay".to_string(),
            event_type: "payment.captured".to_string(),
            webhook_id: webhook_id.clone(),
            payload_json: payload.to_string(),
            signature_verified: true,
            provider_event_id: None,
            raw_signature: None,
        }),
    )
    .await
    .expect("replay ingest should succeed")
    .into_inner()
    .items
    .into_iter()
    .next()
    .expect("second webhook response item");
    assert_eq!(second.status, "processed");
    assert_eq!(second.event_id, first.event_id);

    let updated_intent = payment_intents::Entity::find_by_id(inserted.intent_id)
        .one(&txn)
        .await
        .expect("query updated intent")
        .expect("intent exists");
    assert_eq!(
        updated_intent.status,
        core_db_entities::entity::sea_orm_active_enums::Status::Processed
    );
    assert_eq!(
        updated_intent.razorpay_payment_id.as_deref(),
        Some(payment_id.as_str())
    );

    txn.rollback().await.ok();
}

/// Fresh pending/client_verified webhooks should not be reclaimed concurrently.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_webhook_fresh_in_progress_not_reclaimed() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_webhook_fresh_in_progress_not_reclaimed",
    ) {
        return;
    }

    use core_db_entities::entity::sea_orm_active_enums::Status as WebhookStatus;
    use core_db_entities::entity::webhook_events;
    use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait};

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let webhook_id = format!(
        "razorpay:fresh:{}",
        std::time::SystemTime::now().elapsed().unwrap().as_millis()
    );
    let seeded = webhook_events::ActiveModel {
        event_id: ActiveValue::NotSet,
        provider: ActiveValue::Set("razorpay".to_string()),
        event_type: ActiveValue::Set("noop.event".to_string()),
        webhook_id: ActiveValue::Set(webhook_id.clone()),
        provider_event_id: ActiveValue::Set(None),
        payload: ActiveValue::Set(serde_json::json!({"event":"noop.event"})),
        status: ActiveValue::Set(Some(WebhookStatus::ClientVerified)),
        received_at: ActiveValue::Set(Some(chrono::Utc::now())),
    }
    .insert(&txn)
    .await
    .expect("seed fresh row");

    let replay = core_operations::handlers::webhooks::ingest_webhook(
        &txn,
        Request::new(IngestWebhookRequest {
            provider: "razorpay".to_string(),
            event_type: "noop.event".to_string(),
            webhook_id: webhook_id.clone(),
            payload_json: serde_json::json!({"event":"noop.event"}).to_string(),
            signature_verified: true,
            provider_event_id: None,
            raw_signature: None,
        }),
    )
    .await
    .expect("fresh duplicate should no-op")
    .into_inner()
    .items
    .into_iter()
    .next()
    .expect("response item");
    assert_eq!(replay.event_id, seeded.event_id);
    assert_eq!(replay.status, "client_verified");

    let refreshed = webhook_events::Entity::find_by_id(seeded.event_id)
        .one(&txn)
        .await
        .expect("query refreshed row")
        .expect("row exists");
    assert_eq!(refreshed.status, Some(WebhookStatus::ClientVerified));

    txn.rollback().await.ok();
}

/// Stale pending/client_verified webhooks should be reclaimed and processed.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_webhook_stale_in_progress_reclaimed() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_webhook_stale_in_progress_reclaimed",
    ) {
        return;
    }

    use core_db_entities::entity::sea_orm_active_enums::Status as WebhookStatus;
    use core_db_entities::entity::webhook_events;
    use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait};

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let webhook_id = format!(
        "razorpay:stale:{}",
        std::time::SystemTime::now().elapsed().unwrap().as_millis()
    );
    let seeded = webhook_events::ActiveModel {
        event_id: ActiveValue::NotSet,
        provider: ActiveValue::Set("razorpay".to_string()),
        event_type: ActiveValue::Set("noop.event".to_string()),
        webhook_id: ActiveValue::Set(webhook_id.clone()),
        provider_event_id: ActiveValue::Set(None),
        payload: ActiveValue::Set(serde_json::json!({"event":"noop.event"})),
        status: ActiveValue::Set(Some(WebhookStatus::ClientVerified)),
        received_at: ActiveValue::Set(Some(chrono::Utc::now() - chrono::Duration::minutes(30))),
    }
    .insert(&txn)
    .await
    .expect("seed stale row");

    let replay = core_operations::handlers::webhooks::ingest_webhook(
        &txn,
        Request::new(IngestWebhookRequest {
            provider: "razorpay".to_string(),
            event_type: "noop.event".to_string(),
            webhook_id: webhook_id.clone(),
            payload_json: serde_json::json!({"event":"noop.event"}).to_string(),
            signature_verified: true,
            provider_event_id: None,
            raw_signature: None,
        }),
    )
    .await
    .expect("stale replay should succeed")
    .into_inner()
    .items
    .into_iter()
    .next()
    .expect("response item");
    assert_eq!(replay.event_id, seeded.event_id);
    assert_eq!(replay.status, "processed");

    let refreshed = webhook_events::Entity::find_by_id(seeded.event_id)
        .one(&txn)
        .await
        .expect("query refreshed row")
        .expect("row exists");
    assert_eq!(refreshed.status, Some(WebhookStatus::Processed));

    txn.rollback().await.ok();
}

/// Duplicate provider_event_id can safely retry when the first processing attempt failed.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and schema with provider_event_id (run generate.ps1 after schema)"]
async fn integration_webhook_duplicate_provider_event_id_failed_can_retry() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_webhook_duplicate_provider_event_id_failed_can_retry",
    ) {
        return;
    }

    use core_db_entities::entity::payment_intents;
    use core_db_entities::entity::webhook_events;
    use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait};

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let razorpay_order_id = format!(
        "order_peid_failed_{}",
        std::time::SystemTime::now().elapsed().unwrap().as_millis()
    );
    let payment_id = format!(
        "pay_peid_failed_{}",
        std::time::SystemTime::now().elapsed().unwrap().as_millis()
    );
    let provider_event_id = format!(
        "evt_peid_failed_{}",
        std::time::SystemTime::now().elapsed().unwrap().as_millis()
    );
    let payload = serde_json::json!({
        "event": "payment.captured",
        "payload": {
            "payment": {
                "entity": {
                    "id": payment_id,
                    "order_id": razorpay_order_id,
                    "amount": 18_250,
                    "currency": "INR"
                }
            }
        }
    });

    let first = core_operations::handlers::webhooks::ingest_webhook(
        &txn,
        Request::new(IngestWebhookRequest {
            provider: "razorpay".to_string(),
            event_type: "payment.captured".to_string(),
            webhook_id: format!("razorpay:peid-failed:first:{provider_event_id}"),
            payload_json: payload.to_string(),
            signature_verified: true,
            provider_event_id: Some(provider_event_id.clone()),
            raw_signature: None,
        }),
    )
    .await
    .expect("first ingest should store failed webhook")
    .into_inner()
    .items
    .into_iter()
    .next()
    .expect("first response item");
    assert_eq!(first.status, "failed");

    let intent = payment_intents::ActiveModel {
        intent_id: ActiveValue::NotSet,
        razorpay_order_id: ActiveValue::Set(razorpay_order_id.clone()),
        order_id: ActiveValue::Set(None),
        active_order_id: ActiveValue::NotSet,
        user_id: ActiveValue::Set(None),
        amount_paise: ActiveValue::Set(18_250),
        currency: ActiveValue::Set(Some("INR".to_string())),
        status: ActiveValue::Set(core_db_entities::entity::sea_orm_active_enums::Status::Pending),
        razorpay_payment_id: ActiveValue::Set(None),
        metadata: ActiveValue::Set(None),
        gateway_fee_paise: ActiveValue::Set(None),
        gateway_tax_paise: ActiveValue::Set(None),
        created_at: ActiveValue::Set(None),
        expires_at: ActiveValue::Set(chrono::Utc::now()),
    };
    let inserted = intent.insert(&txn).await.expect("insert replay intent");

    let second = core_operations::handlers::webhooks::ingest_webhook(
        &txn,
        Request::new(IngestWebhookRequest {
            provider: "razorpay".to_string(),
            event_type: "payment.captured".to_string(),
            webhook_id: format!("razorpay:peid-failed:second:{provider_event_id}"),
            payload_json: payload.to_string(),
            signature_verified: true,
            provider_event_id: Some(provider_event_id.clone()),
            raw_signature: None,
        }),
    )
    .await
    .expect("failed provider_event replay should succeed")
    .into_inner()
    .items
    .into_iter()
    .next()
    .expect("second response item");
    assert_eq!(second.status, "processed");
    assert_eq!(second.event_id, first.event_id);

    let rows = webhook_events::Entity::find()
        .filter(webhook_events::Column::ProviderEventId.eq(provider_event_id.as_str()))
        .all(&txn)
        .await
        .expect("query webhook rows");
    assert_eq!(rows.len(), 1);

    let updated_intent = payment_intents::Entity::find_by_id(inserted.intent_id)
        .one(&txn)
        .await
        .expect("query updated intent")
        .expect("intent exists");
    assert_eq!(
        updated_intent.status,
        core_db_entities::entity::sea_orm_active_enums::Status::Processed
    );
    assert_eq!(
        updated_intent.razorpay_payment_id.as_deref(),
        Some(payment_id.as_str())
    );

    txn.rollback().await.ok();
}

/// Internal replay-by-id path supports admin-safe reprocessing of failed events.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_webhook_internal_replay_by_id_reprocesses_failed() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_webhook_internal_replay_by_id_reprocesses_failed",
    ) {
        return;
    }

    use core_db_entities::entity::payment_intents;
    use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait};

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let razorpay_order_id = format!(
        "order_internal_replay_{}",
        std::time::SystemTime::now().elapsed().unwrap().as_millis()
    );
    let payment_id = format!(
        "pay_internal_replay_{}",
        std::time::SystemTime::now().elapsed().unwrap().as_millis()
    );
    let webhook_id = format!(
        "razorpay:internal-replay:{}",
        std::time::SystemTime::now().elapsed().unwrap().as_millis()
    );
    let payload = serde_json::json!({
        "event": "payment.captured",
        "payload": {
            "payment": {
                "entity": {
                    "id": payment_id,
                    "order_id": razorpay_order_id,
                    "amount": 21_000,
                    "currency": "INR"
                }
            }
        }
    });

    let first = core_operations::handlers::webhooks::ingest_webhook(
        &txn,
        Request::new(IngestWebhookRequest {
            provider: "razorpay".to_string(),
            event_type: "payment.captured".to_string(),
            webhook_id: webhook_id.clone(),
            payload_json: payload.to_string(),
            signature_verified: true,
            provider_event_id: None,
            raw_signature: None,
        }),
    )
    .await
    .expect("first ingest should record failed")
    .into_inner()
    .items
    .into_iter()
    .next()
    .expect("first response item");
    assert_eq!(first.status, "failed");

    let intent = payment_intents::ActiveModel {
        intent_id: ActiveValue::NotSet,
        razorpay_order_id: ActiveValue::Set(razorpay_order_id.clone()),
        order_id: ActiveValue::Set(None),
        active_order_id: ActiveValue::NotSet,
        user_id: ActiveValue::Set(None),
        amount_paise: ActiveValue::Set(21_000),
        currency: ActiveValue::Set(Some("INR".to_string())),
        status: ActiveValue::Set(core_db_entities::entity::sea_orm_active_enums::Status::Pending),
        razorpay_payment_id: ActiveValue::Set(None),
        metadata: ActiveValue::Set(None),
        gateway_fee_paise: ActiveValue::Set(None),
        gateway_tax_paise: ActiveValue::Set(None),
        created_at: ActiveValue::Set(None),
        expires_at: ActiveValue::Set(chrono::Utc::now()),
    };
    let inserted = intent.insert(&txn).await.expect("insert replay intent");

    let replay = core_operations::handlers::webhooks::ingest_webhook::replay_webhook_by_id(
        &txn,
        &webhook_id,
    )
    .await
    .expect("internal replay by id should succeed")
    .into_inner()
    .items
    .into_iter()
    .next()
    .expect("replay response item");
    assert_eq!(replay.status, "processed");
    assert_eq!(replay.event_id, first.event_id);

    let updated_intent = payment_intents::Entity::find_by_id(inserted.intent_id)
        .one(&txn)
        .await
        .expect("query updated intent")
        .expect("intent exists");
    assert_eq!(
        updated_intent.status,
        core_db_entities::entity::sea_orm_active_enums::Status::Processed
    );
    assert_eq!(
        updated_intent.razorpay_payment_id.as_deref(),
        Some(payment_id.as_str())
    );

    txn.rollback().await.ok();
}
