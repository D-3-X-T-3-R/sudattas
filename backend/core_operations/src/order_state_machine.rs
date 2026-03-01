//! Phase 7: Central order state machine.
//!
//! Defines allowed order status transitions and a single function to apply them,
//! update the order, and emit order_events.

use crate::handlers::db_errors::map_db_error_to_status;
use crate::handlers::order_events::create_order_event;
use crate::handlers::outbox::{
    enqueue_outbox_event, DELIVERED, PAYMENT_CAPTURED, REFUNDED, SHIPPED,
};
use core_db_entities::entity::sea_orm_active_enums::PaymentStatus;
use core_db_entities::entity::{order_status, orders};
use proto::proto::core::CreateOrderEventRequest;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseTransaction, EntityTrait, IntoActiveModel,
    QueryFilter,
};
use serde_json::json;
use std::collections::HashSet;
use tonic::Request;

/// Order lifecycle states (maps to OrderStatus.StatusName in DB).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OrderState {
    PendingPayment,
    Paid,
    Processing,
    Shipped,
    Delivered,
    Cancelled,
    Refunded,
    NeedsReview,
}

impl OrderState {
    pub fn as_status_name(self) -> &'static str {
        match self {
            OrderState::PendingPayment => "pending",
            OrderState::Paid => "confirmed",
            OrderState::Processing => "processing",
            OrderState::Shipped => "shipped",
            OrderState::Delivered => "delivered",
            OrderState::Cancelled => "cancelled",
            OrderState::Refunded => "refunded",
            OrderState::NeedsReview => "needs_review",
        }
    }
}

/// Allowed transitions: from -> set of to states.
fn allowed_transitions() -> Vec<(OrderState, HashSet<OrderState>)> {
    use OrderState::*;
    vec![
        (
            PendingPayment,
            [Paid, NeedsReview, Cancelled].into_iter().collect(),
        ),
        (
            Paid,
            [Processing, Cancelled, Refunded].into_iter().collect(),
        ),
        (
            Processing,
            [Shipped, Cancelled, Refunded].into_iter().collect(),
        ),
        (Shipped, [Delivered, Refunded].into_iter().collect()),
        (Delivered, [Refunded].into_iter().collect()),
        (
            NeedsReview,
            [PendingPayment, Paid, Cancelled, Refunded]
                .into_iter()
                .collect(),
        ),
        (Cancelled, HashSet::new()),
        (Refunded, HashSet::new()),
    ]
}

/// Returns true if transitioning from `from` to `to` is allowed.
pub fn can_transition(from: OrderState, to: OrderState) -> bool {
    if from == to {
        return true;
    }
    allowed_transitions()
        .into_iter()
        .find(|(s, _)| *s == from)
        .map(|(_, allowed)| allowed.contains(&to))
        .unwrap_or(false)
}

/// Look up StatusID by StatusName (e.g. "pending", "confirmed").
pub async fn get_status_id(
    txn: &DatabaseTransaction,
    status_name: &str,
) -> Result<Option<i64>, tonic::Status> {
    let row = order_status::Entity::find()
        .filter(order_status::Column::StatusName.eq(status_name))
        .one(txn)
        .await
        .map_err(|e| tonic::Status::internal(e.to_string()))?;
    Ok(row.map(|r| r.status_id))
}

/// Look up StatusName by StatusID.
pub async fn get_status_name(
    txn: &DatabaseTransaction,
    status_id: i64,
) -> Result<Option<String>, tonic::Status> {
    let row = order_status::Entity::find_by_id(status_id)
        .one(txn)
        .await
        .map_err(|e| tonic::Status::internal(e.to_string()))?;
    Ok(row.map(|r| r.status_name))
}

/// Returns true if transitioning from `from_status_id` to `to_status_id` is allowed.
pub async fn can_transition_by_ids(
    txn: &DatabaseTransaction,
    from_status_id: i64,
    to_status_id: i64,
) -> Result<bool, tonic::Status> {
    let from_name = get_status_name(txn, from_status_id).await?;
    let to_name = get_status_name(txn, to_status_id).await?;
    let (from_s, to_s) = match (from_name.as_deref(), to_name.as_deref()) {
        (Some(a), Some(b)) => (status_name_to_state(a), status_name_to_state(b)),
        _ => return Ok(false),
    };
    Ok(can_transition(from_s, to_s))
}

/// Transition order to `to_state`. Validates the transition from current order status;
/// updates order.status_id (and payment_status when transitioning to Paid or NeedsReview);
/// emits order_event. Returns error if transition is illegal.
pub async fn transition_order_status(
    txn: &DatabaseTransaction,
    order_id: i64,
    to_state: OrderState,
    event_type: &str,
    actor_type: &str,
    message: Option<&str>,
    set_payment_status: Option<PaymentStatus>,
) -> Result<(), tonic::Status> {
    let order = orders::Entity::find_by_id(order_id)
        .one(txn)
        .await
        .map_err(|e| tonic::Status::internal(e.to_string()))?
        .ok_or_else(|| tonic::Status::not_found(format!("Order {} not found", order_id)))?;

    let current_status_id = order.status_id;
    let current_row = order_status::Entity::find_by_id(current_status_id)
        .one(txn)
        .await
        .map_err(|e| tonic::Status::internal(e.to_string()))?
        .ok_or_else(|| {
            tonic::Status::internal(format!(
                "OrderStatus {} for order {} not found",
                current_status_id, order_id
            ))
        })?;
    let from_state = status_name_to_state(&current_row.status_name);
    let to_status_id = get_status_id(txn, to_state.as_status_name())
        .await?
        .ok_or_else(|| {
            tonic::Status::internal(format!(
                "OrderStatus '{}' not found in DB",
                to_state.as_status_name()
            ))
        })?;

    if !can_transition(from_state, to_state) {
        return Err(tonic::Status::invalid_argument(format!(
            "Illegal order state transition: {} -> {}",
            from_state.as_status_name(),
            to_state.as_status_name()
        )));
    }

    let user_id = order.user_id;
    let mut active: orders::ActiveModel = order.into_active_model();
    active.status_id = ActiveValue::Set(to_status_id);
    if let Some(ps) = set_payment_status {
        active.payment_status = ActiveValue::Set(Some(ps));
    }
    let _ = active.update(txn).await.map_err(map_db_error_to_status)?;

    let _ = create_order_event(
        txn,
        Request::new(CreateOrderEventRequest {
            order_id,
            event_type: event_type.to_string(),
            from_status: Some(current_row.status_name.clone()),
            to_status: Some(to_state.as_status_name().to_string()),
            actor_type: actor_type.to_string(),
            message: message.map(String::from),
        }),
    )
    .await;

    // P1 Outbox: enqueue notification event for transactional emails/SMS
    let outbox_type = match to_state {
        OrderState::Paid => Some(PAYMENT_CAPTURED),
        OrderState::Shipped => Some(SHIPPED),
        OrderState::Delivered => Some(DELIVERED),
        OrderState::Refunded => Some(REFUNDED),
        _ => None,
    };
    if let Some(evt) = outbox_type {
        let payload = json!({ "order_id": order_id, "user_id": user_id });
        let _ = enqueue_outbox_event(txn, evt, "order", &order_id.to_string(), payload).await;
    }

    Ok(())
}

fn status_name_to_state(name: &str) -> OrderState {
    match name {
        "pending" => OrderState::PendingPayment,
        "confirmed" => OrderState::Paid,
        "processing" => OrderState::Processing,
        "shipped" => OrderState::Shipped,
        "delivered" => OrderState::Delivered,
        "cancelled" => OrderState::Cancelled,
        "refunded" => OrderState::Refunded,
        "needs_review" => OrderState::NeedsReview,
        _ => OrderState::NeedsReview, // unknown -> treat as needs_review for safety
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_can_transition_to_paid_needs_review_cancelled() {
        assert!(can_transition(OrderState::PendingPayment, OrderState::Paid));
        assert!(can_transition(
            OrderState::PendingPayment,
            OrderState::NeedsReview
        ));
        assert!(can_transition(
            OrderState::PendingPayment,
            OrderState::Cancelled
        ));
        assert!(!can_transition(
            OrderState::PendingPayment,
            OrderState::Processing
        ));
    }

    #[test]
    fn paid_can_transition_to_processing_cancelled_refunded() {
        assert!(can_transition(OrderState::Paid, OrderState::Processing));
        assert!(can_transition(OrderState::Paid, OrderState::Cancelled));
        assert!(can_transition(OrderState::Paid, OrderState::Refunded));
        assert!(!can_transition(
            OrderState::Paid,
            OrderState::PendingPayment
        ));
    }

    #[test]
    fn terminal_states_have_no_outgoing() {
        assert!(!can_transition(OrderState::Cancelled, OrderState::Paid));
        assert!(!can_transition(
            OrderState::Refunded,
            OrderState::Processing
        ));
        assert!(can_transition(OrderState::Cancelled, OrderState::Cancelled));
    }

    #[test]
    fn same_state_is_allowed() {
        assert!(can_transition(
            OrderState::Processing,
            OrderState::Processing
        ));
    }
}
