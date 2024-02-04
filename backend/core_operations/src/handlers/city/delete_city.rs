use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::cities;
use proto::proto::core::{CitiesResponse, CityResponse, DeleteCityRequest};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn delete_city(
    txn: &DatabaseTransaction,
    request: Request<DeleteCityRequest>,
) -> Result<Response<CitiesResponse>, Status> {
    let req = request.into_inner();

    let city = cities::Entity::find_by_id(req.city_id).one(txn).await;

    match city {
        Ok(Some(model)) => {
            match cities::Entity::delete_many()
                .filter(cities::Column::CityId.eq(req.city_id))
                .exec(txn)
                .await
            {
                Ok(delete_result) => {
                    if delete_result.rows_affected > 0 {
                        let response = CitiesResponse {
                            items: vec![CityResponse {
                                city_name: model.city_name.unwrap(),
                                city_id: model.city_id,
                            }],
                        };
                        Ok(Response::new(response))
                    } else {
                        Err(Status::not_found(format!(
                            "City with ID {} not found.",
                            req.city_id
                        )))
                    }
                }
                Err(e) => Err(map_db_error_to_status(e)),
            }
        }
        Ok(None) => Err(Status::not_found(format!(
            "City with ID {} not found.",
            req.city_id
        ))),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
