use crate::handlers::db_errors::map_db_error_to_status;
use crate::money::{decimal_to_paise, paise_to_decimal};
use core_db_entities::entity::order_details;
use proto::proto::core::{OrderDetailResponse, OrderDetailsResponse, SearchOrderDetailRequest};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryTrait};
use tonic::{Request, Response, Status};

pub async fn search_order_detail(
    txn: &DatabaseTransaction,
    request: Request<SearchOrderDetailRequest>,
) -> Result<Response<OrderDetailsResponse>, Status> {
    let req = request.into_inner();

    match order_details::Entity::find()
        .apply_if(req.order_detail_id, |query, _| {
            query.filter(order_details::Column::OrderDetailId.eq(req.order_detail_id))
        })
        .apply_if(req.order_id, |query, _| {
            query.filter(order_details::Column::OrderId.eq(req.order_id))
        })
        .apply_if(req.product_id, |query, _| {
            query.filter(order_details::Column::ProductId.eq(req.product_id))
        })
        .apply_if(req.quantity, |query, _| {
            query.filter(order_details::Column::Quantity.eq(req.quantity))
        })
        .apply_if(req.price_start_paise, |query, v| {
            query.filter(order_details::Column::Price.gte(paise_to_decimal(v)))
        })
        .apply_if(req.price_end_paise, |query, v| {
            query.filter(order_details::Column::Price.lte(paise_to_decimal(v)))
        })
        .all(txn)
        .await
    {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|model| OrderDetailResponse {
                    order_detail_id: model.order_detail_id,
                    order_id: model.order_id,
                    product_id: model.product_id,
                    quantity: model.quantity,
                    price_paise: decimal_to_paise(&model.price),
                })
                .collect();

            Ok(Response::new(OrderDetailsResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
