use crate::handlers::db_errors::map_db_error_to_status;
use chrono::Utc;
use core_db_entities::entity::order_details;
use proto::proto::core::{OrderDetailResponse, OrderDetailsResponse, UpdateOrderDetailRequest};
use rust_decimal::{
    prelude::{FromPrimitive, ToPrimitive},
    Decimal,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection};
use tonic::{Request, Response, Status};

pub async fn update_order_detail(
    db: &DatabaseConnection,
    request: Request<UpdateOrderDetailRequest>,
) -> Result<Response<OrderDetailsResponse>, Status> {
    let req = request.into_inner();

    let order_details = order_details::ActiveModel {
        order_detail_id: ActiveValue::Set(req.order_detail_id),
        order_id: ActiveValue::Set(req.order_id),
        product_id: ActiveValue::Set(req.product_id),
        quantity: ActiveValue::Set(req.quantity),
        price: ActiveValue::Set(Decimal::from_f64(req.price).unwrap()),
    };
    match order_details.update(db).await {
        Ok(model) => {
            let response = OrderDetailsResponse {
                items: vec![OrderDetailResponse {
                    order_detail_id: model.order_detail_id,
                    order_id: model.order_id,
                    product_id: model.product_id,
                    quantity: model.quantity,
                    price: Decimal::to_f64(&model.price).unwrap(),
                }],
            };
            Ok(Response::new(response))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
