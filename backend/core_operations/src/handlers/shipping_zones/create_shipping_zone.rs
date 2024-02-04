use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::shipping_zones;
use proto::proto::core::{CreateShippingZoneRequest, ShippingZoneResponse, ShippingZonesResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_shipping_zone(
    txn: &DatabaseTransaction,
    request: Request<CreateShippingZoneRequest>,
) -> Result<Response<ShippingZonesResponse>, Status> {
    let req = request.into_inner();
    let shipping_zone = shipping_zones::ActiveModel {
        zone_id: ActiveValue::NotSet,
        zip_code: ActiveValue::Set(req.zip_code),
        description: ActiveValue::Set(req.description),
    };
    match shipping_zone.insert(txn).await {
        Ok(model) => {
            let response = ShippingZonesResponse {
                items: vec![ShippingZoneResponse {
                    zone_id: model.zone_id,
                    zip_code: model.zip_code,
                    description: model.description,
                }],
            };
            Ok(Response::new(response))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
