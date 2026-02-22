use crate::handlers::cart::delete_cart_item;
use crate::handlers::coupons::validate_coupon::check_coupon;
use crate::handlers::order_events::create_order_event;

use crate::handlers::{
    cart::get_cart_items, order_details::create_order_details, orders::create_order,
    payment_intents::create_payment_intent, products::get_products_by_id,
};

use core_db_entities::entity::inventory;
use proto::proto::core::{
    CreateOrderDetailRequest, CreateOrderDetailsRequest, CreateOrderEventRequest,
    CreateOrderRequest, CreatePaymentIntentRequest, DeleteCartItemRequest, GetCartItemsRequest,
    GetProductsByIdRequest, OrdersResponse, PlaceOrderRequest,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use sea_orm::DatabaseTransaction;
use std::collections::HashMap;
use tonic::{Request, Response, Status};

pub async fn place_order(
    txn: &DatabaseTransaction,
    request: Request<PlaceOrderRequest>,
) -> Result<Response<OrdersResponse>, Status> {
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

    let (product_quantity_map, product_ids): (HashMap<i64, i64>, Vec<i64>) = cart_items
        .iter()
        .map(|item| ((item.product_id, item.quantity), item.product_id))
        .unzip();

    // Stock check: reject the order if any item exceeds available inventory.
    for product_id in &product_ids {
        let inv = inventory::Entity::find()
            .filter(inventory::Column::ProductId.eq(*product_id))
            .one(txn)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let quantity_needed = *product_quantity_map.get(product_id).unwrap_or(&0);
        let quantity_available = inv.as_ref().and_then(|i| i.quantity_available).unwrap_or(0);

        if quantity_available < quantity_needed {
            return Err(Status::failed_precondition(format!(
                "Insufficient stock for product {}: {} requested, {} available",
                product_id, quantity_needed, quantity_available
            )));
        }
    }

    let order_products =
        get_products_by_id(txn, Request::new(GetProductsByIdRequest { product_ids }))
            .await?
            .into_inner()
            .items;

    let gross_amount = order_products
        .iter()
        .filter_map(|product| {
            product_quantity_map
                .get(&product.product_id)
                .map(|&quantity| product.price * quantity as f64)
        })
        .sum::<f64>();

    // Apply coupon if provided, deriving the discounted total.
    let total_amount = if let Some(ref code) = req.coupon_code {
        let gross_paise = (gross_amount * 100.0) as i64;
        match check_coupon(txn, code, gross_paise, true).await {
            Ok(result) if result.is_valid => result.final_amount_paise as f64 / 100.0,
            Ok(result) => {
                log::warn!("Coupon '{}' invalid at checkout: {}", code, result.reason);
                gross_amount
            }
            Err(e) => {
                log::warn!("Coupon check failed: {}", e);
                gross_amount
            }
        }
    } else {
        gross_amount
    };

    let create_order = create_order(
        txn,
        Request::new(CreateOrderRequest {
            shipping_address_id: req.shipping_address_id,
            status_id: 2, // Always start with order status is processing
            user_id: req.user_id,
            total_amount: total_amount,
        }),
    )
    .await?
    .into_inner()
    .items
    .first()
    .unwrap()
    .clone();

    let mut order_details: Vec<CreateOrderDetailRequest> = Vec::new();

    for product in order_products.iter() {
        order_details.push(CreateOrderDetailRequest {
            order_id: create_order.order_id,
            product_id: product.product_id,
            quantity: product_quantity_map
                .get(&product.product_id)
                .unwrap()
                .to_owned(),
            price: product.price,
        })
    }

    let _ = create_order_details(
        txn,
        Request::new(CreateOrderDetailsRequest { order_details }),
    )
    .await?
    .into_inner()
    .items;

    // Auto-create a pending payment intent for the new order.
    // razorpay_order_id must be obtained from Razorpay; here we generate a placeholder
    // that callers must replace via CreatePaymentIntent when they have a real Razorpay ID.
    let razorpay_order_id = format!("rzp_pending_{}", create_order.order_id);
    let amount_paise = (create_order.total_amount * 100.0) as i64;
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

    Ok(Response::new(OrdersResponse {
        items: vec![create_order],
    }))
}
