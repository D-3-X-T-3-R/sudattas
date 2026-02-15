use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::product_ratings;
use proto::proto::core::{
    DeleteProductRatingRequest, ProductRatingResponse, ProductRatingsResponse,
};
use sea_orm::{DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn delete_product_rating(
    txn: &DatabaseTransaction,
    request: Request<DeleteProductRatingRequest>,
) -> Result<Response<ProductRatingsResponse>, Status> {
    let req = request.into_inner();

    let found = product_ratings::Entity::find_by_id(req.rating_id)
        .one(txn)
        .await;

    match found {
        Ok(Some(model)) => {
            match product_ratings::Entity::delete_by_id(req.rating_id)
                .exec(txn)
                .await
            {
                Ok(_) => Ok(Response::new(ProductRatingsResponse {
                    items: vec![ProductRatingResponse {
                        rating_id: model.rating_id,
                        product_id: model.product_id.unwrap_or(0),
                        user_id: model.user_id.unwrap_or(0),
                        rating: model.rating.unwrap_or(0),
                    }],
                })),
                Err(e) => Err(map_db_error_to_status(e)),
            }
        }
        Ok(None) => Err(Status::not_found(format!(
            "ProductRating with ID {} not found",
            req.rating_id
        ))),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
