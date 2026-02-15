use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::product_color_mapping;
use proto::proto::core::{
    CreateProductColorMappingRequest, ProductColorMappingResponse,
    ProductColorMappingsResponse,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_product_color_mapping(
    txn: &DatabaseTransaction,
    request: Request<CreateProductColorMappingRequest>,
) -> Result<Response<ProductColorMappingsResponse>, Status> {
    let req = request.into_inner();
    let model = product_color_mapping::ActiveModel {
        product_id: ActiveValue::Set(req.product_id),
        color_id: ActiveValue::Set(req.color_id),
    };

    match model.insert(txn).await {
        Ok(inserted) => Ok(Response::new(ProductColorMappingsResponse {
            items: vec![ProductColorMappingResponse {
                product_id: inserted.product_id,
                color_id: inserted.color_id,
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
