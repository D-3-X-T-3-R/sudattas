use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::event_logs;
use proto::proto::core::{EventLogResponse, EventLogsResponse, SearchEventLogRequest};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryTrait};
use tonic::{Request, Response, Status};

pub async fn search_event_log(
    txn: &DatabaseTransaction,
    request: Request<SearchEventLogRequest>,
) -> Result<Response<EventLogsResponse>, Status> {
    let req = request.into_inner();

    let mut query = event_logs::Entity::find();
    if req.log_id != 0 {
        query = query.filter(event_logs::Column::LogId.eq(req.log_id));
    }

    match query.all(txn).await {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|m| EventLogResponse {
                    log_id: m.log_id,
                    event_type: m.event_type,
                    event_description: m.event_description.unwrap_or_default(),
                    user_id: m.user_id.unwrap_or(0),
                    event_time: m.event_time.to_rfc3339(),
                })
                .collect();
            Ok(Response::new(EventLogsResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
