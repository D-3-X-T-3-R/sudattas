use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::state_city_mapping;
use proto::proto::core::{
    DeleteStateCityMappingRequest, StateCityMappingResponse, StateCityMappingsResponse,
};
use sea_orm::{DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn delete_state_city_mapping(
    txn: &DatabaseTransaction,
    request: Request<DeleteStateCityMappingRequest>,
) -> Result<Response<StateCityMappingsResponse>, Status> {
    let req = request.into_inner();

    let found = state_city_mapping::Entity::find_by_id(req.id)
        .one(txn)
        .await;

    match found {
        Ok(Some(model)) => {
            match state_city_mapping::Entity::delete_by_id(req.id)
                .exec(txn)
                .await
            {
                Ok(_) => Ok(Response::new(StateCityMappingsResponse {
                    items: vec![StateCityMappingResponse {
                        id: model.id,
                        state_id: model.state_id,
                        city_id: model.city_id,
                    }],
                })),
                Err(e) => Err(map_db_error_to_status(e)),
            }
        }
        Ok(None) => Err(Status::not_found(format!(
            "StateCityMapping with ID {} not found",
            req.id
        ))),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
