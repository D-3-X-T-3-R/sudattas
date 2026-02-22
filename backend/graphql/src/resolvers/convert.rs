//! Shared conversion from gRPC (proto) response types to GraphQL schema types.
//! Use these in resolvers to avoid repeating the same mapping logic.

use proto::proto::core::{CartItemResponse, OrderResponse, ProductResponse};

use crate::resolvers::{
    cart::schema::Cart,
    orders::schema::Order,
    product::schema::Product,
};

pub fn product_response_to_gql(p: ProductResponse) -> Product {
    Product {
        product_id: p.product_id.to_string(),
        name: p.name,
        description: p.description,
        price: p.price.to_string(),
        stock_quantity: p.stock_quantity.map(|v| v.to_string()),
        category_id: p.category_id.map(|v| v.to_string()),
    }
}

pub fn order_response_to_gql(o: OrderResponse) -> Order {
    Order {
        order_id: o.order_id.to_string(),
        user_id: o.user_id.to_string(),
        order_date: o.order_date,
        shipping_address_id: o.shipping_address_id.to_string(),
        total_amount: o.total_amount.to_string(),
        status_id: o.status_id.to_string(),
    }
}

pub fn cart_item_response_to_gql(c: CartItemResponse) -> Cart {
    Cart {
        cart_id: c.cart_id.to_string(),
        user_id: c.user_id.to_string(),
        product_id: c.product_id.to_string(),
        quantity: c.quantity.to_string(),
    }
}
