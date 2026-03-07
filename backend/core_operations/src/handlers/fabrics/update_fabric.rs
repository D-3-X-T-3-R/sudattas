use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::fabrics;
use proto::proto::core::{FabricResponse, FabricsResponse, UpdateFabricRequest};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn update_fabric(
    txn: &DatabaseTransaction,
    request: Request<UpdateFabricRequest>,
) -> Result<Response<FabricsResponse>, Status> {
    let req = request.into_inner();
    let model = fabrics::ActiveModel {
        fabric_id: ActiveValue::Set(req.fabric_id),
        name: ActiveValue::Set(req.fabric_name),
    };

    match model.update(txn).await {
        Ok(updated) => Ok(Response::new(FabricsResponse {
            items: vec![FabricResponse {
                fabric_id: updated.fabric_id,
                fabric_name: updated.name,
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
