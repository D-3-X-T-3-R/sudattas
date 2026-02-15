use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::reviews;
use proto::proto::core::{DeleteReviewRequest, ReviewResponse, ReviewsResponse};
use sea_orm::{DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn delete_review(
    txn: &DatabaseTransaction,
    request: Request<DeleteReviewRequest>,
) -> Result<Response<ReviewsResponse>, Status> {
    let req = request.into_inner();

    let found = reviews::Entity::find_by_id(req.review_id).one(txn).await;

    match found {
        Ok(Some(model)) => match reviews::Entity::delete_by_id(req.review_id).exec(txn).await {
            Ok(_) => Ok(Response::new(ReviewsResponse {
                items: vec![ReviewResponse {
                    review_id: model.review_id,
                    product_id: model.product_id.unwrap_or(0),
                    user_id: model.user_id.unwrap_or(0),
                    rating: model.rating.unwrap_or(0),
                    comment: model.comment.unwrap_or_default(),
                }],
            })),
            Err(e) => Err(map_db_error_to_status(e)),
        },
        Ok(None) => Err(Status::not_found(format!(
            "Review with ID {} not found",
            req.review_id
        ))),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
