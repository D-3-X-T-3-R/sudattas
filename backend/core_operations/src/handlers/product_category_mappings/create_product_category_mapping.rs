use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::product_category_mapping;
use proto::proto::core::{
    CreateProductCategoryMappingRequest, ProductCategoryMappingResponse,
    ProductCategoryMappingsResponse,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_product_category_mapping(
    txn: &DatabaseTransaction,
    request: Request<CreateProductCategoryMappingRequest>,
) -> Result<Response<ProductCategoryMappingsResponse>, Status> {
    let req = request.into_inner();
    let model = product_category_mapping::ActiveModel {
        product_id: ActiveValue::Set(req.product_id),
        category_id: ActiveValue::Set(req.category_id),
    };

    match model.insert(txn).await {
        Ok(inserted) => Ok(Response::new(ProductCategoryMappingsResponse {
            items: vec![ProductCategoryMappingResponse {
                product_id: inserted.product_id,
                category_id: inserted.category_id,
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
