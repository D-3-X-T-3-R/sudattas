use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::promotions;
use proto::proto::core::{PromotionResponse, PromotionsResponse, UpdatePromotionRequest};
use sea_orm::entity::prelude::DateTime;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

fn parse_datetime(s: &str) -> Option<DateTime> {
    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
        .or_else(|_| {
            chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .map(|d| d.and_hms_opt(0, 0, 0).unwrap())
        })
        .ok()
}

fn format_datetime(d: DateTime) -> String {
    d.format("%Y-%m-%dT%H:%M:%S").to_string()
}

pub async fn update_promotion(
    txn: &DatabaseTransaction,
    request: Request<UpdatePromotionRequest>,
) -> Result<Response<PromotionsResponse>, Status> {
    let req = request.into_inner();

    let existing = promotions::Entity::find_by_id(req.promotion_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| {
            Status::not_found(format!("Promotion with ID {} not found", req.promotion_id))
        })?;

    let start = req
        .start_date
        .as_deref()
        .and_then(parse_datetime)
        .unwrap_or(existing.start_date);
    let end = req
        .end_date
        .as_deref()
        .and_then(parse_datetime)
        .unwrap_or(existing.end_date);

    let model = promotions::ActiveModel {
        promotion_id: ActiveValue::Set(existing.promotion_id),
        promotion_name: ActiveValue::Set(req.promotion_name.unwrap_or(existing.promotion_name)),
        start_date: ActiveValue::Set(start),
        end_date: ActiveValue::Set(end),
        details: ActiveValue::Set(req.details.or(existing.details)),
    };

    match model.update(txn).await {
        Ok(updated) => Ok(Response::new(PromotionsResponse {
            items: vec![PromotionResponse {
                promotion_id: updated.promotion_id,
                promotion_name: updated.promotion_name,
                start_date: format_datetime(updated.start_date),
                end_date: format_datetime(updated.end_date),
                details: updated.details.unwrap_or_default(),
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
