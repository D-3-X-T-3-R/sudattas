//! P1 Refund workflow: idempotent by gateway_refund_id; update order status + audit trail.

use crate::handlers::db_errors::map_db_error_to_status;
use crate::handlers::order_events::create_order_event;
use crate::order_state_machine;
use chrono::Utc;
use core_db_entities::entity::sea_orm_active_enums::{PaymentStatus, Status as RefundStatus};
use core_db_entities::entity::{order_status, orders, refunds};
use proto::proto::core::{
    CreateOrderEventRequest, CreateRefundRequest, RefundResponse, RefundsResponse,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, TransactionTrait,
};
use tonic::{Request, Response, Status};

pub async fn create_refund(
    txn: &DatabaseTransaction,
    request: Request<CreateRefundRequest>,
) -> Result<Response<RefundsResponse>, Status> {
    let req = request.into_inner();

    if req.gateway_refund_id.is_empty() {
        return Err(Status::invalid_argument("gateway_refund_id is required"));
    }
    if req.amount_paise <= 0 {
        return Err(Status::invalid_argument("amount_paise must be positive"));
    }

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

    let status_name = status_row.status_name.as_str();
    let refundable = matches!(
        status_name,
        "confirmed" | "processing" | "shipped" | "delivered"
    );
    if !refundable {
        return Err(Status::failed_precondition(format!(
            "Order is not in a refundable state (current: {})",
            status_name
        )));
    }

    let existing = refunds::Entity::find()
        .filter(refunds::Column::GatewayRefundId.eq(&req.gateway_refund_id))
        .one(txn)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    if let Some(ex) = existing {
        return Ok(Response::new(RefundsResponse {
            items: vec![model_to_response(&ex)],
        }));
    }

    let currency = req.currency.unwrap_or_else(|| "INR".to_string());
    let line_items_json = req
        .line_items_refunded_json
        .filter(|s| !s.is_empty())
        .and_then(|s| serde_json::from_str(&s).ok());

    let refund = refunds::ActiveModel {
        refund_id: sea_orm::ActiveValue::NotSet,
        order_id: sea_orm::ActiveValue::Set(req.order_id),
        gateway_refund_id: sea_orm::ActiveValue::Set(req.gateway_refund_id.clone()),
        amount_paise: sea_orm::ActiveValue::Set(req.amount_paise as i32),
        currency: sea_orm::ActiveValue::Set(Some(currency.clone())),
        status: sea_orm::ActiveValue::Set(Some(RefundStatus::Processed)),
        line_items_refunded: sea_orm::ActiveValue::Set(line_items_json),
        created_at: sea_orm::ActiveValue::Set(Some(Utc::now())),
    };

    let inserted = refund.insert(txn).await.map_err(map_db_error_to_status)?;

    let grand_total = order.grand_total_minor.unwrap_or(0);
    let total_refunded: i64 = refunds::Entity::find()
        .filter(refunds::Column::OrderId.eq(req.order_id))
        .all(txn)
        .await
        .map_err(|e| Status::internal(e.to_string()))?
        .iter()
        .map(|r| r.amount_paise as i64)
        .sum();
    let is_full_refund = grand_total > 0 && total_refunded >= grand_total;

    if is_full_refund {
        let _ = order_state_machine::transition_order_status(
            txn,
            req.order_id,
            order_state_machine::OrderState::Refunded,
            "refund_recorded",
            "system",
            Some("Full refund processed"),
            Some(PaymentStatus::Failed),
        )
        .await;
    } else {
        let _ = create_order_event(
            txn,
            Request::new(CreateOrderEventRequest {
                order_id: req.order_id,
                event_type: "refund_recorded".to_string(),
                from_status: Some(status_name.to_string()),
                to_status: Some(status_name.to_string()),
                actor_type: "system".to_string(),
                message: Some(format!("Partial refund {} paise", inserted.amount_paise)),
            }),
        )
        .await;
    }

    Ok(Response::new(RefundsResponse {
        items: vec![model_to_response(&inserted)],
    }))
}

fn model_to_response(m: &refunds::Model) -> RefundResponse {
    RefundResponse {
        refund_id: m.refund_id,
        order_id: m.order_id,
        gateway_refund_id: m.gateway_refund_id.clone(),
        amount_paise: m.amount_paise as i64,
        currency: m.currency.clone().unwrap_or_default(),
        status: m
            .status
            .as_ref()
            .map(|s| format!("{:?}", s).to_lowercase())
            .unwrap_or_else(|| "processed".to_string()),
        created_at: m.created_at.map(|t| t.to_string()).unwrap_or_default(),
    }
}
