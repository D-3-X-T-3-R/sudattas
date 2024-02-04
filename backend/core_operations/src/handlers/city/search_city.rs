use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::cities;
use proto::proto::core::{CitiesResponse, CityResponse, SearchCityRequest};

use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryTrait};
use tonic::{Request, Response, Status};

pub async fn search_city(
    txn: &DatabaseTransaction,
    request: Request<SearchCityRequest>,
) -> Result<Response<CitiesResponse>, Status> {
    let req = request.into_inner();

    match cities::Entity::find()
        .apply_if(req.city_id, |query, v| {
            query.filter(cities::Column::CityId.eq(v))
        })
        .apply_if(req.city_name, |query, v| {
            query.filter(cities::Column::CityName.contains(v))
        })
        .all(txn)
        .await
    {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|model| CityResponse {
                    city_name: model.city_name.unwrap(),
                    city_id: model.city_id,
                })
                .collect();

            let response = CitiesResponse { items };
            Ok(Response::new(response))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
