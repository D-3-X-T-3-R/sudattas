use crate::handlers::db_errors::map_db_error_to_status;
use chrono::Utc;
use core_db_entities::entity::shipping_zones;
use proto::proto::core::{ShippingZoneResponse, ShippingZonesResponse, UpdateShippingZoneRequest};
use rust_decimal::{
    prelude::{FromPrimitive, ToPrimitive},
    Decimal,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn update_shipping_zone(
    txn: &DatabaseTransaction,
    request: Request<UpdateShippingZoneRequest>,
) -> Result<Response<ShippingZonesResponse>, Status> {
    let req = request.into_inner();

    let shipping_zone = shipping_zones::ActiveModel {
        zone_id: ActiveValue::Set(req.zone_id),
        zip_code: ActiveValue::Set(req.zip_code),
        description: ActiveValue::Set(req.description),
    };
    match shipping_zone.update(txn).await {
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
