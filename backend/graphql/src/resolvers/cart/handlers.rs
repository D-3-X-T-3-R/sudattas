use proto::proto::core::{
    CreateCartItemRequest, DeleteCartItemRequest, GetCartItemsRequest, UpdateCartItemRequest,
};

use tracing::instrument;

use super::schema::{Cart, CartMutation, NewCart};
use crate::resolvers::{
    error::{Code, GqlError},
    utils::connect_grpc_client,
};

#[instrument]
pub(crate) async fn add_cart_item(cart_item: NewCart) -> Result<Vec<Cart>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let product_id = cart_item
        .product_id
        .parse::<i64>()
        .map_err(|_| GqlError::new("Failed to parse product id", Code::InvalidArgument))?;
    let quantity = cart_item
        .quantity
        .parse::<i64>()
        .map_err(|_| GqlError::new("Failed to parse quantity", Code::InvalidArgument))?;
    let user_id = cart_item
        .user_id
        .parse::<i64>()
        .map_err(|_| GqlError::new("Failed to parse user id", Code::InvalidArgument))?;

    let response = client
        .create_cart_item(CreateCartItemRequest {
            product_id,
            quantity,
            user_id,
        })
        .await
        .map_err(|e| GqlError::new(&format!("gRPC request failed: {}", e), Code::Internal))?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(|cart| Cart {
            cart_id: cart.cart_id.to_string(),
            user_id: cart.user_id.to_string(),
            product_id: cart.product_id.to_string(),
            quantity: cart.quantity.to_string(),
        })
        .collect())
}

pub(crate) async fn get_cart_items(user_id: String) -> Result<Vec<Cart>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let user_id = user_id
        .parse::<i64>()
        .map_err(|_| GqlError::new("Failed to parse user id", Code::InvalidArgument))?;

    let response = client
        .get_cart_items(GetCartItemsRequest { user_id })
        .await
        .map_err(|e| GqlError::new(&format!("gRPC request failed: {}", e), Code::Internal))?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(|cart| Cart {
            cart_id: cart.cart_id.to_string(),
            user_id: cart.user_id.to_string(),
            product_id: cart.product_id.to_string(),
            quantity: cart.quantity.to_string(),
        })
        .collect())
}

pub(crate) async fn delete_cart_item(user_id: String) -> Result<Vec<Cart>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let user_id = user_id
        .parse::<i64>()
        .map_err(|_| GqlError::new("Failed to parse cart id", Code::InvalidArgument))?;

    let response = client
        .delete_cart_item(DeleteCartItemRequest { user_id })
        .await
        .map_err(|e| GqlError::new(&format!("gRPC request failed: {}", e), Code::Internal))?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(|cart| Cart {
            cart_id: cart.cart_id.to_string(),
            user_id: cart.user_id.to_string(),
            product_id: cart.product_id.to_string(),
            quantity: cart.quantity.to_string(),
        })
        .collect())
}

#[instrument]
pub(crate) async fn update_cart_item(cart_item: CartMutation) -> Result<Vec<Cart>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let cart_id = cart_item
        .cart_id
        .parse::<i64>()
        .map_err(|_| GqlError::new("Failed to parse cart id", Code::InvalidArgument))?;
    let product_id = cart_item
        .product_id
        .parse::<i64>()
        .map_err(|_| GqlError::new("Failed to parse product id", Code::InvalidArgument))?;
    let quantity = cart_item
        .quantity
        .parse::<i64>()
        .map_err(|_| GqlError::new("Failed to parse quantity", Code::InvalidArgument))?;
    let user_id = cart_item
        .user_id
        .parse::<i64>()
        .map_err(|_| GqlError::new("Failed to parse user id", Code::InvalidArgument))?;

    let response = client
        .update_cart_item(UpdateCartItemRequest {
            cart_id,
            product_id,
            quantity,
            user_id,
        })
        .await
        .map_err(|e| GqlError::new(&format!("gRPC request failed: {}", e), Code::Internal))?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(|cart| Cart {
            cart_id: cart.cart_id.to_string(),
            user_id: cart.user_id.to_string(),
            product_id: cart.product_id.to_string(),
            quantity: cart.quantity.to_string(),
        })
        .collect())
}
