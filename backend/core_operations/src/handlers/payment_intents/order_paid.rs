use crate::handlers::db_errors::map_db_error_to_status;
use crate::order_state_machine::{self, OrderState};
use chrono::Utc;
use core_db_entities::entity::sea_orm_active_enums::PaymentStatus;
use core_db_entities::entity::{coupon_redemptions, coupons, orders};
use sea_orm::DbErr;
use sea_orm::{
    sea_query::LockType, ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait,
    DatabaseTransaction, EntityTrait, QueryFilter, QuerySelect, Statement,
};
use tonic::Status as TonicStatus;

fn is_duplicate_redemption(err: &DbErr) -> bool {
    match err {
        DbErr::Exec(exec) => {
            let message = exec.to_string();
            message.contains("Duplicate entry") || message.contains("1062")
        }
        _ => false,
    }
}

pub async fn finalize_order_paid(
    txn: &DatabaseTransaction,
    order_id: i64,
    event_type: &str,
    actor_type: &str,
    message: &str,
) -> Result<(), TonicStatus> {
    let order = orders::Entity::find_by_id(order_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| TonicStatus::not_found(format!("Order {} not found", order_id)))?;
    let coupon_id = order.applied_coupon_id;

    if let Some(coupon_id) = coupon_id {
        coupons::Entity::find_by_id(coupon_id)
            .lock(LockType::Update)
            .one(txn)
            .await
            .map_err(map_db_error_to_status)?
            .ok_or_else(|| TonicStatus::not_found(format!("Coupon {} not found", coupon_id)))?;
    }

    order_state_machine::transition_order_status(
        txn,
        order_id,
        OrderState::Paid,
        event_type,
        actor_type,
        Some(message),
        Some(PaymentStatus::Captured),
    )
    .await?;

    if let Some(coupon_id) = coupon_id {
        let redemption = coupon_redemptions::ActiveModel {
            redemption_id: ActiveValue::NotSet,
            coupon_id: ActiveValue::Set(coupon_id),
            user_id: ActiveValue::Set(order.user_id),
            order_id: ActiveValue::Set(order_id),
            redeemed_at: ActiveValue::Set(Some(Utc::now())),
        };
        match redemption.insert(txn).await {
            Ok(_) => {}
            Err(err) if is_duplicate_redemption(&err) => {}
            Err(err) => return Err(map_db_error_to_status(err)),
        }

        let usage_update = txn
            .execute(Statement::from_sql_and_values(
                sea_orm::DbBackend::MySql,
                r#"UPDATE Coupons
                   SET usage_count = COALESCE(usage_count, 0) + 1
                   WHERE coupon_id = ?
                     AND (usage_limit IS NULL OR COALESCE(usage_count, 0) < usage_limit)"#,
                [coupon_id.into()],
            ))
            .await
            .map_err(map_db_error_to_status)?;
        if usage_update.rows_affected() == 0 {
            coupon_redemptions::Entity::delete_many()
                .filter(coupon_redemptions::Column::CouponId.eq(coupon_id))
                .filter(coupon_redemptions::Column::OrderId.eq(order_id))
                .exec(txn)
                .await
                .map_err(map_db_error_to_status)?;
            order_state_machine::transition_order_status(
                txn,
                order_id,
                OrderState::NeedsReview,
                "coupon_usage_limit_contended",
                "system",
                Some("Coupon usage limit exhausted during payment finalization"),
                Some(PaymentStatus::NeedsReview),
            )
            .await?;
        }
    }

    crate::observability::log_operational_event(
        "payment_finalized_paid",
        &[("order_id", order_id.to_string())],
    );

    Ok(())
}
