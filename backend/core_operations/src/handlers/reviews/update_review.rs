use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::reviews;
use proto::proto::core::{ReviewResponse, ReviewsResponse, UpdateReviewRequest};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn update_review(
    txn: &DatabaseTransaction,
    request: Request<UpdateReviewRequest>,
) -> Result<Response<ReviewsResponse>, Status> {
    let req = request.into_inner();

    let existing = reviews::Entity::find_by_id(req.review_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::not_found(format!("Review with ID {} not found", req.review_id)))?;

    let model = reviews::ActiveModel {
        review_id: ActiveValue::Set(existing.review_id),
        product_id: ActiveValue::Set(req.product_id.or(existing.product_id)),
        user_id: ActiveValue::Set(req.user_id.or(existing.user_id)),
        rating: ActiveValue::Set(req.rating.or(existing.rating)),
        comment: ActiveValue::Set(req.comment.or(existing.comment)),
    };

    match model.update(txn).await {
        Ok(updated) => Ok(Response::new(ReviewsResponse {
            items: vec![ReviewResponse {
                review_id: updated.review_id,
                product_id: updated.product_id.unwrap_or(0),
                user_id: updated.user_id.unwrap_or(0),
                rating: updated.rating.unwrap_or(0),
                comment: updated.comment.unwrap_or_default(),
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
