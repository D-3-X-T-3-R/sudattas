use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::shipping_zones;
use proto::proto::core::{SearchShippingZoneRequest, ShippingZoneResponse, ShippingZonesResponse};

use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryTrait};
use tonic::{Request, Response, Status};

pub async fn search_shipping_zone(
    txn: &DatabaseTransaction,
    request: Request<SearchShippingZoneRequest>,
) -> Result<Response<ShippingZonesResponse>, Status> {
    let req = request.into_inner();

    match shipping_zones::Entity::find()
        .apply_if(req.zone_id, |query, v| {
            query.filter(shipping_zones::Column::ZoneId.eq(v))
        })
        .apply_if(req.zip_code, |query, v| {
            query.filter(shipping_zones::Column::ZipCode.starts_with(v.to_string()))
        })
        .all(txn)
        .await
    {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|model| ShippingZoneResponse {
                    zone_id: model.zone_id,
                    zip_code: model.zip_code,
                    description: model.description,
                })
                .collect();

            let response = ShippingZonesResponse { items };
            Ok(Response::new(response))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
