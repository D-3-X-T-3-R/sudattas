use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::shipping_addresses;
use proto::proto::core::{
    CreateShippingAddressRequest, ShippingAddressResponse, ShippingAddressesResponse,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_shipping_address(
    txn: &DatabaseTransaction,
    request: Request<CreateShippingAddressRequest>,
) -> Result<Response<ShippingAddressesResponse>, Status> {
    let req = request.into_inner();
    let model = shipping_addresses::ActiveModel {
        shipping_address_id: ActiveValue::NotSet,
        user_id: ActiveValue::Set(req.user_id),
        country: ActiveValue::Set(req.country),
        state_region: ActiveValue::Set(req.state_region),
        city: ActiveValue::Set(req.city),
        postal_code: ActiveValue::Set(req.postal_code),
        road: ActiveValue::Set(req.road),
        apartment_no_or_name: ActiveValue::Set(req.apartment_no_or_name),
    };

    match model.insert(txn).await {
        Ok(inserted) => Ok(Response::new(ShippingAddressesResponse {
            items: vec![ShippingAddressResponse {
                shipping_address_id: inserted.shipping_address_id,
                user_id: inserted.user_id,
                country: inserted.country,
                state_region: inserted.state_region,
                city: inserted.city,
                postal_code: inserted.postal_code,
                road: inserted.road,
                apartment_no_or_name: inserted.apartment_no_or_name,
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
