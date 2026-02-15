use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::event_logs;
use proto::proto::core::{EventLogResponse, EventLogsResponse, UpdateEventLogRequest};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn update_event_log(
    txn: &DatabaseTransaction,
    request: Request<UpdateEventLogRequest>,
) -> Result<Response<EventLogsResponse>, Status> {
    let req = request.into_inner();

    let existing = event_logs::Entity::find_by_id(req.log_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| {
            Status::not_found(format!("EventLog with ID {} not found", req.log_id))
        })?;

    let model = event_logs::ActiveModel {
        log_id: ActiveValue::Set(existing.log_id),
        event_type: ActiveValue::Set(req.event_type.unwrap_or(existing.event_type)),
        event_description: ActiveValue::Set(req.event_description.or(existing.event_description)),
        user_id: ActiveValue::Set(req.user_id.or(existing.user_id)),
        event_time: ActiveValue::Set(existing.event_time),
    };

    match model.update(txn).await {
        Ok(updated) => Ok(Response::new(EventLogsResponse {
            items: vec![EventLogResponse {
                log_id: updated.log_id,
                event_type: updated.event_type,
                event_description: updated.event_description.unwrap_or_default(),
                user_id: updated.user_id.unwrap_or(0),
                event_time: updated.event_time.to_rfc3339(),
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
