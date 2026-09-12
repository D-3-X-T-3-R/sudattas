//! Server-side rating aggregate: real SQL AVG/COUNT instead of the caller fetching every review
//! row for a product and averaging client-side (mirrors the SUM/GROUP BY approach already used in
//! orders/get_order_stats.rs). average_rating is CEIL(AVG(Rating)), so a 3.2 or 3.8 average both
//! come back as 4 stars, per product decision; 0 when the product has no ratings yet.

use crate::handlers::db_errors::map_db_error_to_status;
use proto::proto::core::{ProductRatingSummaryRequest, ProductRatingSummaryResponse};
use sea_orm::{ConnectionTrait, DatabaseTransaction, DbBackend, Statement};
use tonic::{Request, Response, Status};

pub async fn get_product_rating_summary(
    txn: &DatabaseTransaction,
    request: Request<ProductRatingSummaryRequest>,
) -> Result<Response<ProductRatingSummaryResponse>, Status> {
    let req = request.into_inner();

    let sql = "SELECT \
        CAST(CEIL(COALESCE(AVG(Rating), 0)) AS SIGNED) AS average_rating, \
        CAST(COUNT(*) AS SIGNED) AS rating_count \
        FROM Reviews WHERE ProductID = ?";
    let row = txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::MySql,
            sql,
            [req.product_id.into()],
        ))
        .await
        .map_err(map_db_error_to_status)?;

    let (average_rating, rating_count) = match row {
        Some(r) => (
            r.try_get::<i64>("", "average_rating").unwrap_or(0) as i32,
            r.try_get::<i64>("", "rating_count").unwrap_or(0),
        ),
        None => (0, 0),
    };

    Ok(Response::new(ProductRatingSummaryResponse {
        product_id: req.product_id,
        average_rating,
        rating_count,
    }))
}
