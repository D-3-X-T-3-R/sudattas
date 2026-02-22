use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::shipping_zones;
use proto::proto::core::{ShippingZoneResponse, ShippingZonesResponse, UpdateShippingZoneRequest};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn update_shipping_zone(
    txn: &DatabaseTransaction,
    request: Request<UpdateShippingZoneRequest>,
) -> Result<Response<ShippingZonesResponse>, Status> {
    let req = request.into_inner();

    // Load existing entity
    let existing = shipping_zones::Entity::find_by_id(req.zone_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::not_found("Shipping zone not found"))?;

    #[allow(clippy::unnecessary_map_or)]
    let model = shipping_zones::ActiveModel {
        zone_id: ActiveValue::Set(existing.zone_id),
        zone_name: if req.zone_name.as_ref().map_or(true, |s| s.is_empty()) {
            ActiveValue::Set(existing.zone_name)
        } else {
            ActiveValue::Set(req.zone_name.unwrap())
        },
        description: if req.description.as_ref().map_or(true, |s| s.is_empty()) {
            ActiveValue::Set(existing.description)
        } else {
            ActiveValue::Set(Some(req.description.unwrap()))
        },
    };

    match model.update(txn).await {
        Ok(updated) => Ok(Response::new(ShippingZonesResponse {
            items: vec![ShippingZoneResponse {
                zone_id: updated.zone_id,
                zone_name: updated.zone_name,
                description: updated.description.unwrap_or_default(),
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
