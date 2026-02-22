use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::inventory;
use proto::proto::core::{
    InventoryItemResponse, InventoryItemsResponse, SearchInventoryItemRequest,
};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn search_inventory_item(
    txn: &DatabaseTransaction,
    request: Request<SearchInventoryItemRequest>,
) -> Result<Response<InventoryItemsResponse>, Status> {
    let req = request.into_inner();

    let mut query = inventory::Entity::find();
    if req.inventory_id != 0 {
        query = query.filter(inventory::Column::InventoryId.eq(req.inventory_id));
    }
    if let Some(product_id) = req.product_id {
        query = query.filter(inventory::Column::ProductId.eq(product_id));
    }

    match query.all(txn).await {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|m| InventoryItemResponse {
                    inventory_id: m.inventory_id,
                    product_id: m.product_id.unwrap_or(0),
                    quantity_available: m.quantity_available.unwrap_or(0),
                    reorder_level: m.reorder_level.unwrap_or(0),
                    supplier_id: m.supplier_id.unwrap_or(0),
                })
                .collect();
            Ok(Response::new(InventoryItemsResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
