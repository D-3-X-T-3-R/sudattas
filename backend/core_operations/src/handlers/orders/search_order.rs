// SearchOrderRequest Proto message
use crate::handlers::db_errors::map_db_error_to_status;
use crate::handlers::orders::order_response;
use chrono::{DateTime, Utc};
use core_db_entities::entity::orders;
use proto::proto::core::{OrdersResponse, SearchOrderRequest};
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseTransaction, DbBackend, EntityTrait, QueryFilter,
    QuerySelect, QueryTrait, Statement,
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
            let mut items = Vec::with_capacity(models.len());
            for model in models {
                let mut row = order_response::from_model(&model);
                if let Some(extra) = txn
                    .query_one(Statement::from_sql_and_values(
                        DbBackend::MySql,
                        r#"SELECT cancel_window_ends_at,
                                  earliest_booking_at,
                                  pickup_target_at
                           FROM Orders
                           WHERE OrderID = ?
                           LIMIT 1"#,
                        [model.order_id.into()],
                    ))
                    .await
                    .map_err(map_db_error_to_status)?
                {
                    row.cancel_window_ends_at = extra
                        .try_get::<DateTime<Utc>>("", "cancel_window_ends_at")
                        .ok()
                        .map(|v| v.to_rfc3339());
                    row.earliest_booking_at = extra
                        .try_get::<DateTime<Utc>>("", "earliest_booking_at")
                        .ok()
                        .map(|v| v.to_rfc3339());
                    row.pickup_target_at = extra
                        .try_get::<DateTime<Utc>>("", "pickup_target_at")
                        .ok()
                        .map(|v| v.to_rfc3339());
                }
                items.push(row);
            }

            Ok(Response::new(OrdersResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
