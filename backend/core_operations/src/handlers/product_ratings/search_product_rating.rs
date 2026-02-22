use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::product_ratings;
use proto::proto::core::{
    ProductRatingResponse, ProductRatingsResponse, SearchProductRatingRequest,
};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn search_product_rating(
    txn: &DatabaseTransaction,
    request: Request<SearchProductRatingRequest>,
) -> Result<Response<ProductRatingsResponse>, Status> {
    let req = request.into_inner();

    let mut query = product_ratings::Entity::find();
    if req.rating_id != 0 {
        query = query.filter(product_ratings::Column::RatingId.eq(req.rating_id));
    }

    match query.all(txn).await {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|m| ProductRatingResponse {
                    rating_id: m.rating_id,
                    product_id: m.product_id.unwrap_or(0),
                    user_id: m.user_id.unwrap_or(0),
                    rating: m.rating.unwrap_or(0),
                })
                .collect();
            Ok(Response::new(ProductRatingsResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
