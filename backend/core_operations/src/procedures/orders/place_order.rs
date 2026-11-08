use crate::handlers::coupons::{
    eligibility::{check_coupon_scope, check_per_customer_limit, CartProduct},
    validate_coupon::check_coupon,
};
use crate::handlers::db_errors::is_deadlock_status;

use crate::handlers::order_events::create_order_event;
use crate::integrations::shiprocket::ShiprocketCourierQuote;
use crate::money::{paise_checked_add, paise_checked_mul};

use crate::handlers::{
    cart::get_cart_items, order_details::create_order_details, orders::create_order,
    orders::order_response, payment_intents::create_payment_intent, products::get_products_by_id,
    transactions::create_transaction,
};
use crate::order_state_machine;

use core_db_entities::entity::{
    cart, orders, product_variants, sea_orm_active_enums::PaymentStatus, shipping_addresses,
};
use proto::proto::core::{
    CreateOrderDetailRequest, CreateOrderDetailsRequest, CreateOrderEventRequest,
    CreateOrderRequest, CreatePaymentIntentRequest, CreateTransactionRequest, GetCartItemsRequest,
    GetProductsByIdRequest, OrdersResponse, PlaceOrderRequest,
};
use sea_orm::DbBackend;
use sea_orm::{
    ColumnTrait, Condition, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter,
    Statement, TransactionTrait,
};

use sea_orm::DatabaseTransaction;

use std::collections::{HashMap, HashSet};
use tonic::{Request, Response, Status};
use tracing::info;

#[allow(clippy::result_large_err)]
pub(super) fn validate_selected_cart_ids(selected_cart_ids: &[i64]) -> Result<Vec<i64>, Status> {
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
pub(crate) async fn lock_inventory_row_and_get_available_quantity(
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

/// Data gathered/validated in the short prep phase, needed by both the external calls that
/// follow and the write phase after that.
struct PlaceOrderPrep {
    normalized_payment_mode: String,
    is_cod_checkout: bool,
    cart_items: Vec<proto::proto::core::CartItemResponse>,
    frozen_lines: Vec<FrozenLinePricing>,
    gross_paise: i64,
    applied_discount_total_minor: i64,
    items_total_minor_after_discount: i64,
    total_units: i64,
    coupon_snapshot: Option<(i64, String, i64)>,
    delivery_postcode: String,
    qualifies_free_shipping: bool,
}

pub async fn place_order(
    db: &DatabaseConnection,
    request: Request<PlaceOrderRequest>,
) -> Result<Response<OrdersResponse>, Status> {
    use super::checkout_claim::{self, Claim};
    let key = request
        .metadata()
        .get("idempotency-key")
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned)
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let mut req = request.into_inner();
    let hash = checkout_claim::normalize(&mut req)?;
    let lease = match checkout_claim::claim(db, &req, key, &hash).await? {
        Claim::Replay(response) => return Ok(response),
        Claim::Acquired(lease) => lease,
    };
    let prep_txn = db
        .begin()
        .await
        .map_err(|e| Status::internal(e.to_string()))?;
    // Durable replay was resolved above, BEFORE accessing a mutable cart.
    let prep = match prepare_order(&prep_txn, &req).await {
        Ok(prep) => {
            prep_txn
                .commit()
                .await
                .map_err(|e| Status::internal(e.to_string()))?;
            prep
        }

        Err(e) => {
            prep_txn.rollback().await.ok();
            lease.state(db, "preparing", "retryable", None).await?;
            return Err(e);
        }
    };
    // Eligibility is mandatory even when shipping PRICE is zero.
    let quote = match super::estimate_checkout_shipping::eligible_shipping_quote(
        &prep.delivery_postcode,
        prep.items_total_minor_after_discount,
        prep.total_units,
    )
    .await
    {
        Ok(q) => q,
        Err(e) => {
            lease.state(db, "preparing", "retryable", None).await?;
            return Err(e);
        }
    };
    let shipping_minor = if prep.qualifies_free_shipping {
        0
    } else {
        quote.shipping_amount_minor.max(0)
    };
    let grand_total = paise_checked_add(prep.items_total_minor_after_discount, shipping_minor)
        .map_err(|e| Status::internal(e.to_string()))?;
    lease.total(db, grand_total).await?;
    let gateway = if prep.is_cod_checkout {
        lease.state(db, "preparing", "ready", None).await?;
        None
    } else {
        // Fence and persist BEFORE the outbound write. Crash/timeout here is never
        // a retryable new purchase: receipt + key remain available to reconciliation.
        lease.state(db, "preparing", "submitting", None).await?;
        match crate::razorpay::create_order(grand_total, "INR", &lease.receipt).await {
            Ok(id) => {
                lease.state(db, "submitting", "ready", Some(&id)).await?;
                Some(id)
            }
            Err(_) => {
                lease.state(db, "submitting", "needs_review", None).await?;
                return Err(Status::unavailable("Payment order outcome unresolved; reconciliation required. Do not place another order."));
            }
        }
    };
    let mut attempt = 0;
    loop {
        let txn = db
            .begin()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        let result = async {
            let response = place_order_write(
                &txn,
                &req,
                &prep,
                Some(quote.clone()),
                grand_total,
                shipping_minor,
                gateway.clone(),
            )
            .await?;
            lease
                .complete(&txn, response.get_ref().items[0].order_id)
                .await?;
            Ok::<_, Status>(response)
        }
        .await;
        match result {
            Ok(response) => {
                // An uncertain commit must never release the claim for a second order.
                txn.commit().await.map_err(|_| {
                    Status::unavailable("Checkout commit outcome unknown; retry the same key")
                })?;
                return Ok(response);
            }
            Err(e) => {
                txn.rollback().await.ok();
                if attempt < crate::DEADLOCK_MAX_RETRIES && is_deadlock_status(&e) {
                    attempt += 1;
                    continue;
                }
                lease.state(db, "ready", "needs_review", None).await?;
                return Err(e);
            }
        }
    }
}
async fn prepare_order(
    txn: &DatabaseTransaction,
    req: &PlaceOrderRequest,
) -> Result<PlaceOrderPrep, Status> {
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

    // Only fresh claims reach mutable cart/pricing reads. Replay was resolved in MySQL.
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

    let variant_ids: Vec<i64> = cart_items.iter().map(|item| item.variant_id).collect();

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

    Ok(PlaceOrderPrep {
        normalized_payment_mode,
        is_cod_checkout,
        cart_items,
        frozen_lines,
        gross_paise,
        applied_discount_total_minor,
        items_total_minor_after_discount,
        total_units,
        coupon_snapshot,
        delivery_postcode,
        qualifies_free_shipping,
    })
}

#[allow(clippy::too_many_arguments)]
async fn place_order_write(
    txn: &DatabaseTransaction,
    req: &PlaceOrderRequest,
    prep: &PlaceOrderPrep,
    shipping_quote: Option<ShiprocketCourierQuote>,
    grand_total_paise: i64,
    shipping_minor: i64,
    prepaid_razorpay_order_id: Option<String>,
) -> Result<Response<OrdersResponse>, Status> {
    let pending_status_id = order_state_machine::get_status_id(txn, "active_sale")
        .await
        .map_err(|e| Status::internal(e.to_string()))?
        .or(order_state_machine::get_status_id(txn, "pending")
            .await
            .map_err(|e| Status::internal(e.to_string()))?)
        .ok_or_else(|| Status::internal("OrderStatus 'active_sale' not found"))?;

    let create_order_result = create_order(
        txn,
        Request::new(CreateOrderRequest {
            shipping_address_id: req.shipping_address_id,
            status_id: pending_status_id,
            user_id: req.user_id,
            total_amount_paise: grand_total_paise,
            subtotal_minor: Some(prep.gross_paise),
            shipping_minor: Some(shipping_minor),
            tax_total_minor: Some(0),
            discount_total_minor: Some(prep.applied_discount_total_minor),
            grand_total_minor: Some(grand_total_paise),
            applied_coupon_id: prep.coupon_snapshot.as_ref().map(|s| s.0),
            applied_coupon_code: prep.coupon_snapshot.as_ref().map(|s| s.1.clone()),
            applied_discount_paise: prep.coupon_snapshot.as_ref().map(|s| s.2 as i32),
            payment_method: prep.normalized_payment_mode.clone(),
        }),
    )
    .await?
    .into_inner()
    .items
    .first()
    .cloned()
    .ok_or_else(|| Status::internal("create_order returned no order"))?;

    info!(
        order_id = create_order_result.order_id,
        public_order_ref = %create_order_result.public_order_ref,
        user_id = create_order_result.user_id,
        payment_mode = %prep.normalized_payment_mode,
        "place_order created order"
    );

    let order_created_at = orders::Entity::find_by_id(create_order_result.order_id)
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
            prep.normalized_payment_mode.clone().into(),
            cancel_window_ends_at.into(),
            earliest_booking_at.into(),
            pickup_target_at.into(),
            create_order_result.order_id.into(),
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
            prep.gross_paise.into(),
            prep.items_total_minor_after_discount.into(),
            shipping_minor.into(),
            create_order_result.order_id.into(),
        ],
    ))
    .await
    .map_err(|e| Status::internal(e.to_string()))?;

    let mut order_details: Vec<CreateOrderDetailRequest> = Vec::new();

    for line in &prep.frozen_lines {
        let unit_price_minor = i32::try_from(line.unit_price_minor)
            .map_err(|_| Status::internal("unit_price_minor overflow"))?;
        let line_discount_minor = i32::try_from(line.discount_minor)
            .map_err(|_| Status::internal("line discount overflow"))?;
        order_details.push(CreateOrderDetailRequest {
            order_id: create_order_result.order_id,
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

    if created_order_details.len() != prep.frozen_lines.len() {
        return Err(Status::internal(format!(
            "OrderDetails insert mismatch: expected {}, inserted {}",
            prep.frozen_lines.len(),
            created_order_details.len()
        )));
    }

    // Reserve inventory only after order + order details were fully persisted in this write
    // transaction. Any later failure will roll this reservation back.
    //
    // Rows are locked in sorted VariantID order (not HashMap iteration order, which is
    // unspecified) so two concurrent place_order calls whose carts share overlapping
    // variants always acquire the FOR UPDATE locks in the same order, avoiding an InnoDB
    // deadlock (error 1213) between them.
    let mut variant_quantity_map: HashMap<i64, i64> = HashMap::new();
    for item in &prep.cart_items {
        if item.quantity <= 0 {
            return Err(Status::invalid_argument("Cart quantity must be positive"));
        }
        let total = variant_quantity_map.entry(item.variant_id).or_insert(0);
        *total = total
            .checked_add(item.quantity)
            .ok_or_else(|| Status::invalid_argument("Cart quantity overflow"))?;
    }
    let mut variant_ids_sorted: Vec<i64> = variant_quantity_map.keys().copied().collect();
    variant_ids_sorted.sort_unstable();
    for variant_id in variant_ids_sorted {
        let qty = variant_quantity_map[&variant_id];
        let quantity_available =
            lock_inventory_row_and_get_available_quantity(txn, variant_id).await?;
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

    let selected_snapshot_condition =
        prep.cart_items
            .iter()
            .fold(Condition::any(), |condition, item| {
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
    if delete_result.rows_affected != prep.cart_items.len() as u64 {
        return Err(Status::internal(
            "Selected cart items changed during checkout; please refresh your cart and try again",
        ));
    }

    if !prep.is_cod_checkout {
        // Persist the payment intent using the Razorpay order id obtained in phase 2 —
        // this is a DB-only write here, no network call, so it doesn't reintroduce holding
        // the inventory locks (or a connection) during a Razorpay round-trip.
        let amount_paise = grand_total_paise;
        let _payment_intent = create_payment_intent(
            txn,
            tonic::Request::new(CreatePaymentIntentRequest {
                order_id: create_order_result.order_id,
                user_id: req.user_id,
                amount_paise,
                currency: Some("INR".to_string()),
                razorpay_order_id: prepaid_razorpay_order_id,
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
        // Route through the order state machine (rather than a raw StatusID UPDATE) so
        // COD orders get the same validation, order_event, and PAYMENT_CAPTURED outbox
        // dispatch as online payments — the outbox event is what drives the confirmation
        // email; a raw UPDATE bypassed it, so COD customers never received one.
        order_state_machine::transition_order_status(
            txn,
            create_order_result.order_id,
            order_state_machine::OrderState::Paid,
            "cod_order_confirmed",
            "customer",
            Some("COD order accepted and awaiting fulfillment"),
            Some(PaymentStatus::Captured),
        )
        .await?;

        let _ = crate::handlers::invoices::ensure_invoice_for_order(
            txn,
            create_order_result.order_id,
            "cod_confirmed",
        )
        .await?;

        // Record the COD acceptance as a real transaction — COD orders never get a
        // payment_intents row (that table is Razorpay-gateway-specific), so this is the
        // only automatic ledger entry a COD order ever gets. Safe to insert unconditionally
        // here: this branch only runs once, on the order's initial creation (a replayed
        // idempotency-key request returns the stored response before reaching this code).
        let _ = create_transaction(
            txn,
            tonic::Request::new(CreateTransactionRequest {
                user_id: req.user_id,
                amount_paise: grand_total_paise,
                r#type: "cod_payment".to_string(),
            }),
        )
        .await?;
    }

    // Emit audit event: order placed
    let _ = create_order_event(
        txn,
        tonic::Request::new(CreateOrderEventRequest {
            order_id: create_order_result.order_id,
            event_type: "order_placed".to_string(),
            from_status: None,
            to_status: Some("processing".to_string()),
            actor_type: "customer".to_string(),
            message: Some(format!(
                "Order {} placed successfully{}",
                create_order_result.order_id,
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

    // The caller commits the replay reference with this order and stock mutation.
    let persisted_order = orders::Entity::find_by_id(create_order_result.order_id)
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
