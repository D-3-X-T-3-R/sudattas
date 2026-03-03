use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::shipping_addresses;
use proto::proto::core::{
    DeleteShippingAddressRequest, ShippingAddressResponse, ShippingAddressesResponse,
};
use sea_orm::{DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn delete_shipping_address(
    txn: &DatabaseTransaction,
    request: Request<DeleteShippingAddressRequest>,
) -> Result<Response<ShippingAddressesResponse>, Status> {
    let req = request.into_inner();

    let found = shipping_addresses::Entity::find_by_id(req.shipping_address_id)
        .one(txn)
        .await;

    match found {
        Ok(Some(model)) => {
            match shipping_addresses::Entity::delete_by_id(req.shipping_address_id)
                .exec(txn)
                .await
            {
                Ok(_) => Ok(Response::new(ShippingAddressesResponse {
                    items: vec![ShippingAddressResponse {
                        shipping_address_id: model.shipping_address_id,
                        user_id: model.user_id,
                        country: model.country,
                        state_region: model.state_region,
                        city: model.city,
                        postal_code: model.postal_code,
                        road: model.road,
                        apartment_no_or_name: model.apartment_no_or_name,
                    }],
                })),
                Err(e) => Err(map_db_error_to_status(e)),
            }
        }
        Ok(None) => Err(Status::not_found(format!(
            "ShippingAddress with ID {} not found",
            req.shipping_address_id
        ))),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
