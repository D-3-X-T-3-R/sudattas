use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::product_size_mapping;
use proto::proto::core::{
    CreateProductSizeMappingRequest, ProductSizeMappingResponse,
    ProductSizeMappingsResponse,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_product_size_mapping(
    txn: &DatabaseTransaction,
    request: Request<CreateProductSizeMappingRequest>,
) -> Result<Response<ProductSizeMappingsResponse>, Status> {
    let req = request.into_inner();
    let model = product_size_mapping::ActiveModel {
        product_id: ActiveValue::Set(req.product_id),
        size_id: ActiveValue::Set(req.size_id),
    };

    match model.insert(txn).await {
        Ok(inserted) => Ok(Response::new(ProductSizeMappingsResponse {
            items: vec![ProductSizeMappingResponse {
                product_id: inserted.product_id,
                size_id: inserted.size_id,
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
