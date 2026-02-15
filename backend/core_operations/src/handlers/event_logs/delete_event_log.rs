use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::event_logs;
use proto::proto::core::{DeleteEventLogRequest, EventLogResponse, EventLogsResponse};
use sea_orm::{DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn delete_event_log(
    txn: &DatabaseTransaction,
    request: Request<DeleteEventLogRequest>,
) -> Result<Response<EventLogsResponse>, Status> {
    let req = request.into_inner();

    let found = event_logs::Entity::find_by_id(req.log_id).one(txn).await;

    match found {
        Ok(Some(model)) => match event_logs::Entity::delete_by_id(req.log_id).exec(txn).await {
            Ok(_) => Ok(Response::new(EventLogsResponse {
                items: vec![EventLogResponse {
                    log_id: model.log_id,
                    event_type: model.event_type,
                    event_description: model.event_description.unwrap_or_default(),
                    user_id: model.user_id.unwrap_or(0),
                    event_time: model.event_time.to_rfc3339(),
                }],
            })),
            Err(e) => Err(map_db_error_to_status(e)),
        },
        Ok(None) => Err(Status::not_found(format!(
            "EventLog with ID {} not found",
            req.log_id
        ))),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
