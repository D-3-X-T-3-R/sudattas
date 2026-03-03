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
        user_id: ActiveValue::Set(req.user_id),
        country: ActiveValue::Set(req.country),
        state_region: ActiveValue::Set(req.state_region),
        city: ActiveValue::Set(req.city),
        postal_code: ActiveValue::Set(req.postal_code),
        road: ActiveValue::Set(req.road),
        apartment_no_or_name: ActiveValue::Set(req.apartment_no_or_name),
    };

    match model.update(txn).await {
        Ok(updated) => Ok(Response::new(ShippingAddressesResponse {
            items: vec![ShippingAddressResponse {
                shipping_address_id: updated.shipping_address_id,
                user_id: updated.user_id,
                country: updated.country,
                state_region: updated.state_region,
                city: updated.city,
                postal_code: updated.postal_code,
                road: updated.road,
                apartment_no_or_name: updated.apartment_no_or_name,
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
