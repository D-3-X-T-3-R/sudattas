use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::country_state_mapping;
use proto::proto::core::{
    CountryStateMappingResponse, CountryStateMappingsResponse,
    DeleteCountryStateMappingRequest,
};
use sea_orm::{DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn delete_country_state_mapping(
    txn: &DatabaseTransaction,
    request: Request<DeleteCountryStateMappingRequest>,
) -> Result<Response<CountryStateMappingsResponse>, Status> {
    let req = request.into_inner();

    let found = country_state_mapping::Entity::find_by_id(req.id)
        .one(txn)
        .await;

    match found {
        Ok(Some(model)) => {
            match country_state_mapping::Entity::delete_by_id(req.id).exec(txn).await {
                Ok(_) => Ok(Response::new(CountryStateMappingsResponse {
                    items: vec![CountryStateMappingResponse {
                        id: model.id,
                        country_id: model.country_id,
                        state_id: model.state_id,
                    }],
                })),
                Err(e) => Err(map_db_error_to_status(e)),
            }
        }
        Ok(None) => Err(Status::not_found(format!(
            "CountryStateMapping with ID {} not found",
            req.id
        ))),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
