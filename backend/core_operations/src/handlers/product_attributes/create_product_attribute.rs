use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::product_attributes;
use proto::proto::core::{
    CreateProductAttributeRequest, ProductAttributeResponse, ProductAttributesResponse,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_product_attribute(
    txn: &DatabaseTransaction,
    request: Request<CreateProductAttributeRequest>,
) -> Result<Response<ProductAttributesResponse>, Status> {
    let req = request.into_inner();
    let model = product_attributes::ActiveModel {
        attribute_id: ActiveValue::NotSet,
        product_id: ActiveValue::Set(Some(req.product_id)),
        attribute_name: ActiveValue::Set(Some(req.attribute_name)),
        attribute_value: ActiveValue::Set(Some(req.attribute_value)),
    };

    match model.insert(txn).await {
        Ok(inserted) => Ok(Response::new(ProductAttributesResponse {
            items: vec![ProductAttributeResponse {
                attribute_id: inserted.attribute_id,
                product_id: inserted.product_id.unwrap_or(0),
                attribute_name: inserted.attribute_name.unwrap_or_default(),
                attribute_value: inserted.attribute_value.unwrap_or_default(),
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
