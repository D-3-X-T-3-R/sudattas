use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::product_attributes;
use proto::proto::core::{
    ProductAttributeResponse, ProductAttributesResponse, UpdateProductAttributeRequest,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn update_product_attribute(
    txn: &DatabaseTransaction,
    request: Request<UpdateProductAttributeRequest>,
) -> Result<Response<ProductAttributesResponse>, Status> {
    let req = request.into_inner();

    let existing = product_attributes::Entity::find_by_id(req.attribute_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| {
            Status::not_found(format!(
                "ProductAttribute with ID {} not found",
                req.attribute_id
            ))
        })?;

    let model = product_attributes::ActiveModel {
        attribute_id: ActiveValue::Set(existing.attribute_id),
        attribute_name: ActiveValue::Set(
            req.attribute_name
                .unwrap_or_else(|| existing.attribute_name.clone()),
        ),
        attribute_value: ActiveValue::Set(
            req.attribute_value
                .unwrap_or_else(|| existing.attribute_value.clone()),
        ),
    };

    match model.update(txn).await {
        Ok(updated) => Ok(Response::new(ProductAttributesResponse {
            items: vec![ProductAttributeResponse {
                attribute_id: updated.attribute_id,
                attribute_name: updated.attribute_name,
                attribute_value: updated.attribute_value,
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
