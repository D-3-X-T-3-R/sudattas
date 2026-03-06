use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::fabrics;
use proto::proto::core::{DeleteFabricRequest, FabricResponse, FabricsResponse};
use sea_orm::{DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn delete_fabric(
    txn: &DatabaseTransaction,
    request: Request<DeleteFabricRequest>,
) -> Result<Response<FabricsResponse>, Status> {
    let req = request.into_inner();

    let found = fabrics::Entity::find_by_id(req.fabric_id).one(txn).await;

    match found {
        Ok(Some(model)) => match fabrics::Entity::delete_by_id(req.fabric_id).exec(txn).await {
            Ok(_) => Ok(Response::new(FabricsResponse {
                items: vec![FabricResponse {
                    fabric_id: model.fabric_id,
                    fabric_name: model.name,
                }],
            })),
            Err(e) => Err(map_db_error_to_status(e)),
        },
        Ok(None) => Err(Status::not_found(format!(
            "Fabric with ID {} not found",
            req.fabric_id
        ))),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
