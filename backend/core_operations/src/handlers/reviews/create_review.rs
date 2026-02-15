use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::reviews;
use proto::proto::core::{CreateReviewRequest, ReviewResponse, ReviewsResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_review(
    txn: &DatabaseTransaction,
    request: Request<CreateReviewRequest>,
) -> Result<Response<ReviewsResponse>, Status> {
    let req = request.into_inner();
    let model = reviews::ActiveModel {
        review_id: ActiveValue::NotSet,
        product_id: ActiveValue::Set(Some(req.product_id)),
        user_id: ActiveValue::Set(Some(req.user_id)),
        rating: ActiveValue::Set(Some(req.rating)),
        comment: ActiveValue::Set(Some(req.comment)),
    };

    match model.insert(txn).await {
        Ok(inserted) => Ok(Response::new(ReviewsResponse {
            items: vec![ReviewResponse {
                review_id: inserted.review_id,
                product_id: inserted.product_id.unwrap_or(0),
                user_id: inserted.user_id.unwrap_or(0),
                rating: inserted.rating.unwrap_or(0),
                comment: inserted.comment.unwrap_or_default(),
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
