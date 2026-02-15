use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::state_city_mapping;
use proto::proto::core::{
    SearchStateCityMappingRequest, StateCityMappingResponse, StateCityMappingsResponse,
};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryTrait};
use tonic::{Request, Response, Status};

pub async fn search_state_city_mapping(
    txn: &DatabaseTransaction,
    request: Request<SearchStateCityMappingRequest>,
) -> Result<Response<StateCityMappingsResponse>, Status> {
    let req = request.into_inner();

    match state_city_mapping::Entity::find()
        .apply_if(req.id, |query, v| query.filter(state_city_mapping::Column::Id.eq(v)))
        .apply_if(req.state_id, |query, v| {
            query.filter(state_city_mapping::Column::StateId.eq(v))
        })
        .apply_if(req.city_id, |query, v| {
            query.filter(state_city_mapping::Column::CityId.eq(v))
        })
        .all(txn)
        .await
    {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|m| StateCityMappingResponse {
                    id: m.id,
                    state_id: m.state_id,
                    city_id: m.city_id,
                })
                .collect();
            Ok(Response::new(StateCityMappingsResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
