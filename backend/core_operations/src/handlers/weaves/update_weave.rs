use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::weaves;
use proto::proto::core::{UpdateWeaveRequest, WeaveResponse, WeavesResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn update_weave(
    txn: &DatabaseTransaction,
    request: Request<UpdateWeaveRequest>,
) -> Result<Response<WeavesResponse>, Status> {
    let req = request.into_inner();
    let model = weaves::ActiveModel {
        weave_id: ActiveValue::Set(req.weave_id),
        name: ActiveValue::Set(req.weave_name),
    };

    match model.update(txn).await {
        Ok(updated) => Ok(Response::new(WeavesResponse {
            items: vec![WeaveResponse {
                weave_id: updated.weave_id,
                weave_name: updated.name,
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
