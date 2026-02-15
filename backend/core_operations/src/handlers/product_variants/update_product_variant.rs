use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::product_variants;
use proto::proto::core::{
    ProductVariantResponse, ProductVariantsResponse, UpdateProductVariantRequest,
};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn update_product_variant(
    txn: &DatabaseTransaction,
    request: Request<UpdateProductVariantRequest>,
) -> Result<Response<ProductVariantsResponse>, Status> {
    let req = request.into_inner();

    let existing = product_variants::Entity::find_by_id(req.variant_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| {
            Status::not_found(format!(
                "ProductVariant with ID {} not found",
                req.variant_id
            ))
        })?;

    let additional_price = req
        .additional_price
        .and_then(Decimal::from_f64_retain)
        .or(existing.additional_price);

    let model = product_variants::ActiveModel {
        variant_id: ActiveValue::Set(existing.variant_id),
        product_id: ActiveValue::Set(req.product_id.unwrap_or(existing.product_id)),
        size_id: ActiveValue::Set(req.size_id.or(existing.size_id)),
        color_id: ActiveValue::Set(req.color_id.or(existing.color_id)),
        additional_price: ActiveValue::Set(additional_price),
    };

    match model.update(txn).await {
        Ok(updated) => Ok(Response::new(ProductVariantsResponse {
            items: vec![ProductVariantResponse {
                variant_id: updated.variant_id,
                product_id: updated.product_id,
                size_id: updated.size_id,
                color_id: updated.color_id,
                additional_price: updated
                    .additional_price
                    .as_ref()
                    .and_then(ToPrimitive::to_f64),
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
