use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::order_details;
use proto::proto::core::{OrderDetailResponse, OrderDetailsResponse, SearchOrderDetailRequest};
use rust_decimal::prelude::ToPrimitive;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryTrait};
use tonic::{Request, Response, Status};

pub async fn search_order_detail(
    db: &DatabaseConnection,
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
        .apply_if(req.price_start, |query, _| {
            query.filter(order_details::Column::Price.gte(req.price_start))
        })
        .apply_if(req.price_end, |query, _| {
            query.filter(order_details::Column::Price.lte(req.price_end))
        })
        .all(db)
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
                    price: model.price.to_f64().unwrap(),
                })
                .collect();

            Ok(Response::new(OrderDetailsResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
