//! Shared conversion from gRPC (proto) response types to GraphQL schema types.
//! Use these in resolvers instead of inline mapping to keep handlers concise.

use proto::proto::core::{
    CartItemResponse, CategoryResponse, CountryResponse, OrderDetailResponse, OrderResponse,
    ProductImageResponse, ProductResponse, StateResponse, WishlistItemResponse,
};

use crate::resolvers::{
    cart::schema::Cart, category::schema::Category, country::schema::Country,
    order_details::schema::OrderDetails, orders::schema::Order, product::schema::Product,
    product_images::schema::ProductImage, state::schema::State, wishlist::schema::WishlistItem,
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

pub fn category_response_to_gql(c: CategoryResponse) -> Category {
    Category {
        category_id: c.category_id.to_string(),
        name: c.name,
    }
}

pub fn country_response_to_gql(c: CountryResponse) -> Country {
    Country {
        country_id: c.country_id.to_string(),
        country_name: c.country_name,
    }
}

pub fn state_response_to_gql(s: StateResponse) -> State {
    State {
        state_id: s.state_id.to_string(),
        state_name: s.state_name,
    }
}

pub fn product_image_response_to_gql(p: ProductImageResponse) -> ProductImage {
    ProductImage {
        image_id: p.image_id.to_string(),
        product_id: p.product_id.to_string(),
        image_base64: p.image_base64,
        alt_text: p.alt_text,
        url: p.url,
        cdn_path: p.cdn_path,
        thumbnail_url: p.thumbnail_url,
    }
}

pub fn order_detail_response_to_gql(o: OrderDetailResponse) -> OrderDetails {
    OrderDetails {
        order_detail_id: o.order_detail_id.to_string(),
        order_id: o.order_id.to_string(),
        product_id: o.product_id.to_string(),
        quantity: o.quantity.to_string(),
        price: o.price.to_string(),
    }
}

pub fn wishlist_item_response_to_gql(w: WishlistItemResponse) -> WishlistItem {
    WishlistItem {
        wishlist_id: w.wishlist_id.to_string(),
        user_id: w.user_id.to_string(),
        product_id: w.product_id.to_string(),
        date_added: w.date_added,
    }
}
