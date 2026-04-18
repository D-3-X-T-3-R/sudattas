use crate::handlers::db_errors::map_db_error_to_status;
use crate::handlers::orders::delete_order;
use chrono::Utc;
use core_db_entities::entity::sea_orm_active_enums::Status as PaymentIntentStatus;
use core_db_entities::entity::{order_status, payment_intents};
use proto::proto::core::DeleteOrderRequest;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, QueryFilter, Statement, TransactionTrait,
};
use tonic::{Request, Status};

async fn claim_expired_pending_intent_ids(
    txn: &sea_orm::DatabaseTransaction,
    batch_limit: u64,
) -> Result<Vec<i64>, Status> {
    let rows = txn
        .query_all(Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            r#"SELECT intent_id
               FROM PaymentIntents
               WHERE status = 'pending'
                 AND expires_at < ?
                 AND order_id IS NOT NULL
               ORDER BY expires_at ASC
               LIMIT ?
               FOR UPDATE SKIP LOCKED"#,
            [
                Utc::now().into(),
                i64::try_from(batch_limit).unwrap_or(i64::MAX).into(),
            ],
        ))
        .await
        .map_err(map_db_error_to_status)?;

    let mut ids = Vec::with_capacity(rows.len());
    for row in rows {
        if let Ok(intent_id) = row.try_get::<i64>("", "intent_id") {
            ids.push(intent_id);
        }
    }
    Ok(ids)
}

pub async fn expire_stale_pending_orders(
    db: &DatabaseConnection,
    batch_limit: u64,
) -> Result<u64, Status> {
    let txn = db.begin().await.map_err(map_db_error_to_status)?;
    let pending_status_id = order_status::Entity::find()
        .filter(order_status::Column::StatusName.eq("pending"))
        .one(&txn)
        .await
        .map_err(map_db_error_to_status)?
        .map(|row| row.status_id)
        .ok_or_else(|| Status::internal("OrderStatus 'pending' not found"))?;

    let claimed_intent_ids = claim_expired_pending_intent_ids(&txn, batch_limit).await?;
    if claimed_intent_ids.is_empty() {
        txn.commit().await.map_err(map_db_error_to_status)?;
        return Ok(0);
    }

    let expired_intents = payment_intents::Entity::find()
        .filter(payment_intents::Column::IntentId.is_in(claimed_intent_ids))
        .all(&txn)
        .await
        .map_err(map_db_error_to_status)?;

    let mut expired = 0_u64;
    for intent in expired_intents {
        let Some(order_id) = intent.order_id else {
            continue;
        };
        let order = core_db_entities::entity::orders::Entity::find_by_id(order_id)
            .one(&txn)
            .await
            .map_err(map_db_error_to_status)?;
        let Some(order) = order else {
            let mut active: payment_intents::ActiveModel = intent.into_active_model();
            active.status = ActiveValue::Set(PaymentIntentStatus::Failed);
            active.update(&txn).await.map_err(map_db_error_to_status)?;
            expired += 1;
            continue;
        };
        if order.status_id != pending_status_id {
            let mut active: payment_intents::ActiveModel = intent.into_active_model();
            active.status = ActiveValue::Set(PaymentIntentStatus::Failed);
            active.update(&txn).await.map_err(map_db_error_to_status)?;
            expired += 1;
            continue;
        }

        delete_order(
            &txn,
            Request::new(DeleteOrderRequest {
                order_id,
                acting_user_id: None,
            }),
        )
        .await?;

        let mut active: payment_intents::ActiveModel = intent.into_active_model();
        active.status = ActiveValue::Set(PaymentIntentStatus::Failed);
        active.update(&txn).await.map_err(map_db_error_to_status)?;
        expired += 1;
    }

    txn.commit().await.map_err(map_db_error_to_status)?;
    Ok(expired)
}
