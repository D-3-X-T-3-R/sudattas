use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::country_state_mapping;
use proto::proto::core::{
    CountryStateMappingResponse, CountryStateMappingsResponse,
    SearchCountryStateMappingRequest,
};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryTrait};
use tonic::{Request, Response, Status};

pub async fn search_country_state_mapping(
    txn: &DatabaseTransaction,
    request: Request<SearchCountryStateMappingRequest>,
) -> Result<Response<CountryStateMappingsResponse>, Status> {
    let req = request.into_inner();

    match country_state_mapping::Entity::find()
        .apply_if(req.id, |query, v| {
            query.filter(country_state_mapping::Column::Id.eq(v))
        })
        .apply_if(req.country_id, |query, v| {
            query.filter(country_state_mapping::Column::CountryId.eq(v))
        })
        .apply_if(req.state_id, |query, v| {
            query.filter(country_state_mapping::Column::StateId.eq(v))
        })
        .all(txn)
        .await
    {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|m| CountryStateMappingResponse {
                    id: m.id,
                    country_id: m.country_id,
                    state_id: m.state_id,
                })
                .collect();
            Ok(Response::new(CountryStateMappingsResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
