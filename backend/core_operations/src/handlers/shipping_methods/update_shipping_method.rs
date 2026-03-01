use crate::handlers::db_errors::map_db_error_to_status;
use crate::money::{decimal_to_paise, paise_to_decimal};
use core_db_entities::entity::shipping_methods;
use proto::proto::core::{
    ShippingMethodResponse, ShippingMethodsResponse, UpdateShippingMethodRequest,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn update_shipping_method(
    txn: &DatabaseTransaction,
    request: Request<UpdateShippingMethodRequest>,
) -> Result<Response<ShippingMethodsResponse>, Status> {
    let req = request.into_inner();

    let existing = shipping_methods::Entity::find_by_id(req.method_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| {
            Status::not_found(format!(
                "ShippingMethod with ID {} not found",
                req.method_id
            ))
        })?;

    let cost = req.cost_paise.map(paise_to_decimal).or(existing.cost);

    let model = shipping_methods::ActiveModel {
        method_id: ActiveValue::Set(existing.method_id),
        method_name: ActiveValue::Set(req.method_name.or(existing.method_name)),
        cost: ActiveValue::Set(cost),
        estimated_delivery_time: ActiveValue::Set(
            req.estimated_delivery_time
                .or(existing.estimated_delivery_time),
        ),
    };

    match model.update(txn).await {
        Ok(updated) => Ok(Response::new(ShippingMethodsResponse {
            items: vec![ShippingMethodResponse {
                method_id: updated.method_id,
                method_name: updated.method_name.unwrap_or_default(),
                cost_paise: updated.cost.as_ref().map(decimal_to_paise).unwrap_or(0),
                estimated_delivery_time: updated.estimated_delivery_time.unwrap_or_default(),
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
