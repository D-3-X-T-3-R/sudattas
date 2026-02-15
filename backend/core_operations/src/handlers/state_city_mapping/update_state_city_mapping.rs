use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::state_city_mapping;
use proto::proto::core::{
    StateCityMappingResponse, StateCityMappingsResponse, UpdateStateCityMappingRequest,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn update_state_city_mapping(
    txn: &DatabaseTransaction,
    request: Request<UpdateStateCityMappingRequest>,
) -> Result<Response<StateCityMappingsResponse>, Status> {
    let req = request.into_inner();
    let model = state_city_mapping::ActiveModel {
        id: ActiveValue::Set(req.id),
        state_id: ActiveValue::Set(req.state_id),
        city_id: ActiveValue::Set(req.city_id),
    };

    match model.update(txn).await {
        Ok(updated) => Ok(Response::new(StateCityMappingsResponse {
            items: vec![StateCityMappingResponse {
                id: updated.id,
                state_id: updated.state_id,
                city_id: updated.city_id,
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
