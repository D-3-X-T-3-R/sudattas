use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::shipping_addresses;
use proto::proto::core::{
    ShippingAddressResponse, ShippingAddressesResponse, UpdateShippingAddressRequest,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn update_shipping_address(
    txn: &DatabaseTransaction,
    request: Request<UpdateShippingAddressRequest>,
) -> Result<Response<ShippingAddressesResponse>, Status> {
    let req = request.into_inner();
    let model = shipping_addresses::ActiveModel {
        shipping_address_id: ActiveValue::Set(req.shipping_address_id),
        country_id: ActiveValue::Set(req.country_id),
        state_id: ActiveValue::Set(req.state_id),
        city_id: ActiveValue::Set(req.city_id),
        road: ActiveValue::Set(Some(req.road)),
        apartment_no_or_name: ActiveValue::Set(Some(req.apartment_no_or_name)),
    };

    match model.update(txn).await {
        Ok(updated) => Ok(Response::new(ShippingAddressesResponse {
            items: vec![ShippingAddressResponse {
                shipping_address_id: updated.shipping_address_id,
                country_id: updated.country_id,
                state_id: updated.state_id,
                city_id: updated.city_id,
                road: updated.road.unwrap_or_default(),
                apartment_no_or_name: updated.apartment_no_or_name.unwrap_or_default(),
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
