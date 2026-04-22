use crate::handlers::db_errors::map_db_error_to_status;
use crate::handlers::orders::order_response;
use crate::handlers::orders::public_order_ref::{
    generate_public_order_ref_candidate, is_duplicate_key_error,
};
use crate::money::paise_to_decimal;
use chrono::Utc;
use core_db_entities::entity::orders;
use core_db_entities::entity::sea_orm_active_enums::FulfillmentStatus;
use proto::proto::core::{CreateOrderRequest, OrdersResponse};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ConnectionTrait, DatabaseTransaction, DbBackend, Statement,
};
use tonic::{Request, Response, Status};

pub async fn create_order(
    txn: &DatabaseTransaction,
    request: Request<CreateOrderRequest>,
) -> Result<Response<OrdersResponse>, Status> {
    let CreateOrderRequest {
        user_id,
        shipping_address_id,
        status_id,
        total_amount_paise,
        subtotal_minor,
        shipping_minor,
        tax_total_minor,
        discount_total_minor,
        grand_total_minor,
        applied_coupon_id,
        applied_coupon_code,
        applied_discount_paise,
    } = request.into_inner();

    let order_date = Utc::now();

    for attempt in 0u8..32 {
        let public_order_ref = generate_public_order_ref_candidate(order_date);
        let order = orders::ActiveModel {
            order_id: ActiveValue::NotSet,
            user_id: ActiveValue::Set(user_id),
            order_date: ActiveValue::Set(order_date),
            created_at: ActiveValue::Set(order_date),
            cancel_window_ends_at: ActiveValue::NotSet,
            earliest_booking_at: ActiveValue::NotSet,
            pickup_target_at: ActiveValue::NotSet,
            pickup_target_reason: ActiveValue::NotSet,
            pickup_target_set_by: ActiveValue::NotSet,
            pickup_target_updated_at: ActiveValue::NotSet,
            shipping_address_id: ActiveValue::Set(shipping_address_id),
            total_amount: ActiveValue::Set(Some(paise_to_decimal(total_amount_paise))),
            status_id: ActiveValue::Set(status_id),
            order_number: ActiveValue::NotSet,
            public_order_ref: ActiveValue::Set(public_order_ref),
            payment_status: ActiveValue::NotSet,
            payment_method: ActiveValue::NotSet,
            currency: ActiveValue::NotSet,
            updated_at: ActiveValue::NotSet,
            subtotal_minor: subtotal_minor
                .map(ActiveValue::Set)
                .unwrap_or(ActiveValue::NotSet),
            items_total_minor_before_discount: ActiveValue::NotSet,
            shipping_minor: shipping_minor
                .map(ActiveValue::Set)
                .unwrap_or(ActiveValue::NotSet)
                .into(),
            shipping_charge_minor: ActiveValue::NotSet,
            tax_total_minor: tax_total_minor
                .map(ActiveValue::Set)
                .unwrap_or(ActiveValue::NotSet)
                .into(),
            discount_total_minor: discount_total_minor
                .map(ActiveValue::Set)
                .unwrap_or(ActiveValue::NotSet)
                .into(),
            items_total_minor_after_discount: ActiveValue::NotSet,
            grand_total_minor: grand_total_minor
                .map(ActiveValue::Set)
                .unwrap_or(ActiveValue::NotSet),
            applied_coupon_id: applied_coupon_id
                .map(ActiveValue::Set)
                .unwrap_or(ActiveValue::NotSet)
                .into(),
            applied_coupon_code: applied_coupon_code
                .clone()
                .map(ActiveValue::Set)
                .unwrap_or(ActiveValue::NotSet)
                .into(),
            applied_discount_paise: applied_discount_paise
                .map(ActiveValue::Set)
                .unwrap_or(ActiveValue::NotSet)
                .into(),
            refund_settlement_status: ActiveValue::NotSet,
            fulfillment_status: ActiveValue::Set(FulfillmentStatus::NotCreated),
        };

        match order.insert(txn).await {
            Ok(model) => {
                txn.execute(Statement::from_sql_and_values(
                    DbBackend::MySql,
                    r#"UPDATE Orders
                       SET cancel_window_ends_at = COALESCE(
                               cancel_window_ends_at,
                               DATE_ADD(created_at, INTERVAL ? HOUR)
                           ),
                           earliest_booking_at = COALESCE(
                               earliest_booking_at,
                               COALESCE(cancel_window_ends_at, DATE_ADD(created_at, INTERVAL ? HOUR))
                           ),
                           pickup_target_at = COALESCE(
                               pickup_target_at,
                               DATE_ADD(created_at, INTERVAL ? HOUR)
                           ),
                           pickup_target_set_by = COALESCE(pickup_target_set_by, 'system'),
                           pickup_target_reason = COALESCE(pickup_target_reason, 'order_created'),
                           pickup_target_updated_at = COALESCE(pickup_target_updated_at, UTC_TIMESTAMP())
                       WHERE OrderID = ?"#,
                    [
                        crate::order_policy::cancel_window_hours().into(),
                        crate::order_policy::cancel_window_hours().into(),
                        crate::order_policy::pickup_delay_hours().into(),
                        model.order_id.into(),
                    ],
                ))
                .await
                .map_err(map_db_error_to_status)?;
                let response = OrdersResponse {
                    items: vec![order_response::from_model(&model)],
                };
                return Ok(Response::new(response));
            }
            Err(e) if is_duplicate_key_error(&e) && attempt < 31 => {
                continue;
            }
            Err(e) => return Err(map_db_error_to_status(e)),
        }
    }

    Err(Status::internal(
        "exhausted retries generating a unique public_order_ref",
    ))
}
