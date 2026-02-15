use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::shipping_zones;
use proto::proto::core::{
    DeleteShippingZoneRequest, ShippingZoneResponse, ShippingZonesResponse,
};
use sea_orm::{ActiveModelTrait, DatabaseTransaction, EntityTrait, IntoActiveModel};
use tonic::{Request, Response, Status};

pub async fn delete_shipping_zone(
    txn: &DatabaseTransaction,
    request: Request<DeleteShippingZoneRequest>,
) -> Result<Response<ShippingZonesResponse>, Status> {
    let req = request.into_inner();

    // Load existing entity
    let existing = shipping_zones::Entity::find_by_id(req.zone_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::not_found("Shipping zone not found"))?;

    let zone_response = ShippingZoneResponse {
        zone_id: existing.zone_id,
        zone_name: existing.zone_name.clone(),
        description: existing.description.clone().unwrap_or_default(),
    };

    // Delete the entity
    existing
        .into_active_model()
        .delete(txn)
        .await
        .map_err(map_db_error_to_status)?;

    Ok(Response::new(ShippingZonesResponse {
        items: vec![zone_response],
    }))
}
