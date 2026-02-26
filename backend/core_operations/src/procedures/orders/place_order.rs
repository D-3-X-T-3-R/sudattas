use crate::handlers::cart::delete_cart_item;
use crate::handlers::coupons::validate_coupon::check_coupon;
use crate::handlers::idempotency::compute_request_hash;
use crate::handlers::order_events::create_order_event;
use crate::money::{
    paise_checked_add, paise_checked_mul, paise_from_major_f64, paise_to_major_f64,
};

use crate::handlers::{
    cart::get_cart_items, order_details::create_order_details, orders::create_order,
    payment_intents::create_payment_intent, products::get_products_by_id,
};

use core_db_entities::entity::prelude::IdempotencyKeys;
use core_db_entities::entity::{
    idempotency_keys, orders, sea_orm_active_enums::Status as IdempotencyStatus,
};
use proto::proto::core::{
    CreateOrderDetailRequest, CreateOrderDetailsRequest, CreateOrderEventRequest,
    CreateOrderRequest, CreatePaymentIntentRequest, DeleteCartItemRequest, GetCartItemsRequest,
    GetProductsByIdRequest, OrdersResponse, PlaceOrderRequest,
};
use sea_orm::DbBackend;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set, Statement,
};

use chrono::Utc;
use rust_decimal::prelude::ToPrimitive;
use sea_orm::DatabaseTransaction;
use serde_json::json;
use std::collections::HashMap;
use tonic::{Request, Response, Status};
use tracing::info;

pub async fn place_order(
    txn: &DatabaseTransaction,
    request: Request<PlaceOrderRequest>,
) -> Result<Response<OrdersResponse>, Status> {
    // Extract idempotency key from gRPC metadata, if present.
    let metadata = request.metadata().clone();
    let idempotency_key = metadata
        .get("idempotency-key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let req = request.into_inner();

    let cart_items = get_cart_items(
        txn,
        Request::new(GetCartItemsRequest {
            user_id: Some(req.user_id),
            session_id: None,
        }),
    )
    .await?
    .into_inner()
    .items;

    if cart_items.is_empty() {
        return Err(Status::failed_precondition(
            "Cannot place order: cart is empty",
        ));
    }

    let (product_quantity_map, product_ids): (HashMap<i64, i64>, Vec<i64>) = cart_items
        .iter()
        .map(|item| ((item.product_id, item.quantity), item.product_id))
        .unzip();

    let order_products =
        get_products_by_id(txn, Request::new(GetProductsByIdRequest { product_ids }))
            .await?
            .into_inner()
            .items;

    // Build a stable representation of the logical request payload for idempotency hashing.
    let cart_snapshot: Vec<_> = cart_items
        .iter()
        .map(|item| {
            json!({
                "product_id": item.product_id,
                "quantity": item.quantity,
            })
        })
        .collect();
    let payload_json = json!({
        "user_id": req.user_id,
        "shipping_address_id": req.shipping_address_id,
        "coupon_code": req.coupon_code,
        "cart": cart_snapshot,
    });
    let request_hash = compute_request_hash(&payload_json.to_string());

    // If an idempotency key is provided, consult the idempotency_keys table before mutating state.
    const IDEMPOTENCY_SCOPE: &str = "place_order";
    if let Some(ref key) = idempotency_key {
        if let Some(existing) = IdempotencyKeys::find()
            .filter(idempotency_keys::Column::Scope.eq(IDEMPOTENCY_SCOPE))
            .filter(idempotency_keys::Column::Key.eq(key.as_str()))
            .one(txn)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
        {
            // Same key, different payload → conflict.
            if existing.request_hash != request_hash {
                return Err(Status::already_exists(
                    "Idempotency key reuse with different payload",
                ));
            }

            match existing.status {
                IdempotencyStatus::Processed => {
                    // Reconstruct the response from the stored order_id reference.
                    let order_id: i64 = existing
                        .response_ref
                        .as_ref()
                        .and_then(|s| s.parse().ok())
                        .ok_or_else(|| {
                            Status::internal("Invalid response_ref in idempotency_keys")
                        })?;

                    let existing_order = orders::Entity::find_by_id(order_id)
                        .one(txn)
                        .await
                        .map_err(|e| Status::internal(e.to_string()))?
                        .ok_or_else(|| {
                            Status::internal("Order referenced by idempotency_keys not found")
                        })?;

                    info!(
                        order_id = existing_order.order_id,
                        user_id = existing_order.user_id,
                        "place_order idempotent replay – returning existing order"
                    );
                    return Ok(Response::new(OrdersResponse {
                        items: vec![proto::proto::core::OrderResponse {
                            order_id: existing_order.order_id,
                            user_id: existing_order.user_id,
                            order_date: existing_order.order_date.to_string(),
                            shipping_address_id: existing_order.shipping_address_id,
                            total_amount: existing_order.total_amount.to_f64().unwrap_or(0.0),
                            status_id: existing_order.status_id,
                        }],
                    }));
                }
                IdempotencyStatus::Pending => {
                    return Err(Status::unavailable(
                        "Idempotent place_order still in progress; retry later",
                    ));
                }
                IdempotencyStatus::Failed => {
                    // Allow retry after failure by continuing below.
                }
            }
        } else {
            // Insert a fresh in_progress row. We update it to completed/failed later.
            let ttl_hours = std::env::var("IDEMPOTENCY_WINDOW_HOURS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(24_i64);
            let now = Utc::now();
            let expires = now + chrono::Duration::hours(ttl_hours);

            let active = idempotency_keys::ActiveModel {
                id: Default::default(),
                scope: Set(IDEMPOTENCY_SCOPE.to_string()),
                key: Set(key.to_string()),
                request_hash: Set(request_hash.clone()),
                response_ref: Set(None),
                status: Set(IdempotencyStatus::Pending),
                created_at: Set(now),
                expires_at: Set(expires),
            };

            active
                .insert(txn)
                .await
                .map_err(|e| Status::internal(e.to_string()))?;
        }
    }

    // Compute the gross amount in paise (integer minor units) to avoid float drift.
    let mut gross_paise: i64 = 0;
    for product in &order_products {
        if let Some(&quantity) = product_quantity_map.get(&product.product_id) {
            let price_paise = paise_from_major_f64(product.price);
            let line_paise = paise_checked_mul(price_paise, quantity).map_err(|e| {
                Status::internal(format!(
                    "Overflow computing line total for product {}: {}",
                    product.product_id, e
                ))
            })?;
            gross_paise = paise_checked_add(gross_paise, line_paise).map_err(|e| {
                Status::internal(format!("Overflow computing order total in paise: {}", e))
            })?;
        }
    }

    // Apply coupon if provided, deriving the discounted total in paise and coupon snapshot.
    // Do not increment coupon usage_count here; only on verified payment (Phase 4).
    let (total_paise, coupon_snapshot) = if let Some(ref code) = req.coupon_code {
        match check_coupon(txn, code, gross_paise, false).await {
            Ok(result) if result.is_valid => (
                result.final_amount_paise,
                Some((result.coupon_id, code.clone(), result.discount_amount_paise)),
            ),
            Ok(result) => {
                log::warn!("Coupon '{}' invalid at checkout: {}", code, result.reason);
                (gross_paise, None)
            }
            Err(e) => {
                log::warn!("Coupon check failed: {}", e);
                (gross_paise, None)
            }
        }
    } else {
        (gross_paise, None)
    };

    let discount_total_minor = gross_paise - total_paise;

    let create_order = create_order(
        txn,
        Request::new(CreateOrderRequest {
            shipping_address_id: req.shipping_address_id,
            status_id: 2, // Always start with order status is processing
            user_id: req.user_id,
            total_amount: paise_to_major_f64(total_paise),
            subtotal_minor: Some(gross_paise),
            shipping_minor: Some(0),
            tax_total_minor: Some(0),
            discount_total_minor: Some(discount_total_minor),
            grand_total_minor: Some(total_paise),
            applied_coupon_id: coupon_snapshot.as_ref().map(|s| s.0),
            applied_coupon_code: coupon_snapshot.as_ref().map(|s| s.1.clone()),
            applied_discount_paise: coupon_snapshot.as_ref().map(|s| s.2 as i32),
        }),
    )
    .await?
    .into_inner()
    .items
    .first()
    .unwrap()
    .clone();

    info!(
        order_id = create_order.order_id,
        user_id = create_order.user_id,
        "place_order created order"
    );

    let mut order_details: Vec<CreateOrderDetailRequest> = Vec::new();

    for product in order_products.iter() {
        let quantity = product_quantity_map
            .get(&product.product_id)
            .unwrap()
            .to_owned();
        let unit_price_paise = paise_from_major_f64(product.price);
        order_details.push(CreateOrderDetailRequest {
            order_id: create_order.order_id,
            product_id: product.product_id,
            quantity,
            price: product.price,
            unit_price_minor: Some(unit_price_paise as i32),
            discount_minor: None,
            tax_minor: None,
            sku: None,
            title: Some(product.name.clone()),
        })
    }

    let _ = create_order_details(
        txn,
        Request::new(CreateOrderDetailsRequest { order_details }),
    )
    .await?
    .into_inner()
    .items;

    // Atomic inventory decrement: reserve stock for this order (same transaction).
    for (product_id, quantity) in &product_quantity_map {
        let qty = *quantity;
        let result = txn
            .execute(Statement::from_sql_and_values(
                DbBackend::MySql,
                r#"UPDATE Inventory SET QuantityAvailable = QuantityAvailable - ? WHERE ProductID = ? AND QuantityAvailable >= ?"#,
                [qty.into(), (*product_id).into(), qty.into()],
            ))
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        if result.rows_affected() == 0 {
            return Err(Status::failed_precondition(format!(
                "Insufficient stock for product {} (need {}); inventory update had no effect",
                product_id, qty
            )));
        }
    }

    // Auto-create a pending payment intent for the new order.
    // razorpay_order_id must be obtained from Razorpay; here we generate a placeholder
    // that callers must replace via CreatePaymentIntent when they have a real Razorpay ID.
    let razorpay_order_id = format!("rzp_pending_{}", create_order.order_id);
    let amount_paise = total_paise;
    if let Err(e) = create_payment_intent(
        txn,
        tonic::Request::new(CreatePaymentIntentRequest {
            order_id: create_order.order_id,
            user_id: req.user_id,
            amount_paise,
            currency: Some("INR".to_string()),
            razorpay_order_id,
        }),
    )
    .await
    {
        log::warn!(
            "Failed to create payment intent for order {}: {}",
            create_order.order_id,
            e
        );
    }

    // Emit audit event: order placed
    let _ = create_order_event(
        txn,
        tonic::Request::new(CreateOrderEventRequest {
            order_id: create_order.order_id,
            event_type: "order_placed".to_string(),
            from_status: None,
            to_status: Some("processing".to_string()),
            actor_type: "customer".to_string(),
            message: Some(format!(
                "Order {} placed successfully",
                create_order.order_id
            )),
        }),
    )
    .await;

    let _ = delete_cart_item(
        txn,
        Request::new(DeleteCartItemRequest {
            user_id: Some(req.user_id),
            cart_id: None,
            session_id: None,
        }),
    )
    .await?
    .into_inner()
    .items;

    // If we have an idempotency key, mark this operation as completed and store
    // the created order_id as the response_ref so replays can return it.
    if let Some(key) = idempotency_key {
        if let Some(existing) = IdempotencyKeys::find()
            .filter(idempotency_keys::Column::Scope.eq(IDEMPOTENCY_SCOPE))
            .filter(idempotency_keys::Column::Key.eq(key.as_str()))
            .one(txn)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
        {
            let mut active: idempotency_keys::ActiveModel = existing.into();
            active.status = Set(IdempotencyStatus::Processed);
            active.response_ref = Set(Some(create_order.order_id.to_string()));
            active
                .update(txn)
                .await
                .map_err(|e| Status::internal(e.to_string()))?;
        }
    }

    Ok(Response::new(OrdersResponse {
        items: vec![create_order],
    }))
}
