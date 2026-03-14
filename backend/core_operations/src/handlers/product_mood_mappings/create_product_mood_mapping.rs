use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::product_mood_mapping;
use proto::proto::core::{
    CreateProductMoodMappingRequest, ProductMoodMappingResponse, ProductMoodMappingsResponse,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_product_mood_mapping(
    txn: &DatabaseTransaction,
    request: Request<CreateProductMoodMappingRequest>,
) -> Result<Response<ProductMoodMappingsResponse>, Status> {
    let req = request.into_inner();
    let model = product_mood_mapping::ActiveModel {
        product_id: ActiveValue::Set(req.product_id),
        mood_id: ActiveValue::Set(req.mood_id),
    };

    match model.insert(txn).await {
        Ok(inserted) => Ok(Response::new(ProductMoodMappingsResponse {
            items: vec![ProductMoodMappingResponse {
                product_id: inserted.product_id,
                mood_id: inserted.mood_id,
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
