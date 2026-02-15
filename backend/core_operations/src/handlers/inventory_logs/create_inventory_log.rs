use crate::handlers::db_errors::map_db_error_to_status;
use chrono::Utc;
use core_db_entities::entity::inventory_log;
use proto::proto::core::{
    CreateInventoryLogRequest, InventoryLogResponse, InventoryLogsResponse,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_inventory_log(
    txn: &DatabaseTransaction,
    request: Request<CreateInventoryLogRequest>,
) -> Result<Response<InventoryLogsResponse>, Status> {
    let req = request.into_inner();
    let model = inventory_log::ActiveModel {
        log_id: ActiveValue::NotSet,
        product_id: ActiveValue::Set(req.product_id),
        change_quantity: ActiveValue::Set(req.change_quantity),
        log_time: ActiveValue::Set(Utc::now()),
        reason: ActiveValue::Set(Some(req.reason)),
    };

    match model.insert(txn).await {
        Ok(inserted) => Ok(Response::new(InventoryLogsResponse {
            items: vec![InventoryLogResponse {
                log_id: inserted.log_id,
                product_id: inserted.product_id,
                change_quantity: inserted.change_quantity,
                log_time: inserted.log_time.to_rfc3339(),
                reason: inserted.reason.unwrap_or_default(),
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
