use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::user_activity;
use proto::proto::core::{UpdateUserActivityRequest, UserActivitiesResponse, UserActivityResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn update_user_activity(
    txn: &DatabaseTransaction,
    request: Request<UpdateUserActivityRequest>,
) -> Result<Response<UserActivitiesResponse>, Status> {
    let req = request.into_inner();

    let existing = user_activity::Entity::find_by_id(req.activity_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| {
            Status::not_found(format!(
                "UserActivity with ID {} not found",
                req.activity_id
            ))
        })?;

    let model = user_activity::ActiveModel {
        activity_id: ActiveValue::Set(existing.activity_id),
        user_id: ActiveValue::Set(req.user_id.or(existing.user_id)),
        activity_type: ActiveValue::Set(req.activity_type.unwrap_or(existing.activity_type)),
        activity_time: ActiveValue::Set(existing.activity_time),
        activity_details: ActiveValue::Set(req.activity_details.or(existing.activity_details)),
    };

    match model.update(txn).await {
        Ok(updated) => Ok(Response::new(UserActivitiesResponse {
            items: vec![UserActivityResponse {
                activity_id: updated.activity_id,
                user_id: updated.user_id.unwrap_or(0),
                activity_type: updated.activity_type,
                activity_time: updated.activity_time.to_rfc3339(),
                activity_details: updated.activity_details.unwrap_or_default(),
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
