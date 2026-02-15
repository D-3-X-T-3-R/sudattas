use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::shipping_zones;
use proto::proto::core::{
    CreateShippingZoneRequest, ShippingZoneResponse, ShippingZonesResponse,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_shipping_zone(
    txn: &DatabaseTransaction,
    request: Request<CreateShippingZoneRequest>,
) -> Result<Response<ShippingZonesResponse>, Status> {
    let req = request.into_inner();
    let model = shipping_zones::ActiveModel {
        zone_id: ActiveValue::NotSet,
        zone_name: ActiveValue::Set(req.zone_name),
        description: ActiveValue::Set(Some(req.description)),
    };

    match model.insert(txn).await {
        Ok(inserted) => Ok(Response::new(ShippingZonesResponse {
            items: vec![ShippingZoneResponse {
                zone_id: inserted.zone_id,
                zone_name: inserted.zone_name,
                description: inserted.description.unwrap_or_default(),
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
