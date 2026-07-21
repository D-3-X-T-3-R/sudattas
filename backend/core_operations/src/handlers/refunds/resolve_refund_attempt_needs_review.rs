//! P1 Manual recovery for RefundAttempts stuck in `needs_review`: admin either retries the
//! gateway call (resets the attempt for the worker to pick up again) or marks it manually
//! settled (e.g. the refund was actually issued outside the system). Mirrors
//! core_operations::handlers::orders::resolve_needs_review's pattern for Orders.needs_review.

use crate::handlers::db_errors::map_db_error_to_status;
use crate::handlers::order_events::create_order_event;
use core_db_entities::entity::refund_attempts;
use proto::proto::core::{
    CreateOrderEventRequest, ResolveRefundAttemptNeedsReviewRequest,
    ResolveRefundAttemptNeedsReviewResponse,
};
use sea_orm::{ConnectionTrait, DatabaseTransaction, DbBackend, EntityTrait, Statement};
use tonic::{Request, Response, Status};

pub async fn resolve_refund_attempt_needs_review(
    txn: &DatabaseTransaction,
    request: Request<ResolveRefundAttemptNeedsReviewRequest>,
) -> Result<Response<ResolveRefundAttemptNeedsReviewResponse>, Status> {
    let req = request.into_inner();

    let attempt = refund_attempts::Entity::find_by_id(req.attempt_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::not_found(format!("RefundAttempt {} not found", req.attempt_id)))?;

    if attempt.status != "needs_review" {
        return Err(Status::failed_precondition(format!(
            "RefundAttempt is not in needs_review (current: {})",
            attempt.status
        )));
    }

    let resolution = req.resolution.trim().to_lowercase();
    let (new_status, message) = match resolution.as_str() {
        "retry" => (
            "pending_external",
            format!(
                "Refund attempt {} reset to pending_external for retry by admin {}",
                req.attempt_id, req.actor_id
            ),
        ),
        "mark_settled" => (
            "resolved",
            format!(
                "Refund attempt {} manually marked settled by admin {}",
                req.attempt_id, req.actor_id
            ),
        ),
        _ => {
            return Err(Status::invalid_argument(
                "resolution must be one of: retry, mark_settled",
            ));
        }
    };

    if resolution == "retry" {
        txn.execute(Statement::from_sql_and_values(
            DbBackend::MySql,
            r#"UPDATE RefundAttempts
               SET status = ?,
                   attempt_count = 0,
                   provider_error = NULL,
                   updated_at = UTC_TIMESTAMP()
               WHERE attempt_id = ?"#,
            [new_status.into(), req.attempt_id.into()],
        ))
        .await
        .map_err(map_db_error_to_status)?;
    } else {
        txn.execute(Statement::from_sql_and_values(
            DbBackend::MySql,
            r#"UPDATE RefundAttempts
               SET status = ?,
                   updated_at = UTC_TIMESTAMP()
               WHERE attempt_id = ?"#,
            [new_status.into(), req.attempt_id.into()],
        ))
        .await
        .map_err(map_db_error_to_status)?;
    }

    let _ = create_order_event(
        txn,
        Request::new(CreateOrderEventRequest {
            order_id: attempt.order_id,
            event_type: "refund_attempt_needs_review_resolved".to_string(),
            from_status: None,
            to_status: None,
            actor_type: "admin".to_string(),
            message: Some(message.clone()),
        }),
    )
    .await;

    Ok(Response::new(ResolveRefundAttemptNeedsReviewResponse {
        success: true,
        message,
    }))
}
