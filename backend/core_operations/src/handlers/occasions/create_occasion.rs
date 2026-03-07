use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::occasions;
use proto::proto::core::{CreateOccasionRequest, OccasionResponse, OccasionsResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_occasion(
    txn: &DatabaseTransaction,
    request: Request<CreateOccasionRequest>,
) -> Result<Response<OccasionsResponse>, Status> {
    let req = request.into_inner();
    let model = occasions::ActiveModel {
        occasion_id: ActiveValue::NotSet,
        name: ActiveValue::Set(req.occasion_name),
    };

    match model.insert(txn).await {
        Ok(inserted) => Ok(Response::new(OccasionsResponse {
            items: vec![OccasionResponse {
                occasion_id: inserted.occasion_id,
                occasion_name: inserted.name,
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
