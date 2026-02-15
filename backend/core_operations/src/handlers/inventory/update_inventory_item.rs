use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::inventory;
use proto::proto::core::{
    InventoryItemResponse, InventoryItemsResponse, UpdateInventoryItemRequest,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn update_inventory_item(
    txn: &DatabaseTransaction,
    request: Request<UpdateInventoryItemRequest>,
) -> Result<Response<InventoryItemsResponse>, Status> {
    let req = request.into_inner();

    let existing = inventory::Entity::find_by_id(req.inventory_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| {
            Status::not_found(format!(
                "Inventory item with ID {} not found",
                req.inventory_id
            ))
        })?;

    let model = inventory::ActiveModel {
        inventory_id: ActiveValue::Set(existing.inventory_id),
        product_id: ActiveValue::Set(req.product_id.or(existing.product_id)),
        quantity_available: ActiveValue::Set(req.quantity_available.or(existing.quantity_available)),
        reorder_level: ActiveValue::Set(req.reorder_level.or(existing.reorder_level)),
        supplier_id: ActiveValue::Set(req.supplier_id.or(existing.supplier_id)),
    };

    match model.update(txn).await {
        Ok(updated) => Ok(Response::new(InventoryItemsResponse {
            items: vec![InventoryItemResponse {
                inventory_id: updated.inventory_id,
                product_id: updated.product_id.unwrap_or(0),
                quantity_available: updated.quantity_available.unwrap_or(0),
                reorder_level: updated.reorder_level.unwrap_or(0),
                supplier_id: updated.supplier_id.unwrap_or(0),
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
