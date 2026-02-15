use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::inventory_log;
use proto::proto::core::{
    InventoryLogResponse, InventoryLogsResponse, UpdateInventoryLogRequest,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn update_inventory_log(
    txn: &DatabaseTransaction,
    request: Request<UpdateInventoryLogRequest>,
) -> Result<Response<InventoryLogsResponse>, Status> {
    let req = request.into_inner();

    let existing = inventory_log::Entity::find_by_id(req.log_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| {
            Status::not_found(format!(
                "InventoryLog with ID {} not found",
                req.log_id
            ))
        })?;

    let model = inventory_log::ActiveModel {
        log_id: ActiveValue::Set(existing.log_id),
        product_id: ActiveValue::Set(req.product_id.unwrap_or(existing.product_id)),
        change_quantity: ActiveValue::Set(req.change_quantity.unwrap_or(existing.change_quantity)),
        log_time: ActiveValue::Set(existing.log_time),
        reason: ActiveValue::Set(req.reason.or(existing.reason)),
    };

    match model.update(txn).await {
        Ok(updated) => Ok(Response::new(InventoryLogsResponse {
            items: vec![InventoryLogResponse {
                log_id: updated.log_id,
                product_id: updated.product_id,
                change_quantity: updated.change_quantity,
                log_time: updated.log_time.to_rfc3339(),
                reason: updated.reason.unwrap_or_default(),
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
