use crate::handlers::db_errors::map_db_error_to_status;
use crate::money::{basis_points_to_percentage_decimal, percentage_decimal_to_basis_points};
use core_db_entities::entity::discounts;
use proto::proto::core::{CreateDiscountRequest, DiscountResponse, DiscountsResponse};
use sea_orm::entity::prelude::Date;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

fn parse_date(s: &str) -> Option<Date> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

fn date_to_string(d: Option<Date>) -> String {
    d.map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}

pub async fn create_discount(
    txn: &DatabaseTransaction,
    request: Request<CreateDiscountRequest>,
) -> Result<Response<DiscountsResponse>, Status> {
    let req = request.into_inner();
    let discount_pct = basis_points_to_percentage_decimal(req.discount_percentage_basis_points);
    let start = parse_date(&req.start_date);
    let end = parse_date(&req.end_date);

    let model = discounts::ActiveModel {
        discount_id: ActiveValue::NotSet,
        product_id: ActiveValue::Set(Some(req.product_id)),
        discount_percentage: ActiveValue::Set(Some(discount_pct)),
        start_date: ActiveValue::Set(start),
        end_date: ActiveValue::Set(end),
    };

    match model.insert(txn).await {
        Ok(inserted) => Ok(Response::new(DiscountsResponse {
            items: vec![DiscountResponse {
                discount_id: inserted.discount_id,
                product_id: inserted.product_id.unwrap_or(0),
                discount_percentage_basis_points: percentage_decimal_to_basis_points(
                    inserted.discount_percentage.as_ref(),
                ),
                start_date: date_to_string(inserted.start_date),
                end_date: date_to_string(inserted.end_date),
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
