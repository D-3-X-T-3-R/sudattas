use crate::handlers::db_errors::map_db_error_to_status;
use chrono::Utc;
use core_db_entities::entity::user_activity;
use proto::proto::core::{CreateUserActivityRequest, UserActivitiesResponse, UserActivityResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_user_activity(
    txn: &DatabaseTransaction,
    request: Request<CreateUserActivityRequest>,
) -> Result<Response<UserActivitiesResponse>, Status> {
    let req = request.into_inner();
    let model = user_activity::ActiveModel {
        activity_id: ActiveValue::NotSet,
        user_id: ActiveValue::Set(Some(req.user_id)),
        activity_type: ActiveValue::Set(req.activity_type),
        activity_time: ActiveValue::Set(Utc::now()),
        activity_details: ActiveValue::Set(Some(req.activity_details)),
    };

    match model.insert(txn).await {
        Ok(inserted) => Ok(Response::new(UserActivitiesResponse {
            items: vec![UserActivityResponse {
                activity_id: inserted.activity_id,
                user_id: inserted.user_id.unwrap_or(0),
                activity_type: inserted.activity_type,
                activity_time: inserted.activity_time.to_rfc3339(),
                activity_details: inserted.activity_details.unwrap_or_default(),
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
