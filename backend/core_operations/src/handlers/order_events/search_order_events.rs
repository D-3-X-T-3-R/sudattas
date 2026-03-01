//! P1 Admin audit log viewer: search order events (optional order_id = recent across all).

use crate::handlers::db_errors::map_db_error_to_status;
use crate::handlers::order_events::create_order_event::model_to_response;
use core_db_entities::entity::order_events;
use proto::proto::core::{OrderEventsResponse, SearchOrderEventsRequest};
use sea_orm::{
    ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder, QuerySelect, QueryTrait,
};
use tonic::{Request, Response, Status};

pub async fn search_order_events(
    txn: &DatabaseTransaction,
    request: Request<SearchOrderEventsRequest>,
) -> Result<Response<OrderEventsResponse>, Status> {
    let req = request.into_inner();

    let limit = req.limit.unwrap_or(100).clamp(1, 500) as u64;
    let offset = req.offset.unwrap_or(0).max(0) as u64;

    let events = order_events::Entity::find()
        .apply_if(req.order_id, |q, v| {
            q.filter(order_events::Column::OrderId.eq(v))
        })
        .order_by_desc(order_events::Column::CreatedAt)
        .limit(limit)
        .offset(offset)
        .all(txn)
        .await
        .map_err(map_db_error_to_status)?;

    Ok(Response::new(OrderEventsResponse {
        items: events.into_iter().map(model_to_response).collect(),
    }))
}
