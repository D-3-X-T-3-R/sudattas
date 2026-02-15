use crate::handlers::db_errors::map_db_error_to_status;
use chrono::Utc;
use core_db_entities::entity::event_logs;
use proto::proto::core::{CreateEventLogRequest, EventLogResponse, EventLogsResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_event_log(
    txn: &DatabaseTransaction,
    request: Request<CreateEventLogRequest>,
) -> Result<Response<EventLogsResponse>, Status> {
    let req = request.into_inner();
    let model = event_logs::ActiveModel {
        log_id: ActiveValue::NotSet,
        event_type: ActiveValue::Set(req.event_type),
        event_description: ActiveValue::Set(Some(req.event_description)),
        user_id: ActiveValue::Set(Some(req.user_id)),
        event_time: ActiveValue::Set(Utc::now()),
    };

    match model.insert(txn).await {
        Ok(inserted) => Ok(Response::new(EventLogsResponse {
            items: vec![EventLogResponse {
                log_id: inserted.log_id,
                event_type: inserted.event_type,
                event_description: inserted.event_description.unwrap_or_default(),
                user_id: inserted.user_id.unwrap_or(0),
                event_time: inserted.event_time.to_rfc3339(),
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
