use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::sizes;
use proto::proto::core::{DeleteSizeRequest, SizeResponse, SizesResponse};
use sea_orm::{DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn delete_size(
    txn: &DatabaseTransaction,
    request: Request<DeleteSizeRequest>,
) -> Result<Response<SizesResponse>, Status> {
    let req = request.into_inner();

    let found = sizes::Entity::find_by_id(req.size_id).one(txn).await;

    match found {
        Ok(Some(model)) => match sizes::Entity::delete_by_id(req.size_id).exec(txn).await {
            Ok(_) => Ok(Response::new(SizesResponse {
                items: vec![SizeResponse {
                    size_id: model.size_id,
                    size_name: model.size_name,
                }],
            })),
            Err(e) => Err(map_db_error_to_status(e)),
        },
        Ok(None) => Err(Status::not_found(format!(
            "Size with ID {} not found",
            req.size_id
        ))),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
