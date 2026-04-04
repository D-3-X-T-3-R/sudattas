use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::weaves;
use proto::proto::core::{CreateWeaveRequest, WeaveResponse, WeavesResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_weave(
    txn: &DatabaseTransaction,
    request: Request<CreateWeaveRequest>,
) -> Result<Response<WeavesResponse>, Status> {
    let req = request.into_inner();
    let model = weaves::ActiveModel {
        weave_id: ActiveValue::NotSet,
        name: ActiveValue::Set(req.weave_name),
    };

    match model.insert(txn).await {
        Ok(inserted) => Ok(Response::new(WeavesResponse {
            items: vec![WeaveResponse {
                weave_id: inserted.weave_id,
                weave_name: inserted.name,
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
