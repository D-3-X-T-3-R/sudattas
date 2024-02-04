use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::cities;
use proto::proto::core::{CitiesResponse, CityResponse, CreateCityRequest};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_city(
    txn: &DatabaseTransaction,
    request: Request<CreateCityRequest>,
) -> Result<Response<CitiesResponse>, Status> {
    let req = request.into_inner();
    let city = cities::ActiveModel {
        city_id: ActiveValue::NotSet,
        city_name: ActiveValue::Set(Some(req.city_name)),
    };
    match city.insert(txn).await {
        Ok(model) => {
            let response = CitiesResponse {
                items: vec![CityResponse {
                    city_name: model.city_name.unwrap(),
                    city_id: model.city_id,
                }],
            };
            Ok(Response::new(response))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
