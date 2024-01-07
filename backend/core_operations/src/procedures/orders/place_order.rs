use crate::handlers::cart::delete_cart_item;
use crate::handlers::db_errors::map_db_error_to_status;
use crate::handlers::{
    cart::get_cart_items, order_details::create_order_details, orders::create_order,
    products::get_products_by_id,
};
use chrono::Utc;
use core_db_entities::entity::orders;
use proto::proto::core::{
    CreateOrderDetailRequest, CreateOrderDetailsRequest, CreateOrderRequest, DeleteCartItemRequest,
    GetCartItemsRequest, GetProductsByIdRequest, OrdersResponse, PlaceOrderRequest,
};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection};
use std::collections::HashMap;
use tonic::{Request, Response, Status};

pub async fn place_order(
    db: &DatabaseConnection,
    request: Request<PlaceOrderRequest>,
) -> Result<Response<OrdersResponse>, Status> {
    let req = request.into_inner();

    let cart_items = get_cart_items(
        db,
        Request::new(GetCartItemsRequest {
            user_id: req.user_id,
        }),
    )
    .await?
    .into_inner()
    .items;

    let (product_quantity_map, product_ids): (HashMap<i64, i64>, Vec<i64>) = cart_items
        .iter()
        .map(|item| ((item.product_id, item.quantity), item.product_id))
        .unzip();

    let order_products =
        get_products_by_id(db, Request::new(GetProductsByIdRequest { product_ids }))
            .await?
            .into_inner()
            .items;

    let total_amount = order_products
        .iter()
        .filter_map(|product| {
            product_quantity_map
                .get(&product.product_id)
                .map(|&quantity| product.price * quantity as f64)
        })
        .sum::<f64>();

    let create_order = create_order(
        db,
        Request::new(CreateOrderRequest {
            shipping_address: req.shipping_address,
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
        db,
        Request::new(CreateOrderDetailsRequest { order_details }),
    )
    .await?
    .into_inner()
    .items;

    let _ = delete_cart_item(
        db,
        Request::new(DeleteCartItemRequest {
            user_id: req.user_id,
        }),
    )
    .await?
    .into_inner()
    .items;

    Ok(Response::new(OrdersResponse {
        items: vec![create_order],
    }))
}
