// SearchOrderRequest Proto message
use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::orders;
use proto::proto::core::{OrderResponse, OrdersResponse, SearchOrderRequest};
use rust_decimal::prelude::ToPrimitive;
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryTrait};
use tonic::{Request, Response, Status};

pub async fn search_order(
    txn: &DatabaseTransaction,
    request: Request<SearchOrderRequest>,
) -> Result<Response<OrdersResponse>, Status> {
    let req = request.into_inner();

    match orders::Entity::find()
        .filter(orders::Column::UserId.eq(req.user_id))
        .apply_if(req.order_id, |query, _| {
            query.filter(orders::Column::OrderId.eq(req.order_id))
        })
        .apply_if(req.order_date_start, |query, _| {
            query.filter(orders::Column::OrderDate.gte(req.order_date_start))
        })
        .apply_if(req.order_date_end, |query, _| {
            query.filter(orders::Column::OrderDate.lte(req.order_date_end))
        })
        .apply_if(req.status_id, |query, _| {
            query.filter(orders::Column::StatusId.eq(req.status_id))
        })
        .all(txn)
        .await
    {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|model| OrderResponse {
                    order_id: model.order_id,
                    user_id: model.user_id,
                    order_date: model.order_date.to_string(),
                    shipping_address_id: model.shipping_address_id,
                    total_amount: model.total_amount.to_f64().unwrap(),
                    status_id: model.status_id,
                })
                .collect();

            Ok(Response::new(OrdersResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
