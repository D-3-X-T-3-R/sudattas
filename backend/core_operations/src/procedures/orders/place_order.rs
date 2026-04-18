use crate::handlers::coupons::{
    eligibility::{check_coupon_scope, check_per_customer_limit, CartProduct},
    validate_coupon::check_coupon,
};
use crate::handlers::idempotency::compute_request_hash;
use crate::handlers::order_events::create_order_event;
use crate::integrations::shiprocket::{best_courier_quote_for_checkout, ShiprocketError};
use crate::money::{paise_checked_add, paise_checked_mul};

use crate::handlers::{
    cart::get_cart_items, order_details::create_order_details, orders::create_order,
    orders::order_response, payment_intents::create_payment_intent, products::get_products_by_id,
    shipments::upsert_quote_selection,
};
use crate::order_state_machine;

use core_db_entities::entity::prelude::IdempotencyKeys;
use core_db_entities::entity::{
    cart, idempotency_keys, orders, product_variants,
    sea_orm_active_enums::Status as IdempotencyStatus, shipping_addresses,
};
use proto::proto::core::{
    CreateOrderDetailRequest, CreateOrderDetailsRequest, CreateOrderEventRequest,
    CreateOrderRequest, CreatePaymentIntentRequest, GetCartItemsRequest, GetProductsByIdRequest,
    OrdersResponse, PlaceOrderRequest,
};
use sea_orm::DbBackend;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, EntityTrait, QueryFilter, Set,
    Statement,
};

use chrono::Utc;
use sea_orm::DatabaseTransaction;
use serde_json::json;
use std::collections::{HashMap, HashSet};
use tonic::{Request, Response, Status};
use tracing::{info, warn};

fn map_shipping_quote_error(error: ShiprocketError) -> Status {
    let message = match error {
        ShiprocketError::NotConfigured => {
            "Live shipping quote is unavailable because shipping is not configured"
        }
        _ => "Live shipping quote is unavailable for this checkout",
    };
    Status::unavailable(message)
}

#[allow(clippy::result_large_err)]
fn validate_selected_cart_ids(selected_cart_ids: &[i64]) -> Result<Vec<i64>, Status> {
    if selected_cart_ids.is_empty() {
        return Err(Status::failed_precondition(
            "Cannot place order: no selected cart items provided",
        ));
    }

    let mut seen = HashSet::new();
    let mut normalized = Vec::with_capacity(selected_cart_ids.len());
    for cart_id in selected_cart_ids {
        if *cart_id <= 0 {
            return Err(Status::invalid_argument(format!(
                "Invalid selected cart item id {}",
                cart_id
            )));
        }
        if !seen.insert(*cart_id) {
            return Err(Status::invalid_argument(format!(
                "Duplicate selected cart item id {}",
                cart_id
            )));
        }
        normalized.push(*cart_id);
    }
    Ok(normalized)
}

#[allow(clippy::result_large_err)]
fn pick_selected_cart_items(
    cart_items: Vec<proto::proto::core::CartItemResponse>,
    selected_cart_ids: &[i64],
) -> Result<Vec<proto::proto::core::CartItemResponse>, Status> {
    let mut by_cart_id = cart_items
        .into_iter()
        .map(|item| (item.cart_id, item))
        .collect::<HashMap<_, _>>();

    let mut selected_items = Vec::with_capacity(selected_cart_ids.len());
    for cart_id in selected_cart_ids {
        let item = by_cart_id.remove(cart_id).ok_or_else(|| {
            Status::invalid_argument(format!(
                "Selected cart item {} was not found in the current cart",
                cart_id
            ))
        })?;
        selected_items.push(item);
    }
    Ok(selected_items)
}

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
    let selected_cart_ids = validate_selected_cart_ids(&req.selected_cart_ids)?;
    crate::observability::log_operational_event(
        "order_place_requested",
        &[
            ("user_id", req.user_id.to_string()),
            ("shipping_address_id", req.shipping_address_id.to_string()),
            (
                "selected_cart_ids",
                selected_cart_ids
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
            ),
        ],
    );

    // If an idempotency key is present, check for an existing Processed/Pending result.
    // For Processed we must distinguish replay (same payload) from conflict (different payload):
    // load cart and compare request_hash when cart is non-empty; if different, return AlreadyExists.
    const IDEMPOTENCY_SCOPE: &str = "place_order";
    if let Some(ref key) = idempotency_key {
        if let Some(existing) = IdempotencyKeys::find()
            .filter(idempotency_keys::Column::Scope.eq(IDEMPOTENCY_SCOPE))
            .filter(idempotency_keys::Column::Key.eq(key.as_str()))
            .one(txn)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
        {
            match existing.status {
                IdempotencyStatus::Processed => {
                    let payload_json = json!({
                        "user_id": req.user_id,
                        "shipping_address_id": req.shipping_address_id,
                        "coupon_code": req.coupon_code,
                        "selected_cart_ids": selected_cart_ids,
                    });
                    let incoming_hash = compute_request_hash(&payload_json.to_string());
                    if existing.request_hash != incoming_hash {
                        return Err(Status::already_exists(
                            "Idempotency key reuse with different payload",
                        ));
                    }
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
                        items: vec![order_response::from_model(&existing_order)],
                    }));
                }
                IdempotencyStatus::Pending => {
                    return Err(Status::unavailable(
                        "Idempotent place_order still in progress; retry later",
                    ));
                }
                IdempotencyStatus::Failed => {
                    // Allow retry; fall through to place order and update row later.
                }
                IdempotencyStatus::ClientVerified | IdempotencyStatus::NeedsReview => {
                    // Not used for idempotency_keys; treat like Pending if ever seen.
                    return Err(Status::unavailable(
                        "Idempotent place_order still in progress; retry later",
                    ));
                }
            }
        }
    }

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

    let cart_items = pick_selected_cart_items(cart_items, &selected_cart_ids)?;

    let (variant_quantity_map, variant_ids): (HashMap<i64, i64>, Vec<i64>) = cart_items
        .iter()
        .map(|item| ((item.variant_id, item.quantity), item.variant_id))
        .unzip();

    let variants = product_variants::Entity::find()
        .filter(product_variants::Column::VariantId.is_in(variant_ids.clone()))
        .all(txn)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;
    let product_ids: Vec<i64> = variants.iter().map(|v| v.product_id).collect();
    let order_products =
        get_products_by_id(txn, Request::new(GetProductsByIdRequest { product_ids }))
            .await?
            .into_inner()
            .items;
    let variants_by_id: HashMap<i64, product_variants::Model> =
        variants.into_iter().map(|v| (v.variant_id, v)).collect();
    let products_by_id: HashMap<i64, proto::proto::core::ProductResponse> = order_products
        .iter()
        .map(|p| (p.product_id, p.clone()))
        .collect();

    // Build a stable representation of the logical request payload for idempotency hashing.
    let cart_snapshot: Vec<_> = cart_items
        .iter()
        .map(|item| {
            json!({
                "variant_id": item.variant_id,
                "quantity": item.quantity,
            })
        })
        .collect();
    let payload_json = json!({
        "user_id": req.user_id,
        "shipping_address_id": req.shipping_address_id,
        "coupon_code": req.coupon_code,
        "selected_cart_ids": selected_cart_ids,
        "cart": cart_snapshot,
    });
    let request_hash = compute_request_hash(&payload_json.to_string());

    // If an idempotency key is provided, enforce payload consistency and insert Pending row if new.
    // (Processed/Pending replay already returned above.)
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
            // Existing row is Failed (Processed/Pending already returned above); allow retry, no insert.
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
    for item in &cart_items {
        let variant = variants_by_id.get(&item.variant_id).ok_or_else(|| {
            Status::invalid_argument(format!("Variant {} not found", item.variant_id))
        })?;
        let product = products_by_id.get(&variant.product_id).ok_or_else(|| {
            Status::internal(format!(
                "Product {} for variant {} not found",
                variant.product_id, item.variant_id
            ))
        })?;
        let unit_paise = product.price_paise + i64::from(variant.additional_price.unwrap_or(0));
        let line_paise = paise_checked_mul(unit_paise, item.quantity).map_err(|e| {
            Status::internal(format!(
                "Overflow computing line total for variant {}: {}",
                item.variant_id, e
            ))
        })?;
        gross_paise = paise_checked_add(gross_paise, line_paise).map_err(|e| {
            Status::internal(format!("Overflow computing order total in paise: {}", e))
        })?;
    }

    // Apply coupon if provided, deriving the discounted total in paise and coupon snapshot.
    // Do not increment coupon usage_count here; only on verified payment (Phase 4).
    // P1: enforce per-customer limit and allowlist/denylist scope before applying.
    let (total_paise, coupon_snapshot) = if let Some(ref code) = req.coupon_code {
        match check_coupon(txn, code, gross_paise, false).await {
            Ok(result) if result.is_valid => {
                let cart_for_scope: Vec<CartProduct> = cart_items
                    .iter()
                    .filter_map(|item| {
                        let v = variants_by_id.get(&item.variant_id)?;
                        let p = products_by_id.get(&v.product_id)?;
                        Some(CartProduct {
                            product_id: p.product_id,
                            category_id: Some(p.category_id),
                        })
                    })
                    .collect();
                let ok_per_customer = check_per_customer_limit(txn, result.coupon_id, req.user_id)
                    .await
                    .unwrap_or(false);
                let ok_scope = check_coupon_scope(txn, result.coupon_id, &cart_for_scope)
                    .await
                    .unwrap_or(false);
                if ok_per_customer && ok_scope {
                    (
                        result.final_amount_paise,
                        Some((result.coupon_id, code.clone(), result.discount_amount_paise)),
                    )
                } else {
                    if !ok_per_customer {
                        log::warn!(
                            "Coupon '{}' not applied: per-customer usage limit reached",
                            code
                        );
                    }
                    if !ok_scope {
                        log::warn!(
                            "Coupon '{}' not applied: cart does not meet product/category scope",
                            code
                        );
                    }
                    (gross_paise, None)
                }
            }
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
    let total_units: i64 = cart_items.iter().map(|item| item.quantity.max(1)).sum();

    let shipping_address = shipping_addresses::Entity::find_by_id(req.shipping_address_id)
        .one(txn)
        .await
        .map_err(|e| Status::internal(e.to_string()))?
        .ok_or_else(|| {
            Status::invalid_argument(format!(
                "Shipping address {} not found",
                req.shipping_address_id
            ))
        })?;
    if shipping_address.user_id != Some(req.user_id) {
        return Err(Status::permission_denied(
            "Shipping address does not belong to the requesting user",
        ));
    }
    let delivery_postcode = shipping_address.postal_code.trim().to_string();

    let shipping_quote =
        match best_courier_quote_for_checkout(delivery_postcode.as_str(), total_paise, total_units)
            .await
        {
            Ok(Some(quote)) => quote,
            Ok(None) => {
                warn!("checkout shipping quote unavailable without courier result");
                return Err(Status::unavailable(
                    "Live shipping quote is unavailable for this checkout",
                ));
            }
            Err(error) => {
                warn!("checkout shipping quote failed: {}", error);
                return Err(map_shipping_quote_error(error));
            }
        };
    let shipping_minor = shipping_quote.shipping_amount_minor.max(0);
    let shipping_quote = Some(shipping_quote);
    let grand_total_paise = paise_checked_add(total_paise, shipping_minor)
        .map_err(|e| Status::internal(format!("Overflow computing grand total in paise: {}", e)))?;

    // Reserve inventory before creating the order so that on insufficient stock we fail without creating any order.
    for (variant_id, quantity) in &variant_quantity_map {
        let qty = *quantity;
        let result = txn
            .execute(Statement::from_sql_and_values(
                DbBackend::MySql,
                r#"UPDATE Inventory SET QuantityAvailable = QuantityAvailable - ? WHERE VariantID = ? AND QuantityAvailable >= ?"#,
                [qty.into(), (*variant_id).into(), qty.into()],
            ))
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        if result.rows_affected() == 0 {
            crate::observability::record_inventory_update_failure_total();
            return Err(Status::failed_precondition(format!(
                "Insufficient stock for variant {} (need {}); inventory update had no effect",
                variant_id, qty
            )));
        }
    }

    let pending_status_id = order_state_machine::get_status_id(txn, "pending")
        .await
        .map_err(|e| Status::internal(e.to_string()))?
        .ok_or_else(|| Status::internal("OrderStatus 'pending' not found"))?;

    let create_order = create_order(
        txn,
        Request::new(CreateOrderRequest {
            shipping_address_id: req.shipping_address_id,
            status_id: pending_status_id,
            user_id: req.user_id,
            total_amount_paise: grand_total_paise,
            subtotal_minor: Some(gross_paise),
            shipping_minor: Some(shipping_minor),
            tax_total_minor: Some(0),
            discount_total_minor: Some(discount_total_minor),
            grand_total_minor: Some(grand_total_paise),
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
        public_order_ref = %create_order.public_order_ref,
        user_id = create_order.user_id,
        "place_order created order"
    );

    upsert_quote_selection(
        txn,
        create_order.order_id,
        shipping_quote.as_ref().map(|q| q.courier_id).unwrap_or_default(),
        shipping_quote
            .as_ref()
            .map(|q| q.courier_name.as_str())
            .unwrap_or(""),
        shipping_minor,
        &json!({
            "courier_id": shipping_quote.as_ref().map(|q| q.courier_id),
            "courier_name": shipping_quote.as_ref().map(|q| q.courier_name.clone()),
            "shipping_amount_minor": shipping_minor,
            "estimated_delivery_days": shipping_quote.as_ref().and_then(|q| q.estimated_delivery_days),
        }),
    )
    .await?;

    let mut order_details: Vec<CreateOrderDetailRequest> = Vec::new();

    for item in &cart_items {
        let variant = variants_by_id.get(&item.variant_id).ok_or_else(|| {
            Status::invalid_argument(format!("Variant {} not found", item.variant_id))
        })?;
        let product = products_by_id.get(&variant.product_id).ok_or_else(|| {
            Status::internal(format!(
                "Product {} for variant {} not found",
                variant.product_id, item.variant_id
            ))
        })?;
        let quantity = item.quantity;
        let unit_price_paise =
            product.price_paise + i64::from(variant.additional_price.unwrap_or(0));
        let line_total_paise = paise_checked_mul(unit_price_paise, quantity).unwrap_or(0);
        order_details.push(CreateOrderDetailRequest {
            order_id: create_order.order_id,
            variant_id: item.variant_id,
            quantity,
            price_paise: line_total_paise,
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

    // Auto-create a pending payment intent: backend creates Razorpay order via API (server-authoritative).
    let amount_paise = grand_total_paise;
    let payment_intent = create_payment_intent(
        txn,
        tonic::Request::new(CreatePaymentIntentRequest {
            order_id: create_order.order_id,
            user_id: req.user_id,
            amount_paise,
            currency: Some("INR".to_string()),
            razorpay_order_id: None, // Backend will call Razorpay Orders API and store returned id.
        }),
    )
    .await?
    .into_inner()
    .items
    .into_iter()
    .next()
    .ok_or_else(|| Status::internal("create_payment_intent returned no payment intent"))?;

    if payment_intent.razorpay_order_id.starts_with("rzp_pending_") {
        return Err(Status::unavailable(format!(
            "Failed to create payment intent for order {}",
            create_order.order_id
        )));
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
                "Order {} placed successfully{}",
                create_order.order_id,
                shipping_quote
                    .as_ref()
                    .map(|q| format!(
                        " (courier quote: {} / ₹{:.2})",
                        q.courier_name,
                        (q.shipping_amount_minor as f64) / 100.0
                    ))
                    .unwrap_or_default()
            )),
        }),
    )
    .await;

    let selected_snapshot_condition =
        cart_items.iter().fold(Condition::any(), |condition, item| {
            condition.add(
                Condition::all()
                    .add(cart::Column::CartId.eq(item.cart_id))
                    .add(cart::Column::VariantId.eq(item.variant_id))
                    .add(cart::Column::Quantity.eq(item.quantity)),
            )
        });
    let delete_result = cart::Entity::delete_many()
        .filter(cart::Column::UserId.eq(req.user_id))
        .filter(selected_snapshot_condition)
        .exec(txn)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;
    if delete_result.rows_affected != cart_items.len() as u64 {
        return Err(Status::internal(
            "Selected cart items changed during checkout; please refresh your cart and try again",
        ));
    }

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
