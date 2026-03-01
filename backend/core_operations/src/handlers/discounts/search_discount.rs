use crate::handlers::db_errors::map_db_error_to_status;
use crate::money::percentage_decimal_to_basis_points;
use core_db_entities::entity::discounts;
use proto::proto::core::{DiscountResponse, DiscountsResponse, SearchDiscountRequest};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

fn date_to_string(d: Option<sea_orm::entity::prelude::Date>) -> String {
    d.map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}

pub async fn search_discount(
    txn: &DatabaseTransaction,
    request: Request<SearchDiscountRequest>,
) -> Result<Response<DiscountsResponse>, Status> {
    let req = request.into_inner();

    let mut query = discounts::Entity::find();
    if req.discount_id != 0 {
        query = query.filter(discounts::Column::DiscountId.eq(req.discount_id));
    }

    match query.all(txn).await {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|m| DiscountResponse {
                    discount_id: m.discount_id,
                    product_id: m.product_id.unwrap_or(0),
                    discount_percentage_basis_points: percentage_decimal_to_basis_points(
                        m.discount_percentage.as_ref(),
                    ),
                    start_date: date_to_string(m.start_date),
                    end_date: date_to_string(m.end_date),
                })
                .collect();
            Ok(Response::new(DiscountsResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
