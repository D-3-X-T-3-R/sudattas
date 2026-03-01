//! P2 Review moderation: admin set review status to approved or rejected.

use crate::handlers::db_errors::map_db_error_to_status;
use proto::proto::core::{AdminUpdateReviewStatusRequest, AdminUpdateReviewStatusResponse};
use sea_orm::{ConnectionTrait, DatabaseTransaction, Statement};
use tonic::{Request, Response, Status};

const ALLOWED: &[&str] = &["approved", "rejected"];

pub async fn admin_update_review_status(
    txn: &DatabaseTransaction,
    request: Request<AdminUpdateReviewStatusRequest>,
) -> Result<Response<AdminUpdateReviewStatusResponse>, Status> {
    let req = request.into_inner();
    let status = req.status.trim().to_lowercase();
    if !ALLOWED.contains(&status.as_str()) {
        return Err(Status::invalid_argument(format!(
            "status must be one of: {}",
            ALLOWED.join(", ")
        )));
    }

    let stmt = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::MySql,
        "UPDATE Reviews SET status = ? WHERE ReviewID = ?",
        [status.into(), req.review_id.into()],
    );
    let result = txn.execute(stmt).await.map_err(map_db_error_to_status)?;
    Ok(Response::new(AdminUpdateReviewStatusResponse {
        success: result.rows_affected() > 0,
    }))
}
