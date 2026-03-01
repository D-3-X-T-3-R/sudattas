use crate::handlers::db_errors::map_db_error_to_status;
use crate::money::{decimal_to_paise, paise_to_decimal};
use core_db_entities::entity::order_details;
use proto::proto::core::{CreateOrderDetailsRequest, OrderDetailResponse, OrderDetailsResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_order_details(
    txn: &DatabaseTransaction,
    request: Request<CreateOrderDetailsRequest>,
) -> Result<Response<OrderDetailsResponse>, Status> {
    let req = request.into_inner().order_details;

    let mut response: Vec<OrderDetailResponse> = vec![];
    let mut first_error = None;

    for details in req.iter() {
        let create_order_detail = order_details::ActiveModel {
            order_detail_id: ActiveValue::NotSet,
            order_id: ActiveValue::Set(details.order_id),
            product_id: ActiveValue::Set(details.product_id),
            quantity: ActiveValue::Set(details.quantity),
            price: ActiveValue::Set(paise_to_decimal(details.price_paise)),
            unit_price_minor: details
                .unit_price_minor
                .map(ActiveValue::Set)
                .unwrap_or(ActiveValue::NotSet)
                .into(),
            discount_minor: details
                .discount_minor
                .map(ActiveValue::Set)
                .unwrap_or(ActiveValue::NotSet)
                .into(),
            tax_minor: details
                .tax_minor
                .map(ActiveValue::Set)
                .unwrap_or(ActiveValue::NotSet)
                .into(),
            sku: details
                .sku
                .clone()
                .map(ActiveValue::Set)
                .unwrap_or(ActiveValue::NotSet)
                .into(),
            title: details
                .title
                .clone()
                .map(ActiveValue::Set)
                .unwrap_or(ActiveValue::NotSet)
                .into(),
            line_attrs: ActiveValue::NotSet,
        };

        match create_order_detail.insert(txn).await {
            Ok(model) => {
                response.push(OrderDetailResponse {
                    order_detail_id: model.order_detail_id,
                    order_id: model.order_id,
                    product_id: model.product_id,
                    quantity: model.quantity,
                    price_paise: decimal_to_paise(&model.price),
                });
            }
            Err(e) => {
                first_error.get_or_insert_with(|| map_db_error_to_status(e));
                break;
            }
        }
    }

    Ok(Response::new(OrderDetailsResponse { items: response }))
}
