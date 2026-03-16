use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::product_mood_mapping;
use proto::proto::core::{
    ProductMoodMappingResponse, ProductMoodMappingsResponse, SearchProductMoodMappingRequest,
};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn search_product_mood_mapping(
    txn: &DatabaseTransaction,
    request: Request<SearchProductMoodMappingRequest>,
) -> Result<Response<ProductMoodMappingsResponse>, Status> {
    let req = request.into_inner();

    let mut query = product_mood_mapping::Entity::find()
        .filter(product_mood_mapping::Column::ProductId.eq(req.product_id));
    if let Some(mood_id) = req.mood_id {
        if mood_id != 0 {
            query = query.filter(product_mood_mapping::Column::MoodId.eq(mood_id));
        }
    }

    match query.all(txn).await {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|m| ProductMoodMappingResponse {
                    product_id: m.product_id,
                    mood_id: m.mood_id,
                })
                .collect();
            Ok(Response::new(ProductMoodMappingsResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
