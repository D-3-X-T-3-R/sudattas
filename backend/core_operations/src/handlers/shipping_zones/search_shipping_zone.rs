use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::shipping_zones;
use proto::proto::core::{SearchShippingZoneRequest, ShippingZoneResponse, ShippingZonesResponse};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn search_shipping_zone(
    txn: &DatabaseTransaction,
    request: Request<SearchShippingZoneRequest>,
) -> Result<Response<ShippingZonesResponse>, Status> {
    let req = request.into_inner();

    let mut query = shipping_zones::Entity::find();
    if req.zone_id != 0 {
        query = query.filter(shipping_zones::Column::ZoneId.eq(req.zone_id));
    }

    match query.all(txn).await {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|m| ShippingZoneResponse {
                    zone_id: m.zone_id,
                    zone_name: m.zone_name,
                    description: m.description.unwrap_or_default(),
                })
                .collect();
            Ok(Response::new(ShippingZonesResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
