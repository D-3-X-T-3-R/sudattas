use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::shipping_addresses;
use proto::proto::core::{
    DeleteShippingAddressRequest, ShippingAddressResponse, ShippingAddressesResponse,
};
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter,
};
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
                Ok(_) => {
                    if model.is_default == 1 {
                        if let Some(uid) = model.user_id {
                            if let Some(next_default) = shipping_addresses::Entity::find()
                                .filter(shipping_addresses::Column::UserId.eq(uid))
                                .one(txn)
                                .await
                                .map_err(map_db_error_to_status)?
                            {
                                shipping_addresses::Entity::update_many()
                                    .col_expr(shipping_addresses::Column::IsDefault, Expr::value(0))
                                    .filter(shipping_addresses::Column::UserId.eq(uid))
                                    .exec(txn)
                                    .await
                                    .map_err(map_db_error_to_status)?;
                                let mut active: shipping_addresses::ActiveModel =
                                    next_default.into();
                                active.is_default = ActiveValue::Set(1);
                                active.update(txn).await.map_err(map_db_error_to_status)?;
                            }
                        }
                    }

                    Ok(Response::new(ShippingAddressesResponse {
                        items: vec![ShippingAddressResponse {
                            shipping_address_id: model.shipping_address_id,
                            user_id: model.user_id,
                            is_default: model.is_default == 1,
                            country: model.country,
                            state_region: model.state_region,
                            city: model.city,
                            postal_code: model.postal_code,
                            road: model.road,
                            apartment_no_or_name: model.apartment_no_or_name,
                            recipient_name: model.recipient_name,
                            phone_number: model.phone_number,
                        }],
                    }))
                }
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
