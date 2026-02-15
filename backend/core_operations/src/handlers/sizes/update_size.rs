use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::sizes;
use proto::proto::core::{SizeResponse, SizesResponse, UpdateSizeRequest};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn update_size(
    txn: &DatabaseTransaction,
    request: Request<UpdateSizeRequest>,
) -> Result<Response<SizesResponse>, Status> {
    let req = request.into_inner();
    let model = sizes::ActiveModel {
        size_id: ActiveValue::Set(req.size_id),
        size_name: ActiveValue::Set(req.size_name),
    };

    match model.update(txn).await {
        Ok(updated) => Ok(Response::new(SizesResponse {
            items: vec![SizeResponse {
                size_id: updated.size_id,
                size_name: updated.size_name,
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
