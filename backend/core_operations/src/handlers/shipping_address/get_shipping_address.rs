use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::shipping_addresses;
use proto::proto::core::{GetShippingAddressRequest, ShippingAddressResponse, ShippingAddressesResponse};
use sea_orm::{DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn get_shipping_address(
    txn: &DatabaseTransaction,
    _request: Request<GetShippingAddressRequest>,
) -> Result<Response<ShippingAddressesResponse>, Status> {
    match shipping_addresses::Entity::find().all(txn).await {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|m| ShippingAddressResponse {
                    shipping_address_id: m.shipping_address_id,
                    country_id: m.country_id,
                    state_id: m.state_id,
                    city_id: m.city_id,
                    road: m.road.unwrap_or_default(),
                    apartment_no_or_name: m.apartment_no_or_name.unwrap_or_default(),
                })
                .collect();
            Ok(Response::new(ShippingAddressesResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
