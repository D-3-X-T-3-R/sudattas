use crate::handlers::cart::delete_cart_item;

use crate::handlers::{
    cart::get_cart_items, order_details::create_order_details, orders::create_order,
    products::get_products_by_id,
};

use proto::proto::core::{
    CreateOrderDetailRequest, CreateOrderDetailsRequest, CreateOrderRequest, DeleteCartItemRequest,
    GetCartItemsRequest, GetProductsByIdRequest, OrdersResponse, PlaceOrderRequest,
};

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
        get_products_by_id(txn, Request::new(GetProductsByIdRequest { product_ids }))
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

    let _ = delete_cart_item(
        txn,
        Request::new(DeleteCartItemRequest {
            user_id: req.user_id,
            cart_id: None,
        }),
    )
    .await?
    .into_inner()
    .items;

    Ok(Response::new(OrdersResponse {
        items: vec![create_order],
    }))
}
