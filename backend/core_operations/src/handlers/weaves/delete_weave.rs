use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::weaves;
use proto::proto::core::{DeleteWeaveRequest, WeaveResponse, WeavesResponse};
use sea_orm::{DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn delete_weave(
    txn: &DatabaseTransaction,
    request: Request<DeleteWeaveRequest>,
) -> Result<Response<WeavesResponse>, Status> {
    let req = request.into_inner();

    let found = weaves::Entity::find_by_id(req.weave_id).one(txn).await;

    match found {
        Ok(Some(model)) => match weaves::Entity::delete_by_id(req.weave_id).exec(txn).await {
            Ok(_) => Ok(Response::new(WeavesResponse {
                items: vec![WeaveResponse {
                    weave_id: model.weave_id,
                    weave_name: model.name,
                }],
            })),
            Err(e) => Err(map_db_error_to_status(e)),
        },
        Ok(None) => Err(Status::not_found(format!(
            "Weave with ID {} not found",
            req.weave_id
        ))),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
