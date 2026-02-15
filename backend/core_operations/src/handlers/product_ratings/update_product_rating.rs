use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::product_ratings;
use proto::proto::core::{
    ProductRatingResponse, ProductRatingsResponse, UpdateProductRatingRequest,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn update_product_rating(
    txn: &DatabaseTransaction,
    request: Request<UpdateProductRatingRequest>,
) -> Result<Response<ProductRatingsResponse>, Status> {
    let req = request.into_inner();

    let existing = product_ratings::Entity::find_by_id(req.rating_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| {
            Status::not_found(format!(
                "ProductRating with ID {} not found",
                req.rating_id
            ))
        })?;

    let model = product_ratings::ActiveModel {
        rating_id: ActiveValue::Set(existing.rating_id),
        product_id: ActiveValue::Set(req.product_id.or(existing.product_id)),
        user_id: ActiveValue::Set(req.user_id.or(existing.user_id)),
        rating: ActiveValue::Set(req.rating.or(existing.rating)),
    };

    match model.update(txn).await {
        Ok(updated) => Ok(Response::new(ProductRatingsResponse {
            items: vec![ProductRatingResponse {
                rating_id: updated.rating_id,
                product_id: updated.product_id.unwrap_or(0),
                user_id: updated.user_id.unwrap_or(0),
                rating: updated.rating.unwrap_or(0),
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
