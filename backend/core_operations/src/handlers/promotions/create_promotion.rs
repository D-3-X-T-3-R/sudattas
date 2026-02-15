use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::promotions;
use proto::proto::core::{CreatePromotionRequest, PromotionResponse, PromotionsResponse};
use sea_orm::entity::prelude::DateTime;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
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

pub async fn create_promotion(
    txn: &DatabaseTransaction,
    request: Request<CreatePromotionRequest>,
) -> Result<Response<PromotionsResponse>, Status> {
    let req = request.into_inner();
    let start = parse_datetime(&req.start_date)
        .ok_or_else(|| Status::invalid_argument("Invalid start_date format"))?;
    let end = parse_datetime(&req.end_date)
        .ok_or_else(|| Status::invalid_argument("Invalid end_date format"))?;
    let model = promotions::ActiveModel {
        promotion_id: ActiveValue::NotSet,
        promotion_name: ActiveValue::Set(req.promotion_name),
        start_date: ActiveValue::Set(start),
        end_date: ActiveValue::Set(end),
        details: ActiveValue::Set(Some(req.details)),
    };

    match model.insert(txn).await {
        Ok(inserted) => Ok(Response::new(PromotionsResponse {
            items: vec![PromotionResponse {
                promotion_id: inserted.promotion_id,
                promotion_name: inserted.promotion_name,
                start_date: format_datetime(inserted.start_date),
                end_date: format_datetime(inserted.end_date),
                details: inserted.details.unwrap_or_default(),
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
