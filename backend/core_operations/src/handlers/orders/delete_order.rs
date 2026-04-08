use crate::handlers::db_errors::map_db_error_to_status;
use crate::handlers::orders::update_order;
use core_db_entities::entity::{order_status, orders};
use proto::proto::core::{DeleteOrderRequest, OrderResponse, OrdersResponse, UpdateOrderRequest};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

/// Cancels an order (status → `cancelled`) using the same rules and side effects as [update_order].
/// Does not remove rows from the database.
///
/// When `acting_user_id` is set, the order must belong to that user; otherwise the caller is treated
/// as an admin/service and may cancel any order by id.
pub async fn delete_order(
    txn: &DatabaseTransaction,
    request: Request<DeleteOrderRequest>,
) -> Result<Response<OrdersResponse>, Status> {
    let req = request.into_inner();

    let existing = orders::Entity::find_by_id(req.order_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?;
    let existing = existing.ok_or_else(|| Status::not_found("Order not found"))?;

    if let Some(uid) = req.acting_user_id {
        if existing.user_id != uid {
            return Err(Status::not_found("Order not found"));
        }
    }

    let cancelled_row = order_status::Entity::find()
        .filter(order_status::Column::StatusName.eq("cancelled"))
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::internal("Cancelled status not configured"))?;

    if existing.status_id == cancelled_row.status_id {
        let total_amount_paise = existing.grand_total_minor;
        return Ok(Response::new(OrdersResponse {
            items: vec![OrderResponse {
                order_id: existing.order_id,
                user_id: existing.user_id,
                order_date: existing.order_date.to_string(),
                shipping_address_id: existing.shipping_address_id,
                total_amount_paise,
                status_id: existing.status_id,
            }],
        }));
    }

    update_order(
        txn,
        Request::new(UpdateOrderRequest {
            order_id: existing.order_id,
            user_id: existing.user_id,
            shipping_address_id: existing.shipping_address_id,
            total_amount_paise: existing.grand_total_minor,
            status_id: cancelled_row.status_id,
        }),
    )
    .await
}
