use proto::proto::core::{CreateOrderEventRequest, GetOrderEventsRequest, OrderEventResponse};
use tracing::instrument;

use super::schema::{NewOrderEvent, OrderEvent};
use crate::resolvers::{
    error::GqlError,
    utils::{connect_grpc_client, parse_i64},
};

fn event_response_to_gql(e: OrderEventResponse) -> OrderEvent {
    OrderEvent {
        event_id: e.event_id.to_string(),
        order_id: e.order_id.to_string(),
        event_type: e.event_type,
        from_status: e.from_status,
        to_status: e.to_status,
        actor_type: e.actor_type,
        message: e.message,
        created_at: e.created_at,
    }
}

#[instrument]
pub(crate) async fn get_order_events(order_id: String) -> Result<Vec<OrderEvent>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .get_order_events(GetOrderEventsRequest {
            order_id: parse_i64(&order_id, "order id")?,
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(event_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn create_order_event(input: NewOrderEvent) -> Result<Vec<OrderEvent>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .create_order_event(CreateOrderEventRequest {
            order_id: parse_i64(&input.order_id, "order id")?,
            event_type: input.event_type,
            from_status: input.from_status,
            to_status: input.to_status,
            actor_type: input.actor_type,
            message: input.message,
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(event_response_to_gql)
        .collect())
}
