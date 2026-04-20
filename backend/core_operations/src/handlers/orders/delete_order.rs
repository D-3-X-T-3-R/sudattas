use crate::handlers::db_errors::map_db_error_to_status;
use crate::handlers::orders::{cancel_order_items, order_response};
use chrono::Utc;
use core_db_entities::entity::sea_orm_active_enums::FulfillmentStatus;
use core_db_entities::entity::{order_details, orders};
use proto::proto::core::{CancelOrderItemsRequest, DeleteOrderRequest, OrdersResponse};
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

    if existing.fulfillment_status != FulfillmentStatus::NotCreated {
        return Err(Status::failed_precondition(
            "Cancellation window closed. You can refuse delivery.",
        ));
    }

    if !crate::order_policy::is_within_cancel_window(existing.created_at, Utc::now()) {
        return Err(Status::failed_precondition(
            "Cancellation window closed. You can refuse delivery.",
        ));
    }

    let active_details = order_details::Entity::find()
        .filter(order_details::Column::OrderId.eq(req.order_id))
        .filter(order_details::Column::ItemStatus.eq("active"))
        .all(txn)
        .await
        .map_err(map_db_error_to_status)?;
    if active_details.is_empty() {
        return Ok(Response::new(OrdersResponse {
            items: vec![order_response::from_model(&existing)],
        }));
    }

    cancel_order_items(
        txn,
        Request::new(CancelOrderItemsRequest {
            order_id: req.order_id,
            order_detail_ids: active_details
                .into_iter()
                .map(|d| d.order_detail_id)
                .collect(),
            acting_user_id: req.acting_user_id,
        }),
    )
    .await
}
