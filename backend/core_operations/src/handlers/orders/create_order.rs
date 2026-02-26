use crate::handlers::db_errors::map_db_error_to_status;
use chrono::Utc;
use core_db_entities::entity::orders;
use proto::proto::core::{CreateOrderRequest, OrderResponse, OrdersResponse};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_order(
    txn: &DatabaseTransaction,
    request: Request<CreateOrderRequest>,
) -> Result<Response<OrdersResponse>, Status> {
    let req = request.into_inner();

    let order = orders::ActiveModel {
        order_id: ActiveValue::NotSet,
        user_id: ActiveValue::Set(req.user_id),
        order_date: ActiveValue::Set(Utc::now()),
        shipping_address_id: ActiveValue::Set(req.shipping_address_id),
        total_amount: ActiveValue::Set(Decimal::from_f64(req.total_amount).unwrap()),
        status_id: ActiveValue::Set(req.status_id),
        order_number: ActiveValue::NotSet,
        payment_status: ActiveValue::NotSet,
        currency: ActiveValue::NotSet,
        updated_at: ActiveValue::NotSet,
        subtotal_minor: req
            .subtotal_minor
            .map(ActiveValue::Set)
            .unwrap_or(ActiveValue::NotSet)
            .into(),
        shipping_minor: req
            .shipping_minor
            .map(ActiveValue::Set)
            .unwrap_or(ActiveValue::NotSet)
            .into(),
        tax_total_minor: req
            .tax_total_minor
            .map(ActiveValue::Set)
            .unwrap_or(ActiveValue::NotSet)
            .into(),
        discount_total_minor: req
            .discount_total_minor
            .map(ActiveValue::Set)
            .unwrap_or(ActiveValue::NotSet)
            .into(),
        grand_total_minor: req
            .grand_total_minor
            .map(ActiveValue::Set)
            .unwrap_or(ActiveValue::NotSet)
            .into(),
        applied_coupon_id: req
            .applied_coupon_id
            .map(ActiveValue::Set)
            .unwrap_or(ActiveValue::NotSet)
            .into(),
        applied_coupon_code: req
            .applied_coupon_code
            .map(ActiveValue::Set)
            .unwrap_or(ActiveValue::NotSet)
            .into(),
        applied_discount_paise: req
            .applied_discount_paise
            .map(ActiveValue::Set)
            .unwrap_or(ActiveValue::NotSet)
            .into(),
    };
    match order.insert(txn).await {
        Ok(model) => {
            let response = OrdersResponse {
                items: vec![OrderResponse {
                    order_id: model.order_id,
                    user_id: model.user_id,
                    order_date: model.order_date.to_string(),
                    shipping_address_id: model.shipping_address_id,
                    total_amount: Decimal::to_f64(&model.total_amount).unwrap(),
                    status_id: model.status_id,
                }],
            };
            Ok(Response::new(response))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
