use crate::handlers::db_errors::map_db_error_to_status;
use crate::money::{decimal_to_paise, paise_to_decimal};
use core_db_entities::entity::order_details;
use proto::proto::core::{OrderDetailResponse, OrderDetailsResponse, UpdateOrderDetailRequest};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn update_order_detail(
    txn: &DatabaseTransaction,
    request: Request<UpdateOrderDetailRequest>,
) -> Result<Response<OrderDetailsResponse>, Status> {
    let req = request.into_inner();

    let order_details = order_details::ActiveModel {
        order_detail_id: ActiveValue::Set(req.order_detail_id),
        order_id: ActiveValue::Set(req.order_id),
        variant_id: ActiveValue::Set(req.variant_id),
        quantity: ActiveValue::Set(req.quantity),
        price: ActiveValue::Set(Some(paise_to_decimal(req.price_paise))),
        unit_price_minor: ActiveValue::NotSet,
        discount_minor: ActiveValue::NotSet,
        tax_minor: ActiveValue::NotSet,
        sku: ActiveValue::NotSet,
        title: ActiveValue::NotSet,
        line_attrs: ActiveValue::NotSet,
    };
    match order_details.update(txn).await {
        Ok(model) => {
            let response = OrderDetailsResponse {
                items: vec![OrderDetailResponse {
                    order_detail_id: model.order_detail_id,
                    order_id: model.order_id,
                    variant_id: model.variant_id,
                    quantity: model.quantity,
                    price_paise: model.price.as_ref().map(decimal_to_paise).unwrap_or(0),
                }],
            };
            Ok(Response::new(response))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
