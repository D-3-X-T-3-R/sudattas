use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::inventory;
use proto::proto::core::{
    CreateInventoryItemRequest, InventoryItemResponse, InventoryItemsResponse,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_inventory_item(
    txn: &DatabaseTransaction,
    request: Request<CreateInventoryItemRequest>,
) -> Result<Response<InventoryItemsResponse>, Status> {
    let req = request.into_inner();
    let model = inventory::ActiveModel {
        inventory_id: ActiveValue::NotSet,
        product_id: ActiveValue::Set(Some(req.product_id)),
        quantity_available: ActiveValue::Set(Some(req.quantity_available)),
        reorder_level: ActiveValue::Set(Some(req.reorder_level)),
        supplier_id: ActiveValue::Set(Some(req.supplier_id)),
    };

    match model.insert(txn).await {
        Ok(inserted) => Ok(Response::new(InventoryItemsResponse {
            items: vec![InventoryItemResponse {
                inventory_id: inserted.inventory_id,
                product_id: inserted.product_id.unwrap_or(0),
                quantity_available: inserted.quantity_available.unwrap_or(0),
                reorder_level: inserted.reorder_level.unwrap_or(0),
                supplier_id: inserted.supplier_id.unwrap_or(0),
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
