use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::promotions;
use proto::proto::core::{PromotionResponse, PromotionsResponse, SearchPromotionRequest};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

fn format_datetime(d: sea_orm::entity::prelude::DateTime) -> String {
    d.format("%Y-%m-%dT%H:%M:%S").to_string()
}

pub async fn search_promotion(
    txn: &DatabaseTransaction,
    request: Request<SearchPromotionRequest>,
) -> Result<Response<PromotionsResponse>, Status> {
    let req = request.into_inner();

    let mut query = promotions::Entity::find();
    if req.promotion_id != 0 {
        query = query.filter(promotions::Column::PromotionId.eq(req.promotion_id));
    }

    match query.all(txn).await {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|m| PromotionResponse {
                    promotion_id: m.promotion_id,
                    promotion_name: m.promotion_name,
                    start_date: format_datetime(m.start_date),
                    end_date: format_datetime(m.end_date),
                    details: m.details.unwrap_or_default(),
                })
                .collect();
            Ok(Response::new(PromotionsResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
