use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::{cart, inventory, inventory_log, order_details, product_variants};
use proto::proto::core::{
    DeleteProductVariantRequest, ProductVariantResponse, ProductVariantsResponse,
};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn delete_product_variant(
    txn: &DatabaseTransaction,
    request: Request<DeleteProductVariantRequest>,
) -> Result<Response<ProductVariantsResponse>, Status> {
    let req = request.into_inner();

    let found = product_variants::Entity::find_by_id(req.variant_id)
        .one(txn)
        .await;

    match found {
        Ok(Some(model)) => {
            // Refuse outright if this variant has real order history — deleting it out from
            // under a placed order would corrupt that order's line items. Mirrors the same
            // safety check permanently_delete_product does at the product level.
            let has_order_history = order_details::Entity::find()
                .filter(order_details::Column::VariantId.eq(req.variant_id))
                .one(txn)
                .await
                .map_err(map_db_error_to_status)?
                .is_some();
            if has_order_history {
                return Err(Status::failed_precondition(format!(
                    "ProductVariant {} has order history and cannot be deleted.",
                    req.variant_id
                )));
            }

            // Cart/Inventory/InventoryLog all FK-reference VariantID with no ON DELETE CASCADE
            // (RESTRICT by default) — a plain delete_by_id below would otherwise fail outright
            // the moment any of these exist, which they normally do (every variant gets an
            // Inventory row when created). None of these represent order history, so it's safe
            // to clear them here rather than block the deletion.
            cart::Entity::delete_many()
                .filter(cart::Column::VariantId.eq(req.variant_id))
                .exec(txn)
                .await
                .map_err(map_db_error_to_status)?;
            inventory_log::Entity::delete_many()
                .filter(inventory_log::Column::VariantId.eq(req.variant_id))
                .exec(txn)
                .await
                .map_err(map_db_error_to_status)?;
            inventory::Entity::delete_many()
                .filter(inventory::Column::VariantId.eq(req.variant_id))
                .exec(txn)
                .await
                .map_err(map_db_error_to_status)?;

            match product_variants::Entity::delete_by_id(req.variant_id)
                .exec(txn)
                .await
            {
                Ok(_) => Ok(Response::new(ProductVariantsResponse {
                    items: vec![ProductVariantResponse {
                        variant_id: model.variant_id,
                        product_id: model.product_id,
                        size_id: model.size_id,
                        color_id: model.color_id,
                        additional_price_paise: model.additional_price.map(i64::from),
                    }],
                })),
                Err(e) => Err(map_db_error_to_status(e)),
            }
        }
        Ok(None) => Err(Status::not_found(format!(
            "ProductVariant with ID {} not found",
            req.variant_id
        ))),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
