use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::shipping_zones;
use proto::proto::core::{DeleteShippingZoneRequest, ShippingZoneResponse, ShippingZonesResponse};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn delete_shipping_zone(
    txn: &DatabaseTransaction,
    request: Request<DeleteShippingZoneRequest>,
) -> Result<Response<ShippingZonesResponse>, Status> {
    let req = request.into_inner();

    let shipping_zone = shipping_zones::Entity::find_by_id(req.zone_id)
        .one(txn)
        .await;

    match shipping_zone {
        Ok(Some(model)) => {
            match shipping_zones::Entity::delete_many()
                .filter(shipping_zones::Column::ZoneId.eq(req.zone_id))
                .exec(txn)
                .await
            {
                Ok(delete_result) => {
                    if delete_result.rows_affected > 0 {
                        let response = ShippingZonesResponse {
                            items: vec![ShippingZoneResponse {
                                zone_id: model.zone_id,
                                zip_code: model.zip_code,
                                description: model.description,
                            }],
                        };
                        Ok(Response::new(response))
                    } else {
                        Err(Status::not_found(format!(
                            "ShippingZone with ID {} not found.",
                            req.zone_id
                        )))
                    }
                }
                Err(e) => Err(map_db_error_to_status(e)),
            }
        }
        Ok(None) => Err(Status::not_found(format!(
            "ShippingZone with ID {} not found.",
            req.zone_id
        ))),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
