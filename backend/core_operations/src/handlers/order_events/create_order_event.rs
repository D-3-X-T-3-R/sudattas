use crate::handlers::db_errors::map_db_error_to_status;
use chrono::Utc;
use core_db_entities::entity::order_events;
use core_db_entities::entity::sea_orm_active_enums::ActorType;
use proto::proto::core::{CreateOrderEventRequest, OrderEventResponse, OrderEventsResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_order_event(
    txn: &DatabaseTransaction,
    request: Request<CreateOrderEventRequest>,
) -> Result<Response<OrderEventsResponse>, Status> {
    let req = request.into_inner();

    let actor_type = match req.actor_type.as_str() {
        "admin" => ActorType::Admin,
        "customer" => ActorType::Customer,
        _ => ActorType::System,
    };

    let event = order_events::ActiveModel {
        event_id: ActiveValue::NotSet,
        order_id: ActiveValue::Set(req.order_id),
        event_type: ActiveValue::Set(req.event_type),
        from_status: ActiveValue::Set(req.from_status),
        to_status: ActiveValue::Set(req.to_status),
        actor_type: ActiveValue::Set(actor_type),
        message: ActiveValue::Set(req.message),
        created_at: ActiveValue::Set(Some(Utc::now())),
    };

    match event.insert(txn).await {
        Ok(m) => Ok(Response::new(OrderEventsResponse {
            items: vec![model_to_response(m)],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}

pub fn model_to_response(m: order_events::Model) -> OrderEventResponse {
    OrderEventResponse {
        event_id: m.event_id,
        order_id: m.order_id,
        event_type: m.event_type,
        from_status: m.from_status.unwrap_or_default(),
        to_status: m.to_status.unwrap_or_default(),
        actor_type: format!("{:?}", m.actor_type).to_lowercase(),
        message: m.message.unwrap_or_default(),
        created_at: m.created_at.map(|t| t.to_string()).unwrap_or_default(),
    }
}
