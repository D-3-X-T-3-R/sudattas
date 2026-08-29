//! Admin: place a full order on a customer's behalf (phone/in-person sale, etc.).
//!
//! Deliberately mirrors real checkout's COD path exactly — inventory is locked/checked/
//! decremented the same way, order creation, immediate order_state_machine transition to
//! `Paid` (status "confirmed", payment_status "captured"), invoice generation, and an
//! `order_placed` order_event — so an admin-placed order joins the same downstream lifecycle
//! (cancel window, shipment-booking eligibility, inventory accounting) as a real order, with
//! the admin acting in place of the customer. The only real differences from real checkout:
//! line items come from the admin directly (not a persisted cart), and there is never a live
//! Razorpay step for either payment method — "prepaid" here means payment already happened
//! outside this system, not "collect it now".

use super::place_order::lock_inventory_row_and_get_available_quantity;
use crate::handlers::invoices::ensure_invoice_for_order;
use crate::handlers::order_details::create_order_details;
use crate::handlers::order_events::create_order_event;
use crate::handlers::orders::create_order;
use crate::order_state_machine::{self, OrderState};
use core_db_entities::entity::sea_orm_active_enums::PaymentStatus;
use proto::proto::core::{
    CreateOrderDetailRequest, CreateOrderDetailsRequest, CreateOrderEventRequest,
    CreateOrderRequest, OrdersResponse, PlaceOrderAdminRequest,
};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement, TransactionTrait};
use std::collections::HashMap;
use tonic::{Request, Response, Status};

pub async fn place_order_admin(
    db: &DatabaseConnection,
    request: Request<PlaceOrderAdminRequest>,
) -> Result<Response<OrdersResponse>, Status> {
    let req = request.into_inner();

    let normalized_payment_method = req.payment_method.trim().to_lowercase();
    if normalized_payment_method != "cod" && normalized_payment_method != "prepaid" {
        return Err(Status::invalid_argument(
            "payment_method must be \"cod\" or \"prepaid\"",
        ));
    }
    if req.line_items.is_empty() {
        return Err(Status::invalid_argument(
            "At least one line item is required",
        ));
    }

    let gross_paise: i64 = req.line_items.iter().map(|l| l.price_paise).sum();
    let shipping_minor = req.shipping_minor.unwrap_or(0);
    let grand_total_paise = gross_paise + shipping_minor;

    let txn = db
        .begin()
        .await
        .map_err(|e| Status::internal(format!("failed to begin place_order_admin txn: {e}")))?;

    // Same starting status real checkout uses — it never stays there; the transition below
    // immediately moves it to "confirmed" in the same transaction.
    let pending_status_id = order_state_machine::get_status_id(&txn, "active_sale")
        .await
        .map_err(|e| Status::internal(e.to_string()))?
        .or(order_state_machine::get_status_id(&txn, "pending")
            .await
            .map_err(|e| Status::internal(e.to_string()))?)
        .ok_or_else(|| Status::internal("OrderStatus 'active_sale' not found"))?;

    let created_order = create_order(
        &txn,
        Request::new(CreateOrderRequest {
            user_id: req.user_id,
            shipping_address_id: req.shipping_address_id,
            status_id: pending_status_id,
            total_amount_paise: grand_total_paise,
            subtotal_minor: Some(gross_paise),
            shipping_minor: Some(shipping_minor),
            tax_total_minor: Some(0),
            discount_total_minor: req.applied_discount_paise.map(i64::from),
            grand_total_minor: Some(grand_total_paise),
            applied_coupon_id: req.applied_coupon_id,
            applied_coupon_code: req.applied_coupon_code.clone(),
            applied_discount_paise: req.applied_discount_paise,
            payment_method: normalized_payment_method,
        }),
    )
    .await?
    .into_inner()
    .items
    .into_iter()
    .next()
    .ok_or_else(|| Status::internal("create_order returned no order"))?;

    let order_id = created_order.order_id;

    let line_items: Vec<CreateOrderDetailRequest> = req
        .line_items
        .iter()
        .map(|l| CreateOrderDetailRequest {
            order_id,
            variant_id: l.variant_id,
            quantity: l.quantity,
            price_paise: l.price_paise,
            unit_price_minor: None,
            discount_minor: None,
            tax_minor: None,
            sku: None,
            title: None,
        })
        .collect();

    create_order_details(
        &txn,
        Request::new(CreateOrderDetailsRequest {
            order_details: line_items,
        }),
    )
    .await?;

    // Reserve inventory the same way real checkout does: lock each variant's row, verify
    // enough stock is available, then decrement — all inside this transaction, so a failure
    // here (or later) rolls the whole order back. Rows are locked in sorted VariantID order to
    // match place_order's deadlock-avoidance convention.
    let variant_quantity_map: HashMap<i64, i64> = req
        .line_items
        .iter()
        .map(|l| (l.variant_id, l.quantity))
        .collect();
    let mut variant_ids_sorted: Vec<i64> = variant_quantity_map.keys().copied().collect();
    variant_ids_sorted.sort_unstable();
    for variant_id in variant_ids_sorted {
        let qty = variant_quantity_map[&variant_id];
        let quantity_available =
            lock_inventory_row_and_get_available_quantity(&txn, variant_id).await?;
        if quantity_available < qty {
            crate::observability::record_inventory_update_failure_total();
            return Err(Status::failed_precondition(format!(
                "Insufficient stock for variant {} (need {}, available {})",
                variant_id, qty, quantity_available
            )));
        }
        let result = txn
            .execute(Statement::from_sql_and_values(
                DbBackend::MySql,
                r#"UPDATE Inventory
                   SET QuantityAvailable = QuantityAvailable - ?
                   WHERE VariantID = ?"#,
                [qty.into(), variant_id.into()],
            ))
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        if result.rows_affected() == 0 {
            crate::observability::record_inventory_update_failure_total();
            return Err(Status::failed_precondition(format!(
                "No inventory row exists for variant {}",
                variant_id
            )));
        }
        if result.rows_affected() > 1 {
            crate::observability::record_inventory_update_failure_total();
            return Err(Status::internal(format!(
                "Inventory data corruption: reserve update touched {} rows for variant {}",
                result.rows_affected(),
                variant_id
            )));
        }
    }

    // Both payment methods are treated as already settled — no live Razorpay step here, ever.
    order_state_machine::transition_order_status(
        &txn,
        order_id,
        OrderState::Paid,
        "admin_order_placed",
        "admin",
        Some("Order placed by admin on behalf of customer"),
        Some(PaymentStatus::Captured),
    )
    .await?;

    let _ = ensure_invoice_for_order(&txn, order_id, "admin_order_confirmed").await?;

    let _ = create_order_event(
        &txn,
        Request::new(CreateOrderEventRequest {
            order_id,
            event_type: "order_placed".to_string(),
            from_status: None,
            to_status: Some("confirmed".to_string()),
            actor_type: "admin".to_string(),
            message: Some(format!(
                "Order {order_id} placed by admin on behalf of customer"
            )),
        }),
    )
    .await;

    txn.commit()
        .await
        .map_err(|e| Status::internal(format!("failed to commit place_order_admin txn: {e}")))?;

    Ok(Response::new(OrdersResponse {
        items: vec![created_order],
    }))
}
