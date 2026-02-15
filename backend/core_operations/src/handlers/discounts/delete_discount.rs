use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::discounts;
use proto::proto::core::{DeleteDiscountRequest, DiscountResponse, DiscountsResponse};
use rust_decimal::prelude::ToPrimitive;
use sea_orm::{DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

fn date_to_string(d: Option<sea_orm::entity::prelude::Date>) -> String {
    d.map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}

pub async fn delete_discount(
    txn: &DatabaseTransaction,
    request: Request<DeleteDiscountRequest>,
) -> Result<Response<DiscountsResponse>, Status> {
    let req = request.into_inner();

    let found = discounts::Entity::find_by_id(req.discount_id)
        .one(txn)
        .await;

    match found {
        Ok(Some(model)) => {
            match discounts::Entity::delete_by_id(req.discount_id)
                .exec(txn)
                .await
            {
                Ok(_) => Ok(Response::new(DiscountsResponse {
                    items: vec![DiscountResponse {
                        discount_id: model.discount_id,
                        product_id: model.product_id.unwrap_or(0),
                        discount_percentage: model
                            .discount_percentage
                            .as_ref()
                            .and_then(ToPrimitive::to_f64)
                            .unwrap_or(0.0),
                        start_date: date_to_string(model.start_date),
                        end_date: date_to_string(model.end_date),
                    }],
                })),
                Err(e) => Err(map_db_error_to_status(e)),
            }
        }
        Ok(None) => Err(Status::not_found(format!(
            "Discount with ID {} not found",
            req.discount_id
        ))),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
