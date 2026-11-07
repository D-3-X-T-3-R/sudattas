//! Bounded inbox retries and read-only recovery of browser-verified payments.
use crate::handlers::db_errors::map_db_error_to_status as db_err;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement, TransactionTrait};
use tonic::Status;

pub async fn process(db: &DatabaseConnection) -> Result<(), Status> {
    let abandoned=db.execute(Statement::from_string(DbBackend::MySql,
        "UPDATE CheckoutAttemptsV2 SET state='needs_review',last_error='Abandoned provider submission or local commit; reconcile using receipt and gateway ID',updated_at=UTC_TIMESTAMP(6) WHERE state IN ('submitting','ready') AND updated_at<DATE_SUB(UTC_TIMESTAMP(6),INTERVAL 15 MINUTE)".to_owned())).await.map_err(db_err)?;
    if abandoned.rows_affected() > 0 {
        tracing::error!(
            count = abandoned.rows_affected(),
            "checkout attempts require provider reconciliation"
        );
    }
    let inbox=db.query_all(Statement::from_string(DbBackend::MySql,
        "SELECT e.event_id FROM WebhookEvents e JOIN WebhookVerificationV2 v ON v.event_id=e.event_id WHERE v.exhausted=FALSE AND v.next_attempt_at<=UTC_TIMESTAMP(6) AND (e.status='failed' OR (e.status IN ('pending','client_verified') AND e.received_at<DATE_SUB(UTC_TIMESTAMP(),INTERVAL 15 MINUTE))) ORDER BY v.next_attempt_at LIMIT 25".to_owned()))
        .await.map_err(db_err)?;
    for row in inbox {
        let id: i64 = row
            .try_get("", "event_id")
            .map_err(|e| Status::internal(e.to_string()))?;
        let txn = db.begin().await.map_err(db_err)?;
        match crate::handlers::webhooks::ingest_webhook::replay_verified_event_by_id(&txn, id).await
        {
            Ok(_) => txn.commit().await.map_err(db_err)?,
            Err(e) => {
                txn.rollback().await.map_err(db_err)?;
                tracing::warn!(error=%e,"webhook replay deferred");
            }
        }
    }
    let payments=db.query_all(Statement::from_string(DbBackend::MySql,
        "SELECT p.intent_id,p.razorpay_payment_id,p.razorpay_order_id FROM PaymentIntents p LEFT JOIN PaymentReconciliationV2 r ON r.intent_id=p.intent_id WHERE p.status='client_verified' AND p.razorpay_payment_id IS NOT NULL AND (r.intent_id IS NULL OR r.next_attempt_at<=UTC_TIMESTAMP(6)) ORDER BY p.intent_id LIMIT 25".to_owned()))
        .await.map_err(db_err)?;
    for row in payments {
        let intent: i64 = row
            .try_get("", "intent_id")
            .map_err(|e| Status::internal(e.to_string()))?;
        let payment: String = row
            .try_get("", "razorpay_payment_id")
            .map_err(|e| Status::internal(e.to_string()))?;
        let gateway: String = row
            .try_get("", "razorpay_order_id")
            .map_err(|e| Status::internal(e.to_string()))?;
        let claim=db.execute(Statement::from_sql_and_values(DbBackend::MySql,
            "INSERT INTO PaymentReconciliationV2(intent_id,attempts,next_attempt_at) VALUES (?,1,DATE_ADD(UTC_TIMESTAMP(6),INTERVAL 5 MINUTE)) ON DUPLICATE KEY UPDATE attempts=IF(next_attempt_at<=UTC_TIMESTAMP(6),attempts+1,attempts),next_attempt_at=IF(next_attempt_at<=UTC_TIMESTAMP(6),DATE_ADD(UTC_TIMESTAMP(6),INTERVAL 5 MINUTE),next_attempt_at)",
            [intent.into()])).await.map_err(db_err)?;
        if claim.rows_affected() == 0 {
            continue;
        }
        let state = db
            .query_one(Statement::from_sql_and_values(
                DbBackend::MySql,
                "SELECT attempts FROM PaymentReconciliationV2 WHERE intent_id=?",
                [intent.into()],
            ))
            .await
            .map_err(db_err)?
            .unwrap();
        let attempts: i32 = state
            .try_get("", "attempts")
            .map_err(|e| Status::internal(e.to_string()))?;
        // Network call outside a DB transaction. No create/capture/refund submission.
        let evidence = crate::razorpay::fetch_payment(&payment).await;
        if let Ok(e) = evidence {
            if e.id == payment && e.order_id == gateway && e.status == "captured" && e.captured {
                let txn = db.begin().await.map_err(db_err)?;
                match crate::handlers::payment_intents::capture_evidence::apply_captured_evidence(
                    &txn,
                    &e.order_id,
                    &e.id,
                    e.amount,
                    &e.currency,
                )
                .await
                {
                    Ok(()) => {
                        txn.commit().await.map_err(db_err)?;
                        continue;
                    }
                    Err(err) => {
                        txn.rollback().await.map_err(db_err)?;
                        tracing::warn!(intent,error=%err,"capture reconciliation deferred");
                    }
                }
            }
        }
        if attempts >= 12 {
            db.execute(Statement::from_sql_and_values(DbBackend::MySql,
                "UPDATE PaymentIntents SET status='needs_review' WHERE intent_id=? AND status='client_verified'",[intent.into()])).await.map_err(db_err)?;
            tracing::error!(
                intent,
                "payment confirmation exhausted; operator reconciliation required"
            );
        }
    }
    Ok(())
}
