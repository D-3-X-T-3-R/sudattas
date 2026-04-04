use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::occasions;
use proto::proto::core::{OccasionResponse, OccasionsResponse, UpdateOccasionRequest};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn update_occasion(
    txn: &DatabaseTransaction,
    request: Request<UpdateOccasionRequest>,
) -> Result<Response<OccasionsResponse>, Status> {
    let req = request.into_inner();
    let model = occasions::ActiveModel {
        occasion_id: ActiveValue::Set(req.occasion_id),
        name: ActiveValue::Set(req.occasion_name),
    };

    match model.update(txn).await {
        Ok(updated) => Ok(Response::new(OccasionsResponse {
            items: vec![OccasionResponse {
                occasion_id: updated.occasion_id,
                occasion_name: updated.name,
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
