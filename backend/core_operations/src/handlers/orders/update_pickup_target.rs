use crate::handlers::db_errors::map_db_error_to_status;
use chrono::{DateTime, Utc};
use core_db_entities::entity::{order_status, orders};
use proto::proto::core::{UpdatePickupTargetRequest, UpdatePickupTargetResponse};
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseTransaction, DbBackend, EntityTrait, QueryFilter,
    Statement,
};
use tonic::{Request, Response, Status};

pub async fn update_pickup_target(
    txn: &DatabaseTransaction,
    request: Request<UpdatePickupTargetRequest>,
) -> Result<Response<UpdatePickupTargetResponse>, Status> {
    let req = request.into_inner();
    let target_raw = req.pickup_target_at.trim();
    if target_raw.is_empty() {
        return Err(Status::invalid_argument("pickup_target_at is required"));
    }

    let parsed_target = DateTime::parse_from_rfc3339(target_raw)
        .map_err(|_| Status::invalid_argument("pickup_target_at must be RFC3339"))?
        .with_timezone(&Utc);

    let order = orders::Entity::find_by_id(req.order_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::not_found("Order not found"))?;
    let status_name = order_status::Entity::find()
        .filter(order_status::Column::StatusId.eq(order.status_id))
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .map(|s| s.status_name.to_lowercase())
        .unwrap_or_default();
    if status_name == "cancelled" {
        return Err(Status::failed_precondition(
            "Cannot update pickup target for a cancelled order",
        ));
    }

    let actor = req
        .actor_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("admin")
        .to_string();
    let reason = req
        .reason
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(std::string::ToString::to_string);

    txn.execute(Statement::from_sql_and_values(
        DbBackend::MySql,
        r#"UPDATE Orders
           SET pickup_target_at = ?,
               pickup_target_reason = ?,
               pickup_target_set_by = ?,
               pickup_target_updated_at = UTC_TIMESTAMP(),
               updated_at = UTC_TIMESTAMP()
           WHERE OrderID = ?"#,
        [
            parsed_target.into(),
            reason.clone().into(),
            actor.clone().into(),
            req.order_id.into(),
        ],
    ))
    .await
    .map_err(map_db_error_to_status)?;

    let row = txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::MySql,
            r#"SELECT pickup_target_at,
                      pickup_target_reason,
                      pickup_target_set_by,
                      pickup_target_updated_at
               FROM Orders
               WHERE OrderID = ?
               LIMIT 1"#,
            [req.order_id.into()],
        ))
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::not_found("Order not found"))?;

    let pickup_target_at: DateTime<Utc> = row
        .try_get("", "pickup_target_at")
        .map_err(|e| Status::internal(e.to_string()))?;
    let pickup_target_reason: Option<String> = row.try_get("", "pickup_target_reason").ok();
    let pickup_target_set_by: Option<String> = row.try_get("", "pickup_target_set_by").ok();
    let pickup_target_updated_at: DateTime<Utc> = row
        .try_get("", "pickup_target_updated_at")
        .map_err(|e| Status::internal(e.to_string()))?;

    Ok(Response::new(UpdatePickupTargetResponse {
        order_id: req.order_id,
        pickup_target_at: pickup_target_at.to_rfc3339(),
        pickup_target_reason,
        pickup_target_set_by,
        pickup_target_updated_at: pickup_target_updated_at.to_rfc3339(),
    }))
}
