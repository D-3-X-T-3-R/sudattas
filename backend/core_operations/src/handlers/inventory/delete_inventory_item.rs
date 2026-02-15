use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::inventory;
use proto::proto::core::{
    DeleteInventoryItemRequest, InventoryItemResponse, InventoryItemsResponse,
};
use sea_orm::{DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn delete_inventory_item(
    txn: &DatabaseTransaction,
    request: Request<DeleteInventoryItemRequest>,
) -> Result<Response<InventoryItemsResponse>, Status> {
    let req = request.into_inner();

    let found = inventory::Entity::find_by_id(req.inventory_id)
        .one(txn)
        .await;

    match found {
        Ok(Some(model)) => {
            match inventory::Entity::delete_by_id(req.inventory_id).exec(txn).await {
                Ok(_) => Ok(Response::new(InventoryItemsResponse {
                    items: vec![InventoryItemResponse {
                        inventory_id: model.inventory_id,
                        product_id: model.product_id.unwrap_or(0),
                        quantity_available: model.quantity_available.unwrap_or(0),
                        reorder_level: model.reorder_level.unwrap_or(0),
                        supplier_id: model.supplier_id.unwrap_or(0),
                    }],
                })),
                Err(e) => Err(map_db_error_to_status(e)),
            }
        }
        Ok(None) => Err(Status::not_found(format!(
            "Inventory item with ID {} not found",
            req.inventory_id
        ))),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
