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
    Statement, TransactionTrait,
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

fn pending_idempotency_ttl_minutes() -> i64 {
    std::env::var("IDEMPOTENCY_PENDING_TIMEOUT_MINUTES")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(15)
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

#[derive(Debug, Clone)]
struct FrozenLinePricing {
    variant_id: i64,
    quantity: i64,
    unit_price_minor: i64,
    gross_line_minor: i64,
    discount_minor: i64,
    net_line_minor: i64,
    title: String,
}

fn allocate_discount_across_lines(
    gross_line_totals_minor: &[i64],
    requested_discount_minor: i64,
) -> Vec<i64> {
    if gross_line_totals_minor.is_empty() {
        return Vec::new();
    }

    let safe_lines: Vec<i64> = gross_line_totals_minor
        .iter()
        .map(|v| (*v).max(0))
        .collect();
    let gross_total_minor: i64 = safe_lines.iter().sum();
    if gross_total_minor <= 0 {
        return vec![0; safe_lines.len()];
    }

    let discount_minor = requested_discount_minor.clamp(0, gross_total_minor);
    if discount_minor == 0 {
        return vec![0; safe_lines.len()];
    }

    let discount_i128 = i128::from(discount_minor);
    let gross_total_i128 = i128::from(gross_total_minor);

    let mut discounts = vec![0_i64; safe_lines.len()];
    let mut floors_sum = 0_i64;
    let mut remainders: Vec<(usize, i128)> = Vec::with_capacity(safe_lines.len());

    for (idx, line_minor) in safe_lines.iter().enumerate() {
        let line_i128 = i128::from(*line_minor);
        let numerator = discount_i128.saturating_mul(line_i128);
        let floor_share = (numerator / gross_total_i128) as i64;
        let remainder = numerator % gross_total_i128;
        discounts[idx] = floor_share;
        floors_sum += floor_share;
        remainders.push((idx, remainder));
    }

    let mut remainder_to_distribute = discount_minor.saturating_sub(floors_sum);
    if remainder_to_distribute <= 0 {
        return discounts;
    }

    remainders
        .sort_by(|(idx_a, rem_a), (idx_b, rem_b)| rem_b.cmp(rem_a).then_with(|| idx_a.cmp(idx_b)));

    for (idx, _) in remainders {
        if remainder_to_distribute == 0 {
            break;
        }
        discounts[idx] += 1;
        remainder_to_distribute -= 1;
    }

    discounts
}

fn apply_frozen_line_discounts(
    lines: &mut [FrozenLinePricing],
    requested_discount_minor: i64,
) -> i64 {
    let gross_lines: Vec<i64> = lines.iter().map(|l| l.gross_line_minor.max(0)).collect();
    let discounts = allocate_discount_across_lines(&gross_lines, requested_discount_minor);
    let mut applied_discount_minor = 0_i64;
    for (line, discount_minor) in lines.iter_mut().zip(discounts) {
        let clamped = discount_minor.clamp(0, line.gross_line_minor.max(0));
        line.discount_minor = clamped;
        line.net_line_minor = line.gross_line_minor.saturating_sub(clamped).max(0);
        applied_discount_minor = applied_discount_minor.saturating_add(clamped);
    }
    applied_discount_minor
}

fn qualifies_for_free_shipping(
    items_total_after_discount_minor: i64,
    threshold_minor: i64,
) -> bool {
    items_total_after_discount_minor >= threshold_minor
}

#[allow(clippy::result_large_err)]
async fn lock_inventory_row_and_get_available_quantity(
    txn: &DatabaseTransaction,
    variant_id: i64,
) -> Result<i64, Status> {
    let row = txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::MySql,
            r#"SELECT COUNT(*) AS row_count,
                      COALESCE(MAX(QuantityAvailable), 0) AS quantity_available
               FROM Inventory
               WHERE VariantID = ?
               FOR UPDATE"#,
            [variant_id.into()],
        ))
        .await
        .map_err(|e| Status::internal(e.to_string()))?
        .ok_or_else(|| Status::internal("Inventory row-count query returned no row"))?;
    let row_count = row
        .try_get::<i64>("", "row_count")
        .map_err(|e| Status::internal(e.to_string()))?;
    let quantity_available = row
        .try_get::<i64>("", "quantity_available")
        .map_err(|e| Status::internal(e.to_string()))?;

    match row_count {
        1 => Ok(quantity_available),
        0 => Err(Status::failed_precondition(format!(
            "No inventory row exists for variant {}",
            variant_id
        ))),
        _ => Err(Status::internal(format!(
            "Inventory data corruption: expected exactly 1 row for variant {}, found {}",
            variant_id, row_count
        ))),
    }
}

pub async fn place_order(
    txn: &DatabaseTransaction,
    request: Request<PlaceOrderRequest>,
) -> Result<Response<OrdersResponse>, Status> {
    // Run checkout within a nested transaction (savepoint) so any failure
    // rolls back all place_order side effects before returning to caller.
    let nested_txn = txn
        .begin()
        .await
        .map_err(|e| Status::internal(format!("failed to begin nested place_order txn: {e}")))?;

    let result = place_order_in_txn(&nested_txn, request).await;
    match result {
        Ok(response) => {
            nested_txn.commit().await.map_err(|e| {
                Status::internal(format!("failed to commit nested place_order txn: {e}"))
            })?;
            Ok(response)
        }
        Err(err) => {
            let _ = nested_txn.rollback().await;
            Err(err)
        }
    }
}

async fn place_order_in_txn(
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
    let normalized_payment_mode = req
        .payment_mode
        .as_deref()
        .unwrap_or("prepaid")
        .trim()
        .to_lowercase();
    if normalized_payment_mode != "prepaid" && normalized_payment_mode != "cod" {
        return Err(Status::invalid_argument(
            "payment_mode must be either 'prepaid' or 'cod'",
        ));
    }
    let is_cod_checkout = normalized_payment_mode == "cod";
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
                        "payment_mode": normalized_payment_mode.as_str(),
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
                    let stale_before =
                        Utc::now() - chrono::Duration::minutes(pending_idempotency_ttl_minutes());
                    if existing.created_at < stale_before {
                        warn!(
                            idempotency_key = %key,
                            created_at = %existing.created_at,
                            "stale pending idempotency key detected; marking failed for safe retry"
                        );
                        let mut active: idempotency_keys::ActiveModel = existing.into();
                        active.status = Set(IdempotencyStatus::Failed);
                        active
                            .update(txn)
                            .await
                            .map_err(|e| Status::internal(e.to_string()))?;
                    } else {
                        return Err(Status::unavailable(
                            "Idempotent place_order still in progress; retry later",
                        ));
                    }
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
        "payment_mode": normalized_payment_mode.as_str(),
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

    // Compute immutable line snapshots and gross amount in paise.
    let mut frozen_lines: Vec<FrozenLinePricing> = Vec::with_capacity(cart_items.len());
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
        frozen_lines.push(FrozenLinePricing {
            variant_id: item.variant_id,
            quantity: item.quantity,
            unit_price_minor: unit_paise,
            gross_line_minor: line_paise,
            discount_minor: 0,
            net_line_minor: line_paise,
            title: product.name.clone(),
        });
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

    let requested_discount_total_minor = gross_paise.saturating_sub(total_paise);
    let applied_discount_total_minor =
        apply_frozen_line_discounts(&mut frozen_lines, requested_discount_total_minor);
    let items_total_minor_after_discount = gross_paise.saturating_sub(applied_discount_total_minor);
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

    let free_shipping_threshold_minor = crate::order_policy::free_shipping_threshold_minor();
    let qualifies_free_shipping = qualifies_for_free_shipping(
        items_total_minor_after_discount,
        free_shipping_threshold_minor,
    );
    let shipping_quote = if qualifies_free_shipping {
        None
    } else {
        Some(
            match best_courier_quote_for_checkout(
                delivery_postcode.as_str(),
                items_total_minor_after_discount,
                total_units,
            )
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
            },
        )
    };
    let shipping_minor = shipping_quote
        .as_ref()
        .map(|q| q.shipping_amount_minor.max(0))
        .unwrap_or(0);
    let grand_total_paise = paise_checked_add(items_total_minor_after_discount, shipping_minor)
        .map_err(|e| Status::internal(format!("Overflow computing grand total in paise: {}", e)))?;

    let pending_status_id = order_state_machine::get_status_id(txn, "active_sale")
        .await
        .map_err(|e| Status::internal(e.to_string()))?
        .or(order_state_machine::get_status_id(txn, "pending")
            .await
            .map_err(|e| Status::internal(e.to_string()))?)
        .ok_or_else(|| Status::internal("OrderStatus 'active_sale' not found"))?;

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
            discount_total_minor: Some(applied_discount_total_minor),
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
        payment_mode = %normalized_payment_mode,
        "place_order created order"
    );

    let order_created_at = orders::Entity::find_by_id(create_order.order_id)
        .one(txn)
        .await
        .map_err(|e| Status::internal(e.to_string()))?
        .map(|o| o.created_at)
        .ok_or_else(|| Status::internal("Created order not found for timestamp policy"))?;
    let cancel_window_ends_at = crate::order_policy::cancel_window_deadline(order_created_at);
    let earliest_booking_at = crate::order_policy::earliest_booking_deadline(order_created_at);
    let pickup_target_at = crate::order_policy::default_pickup_target(order_created_at);

    txn.execute(Statement::from_sql_and_values(
        DbBackend::MySql,
        r#"UPDATE Orders
           SET payment_method = ?,
               payment_status = 'pending',
               cancel_window_ends_at = ?,
               earliest_booking_at = ?,
               pickup_target_at = ?,
               pickup_target_set_by = 'system',
               pickup_target_reason = COALESCE(pickup_target_reason, 'order_created'),
               pickup_target_updated_at = UTC_TIMESTAMP(),
               updated_at = UTC_TIMESTAMP()
           WHERE OrderID = ?"#,
        [
            normalized_payment_mode.clone().into(),
            cancel_window_ends_at.into(),
            earliest_booking_at.into(),
            pickup_target_at.into(),
            create_order.order_id.into(),
        ],
    ))
    .await
    .map_err(|e| Status::internal(e.to_string()))?;

    // Freeze phase-1 pricing snapshots with explicit columns used by refund and audit logic.
    txn.execute(Statement::from_sql_and_values(
        DbBackend::MySql,
        r#"UPDATE Orders
           SET items_total_minor_before_discount = ?,
               items_total_minor_after_discount = ?,
               shipping_charge_minor = ?,
               updated_at = UTC_TIMESTAMP()
           WHERE OrderID = ?"#,
        [
            gross_paise.into(),
            items_total_minor_after_discount.into(),
            shipping_minor.into(),
            create_order.order_id.into(),
        ],
    ))
    .await
    .map_err(|e| Status::internal(e.to_string()))?;

    let mut order_details: Vec<CreateOrderDetailRequest> = Vec::new();

    for line in &frozen_lines {
        let unit_price_minor = i32::try_from(line.unit_price_minor)
            .map_err(|_| Status::internal("unit_price_minor overflow"))?;
        let line_discount_minor = i32::try_from(line.discount_minor)
            .map_err(|_| Status::internal("line discount overflow"))?;
        order_details.push(CreateOrderDetailRequest {
            order_id: create_order.order_id,
            variant_id: line.variant_id,
            quantity: line.quantity,
            price_paise: line.net_line_minor,
            unit_price_minor: Some(unit_price_minor),
            discount_minor: Some(line_discount_minor),
            tax_minor: None,
            sku: None,
            title: Some(line.title.clone()),
        })
    }

    let created_order_details = create_order_details(
        txn,
        Request::new(CreateOrderDetailsRequest { order_details }),
    )
    .await?
    .into_inner()
    .items;

    if created_order_details.len() != frozen_lines.len() {
        return Err(Status::internal(format!(
            "OrderDetails insert mismatch: expected {}, inserted {}",
            frozen_lines.len(),
            created_order_details.len()
        )));
    }

    // Reserve inventory only after order + order details were fully persisted in the
    // nested place_order transaction. Any later failure will roll this reservation back.
    for (variant_id, quantity) in &variant_quantity_map {
        let qty = *quantity;
        let quantity_available =
            lock_inventory_row_and_get_available_quantity(txn, *variant_id).await?;
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
                [qty.into(), (*variant_id).into()],
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

    if !is_cod_checkout {
        // Create a pending payment intent with a real gateway order id before
        // returning checkout data to the frontend.
        let amount_paise = grand_total_paise;
        let _payment_intent = create_payment_intent(
            txn,
            tonic::Request::new(CreatePaymentIntentRequest {
                order_id: create_order.order_id,
                user_id: req.user_id,
                amount_paise,
                currency: Some("INR".to_string()),
                razorpay_order_id: None,
            }),
        )
        .await?
        .into_inner()
        .items
        .into_iter()
        .next()
        .ok_or_else(|| Status::internal("create_payment_intent returned no payment intent"))?;
    } else {
        // COD orders are accepted immediately without creating a Razorpay payment intent.
        let confirmed_status_id = order_state_machine::get_status_id(txn, "confirmed")
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::internal("OrderStatus 'confirmed' not found"))?;
        txn.execute(Statement::from_sql_and_values(
            DbBackend::MySql,
            r#"UPDATE Orders
               SET StatusID = ?,
                   updated_at = UTC_TIMESTAMP()
               WHERE OrderID = ?"#,
            [confirmed_status_id.into(), create_order.order_id.into()],
        ))
        .await
        .map_err(|e| Status::internal(e.to_string()))?;
        let _ = create_order_event(
            txn,
            tonic::Request::new(CreateOrderEventRequest {
                order_id: create_order.order_id,
                event_type: "cod_order_confirmed".to_string(),
                from_status: Some("active_sale".to_string()),
                to_status: Some("confirmed".to_string()),
                actor_type: "customer".to_string(),
                message: Some("COD order accepted and awaiting fulfillment".to_string()),
            }),
        )
        .await;

        let _ = crate::handlers::invoices::ensure_invoice_for_order(
            txn,
            create_order.order_id,
            "cod_confirmed",
        )
        .await?;
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

    let persisted_order = orders::Entity::find_by_id(create_order.order_id)
        .one(txn)
        .await
        .map_err(|e| Status::internal(e.to_string()))?
        .ok_or_else(|| Status::internal("Created order not found"))?;
    let mut wire_order = order_response::from_model(&persisted_order);
    wire_order.cancel_window_ends_at = Some(cancel_window_ends_at.to_rfc3339());
    wire_order.earliest_booking_at = Some(earliest_booking_at.to_rfc3339());
    wire_order.pickup_target_at = Some(pickup_target_at.to_rfc3339());

    Ok(Response::new(OrdersResponse {
        items: vec![wire_order],
    }))
}

#[cfg(test)]
mod tests {
    use super::{
        allocate_discount_across_lines, apply_frozen_line_discounts, qualifies_for_free_shipping,
        FrozenLinePricing,
    };

    #[test]
    fn allocates_coupon_discount_across_multiple_lines_deterministically() {
        let gross_lines = vec![3_000, 2_000, 1_000];
        let discounts = allocate_discount_across_lines(&gross_lines, 1_000);
        assert_eq!(discounts, vec![500, 333, 167]);
    }

    #[test]
    fn allocation_rounding_is_deterministic_with_equal_remainders() {
        let gross_lines = vec![100, 100, 100];
        let d1 = allocate_discount_across_lines(&gross_lines, 1);
        let d2 = allocate_discount_across_lines(&gross_lines, 1);
        assert_eq!(d1, d2);
        assert_eq!(d1, vec![1, 0, 0]);
    }

    #[test]
    fn lines_plus_shipping_match_grand_total_after_discount_allocation() {
        let mut lines = vec![
            FrozenLinePricing {
                variant_id: 1,
                quantity: 1,
                unit_price_minor: 2_000,
                gross_line_minor: 2_000,
                discount_minor: 0,
                net_line_minor: 2_000,
                title: "A".to_string(),
            },
            FrozenLinePricing {
                variant_id: 2,
                quantity: 1,
                unit_price_minor: 1_000,
                gross_line_minor: 1_000,
                discount_minor: 0,
                net_line_minor: 1_000,
                title: "B".to_string(),
            },
        ];
        let applied_discount = apply_frozen_line_discounts(&mut lines, 333);
        assert_eq!(applied_discount, 333);
        let items_after_discount: i64 = lines.iter().map(|l| l.net_line_minor).sum();
        let shipping_charge = 149;
        let grand_total = items_after_discount + shipping_charge;
        assert_eq!(items_after_discount, 2_667);
        assert_eq!(grand_total, 2_816);
    }

    #[test]
    fn free_shipping_threshold_is_evaluated_on_post_discount_items_total() {
        assert!(!qualifies_for_free_shipping(9_999, 10_000));
        assert!(qualifies_for_free_shipping(10_000, 10_000));
    }
}
