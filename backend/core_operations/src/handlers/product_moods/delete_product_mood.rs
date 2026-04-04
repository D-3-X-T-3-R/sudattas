use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::product_moods;
use proto::proto::core::{DeleteProductMoodRequest, ProductMoodResponse, ProductMoodsResponse};
use sea_orm::{DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn delete_product_mood(
    txn: &DatabaseTransaction,
    request: Request<DeleteProductMoodRequest>,
) -> Result<Response<ProductMoodsResponse>, Status> {
    let req = request.into_inner();

    let found = product_moods::Entity::find_by_id(req.mood_id)
        .one(txn)
        .await;

    match found {
        Ok(Some(model)) => {
            match product_moods::Entity::delete_by_id(req.mood_id)
                .exec(txn)
                .await
            {
                Ok(_) => Ok(Response::new(ProductMoodsResponse {
                    items: vec![ProductMoodResponse {
                        mood_id: model.mood_id,
                        mood_name: model.mood_name,
                    }],
                })),
                Err(e) => Err(map_db_error_to_status(e)),
            }
        }
        Ok(None) => Err(Status::not_found(format!(
            "ProductMood with ID {} not found",
            req.mood_id
        ))),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
