use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::product_ratings;
use proto::proto::core::{
    CreateProductRatingRequest, ProductRatingResponse, ProductRatingsResponse,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_product_rating(
    txn: &DatabaseTransaction,
    request: Request<CreateProductRatingRequest>,
) -> Result<Response<ProductRatingsResponse>, Status> {
    let req = request.into_inner();
    let model = product_ratings::ActiveModel {
        rating_id: ActiveValue::NotSet,
        product_id: ActiveValue::Set(Some(req.product_id)),
        user_id: ActiveValue::Set(Some(req.user_id)),
        rating: ActiveValue::Set(Some(req.rating)),
    };

    match model.insert(txn).await {
        Ok(inserted) => Ok(Response::new(ProductRatingsResponse {
            items: vec![ProductRatingResponse {
                rating_id: inserted.rating_id,
                product_id: inserted.product_id.unwrap_or(0),
                user_id: inserted.user_id.unwrap_or(0),
                rating: inserted.rating.unwrap_or(0),
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
