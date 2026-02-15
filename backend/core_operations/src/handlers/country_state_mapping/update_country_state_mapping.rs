use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::country_state_mapping;
use proto::proto::core::{
    CountryStateMappingResponse, CountryStateMappingsResponse,
    UpdateCountryStateMappingRequest,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn update_country_state_mapping(
    txn: &DatabaseTransaction,
    request: Request<UpdateCountryStateMappingRequest>,
) -> Result<Response<CountryStateMappingsResponse>, Status> {
    let req = request.into_inner();

    let existing = country_state_mapping::Entity::find_by_id(req.id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?;

    let existing = existing.ok_or_else(|| {
        Status::not_found(format!(
            "CountryStateMapping with ID {} not found",
            req.id
        ))
    })?;

    let country_id = req.country_id.unwrap_or(existing.country_id);
    let state_id = req.state_id.unwrap_or(existing.state_id);

    let model = country_state_mapping::ActiveModel {
        id: ActiveValue::Set(req.id),
        country_id: ActiveValue::Set(country_id),
        state_id: ActiveValue::Set(state_id),
    };

    match model.update(txn).await {
        Ok(updated) => Ok(Response::new(CountryStateMappingsResponse {
            items: vec![CountryStateMappingResponse {
                id: updated.id,
                country_id: updated.country_id,
                state_id: updated.state_id,
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
