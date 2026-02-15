use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::product_attribute_mapping;
use proto::proto::core::{
    CreateProductAttributeMappingRequest, ProductAttributeMappingResponse,
    ProductAttributeMappingsResponse,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_product_attribute_mapping(
    txn: &DatabaseTransaction,
    request: Request<CreateProductAttributeMappingRequest>,
) -> Result<Response<ProductAttributeMappingsResponse>, Status> {
    let req = request.into_inner();
    let model = product_attribute_mapping::ActiveModel {
        product_id: ActiveValue::Set(req.product_id),
        attribute_id: ActiveValue::Set(req.attribute_id),
    };

    match model.insert(txn).await {
        Ok(inserted) => Ok(Response::new(ProductAttributeMappingsResponse {
            items: vec![ProductAttributeMappingResponse {
                product_id: inserted.product_id,
                attribute_id: inserted.attribute_id,
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
