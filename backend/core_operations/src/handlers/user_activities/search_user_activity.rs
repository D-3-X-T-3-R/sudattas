use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::user_activity;
use proto::proto::core::{
    SearchUserActivityRequest, UserActivityResponse, UserActivitiesResponse,
};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryTrait};
use tonic::{Request, Response, Status};

pub async fn search_user_activity(
    txn: &DatabaseTransaction,
    request: Request<SearchUserActivityRequest>,
) -> Result<Response<UserActivitiesResponse>, Status> {
    let req = request.into_inner();

    let mut query = user_activity::Entity::find();
    if req.activity_id != 0 {
        query = query.filter(user_activity::Column::ActivityId.eq(req.activity_id));
    }

    match query.all(txn).await {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|m| UserActivityResponse {
                    activity_id: m.activity_id,
                    user_id: m.user_id.unwrap_or(0),
                    activity_type: m.activity_type,
                    activity_time: m.activity_time.to_rfc3339(),
                    activity_details: m.activity_details.unwrap_or_default(),
                })
                .collect();
            Ok(Response::new(UserActivitiesResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
