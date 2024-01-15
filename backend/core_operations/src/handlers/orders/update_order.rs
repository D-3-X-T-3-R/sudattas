use crate::handlers::db_errors::map_db_error_to_status;
use chrono::Utc;
use core_db_entities::entity::orders;
use proto::proto::core::{OrderResponse, OrdersResponse, UpdateOrderRequest};
use rust_decimal::{
    prelude::{FromPrimitive, ToPrimitive},
    Decimal,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn update_order(
    txn: &DatabaseTransaction,
    request: Request<UpdateOrderRequest>,
) -> Result<Response<OrdersResponse>, Status> {
    let req = request.into_inner();

    let orders = orders::ActiveModel {
        order_id: ActiveValue::Set(req.order_id),
        user_id: ActiveValue::Set(req.user_id),
        order_date: ActiveValue::Set(Utc::now()),
        shipping_address_id: ActiveValue::Set(req.shipping_address_id),
        total_amount: ActiveValue::Set(Decimal::from_f64(req.total_amount).unwrap()),
        status_id: ActiveValue::Set(req.status_id),
    };
    match orders.update(txn).await {
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
