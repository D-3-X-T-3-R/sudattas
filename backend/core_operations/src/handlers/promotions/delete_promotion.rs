use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::promotions;
use proto::proto::core::{DeletePromotionRequest, PromotionResponse, PromotionsResponse};
use sea_orm::{DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

fn format_datetime(d: sea_orm::entity::prelude::DateTime) -> String {
    d.format("%Y-%m-%dT%H:%M:%S").to_string()
}

pub async fn delete_promotion(
    txn: &DatabaseTransaction,
    request: Request<DeletePromotionRequest>,
) -> Result<Response<PromotionsResponse>, Status> {
    let req = request.into_inner();

    let found = promotions::Entity::find_by_id(req.promotion_id)
        .one(txn)
        .await;

    match found {
        Ok(Some(model)) => {
            match promotions::Entity::delete_by_id(req.promotion_id)
                .exec(txn)
                .await
            {
                Ok(_) => Ok(Response::new(PromotionsResponse {
                    items: vec![PromotionResponse {
                        promotion_id: model.promotion_id,
                        promotion_name: model.promotion_name,
                        start_date: format_datetime(model.start_date),
                        end_date: format_datetime(model.end_date),
                        details: model.details.unwrap_or_default(),
                    }],
                })),
                Err(e) => Err(map_db_error_to_status(e)),
            }
        }
        Ok(None) => Err(Status::not_found(format!(
            "Promotion with ID {} not found",
            req.promotion_id
        ))),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
