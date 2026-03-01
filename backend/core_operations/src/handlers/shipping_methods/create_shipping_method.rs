use crate::handlers::db_errors::map_db_error_to_status;
use crate::money::{decimal_to_paise, paise_to_decimal};
use core_db_entities::entity::shipping_methods;
use proto::proto::core::{
    CreateShippingMethodRequest, ShippingMethodResponse, ShippingMethodsResponse,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_shipping_method(
    txn: &DatabaseTransaction,
    request: Request<CreateShippingMethodRequest>,
) -> Result<Response<ShippingMethodsResponse>, Status> {
    let req = request.into_inner();
    let model = shipping_methods::ActiveModel {
        method_id: ActiveValue::NotSet,
        method_name: ActiveValue::Set(Some(req.method_name)),
        cost: ActiveValue::Set(Some(paise_to_decimal(req.cost_paise))),
        estimated_delivery_time: ActiveValue::Set(Some(req.estimated_delivery_time)),
    };

    match model.insert(txn).await {
        Ok(inserted) => Ok(Response::new(ShippingMethodsResponse {
            items: vec![ShippingMethodResponse {
                method_id: inserted.method_id,
                method_name: inserted.method_name.unwrap_or_default(),
                cost_paise: inserted.cost.as_ref().map(decimal_to_paise).unwrap_or(0),
                estimated_delivery_time: inserted.estimated_delivery_time.unwrap_or_default(),
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
