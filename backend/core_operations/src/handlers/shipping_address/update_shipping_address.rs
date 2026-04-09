use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::shipping_addresses;
use proto::proto::core::{
    ShippingAddressResponse, ShippingAddressesResponse, UpdateShippingAddressRequest,
};
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter,
};
use tonic::{Request, Response, Status};

pub async fn update_shipping_address(
    txn: &DatabaseTransaction,
    request: Request<UpdateShippingAddressRequest>,
) -> Result<Response<ShippingAddressesResponse>, Status> {
    let req = request.into_inner();
    let mut final_is_default = req.is_default;

    if let Some(user_id) = req.user_id {
        if req.is_default {
            shipping_addresses::Entity::update_many()
                .col_expr(shipping_addresses::Column::IsDefault, Expr::value(0))
                .filter(shipping_addresses::Column::UserId.eq(user_id))
                .exec(txn)
                .await
                .map_err(map_db_error_to_status)?;
        } else {
            let has_other_default = shipping_addresses::Entity::find()
                .filter(shipping_addresses::Column::UserId.eq(user_id))
                .filter(shipping_addresses::Column::ShippingAddressId.ne(req.shipping_address_id))
                .filter(shipping_addresses::Column::IsDefault.eq(1))
                .one(txn)
                .await
                .map_err(map_db_error_to_status)?
                .is_some();
            if !has_other_default {
                final_is_default = true;
                shipping_addresses::Entity::update_many()
                    .col_expr(shipping_addresses::Column::IsDefault, Expr::value(0))
                    .filter(shipping_addresses::Column::UserId.eq(user_id))
                    .exec(txn)
                    .await
                    .map_err(map_db_error_to_status)?;
            }
        }
    }

    let is_default_value = if final_is_default { 1 } else { 0 };
    let model = shipping_addresses::ActiveModel {
        shipping_address_id: ActiveValue::Set(req.shipping_address_id),
        user_id: ActiveValue::Set(req.user_id),
        is_default: ActiveValue::Set(is_default_value),
        country: ActiveValue::Set(req.country),
        state_region: ActiveValue::Set(req.state_region),
        city: ActiveValue::Set(req.city),
        postal_code: ActiveValue::Set(req.postal_code),
        road: ActiveValue::Set(req.road),
        apartment_no_or_name: ActiveValue::Set(req.apartment_no_or_name),
        recipient_name: ActiveValue::Set(req.recipient_name),
        phone_number: ActiveValue::Set(req.phone_number),
    };

    match model.update(txn).await {
        Ok(updated) => Ok(Response::new(ShippingAddressesResponse {
            items: vec![ShippingAddressResponse {
                shipping_address_id: updated.shipping_address_id,
                user_id: updated.user_id,
                is_default: updated.is_default == 1,
                country: updated.country,
                state_region: updated.state_region,
                city: updated.city,
                postal_code: updated.postal_code,
                road: updated.road,
                apartment_no_or_name: updated.apartment_no_or_name,
                recipient_name: updated.recipient_name,
                phone_number: updated.phone_number,
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
