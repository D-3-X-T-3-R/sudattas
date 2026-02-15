use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::sizes;
use proto::proto::core::{CreateSizeRequest, SizeResponse, SizesResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_size(
    txn: &DatabaseTransaction,
    request: Request<CreateSizeRequest>,
) -> Result<Response<SizesResponse>, Status> {
    let req = request.into_inner();
    let model = sizes::ActiveModel {
        size_id: ActiveValue::NotSet,
        size_name: ActiveValue::Set(req.size_name),
    };

    match model.insert(txn).await {
        Ok(inserted) => Ok(Response::new(SizesResponse {
            items: vec![SizeResponse {
                size_id: inserted.size_id,
                size_name: inserted.size_name,
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
