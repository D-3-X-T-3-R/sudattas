use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::fabrics;
use proto::proto::core::{CreateFabricRequest, FabricResponse, FabricsResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_fabric(
    txn: &DatabaseTransaction,
    request: Request<CreateFabricRequest>,
) -> Result<Response<FabricsResponse>, Status> {
    let req = request.into_inner();
    let model = fabrics::ActiveModel {
        fabric_id: ActiveValue::NotSet,
        name: ActiveValue::Set(req.fabric_name),
    };

    match model.insert(txn).await {
        Ok(inserted) => Ok(Response::new(FabricsResponse {
            items: vec![FabricResponse {
                fabric_id: inserted.fabric_id,
                fabric_name: inserted.name,
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
