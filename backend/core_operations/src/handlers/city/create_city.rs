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
    let model = cities::ActiveModel {
        city_id: ActiveValue::NotSet,
        city_name: ActiveValue::Set(Some(req.city_name)),
    };

    match model.insert(txn).await {
        Ok(inserted) => {
            let response = CitiesResponse {
                items: vec![CityResponse {
                    city_id: inserted.city_id,
                    city_name: inserted.city_name.unwrap_or_default(),
                }],
            };
            Ok(Response::new(response))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
