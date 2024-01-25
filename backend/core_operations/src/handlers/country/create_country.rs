use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::countries;
use proto::proto::core::{CountriesResponse, CountryResponse, CreateCountryRequest};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_country(
    txn: &DatabaseTransaction,
    request: Request<CreateCountryRequest>,
) -> Result<Response<CountriesResponse>, Status> {
    let req = request.into_inner();
    let country = countries::ActiveModel {
        country_id: ActiveValue::NotSet,
        country_name: ActiveValue::Set(Some(req.country_name)),
    };
    match country.insert(txn).await {
        Ok(model) => {
            let response = CountriesResponse {
                items: vec![CountryResponse {
                    country_name: model.country_name.unwrap(),
                    country_id: model.country_id,
                }],
            };
            Ok(Response::new(response))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
