//! List all order statuses (OrderStatus table) for admin dropdowns.

use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::order_status;
use proto::proto::core::{OrderStatusResponse, OrderStatusesResponse, SearchOrderStatusRequest};
use sea_orm::{DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn search_order_status(
    txn: &DatabaseTransaction,
    _request: Request<SearchOrderStatusRequest>,
) -> Result<Response<OrderStatusesResponse>, Status> {
    match order_status::Entity::find().all(txn).await {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|m| OrderStatusResponse {
                    status_id: m.status_id,
                    status_name: m.status_name,
                })
                .collect();
            Ok(Response::new(OrderStatusesResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
