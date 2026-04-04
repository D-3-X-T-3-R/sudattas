use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::product_moods;
use proto::proto::core::{ProductMoodResponse, ProductMoodsResponse, SearchProductMoodRequest};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn search_product_mood(
    txn: &DatabaseTransaction,
    request: Request<SearchProductMoodRequest>,
) -> Result<Response<ProductMoodsResponse>, Status> {
    let req = request.into_inner();

    let mut query = product_moods::Entity::find();
    if let Some(mood_id) = req.mood_id {
        if mood_id != 0 {
            query = query.filter(product_moods::Column::MoodId.eq(mood_id));
        }
    }
    if let Some(ref name) = req.mood_name {
        query = query.filter(product_moods::Column::MoodName.eq(name.as_str()));
    }

    match query.all(txn).await {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|m| ProductMoodResponse {
                    mood_id: m.mood_id,
                    mood_name: m.mood_name,
                })
                .collect();
            Ok(Response::new(ProductMoodsResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
