use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::inventory_log;
use proto::proto::core::{InventoryLogResponse, InventoryLogsResponse, SearchInventoryLogRequest};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryTrait};
use tonic::{Request, Response, Status};

pub async fn search_inventory_log(
    txn: &DatabaseTransaction,
    request: Request<SearchInventoryLogRequest>,
) -> Result<Response<InventoryLogsResponse>, Status> {
    let req = request.into_inner();

    let mut query = inventory_log::Entity::find();
    if req.log_id != 0 {
        query = query.filter(inventory_log::Column::LogId.eq(req.log_id));
    }

    match query.all(txn).await {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|m| InventoryLogResponse {
                    log_id: m.log_id,
                    product_id: m.product_id,
                    change_quantity: m.change_quantity,
                    log_time: m.log_time.to_rfc3339(),
                    reason: m.reason.unwrap_or_default(),
                })
                .collect();
            Ok(Response::new(InventoryLogsResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
