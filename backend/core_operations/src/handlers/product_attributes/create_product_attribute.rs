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
        attribute_name: ActiveValue::Set(req.attribute_name),
        attribute_value: ActiveValue::Set(req.attribute_value),
    };

    match model.insert(txn).await {
        Ok(inserted) => Ok(Response::new(ProductAttributesResponse {
            items: vec![ProductAttributeResponse {
                attribute_id: inserted.attribute_id,
                attribute_name: inserted.attribute_name,
                attribute_value: inserted.attribute_value,
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
