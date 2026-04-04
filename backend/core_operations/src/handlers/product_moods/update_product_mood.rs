use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::product_moods;
use proto::proto::core::{ProductMoodResponse, ProductMoodsResponse, UpdateProductMoodRequest};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn update_product_mood(
    txn: &DatabaseTransaction,
    request: Request<UpdateProductMoodRequest>,
) -> Result<Response<ProductMoodsResponse>, Status> {
    let req = request.into_inner();

    let existing = product_moods::Entity::find_by_id(req.mood_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| {
            Status::not_found(format!("ProductMood with ID {} not found", req.mood_id))
        })?;

    let model = product_moods::ActiveModel {
        mood_id: ActiveValue::Set(existing.mood_id),
        mood_name: ActiveValue::Set(req.mood_name.unwrap_or_else(|| existing.mood_name.clone())),
    };

    match model.update(txn).await {
        Ok(updated) => Ok(Response::new(ProductMoodsResponse {
            items: vec![ProductMoodResponse {
                mood_id: updated.mood_id,
                mood_name: updated.mood_name,
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
