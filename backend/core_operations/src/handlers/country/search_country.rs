use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::countries;
use proto::proto::core::{CountriesResponse, CountryResponse, SearchCountryRequest};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryTrait};
use tonic::{Request, Response, Status};

pub async fn search_country(
    txn: &DatabaseTransaction,
    request: Request<SearchCountryRequest>,
) -> Result<Response<CountriesResponse>, Status> {
    let req = request.into_inner();

    match countries::Entity::find()
        .apply_if(req.country_id, |query, v| {
            query.filter(countries::Column::CountryId.eq(v))
        })
        .apply_if(req.country_name, |query, v| {
            query.filter(countries::Column::CountryName.contains(v))
        })
        .all(txn)
        .await
    {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|model| CountryResponse {
                    country_name: model.country_name.unwrap(),
                    country_id: model.country_id,
                })
                .collect();

            let response = CountriesResponse { items };
            Ok(Response::new(response))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
