use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::shipping_addresses;
use proto::proto::core::{
    CreateShippingAddressRequest, ShippingAddressResponse, ShippingAddressesResponse,
};
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseTransaction, EntityTrait, PaginatorTrait,
    QueryFilter,
};
use tonic::{Request, Response, Status};

pub async fn create_shipping_address(
    txn: &DatabaseTransaction,
    request: Request<CreateShippingAddressRequest>,
) -> Result<Response<ShippingAddressesResponse>, Status> {
    let req = request.into_inner();
    let mut is_default = req.is_default;

    if let Some(user_id) = req.user_id {
        if !is_default {
            let existing_count = shipping_addresses::Entity::find()
                .filter(shipping_addresses::Column::UserId.eq(user_id))
                .count(txn)
                .await
                .map_err(map_db_error_to_status)?;
            if existing_count == 0 {
                is_default = true;
            }
        }

        if is_default {
            shipping_addresses::Entity::update_many()
                .col_expr(shipping_addresses::Column::IsDefault, Expr::value(0))
                .filter(shipping_addresses::Column::UserId.eq(user_id))
                .exec(txn)
                .await
                .map_err(map_db_error_to_status)?;
        }
    }

    let model = shipping_addresses::ActiveModel {
        shipping_address_id: ActiveValue::NotSet,
        user_id: ActiveValue::Set(req.user_id),
        is_default: ActiveValue::Set(if is_default { 1 } else { 0 }),
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
                is_default: inserted.is_default == 1,
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
