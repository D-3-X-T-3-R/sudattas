use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::inventory_log;
use proto::proto::core::{
    DeleteInventoryLogRequest, InventoryLogResponse, InventoryLogsResponse,
};
use sea_orm::{DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn delete_inventory_log(
    txn: &DatabaseTransaction,
    request: Request<DeleteInventoryLogRequest>,
) -> Result<Response<InventoryLogsResponse>, Status> {
    let req = request.into_inner();

    let found = inventory_log::Entity::find_by_id(req.log_id).one(txn).await;

    match found {
        Ok(Some(model)) => {
            match inventory_log::Entity::delete_by_id(req.log_id).exec(txn).await {
                Ok(_) => Ok(Response::new(InventoryLogsResponse {
                    items: vec![InventoryLogResponse {
                        log_id: model.log_id,
                        product_id: model.product_id,
                        change_quantity: model.change_quantity,
                        log_time: model.log_time.to_rfc3339(),
                        reason: model.reason.unwrap_or_default(),
                    }],
                })),
                Err(e) => Err(map_db_error_to_status(e)),
            }
        }
        Ok(None) => Err(Status::not_found(format!(
            "InventoryLog with ID {} not found",
            req.log_id
        ))),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
