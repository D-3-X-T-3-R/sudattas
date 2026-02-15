use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::shipping_methods;
use proto::proto::core::{
    SearchShippingMethodRequest, ShippingMethodResponse, ShippingMethodsResponse,
};
use rust_decimal::prelude::ToPrimitive;
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryTrait};
use tonic::{Request, Response, Status};

pub async fn search_shipping_method(
    txn: &DatabaseTransaction,
    request: Request<SearchShippingMethodRequest>,
) -> Result<Response<ShippingMethodsResponse>, Status> {
    let req = request.into_inner();

    let mut query = shipping_methods::Entity::find();
    if req.method_id != 0 {
        query = query.filter(shipping_methods::Column::MethodId.eq(req.method_id));
    }

    match query.all(txn).await {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|m| ShippingMethodResponse {
                    method_id: m.method_id,
                    method_name: m.method_name.unwrap_or_default(),
                    cost: m.cost.as_ref().and_then(ToPrimitive::to_f64).unwrap_or(0.0),
                    estimated_delivery_time: m.estimated_delivery_time.unwrap_or_default(),
                })
                .collect();
            Ok(Response::new(ShippingMethodsResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
