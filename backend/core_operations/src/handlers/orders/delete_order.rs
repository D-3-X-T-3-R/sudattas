use crate::handlers::db_errors::map_db_error_to_status;
use crate::money::decimal_to_paise;
use core_db_entities::entity::orders;
use proto::proto::core::{DeleteOrderRequest, OrderResponse, OrdersResponse};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn delete_order(
    txn: &DatabaseTransaction,
    request: Request<DeleteOrderRequest>,
) -> Result<Response<OrdersResponse>, Status> {
    let req = request.into_inner();

    let order = orders::Entity::find_by_id(req.order_id).one(txn).await;

    match order {
        Ok(Some(model)) => {
            match orders::Entity::delete_many()
                .filter(orders::Column::OrderId.eq(req.order_id))
                .exec(txn)
                .await
            {
                Ok(delete_result) => {
                    if delete_result.rows_affected > 0 {
                        let total_amount_paise = model
                            .grand_total_minor
                            .unwrap_or_else(|| decimal_to_paise(&model.total_amount));
                        let response = OrdersResponse {
                            items: vec![OrderResponse {
                                user_id: model.user_id,
                                order_id: model.order_id,
                                order_date: model.order_date.to_string(),
                                total_amount_paise,
                                status_id: model.status_id,
                                shipping_address_id: model.shipping_address_id,
                            }],
                        };
                        Ok(Response::new(response))
                    } else {
                        Err(Status::not_found(format!(
                            "Order with ID {} not found.",
                            req.order_id
                        )))
                    }
                }
                Err(e) => Err(map_db_error_to_status(e)),
            }
        }
        Ok(None) => Err(Status::not_found(format!(
            "Order with ID {} not found.",
            req.order_id
        ))),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
