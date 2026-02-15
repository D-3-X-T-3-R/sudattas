use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::user_activity;
use proto::proto::core::{DeleteUserActivityRequest, UserActivitiesResponse, UserActivityResponse};
use sea_orm::{DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn delete_user_activity(
    txn: &DatabaseTransaction,
    request: Request<DeleteUserActivityRequest>,
) -> Result<Response<UserActivitiesResponse>, Status> {
    let req = request.into_inner();

    let found = user_activity::Entity::find_by_id(req.activity_id)
        .one(txn)
        .await;

    match found {
        Ok(Some(model)) => {
            match user_activity::Entity::delete_by_id(req.activity_id)
                .exec(txn)
                .await
            {
                Ok(_) => Ok(Response::new(UserActivitiesResponse {
                    items: vec![UserActivityResponse {
                        activity_id: model.activity_id,
                        user_id: model.user_id.unwrap_or(0),
                        activity_type: model.activity_type,
                        activity_time: model.activity_time.to_rfc3339(),
                        activity_details: model.activity_details.unwrap_or_default(),
                    }],
                })),
                Err(e) => Err(map_db_error_to_status(e)),
            }
        }
        Ok(None) => Err(Status::not_found(format!(
            "UserActivity with ID {} not found",
            req.activity_id
        ))),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
