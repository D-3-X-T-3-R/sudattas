use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::state_city_mapping;
use proto::proto::core::{
    CreateStateCityMappingRequest, StateCityMappingResponse, StateCityMappingsResponse,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_state_city_mapping(
    txn: &DatabaseTransaction,
    request: Request<CreateStateCityMappingRequest>,
) -> Result<Response<StateCityMappingsResponse>, Status> {
    let req = request.into_inner();
    let model = state_city_mapping::ActiveModel {
        id: ActiveValue::NotSet,
        state_id: ActiveValue::Set(req.state_id),
        city_id: ActiveValue::Set(req.city_id),
    };

    match model.insert(txn).await {
        Ok(inserted) => Ok(Response::new(StateCityMappingsResponse {
            items: vec![StateCityMappingResponse {
                id: inserted.id,
                state_id: inserted.state_id,
                city_id: inserted.city_id,
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
