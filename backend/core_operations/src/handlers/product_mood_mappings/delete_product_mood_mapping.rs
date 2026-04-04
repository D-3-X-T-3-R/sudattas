use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::product_mood_mapping;
use proto::proto::core::{
    DeleteProductMoodMappingRequest, ProductMoodMappingResponse, ProductMoodMappingsResponse,
};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn delete_product_mood_mapping(
    txn: &DatabaseTransaction,
    request: Request<DeleteProductMoodMappingRequest>,
) -> Result<Response<ProductMoodMappingsResponse>, Status> {
    let req = request.into_inner();

    let found = product_mood_mapping::Entity::find()
        .filter(product_mood_mapping::Column::ProductId.eq(req.product_id))
        .filter(product_mood_mapping::Column::MoodId.eq(req.mood_id))
        .one(txn)
        .await;

    match found {
        Ok(Some(model)) => {
            match product_mood_mapping::Entity::delete_many()
                .filter(product_mood_mapping::Column::ProductId.eq(req.product_id))
                .filter(product_mood_mapping::Column::MoodId.eq(req.mood_id))
                .exec(txn)
                .await
            {
                Ok(_) => Ok(Response::new(ProductMoodMappingsResponse {
                    items: vec![ProductMoodMappingResponse {
                        product_id: model.product_id,
                        mood_id: model.mood_id,
                    }],
                })),
                Err(e) => Err(map_db_error_to_status(e)),
            }
        }
        Ok(None) => Err(Status::not_found("ProductMoodMapping not found")),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
