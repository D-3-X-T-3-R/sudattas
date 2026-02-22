use crate::handlers::db_errors::map_db_error_to_status;
use crate::handlers::order_events::create_order_event::model_to_response;
use core_db_entities::entity::order_events;
use proto::proto::core::{GetOrderEventsRequest, OrderEventsResponse};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder};
use tonic::{Request, Response, Status};

pub async fn get_order_events(
    txn: &DatabaseTransaction,
    request: Request<GetOrderEventsRequest>,
) -> Result<Response<OrderEventsResponse>, Status> {
    let req = request.into_inner();

    let events = order_events::Entity::find()
        .filter(order_events::Column::OrderId.eq(req.order_id))
        .order_by_asc(order_events::Column::CreatedAt)
        .all(txn)
        .await
        .map_err(map_db_error_to_status)?;

    Ok(Response::new(OrderEventsResponse {
        items: events.into_iter().map(model_to_response).collect(),
    }))
}
