//! One monotonic refund transition for provider submissions, GETs and webhooks.
use super::create_refund;
use crate::handlers::db_errors::map_db_error_to_status as db_err;
use core_db_entities::entity::sea_orm_active_enums::Status as DbStatus;
use core_db_entities::entity::{orders, payment_intents, refund_attempts, refunds};
use sea_orm::{
    sea_query::LockType, ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait,
    DatabaseTransaction, DbBackend, EntityTrait, IntoActiveModel, QueryFilter, QuerySelect,
    Statement,
};
use tonic::Status;

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Pending,
    Processed,
    Failed,
    Review,
}

async fn review(txn: &DatabaseTransaction, order: i64, id: &str) -> Result<Outcome, Status> {
    txn.execute(Statement::from_sql_and_values(DbBackend::MySql,
        "UPDATE RefundEvidenceV2 SET needs_review=TRUE,updated_at=UTC_TIMESTAMP(6) WHERE gateway_refund_id=?",[id.into()])).await.map_err(db_err)?;
    txn.execute(Statement::from_sql_and_values(
        DbBackend::MySql,
        "UPDATE Orders SET refund_settlement_status='refund_failed' WHERE OrderID=?",
        [order.into()],
    ))
    .await
    .map_err(db_err)?;
    txn.execute(Statement::from_sql_and_values(DbBackend::MySql,
        "UPDATE RefundAttempts SET status='needs_review',provider_error='conflicting provider evidence' WHERE order_id=? AND gateway_refund_id=?",
        [order.into(),id.into()])).await.map_err(db_err)?;
    tracing::error!(
        order_id = order,
        refund_id = id,
        "refund evidence conflict; operator reconciliation required"
    );
    Ok(Outcome::Review)
}

pub async fn apply(
    txn: &DatabaseTransaction,
    order_id: i64,
    payment: &str,
    id: &str,
    amount: i64,
    currency: &str,
    status: &str,
) -> Result<Outcome, Status> {
    if id.is_empty() || payment.is_empty() || amount <= 0 || amount > i32::MAX as i64 {
        return Err(Status::invalid_argument("Incomplete refund association"));
    }
    if !matches!(status, "pending" | "processed" | "failed") {
        return Err(Status::failed_precondition(
            "Unknown refund status; reconciliation required",
        ));
    }
    let order = orders::Entity::find_by_id(order_id)
        .lock(LockType::Update)
        .one(txn)
        .await
        .map_err(db_err)?
        .ok_or_else(|| Status::not_found("Refund order not found"))?;
    let intent = payment_intents::Entity::find()
        .filter(payment_intents::Column::OrderId.eq(order_id))
        .filter(payment_intents::Column::RazorpayPaymentId.eq(payment))
        .lock(LockType::Update)
        .one(txn)
        .await
        .map_err(db_err)?
        .ok_or_else(|| {
            Status::failed_precondition(
                "Refund payment association missing; reconciliation required",
            )
        })?;
    if intent.user_id != Some(order.user_id)
        || intent.currency.as_deref() != Some(currency)
        || order.currency.as_deref() != Some(currency)
        || amount > i64::from(intent.amount_paise)
        || amount > order.grand_total_minor
        || intent.status != DbStatus::Processed
    {
        return Err(Status::failed_precondition(
            "Refund payment/order/amount/currency mismatch; reconciliation required",
        ));
    }
    txn.execute(Statement::from_sql_and_values(DbBackend::MySql,
        "INSERT INTO RefundEvidenceV2(gateway_refund_id,payment_id,order_id,amount_paise,currency,provider_status) VALUES (?,?,?,?,?,?) ON DUPLICATE KEY UPDATE gateway_refund_id=gateway_refund_id",
        [id.into(),payment.into(),order_id.into(),amount.into(),currency.into(),status.into()])).await.map_err(db_err)?;
    let row = txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::MySql,
            "SELECT * FROM RefundEvidenceV2 WHERE gateway_refund_id=? FOR UPDATE",
            [id.into()],
        ))
        .await
        .map_err(db_err)?
        .unwrap();
    let old_payment: String = row
        .try_get("", "payment_id")
        .map_err(|e| Status::internal(e.to_string()))?;
    let old_order: i64 = row
        .try_get("", "order_id")
        .map_err(|e| Status::internal(e.to_string()))?;
    let old_amount: i64 = row
        .try_get("", "amount_paise")
        .map_err(|e| Status::internal(e.to_string()))?;
    let old_currency: String = row
        .try_get("", "currency")
        .map_err(|e| Status::internal(e.to_string()))?;
    let old_status: String = row
        .try_get("", "provider_status")
        .map_err(|e| Status::internal(e.to_string()))?;
    let needs_review: i8 = row
        .try_get("", "needs_review")
        .map_err(|e| Status::internal(e.to_string()))?;
    if old_payment != payment
        || old_order != order_id
        || old_amount != amount
        || old_currency != currency
        || needs_review != 0
    {
        return review(txn, order_id, id).await;
    }
    let existing = refunds::Entity::find()
        .filter(refunds::Column::GatewayRefundId.eq(id))
        .lock(LockType::Update)
        .one(txn)
        .await
        .map_err(db_err)?;
    if let Some(ref r) = existing {
        if r.order_id != order_id
            || i64::from(r.amount_paise) != amount
            || r.currency.as_deref() != Some(currency)
        {
            return review(txn, order_id, id).await;
        }
    }
    let was_processed = old_status == "processed"
        || existing
            .as_ref()
            .is_some_and(|r| r.status == Some(DbStatus::Processed));
    let was_failed = old_status == "failed"
        || existing
            .as_ref()
            .is_some_and(|r| r.status == Some(DbStatus::Failed));
    if (was_processed && status == "failed") || (was_failed && status == "processed") {
        return review(txn, order_id, id).await;
    }
    let effective = if was_processed {
        "processed"
    } else if was_failed {
        "failed"
    } else {
        status
    };
    if effective == "processed" {
        create_refund(
            txn,
            tonic::Request::new(proto::proto::core::CreateRefundRequest {
                order_id,
                gateway_refund_id: id.into(),
                amount_paise: amount,
                currency: Some(currency.into()),
                line_items_refunded_json: None,
            }),
        )
        .await?;
    } else if let Some(row) = existing {
        let mut active = row.into_active_model();
        active.status = ActiveValue::Set(Some(if effective == "failed" {
            DbStatus::Failed
        } else {
            DbStatus::Pending
        }));
        active.update(txn).await.map_err(db_err)?;
    } else {
        refunds::ActiveModel {
            refund_id: ActiveValue::NotSet,
            order_id: ActiveValue::Set(order_id),
            gateway_refund_id: ActiveValue::Set(id.into()),
            amount_paise: ActiveValue::Set(amount as i32),
            currency: ActiveValue::Set(Some(currency.into())),
            status: ActiveValue::Set(Some(if effective == "failed" {
                DbStatus::Failed
            } else {
                DbStatus::Pending
            })),
            line_items_refunded: ActiveValue::Set(None),
            created_at: ActiveValue::Set(Some(chrono::Utc::now())),
        }
        .insert(txn)
        .await
        .map_err(db_err)?;
    }
    txn.execute(Statement::from_sql_and_values(DbBackend::MySql,
        "UPDATE RefundEvidenceV2 SET provider_status=?,updated_at=UTC_TIMESTAMP(6) WHERE gateway_refund_id=?",
        [effective.into(),id.into()])).await.map_err(db_err)?;
    // Prefer the exact provider ID. An unbound attempt can only be associated when
    // there is precisely one matching open request; never guess among partials.
    let mut attempts = refund_attempts::Entity::find()
        .filter(refund_attempts::Column::OrderId.eq(order_id))
        .filter(refund_attempts::Column::GatewayRefundId.eq(id))
        .lock(LockType::Update)
        .all(txn)
        .await
        .map_err(db_err)?;
    if attempts.is_empty() {
        attempts = refund_attempts::Entity::find()
            .filter(refund_attempts::Column::OrderId.eq(order_id))
            .filter(refund_attempts::Column::RazorpayPaymentId.eq(payment))
            .filter(refund_attempts::Column::AmountSentToGatewayPaise.eq(amount))
            .filter(refund_attempts::Column::GatewayRefundId.is_null())
            .filter(refund_attempts::Column::Status.is_in([
                "pending_external",
                "submitting",
                "submitted",
                "needs_review",
            ]))
            .lock(LockType::Update)
            .all(txn)
            .await
            .map_err(db_err)?;
    }
    if attempts.len() > 1 {
        return review(txn, order_id, id).await;
    }
    for attempt in attempts {
        let attempt_id = attempt.attempt_id;
        let mut active = attempt.into_active_model();
        active.gateway_refund_id = ActiveValue::Set(Some(id.into()));
        active.status = ActiveValue::Set(
            match effective {
                "processed" => "processed",
                "failed" => "needs_review",
                _ => "submitted",
            }
            .into(),
        );
        active.provider_error = ActiveValue::Set(None);
        active.updated_at = ActiveValue::Set(chrono::Utc::now());
        active.update(txn).await.map_err(db_err)?;
        if let Some(ret) =
            crate::handlers::returns::find_return_by_refund_attempt(txn, attempt_id).await?
        {
            crate::handlers::returns::set_return_status_and_items(
                txn,
                ret.return_id,
                if effective == "processed" {
                    "refunded"
                } else {
                    "refund_pending"
                },
            )
            .await?;
        }
    }
    txn.execute(Statement::from_sql_and_values(DbBackend::MySql,
        "UPDATE Orders SET refund_settlement_status=CASE WHEN ?='failed' THEN 'refund_failed' WHEN ?='pending' OR EXISTS(SELECT 1 FROM Refunds WHERE order_id=? AND status='pending') OR EXISTS(SELECT 1 FROM RefundAttempts WHERE order_id=? AND status IN ('pending_external','submitting','submitted','needs_review')) THEN 'refund_pending' ELSE 'refund_processed' END WHERE OrderID=?",
        [effective.into(),effective.into(),order_id.into(),order_id.into(),order_id.into()])).await.map_err(db_err)?;
    Ok(match effective {
        "processed" => Outcome::Processed,
        "failed" => Outcome::Failed,
        _ => Outcome::Pending,
    })
}
