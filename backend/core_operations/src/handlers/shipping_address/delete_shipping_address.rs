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
        // Soft delete: Orders.ShippingAddressID is NOT NULL and references this table with no
        // ON DELETE CASCADE, so a hard delete_by_id always fails once the address has been used
        // on a real order — which, for any customer with order history, is every address they
        // have. Same "never hard-delete something order history points at" precedent already
        // used for user accounts (decision #3) and products (archive vs permanently-delete).
        Ok(Some(model)) => {
            let mut active: shipping_addresses::ActiveModel = model.clone().into();
            active.is_deleted = ActiveValue::Set(1);
            active.is_default = ActiveValue::Set(0);
            let updated = active.update(txn).await.map_err(map_db_error_to_status)?;

            if model.is_default == 1 {
                if let Some(uid) = model.user_id {
                    if let Some(next_default) = shipping_addresses::Entity::find()
                        .filter(shipping_addresses::Column::UserId.eq(uid))
                        .filter(shipping_addresses::Column::IsDeleted.eq(0))
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
                        let mut next_active: shipping_addresses::ActiveModel =
                            next_default.into();
                        next_active.is_default = ActiveValue::Set(1);
                        next_active
                            .update(txn)
                            .await
                            .map_err(map_db_error_to_status)?;
                    }
                }
            }

            Ok(Response::new(ShippingAddressesResponse {
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
            }))
        }
        Ok(None) => Err(Status::not_found(format!(
            "ShippingAddress with ID {} not found",
            req.shipping_address_id
        ))),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
