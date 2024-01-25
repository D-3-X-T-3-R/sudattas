use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::countries;
use proto::proto::core::{CountriesResponse, CountryResponse, DeleteCountryRequest};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn delete_country(
    txn: &DatabaseTransaction,
    request: Request<DeleteCountryRequest>,
) -> Result<Response<CountriesResponse>, Status> {
    let req = request.into_inner();

    let country = countries::Entity::find_by_id(req.country_id).one(txn).await;

    match country {
        Ok(Some(model)) => {
            match countries::Entity::delete_many()
                .filter(countries::Column::CountryId.eq(req.country_id))
                .exec(txn)
                .await
            {
                Ok(delete_result) => {
                    if delete_result.rows_affected > 0 {
                        let response = CountriesResponse {
                            items: vec![CountryResponse {
                                country_name: model.country_name.unwrap(),
                                country_id: model.country_id,
                            }],
                        };
                        Ok(Response::new(response))
                    } else {
                        Err(Status::not_found(format!(
                            "Country with ID {} not found.",
                            req.country_id
                        )))
                    }
                }
                Err(e) => Err(map_db_error_to_status(e)),
            }
        }
        Ok(None) => Err(Status::not_found(format!(
            "Country with ID {} not found.",
            req.country_id
        ))),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
