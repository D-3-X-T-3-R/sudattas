//! P1 Manual NeedsReview resolution: admin marks order as paid / cancelled / refunded with audit.

use crate::order_state_machine;
use core_db_entities::entity::sea_orm_active_enums::PaymentStatus;
use core_db_entities::entity::{order_status, orders};
use proto::proto::core::{ResolveNeedsReviewRequest, ResolveNeedsReviewResponse};
use sea_orm::{DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn resolve_needs_review(
    txn: &DatabaseTransaction,
    request: Request<ResolveNeedsReviewRequest>,
) -> Result<Response<ResolveNeedsReviewResponse>, Status> {
    let req = request.into_inner();

    let order = orders::Entity::find_by_id(req.order_id)
        .one(txn)
        .await
        .map_err(|e| Status::internal(e.to_string()))?
        .ok_or_else(|| Status::not_found(format!("Order {} not found", req.order_id)))?;

    let status_row = order_status::Entity::find_by_id(order.status_id)
        .one(txn)
        .await
        .map_err(|e| Status::internal(e.to_string()))?
        .ok_or_else(|| Status::internal("OrderStatus not found"))?;

    if status_row.status_name != "needs_review" {
        return Err(Status::failed_precondition(format!(
            "Order is not in needs_review (current: {})",
            status_row.status_name
        )));
    }

    let resolution = req.resolution.trim().to_lowercase();
    let (to_state, set_payment_status, message) = match resolution.as_str() {
        "paid" => (
            order_state_machine::OrderState::Paid,
            Some(PaymentStatus::Captured),
            format!("Order marked as paid by admin {}", req.actor_id),
        ),
        "cancelled" => (
            order_state_machine::OrderState::Cancelled,
            None,
            format!("Order cancelled by admin {}", req.actor_id),
        ),
        "refunded" => (
            order_state_machine::OrderState::Refunded,
            Some(PaymentStatus::Failed),
            format!("Order marked as refunded by admin {}", req.actor_id),
        ),
        _ => {
            return Err(Status::invalid_argument(
                "resolution must be one of: paid, cancelled, refunded",
            ));
        }
    };

    order_state_machine::transition_order_status(
        txn,
        req.order_id,
        to_state,
        "needs_review_resolved",
        "admin",
        Some(&message),
        set_payment_status,
    )
    .await?;

    let short_message = match resolution.as_str() {
        "paid" => "Order marked as paid",
        "cancelled" => "Order cancelled",
        "refunded" => "Order marked as refunded",
        _ => "Resolved",
    };

    Ok(Response::new(ResolveNeedsReviewResponse {
        success: true,
        message: short_message.to_string(),
    }))
}
