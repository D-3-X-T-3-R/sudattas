use crate::handlers::db_errors::map_db_error_to_status;
use crate::money::decimal_to_paise;
use core_db_entities::entity::shipping_methods;
use proto::proto::core::{
    DeleteShippingMethodRequest, ShippingMethodResponse, ShippingMethodsResponse,
};
use sea_orm::{DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn delete_shipping_method(
    txn: &DatabaseTransaction,
    request: Request<DeleteShippingMethodRequest>,
) -> Result<Response<ShippingMethodsResponse>, Status> {
    let req = request.into_inner();

    let found = shipping_methods::Entity::find_by_id(req.method_id)
        .one(txn)
        .await;

    match found {
        Ok(Some(model)) => {
            match shipping_methods::Entity::delete_by_id(req.method_id)
                .exec(txn)
                .await
            {
                Ok(_) => Ok(Response::new(ShippingMethodsResponse {
                    items: vec![ShippingMethodResponse {
                        method_id: model.method_id,
                        method_name: model.method_name.unwrap_or_default(),
                        cost_paise: model.cost.as_ref().map(decimal_to_paise).unwrap_or(0),
                        estimated_delivery_time: model.estimated_delivery_time.unwrap_or_default(),
                    }],
                })),
                Err(e) => Err(map_db_error_to_status(e)),
            }
        }
        Ok(None) => Err(Status::not_found(format!(
            "ShippingMethod with ID {} not found",
            req.method_id
        ))),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
