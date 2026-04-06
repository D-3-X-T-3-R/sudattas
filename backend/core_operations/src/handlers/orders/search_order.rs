// SearchOrderRequest Proto message
use crate::handlers::db_errors::map_db_error_to_status;
use chrono::{DateTime, Utc};
use core_db_entities::entity::orders;
use proto::proto::core::{OrderResponse, OrdersResponse, SearchOrderRequest};
use sea_orm::{
    ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QuerySelect, QueryTrait,
};
use tonic::{Request, Response, Status};

pub async fn search_order(
    txn: &DatabaseTransaction,
    request: Request<SearchOrderRequest>,
) -> Result<Response<OrdersResponse>, Status> {
    let req = request.into_inner();

    match orders::Entity::find()
        .apply_if(req.user_id, |query, v| {
            query.filter(orders::Column::UserId.eq(v))
        })
        .apply_if(req.order_id, |query, id| {
            query.filter(orders::Column::OrderId.eq(id))
        })
        .apply_if(
            req.order_date_start,
            |query, ts| match DateTime::<Utc>::from_timestamp(ts, 0) {
                Some(dt) => query.filter(orders::Column::OrderDate.gte(dt)),
                None => query,
            },
        )
        .apply_if(
            req.order_date_end,
            |query, ts| match DateTime::<Utc>::from_timestamp(ts, 0) {
                Some(dt) => query.filter(orders::Column::OrderDate.lte(dt)),
                None => query,
            },
        )
        .apply_if(req.status_id, |query, sid| {
            query.filter(orders::Column::StatusId.eq(sid))
        })
        .apply_if(req.limit, |query, v| query.limit(v as u64))
        .apply_if(req.offset, |query, v| query.offset(v as u64))
        .all(txn)
        .await
    {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|model| {
                    let total_amount_paise = model.grand_total_minor;
                    OrderResponse {
                        order_id: model.order_id,
                        user_id: model.user_id,
                        order_date: model.order_date.to_string(),
                        shipping_address_id: model.shipping_address_id,
                        total_amount_paise,
                        status_id: model.status_id,
                    }
                })
                .collect();

            Ok(Response::new(OrdersResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
