use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::country_state_mapping;
use proto::proto::core::{
    CountryStateMappingResponse, CountryStateMappingsResponse, CreateCountryStateMappingRequest,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_country_state_mapping(
    txn: &DatabaseTransaction,
    request: Request<CreateCountryStateMappingRequest>,
) -> Result<Response<CountryStateMappingsResponse>, Status> {
    let req = request.into_inner();
    let model = country_state_mapping::ActiveModel {
        id: ActiveValue::NotSet,
        country_id: ActiveValue::Set(req.country_id),
        state_id: ActiveValue::Set(req.state_id),
    };

    match model.insert(txn).await {
        Ok(inserted) => Ok(Response::new(CountryStateMappingsResponse {
            items: vec![CountryStateMappingResponse {
                id: inserted.id,
                country_id: inserted.country_id,
                state_id: inserted.state_id,
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
