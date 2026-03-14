use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::product_moods;
use proto::proto::core::{CreateProductMoodRequest, ProductMoodResponse, ProductMoodsResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_product_mood(
    txn: &DatabaseTransaction,
    request: Request<CreateProductMoodRequest>,
) -> Result<Response<ProductMoodsResponse>, Status> {
    let req = request.into_inner();
    let model = product_moods::ActiveModel {
        mood_id: ActiveValue::NotSet,
        mood_name: ActiveValue::Set(req.mood_name),
    };

    match model.insert(txn).await {
        Ok(inserted) => Ok(Response::new(ProductMoodsResponse {
            items: vec![ProductMoodResponse {
                mood_id: inserted.mood_id,
                mood_name: inserted.mood_name,
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
