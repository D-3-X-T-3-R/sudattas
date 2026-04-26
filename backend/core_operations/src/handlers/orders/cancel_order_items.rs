use crate::cancellation_saga;
use crate::handlers::db_errors::map_db_error_to_status;
use crate::handlers::orders::order_response;
use chrono::Utc;
use core_db_entities::entity::sea_orm_active_enums::FulfillmentStatus;
use core_db_entities::entity::{
    order_details, order_status, orders, return_request_items, return_requests,
};
use proto::proto::core::{CancelOrderItemsRequest, OrdersResponse};
use sea_orm::{
    sea_query::LockType, ColumnTrait, ConnectionTrait, DatabaseTransaction, DbBackend, EntityTrait,
    JoinType, PaginatorTrait, QueryFilter, QuerySelect, RelationTrait, Statement,
};
use std::collections::HashSet;
use tonic::{Request, Response, Status};

pub async fn cancel_order_items(
    txn: &DatabaseTransaction,
    request: Request<CancelOrderItemsRequest>,
) -> Result<Response<OrdersResponse>, Status> {
    let req = request.into_inner();
    if req.order_detail_ids.is_empty() {
        return Err(Status::invalid_argument(
            "At least one order_detail_id is required",
        ));
    }

    let order = orders::Entity::find_by_id(req.order_id)
        .lock(LockType::Update)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::not_found("Order not found"))?;

    if let Some(uid) = req.acting_user_id {
        if order.user_id != uid {
            return Err(Status::not_found("Order not found"));
        }
    }

    let order_status_name = order_status::Entity::find_by_id(order.status_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .map(|row| row.status_name)
        .unwrap_or_default();
    if order_status_name.eq_ignore_ascii_case("delivered") {
        return Err(Status::failed_precondition(
            "Cancellation window closed. You can refuse delivery.",
        ));
    }

    if order.fulfillment_status != FulfillmentStatus::NotCreated {
        return Err(Status::failed_precondition(
            "Cancellation window closed. You can refuse delivery.",
        ));
    }

    let cancel_window_row = txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::MySql,
            r#"SELECT COALESCE(cancel_window_ends_at, DATE_ADD(created_at, INTERVAL ? HOUR)) AS cancel_window_ends_at
               FROM Orders
               WHERE OrderID = ?
               LIMIT 1"#,
            [
                crate::order_policy::cancel_window_hours().into(),
                req.order_id.into(),
            ],
        ))
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::not_found("Order not found"))?;
    let cancel_window_ends_at: chrono::DateTime<Utc> = cancel_window_row
        .try_get("", "cancel_window_ends_at")
        .map_err(|e| Status::internal(e.to_string()))?;

    if !crate::order_policy::is_before_deadline(Utc::now(), cancel_window_ends_at) {
        return Err(Status::failed_precondition(
            "Cancellation window closed. You can refuse delivery.",
        ));
    }

    let requested_ids: HashSet<i64> = req.order_detail_ids.iter().copied().collect();
    let mut items = order_details::Entity::find()
        .filter(order_details::Column::OrderId.eq(req.order_id))
        .filter(order_details::Column::OrderDetailId.is_in(requested_ids.iter().copied()))
        .all(txn)
        .await
        .map_err(map_db_error_to_status)?;

    if items.len() != requested_ids.len() {
        return Err(Status::not_found(
            "One or more order_detail_ids do not belong to the order",
        ));
    }
    if items
        .iter()
        .any(|row| row.item_status.eq_ignore_ascii_case("cancelled"))
    {
        return Err(Status::failed_precondition(
            "One or more order items are already cancelled",
        ));
    }

    let open_return_count = return_request_items::Entity::find()
        .join(
            JoinType::InnerJoin,
            return_request_items::Relation::ReturnRequests.def(),
        )
        .filter(return_requests::Column::OrderId.eq(req.order_id))
        .filter(return_request_items::Column::OrderDetailId.is_in(requested_ids.iter().copied()))
        .filter(return_requests::Column::Status.is_not_in(["rejected", "cancelled"]))
        .count(txn)
        .await
        .map_err(map_db_error_to_status)?;
    if open_return_count > 0 {
        return Err(Status::failed_precondition(
            "One or more order items already have an active return request",
        ));
    }

    let mut changed = 0_u64;
    for row in &mut items {
        let result = txn
            .execute(Statement::from_sql_and_values(
                DbBackend::MySql,
                r#"UPDATE OrderDetails
                   SET item_status = 'cancelled',
                       cancelled_at = COALESCE(cancelled_at, UTC_TIMESTAMP())
                   WHERE OrderID = ?
                     AND OrderDetailID = ?
                     AND item_status <> 'cancelled'"#,
                [req.order_id.into(), row.order_detail_id.into()],
            ))
            .await
            .map_err(map_db_error_to_status)?;
        changed += result.rows_affected();
    }

    if changed == 0 {
        return Err(Status::failed_precondition(
            "Selected order items were already cancelled",
        ));
    }

    cancellation_saga::restore_inventory_for_items(txn, req.order_id, &requested_ids).await?;

    let active_count = order_details::Entity::find()
        .filter(order_details::Column::OrderId.eq(req.order_id))
        .filter(order_details::Column::ItemStatus.eq("active"))
        .count(txn)
        .await
        .map_err(map_db_error_to_status)?;

    let target_status = if active_count == 0 {
        "cancelled"
    } else {
        "partially_cancelled"
    };
    let target_status_id = order_status::Entity::find()
        .filter(order_status::Column::StatusName.eq(target_status))
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::internal("Target order status is not configured"))?
        .status_id;

    txn.execute(Statement::from_sql_and_values(
        DbBackend::MySql,
        r#"UPDATE Orders
           SET StatusID = ?,
               updated_at = UTC_TIMESTAMP()
           WHERE OrderID = ?"#,
        [target_status_id.into(), req.order_id.into()],
    ))
    .await
    .map_err(map_db_error_to_status)?;

    cancellation_saga::run_order_settlement(txn, req.order_id).await?;

    let updated_order = orders::Entity::find_by_id(req.order_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::not_found("Order not found"))?;

    Ok(Response::new(OrdersResponse {
        items: vec![order_response::from_model(&updated_order)],
    }))
}
