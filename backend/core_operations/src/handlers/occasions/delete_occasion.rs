use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::occasions;
use proto::proto::core::{DeleteOccasionRequest, OccasionResponse, OccasionsResponse};
use sea_orm::{DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn delete_occasion(
    txn: &DatabaseTransaction,
    request: Request<DeleteOccasionRequest>,
) -> Result<Response<OccasionsResponse>, Status> {
    let req = request.into_inner();

    let found = occasions::Entity::find_by_id(req.occasion_id)
        .one(txn)
        .await;

    match found {
        Ok(Some(model)) => match occasions::Entity::delete_by_id(req.occasion_id)
            .exec(txn)
            .await
        {
            Ok(_) => Ok(Response::new(OccasionsResponse {
                items: vec![OccasionResponse {
                    occasion_id: model.occasion_id,
                    occasion_name: model.name,
                }],
            })),
            Err(e) => Err(map_db_error_to_status(e)),
        },
        Ok(None) => Err(Status::not_found(format!(
            "Occasion with ID {} not found",
            req.occasion_id
        ))),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
