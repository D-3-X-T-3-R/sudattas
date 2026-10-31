use crate::handlers::coupons::{
    eligibility::{check_coupon_scope, check_per_customer_limit, CartProduct},
    validate_coupon::check_coupon,
};
use crate::handlers::{cart::get_cart_items, products::get_products_by_id};
use crate::integrations::shiprocket::{best_courier_quote_for_checkout, ShiprocketError};
use crate::money::{paise_checked_add, paise_checked_mul};

use core_db_entities::entity::{product_variants, shipping_addresses};
use proto::proto::core::{
    EstimateCheckoutShippingRequest, EstimateCheckoutShippingResponse, GetCartItemsRequest,
    GetProductsByIdRequest,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, TransactionTrait};
use tonic::{Request, Response, Status};
use tracing::warn;

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
            "Cannot estimate checkout shipping: no selected cart items provided",
        ));
    }
    let mut seen = std::collections::HashSet::new();
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
        .collect::<std::collections::HashMap<_, _>>();

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

pub async fn estimate_checkout_shipping(
    db: &DatabaseConnection,
    request: Request<EstimateCheckoutShippingRequest>,
) -> Result<Response<EstimateCheckoutShippingResponse>, Status> {
    let req = request.into_inner();
    let selected_cart_ids = validate_selected_cart_ids(&req.selected_cart_ids)?;

    // This endpoint is read-only (no writes), so the DB transaction only needs to stay open
    // for the reads below; it is committed before the Shiprocket call further down so that
    // call (up to 90s) doesn't hold a pooled connection idle.
    let txn = db
        .begin()
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    let cart_items = get_cart_items(
        &txn,
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
        .filter(product_variants::Column::VariantId.is_in(variant_ids))
        .all(&txn)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;
    let product_ids: Vec<i64> = variants.iter().map(|v| v.product_id).collect();
    let products = get_products_by_id(&txn, Request::new(GetProductsByIdRequest { product_ids }))
        .await?
        .into_inner()
        .items;

    let variants_by_id: std::collections::HashMap<i64, product_variants::Model> =
        variants.into_iter().map(|v| (v.variant_id, v)).collect();
    let products_by_id: std::collections::HashMap<i64, proto::proto::core::ProductResponse> =
        products.into_iter().map(|p| (p.product_id, p)).collect();

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

    let (total_paise, note_coupon) = if let Some(ref code) = req.coupon_code {
        match check_coupon(&txn, code, gross_paise, false).await {
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
                let ok_per_customer = check_per_customer_limit(&txn, result.coupon_id, req.user_id)
                    .await
                    .unwrap_or(false);
                let ok_scope = check_coupon_scope(&txn, result.coupon_id, &cart_for_scope)
                    .await
                    .unwrap_or(false);
                if ok_per_customer && ok_scope {
                    (result.final_amount_paise, None)
                } else {
                    (
                        gross_paise,
                        Some("Coupon not applicable to this cart".to_string()),
                    )
                }
            }
            Ok(result) => (gross_paise, Some(result.reason)),
            Err(e) => {
                warn!("coupon check failed during shipping estimate: {}", e);
                (gross_paise, Some("Coupon check failed".to_string()))
            }
        }
    } else {
        (gross_paise, None)
    };

    let shipping_address = shipping_addresses::Entity::find_by_id(req.shipping_address_id)
        .one(&txn)
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
    let total_units: i64 = cart_items.iter().map(|item| item.quantity.max(1)).sum();

    // All reads for this estimate are done; release the connection before the Shiprocket
    // call below (up to 90s) rather than holding it idle for the round-trip.
    txn.commit()
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    let free_shipping_threshold_minor = crate::order_policy::free_shipping_threshold_minor();
    let qualifies_free_shipping = total_paise >= free_shipping_threshold_minor;
    let quote = if qualifies_free_shipping {
        None
    } else {
        Some(
            match best_courier_quote_for_checkout(
                delivery_postcode.as_str(),
                total_paise,
                total_units,
            )
            .await
            {
                Ok(Some(value)) => value,
                Ok(None) => {
                    warn!("checkout shipping estimate quote unavailable without courier result");
                    return Err(Status::unavailable(
                        "Live shipping quote is unavailable for this checkout",
                    ));
                }
                Err(error) => {
                    warn!("checkout shipping estimate quote failed: {}", error);
                    return Err(map_shipping_quote_error(error));
                }
            },
        )
    };

    let shipping_amount_paise = quote
        .as_ref()
        .map(|q| q.shipping_amount_minor.max(0))
        .unwrap_or(0);
    let order_total_paise = paise_checked_add(total_paise, shipping_amount_paise)
        .map_err(|e| Status::internal(format!("Overflow computing order total: {}", e)))?;

    let note = note_coupon;

    Ok(Response::new(EstimateCheckoutShippingResponse {
        shipping_amount_paise,
        courier_name: quote.as_ref().map(|q| q.courier_name.clone()),
        estimated_delivery_days: quote.as_ref().and_then(|q| q.estimated_delivery_days),
        item_subtotal_paise: total_paise,
        order_total_paise,
        quote_available: true,
        note,
    }))
}
