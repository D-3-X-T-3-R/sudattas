use proto::proto::core::{
    CreateEventLogRequest, DeleteEventLogRequest, EventLogResponse, EventLogsResponse,
    SearchEventLogRequest, UpdateEventLogRequest,
};
use tracing::instrument;

use super::schema::{
    DeleteEventLogInput, EventLog, EventLogMutation, NewEventLog, SearchEventLogInput,
};
use crate::resolvers::{
    error::GqlError,
    utils::{connect_grpc_client, parse_i64},
};

fn log_response_to_gql(e: EventLogResponse) -> EventLog {
    EventLog {
        log_id: e.log_id.to_string(),
        event_type: e.event_type,
        event_description: e.event_description,
        user_id: e.user_id.to_string(),
        event_time: e.event_time,
    }
}

fn logs_response_to_vec(resp: EventLogsResponse) -> Vec<EventLog> {
    resp.items.into_iter().map(log_response_to_gql).collect()
}

#[instrument]
pub(crate) async fn create_event_log(input: NewEventLog) -> Result<Vec<EventLog>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .create_event_log(CreateEventLogRequest {
            event_type: input.event_type,
            event_description: input.event_description,
            user_id: parse_i64(&input.user_id, "user_id")?,
        })
        .await?;
    Ok(logs_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn search_event_log(
    input: SearchEventLogInput,
) -> Result<Vec<EventLog>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .search_event_log(SearchEventLogRequest {
            log_id: parse_i64(&input.log_id, "log_id")?,
        })
        .await?;
    Ok(logs_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn update_event_log(input: EventLogMutation) -> Result<Vec<EventLog>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .update_event_log(UpdateEventLogRequest {
            log_id: parse_i64(&input.log_id, "log_id")?,
            event_type: input.event_type,
            event_description: input.event_description,
            user_id: input
                .user_id
                .as_deref()
                .map(|s| parse_i64(s, "user_id"))
                .transpose()?,
        })
        .await?;
    Ok(logs_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn delete_event_log(
    input: DeleteEventLogInput,
) -> Result<Vec<EventLog>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .delete_event_log(DeleteEventLogRequest {
            log_id: parse_i64(&input.log_id, "log_id")?,
        })
        .await?;
    Ok(logs_response_to_vec(resp.into_inner()))
}
