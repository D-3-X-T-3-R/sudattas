use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::reviews;
use proto::proto::core::{ReviewResponse, ReviewsResponse, SearchReviewRequest};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryTrait};
use tonic::{Request, Response, Status};

pub async fn search_review(
    txn: &DatabaseTransaction,
    request: Request<SearchReviewRequest>,
) -> Result<Response<ReviewsResponse>, Status> {
    let req = request.into_inner();

    let mut query = reviews::Entity::find();
    if req.review_id != 0 {
        query = query.filter(reviews::Column::ReviewId.eq(req.review_id));
    }

    match query.all(txn).await {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|m| ReviewResponse {
                    review_id: m.review_id,
                    product_id: m.product_id.unwrap_or(0),
                    user_id: m.user_id.unwrap_or(0),
                    rating: m.rating.unwrap_or(0),
                    comment: m.comment.unwrap_or_default(),
                })
                .collect();
            Ok(Response::new(ReviewsResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
