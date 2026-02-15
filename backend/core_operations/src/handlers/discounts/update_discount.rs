use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::discounts;
use proto::proto::core::{DiscountResponse, DiscountsResponse, UpdateDiscountRequest};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

fn parse_date(s: &str) -> Option<sea_orm::entity::prelude::Date> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

fn date_to_string(d: Option<sea_orm::entity::prelude::Date>) -> String {
    d.map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}

pub async fn update_discount(
    txn: &DatabaseTransaction,
    request: Request<UpdateDiscountRequest>,
) -> Result<Response<DiscountsResponse>, Status> {
    let req = request.into_inner();

    let existing = discounts::Entity::find_by_id(req.discount_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| {
            Status::not_found(format!("Discount with ID {} not found", req.discount_id))
        })?;

    let discount_pct = req
        .discount_percentage
        .and_then(Decimal::from_f64_retain)
        .or(existing.discount_percentage);
    let start = req
        .start_date
        .as_deref()
        .and_then(|s| parse_date(s))
        .or(existing.start_date);
    let end = req
        .end_date
        .as_deref()
        .and_then(|s| parse_date(s))
        .or(existing.end_date);

    let model = discounts::ActiveModel {
        discount_id: ActiveValue::Set(existing.discount_id),
        product_id: ActiveValue::Set(req.product_id.or(existing.product_id)),
        discount_percentage: ActiveValue::Set(discount_pct),
        start_date: ActiveValue::Set(start),
        end_date: ActiveValue::Set(end),
    };

    match model.update(txn).await {
        Ok(updated) => Ok(Response::new(DiscountsResponse {
            items: vec![DiscountResponse {
                discount_id: updated.discount_id,
                product_id: updated.product_id.unwrap_or(0),
                discount_percentage: updated
                    .discount_percentage
                    .as_ref()
                    .and_then(ToPrimitive::to_f64)
                    .unwrap_or(0.0),
                start_date: date_to_string(updated.start_date),
                end_date: date_to_string(updated.end_date),
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
