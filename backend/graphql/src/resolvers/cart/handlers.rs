use proto::proto::core::{
    CreateCartItemRequest, DeleteCartItemRequest, EnqueueAbandonedCartRequest,
    EnqueueAbandonedCartResponse, GetCartItemsRequest, UpdateCartItemRequest,
};

use tracing::instrument;

use super::schema::{Cart, CartMutation, DeleteCartItem, NewCart};
use crate::resolvers::{
    convert,
    error::{Code, GqlError},
    utils::{connect_grpc_client, parse_i64, to_i64, to_option_i64},
};
use crate::validation;

#[instrument]
pub(crate) async fn add_cart_item(cart_item: NewCart) -> Result<Vec<Cart>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let user_id = to_option_i64(Some(cart_item.user_id.clone()));
    let session_id = cart_item.session_id.clone();
    if user_id.is_none() && session_id.is_none() {
        return Err(GqlError::new(
            "Either user_id or session_id must be set for cart",
            Code::InvalidArgument,
        ));
    }
    let qty = to_i64(cart_item.quantity);
    validation::validate_quantity(qty, "quantity")?;
    let response = client
        .create_cart_item(CreateCartItemRequest {
            user_id,
            variant_id: to_i64(cart_item.variant_id),
            quantity: qty,
            session_id,
        })
        .await?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(convert::cart_item_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn enqueue_abandoned_cart(
    delay_hours: Option<String>,
) -> Result<EnqueueAbandonedCartResponse, GqlError> {
    let mut client = connect_grpc_client().await?;
    let delay_hours_parsed = match delay_hours {
        Some(ref s) => Some(parse_i64(s, "delay_hours")?),
        None => None,
    };
    let response = client
        .enqueue_abandoned_cart(EnqueueAbandonedCartRequest {
            delay_hours: delay_hours_parsed,
        })
        .await?;
    Ok(response.into_inner())
}

pub(crate) async fn get_cart_items(
    user_id: Option<String>,
    session_id: Option<String>,
) -> Result<Vec<Cart>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let uid = user_id.and_then(|s| s.parse().ok());
    if uid.is_none() && session_id.is_none() {
        return Err(GqlError::new(
            "Either user_id or session_id must be set for get_cart_items",
            Code::InvalidArgument,
        ));
    }
    let response = client
        .get_cart_items(GetCartItemsRequest {
            user_id: uid,
            session_id,
        })
        .await?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(convert::cart_item_response_to_gql)
        .collect())
}

pub(crate) async fn delete_cart_item(delete: DeleteCartItem) -> Result<Vec<Cart>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let user_id = to_option_i64(Some(delete.user_id.clone()));
    let session_id = delete.session_id.clone();
    if user_id.is_none() && session_id.is_none() {
        return Err(GqlError::new(
            "Either user_id or session_id must be set for delete_cart_item",
            Code::InvalidArgument,
        ));
    }
    let response = client
        .delete_cart_item(DeleteCartItemRequest {
            user_id,
            cart_id: to_option_i64(delete.cart_id),
            session_id,
        })
        .await?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(convert::cart_item_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn update_cart_item(cart_item: CartMutation) -> Result<Vec<Cart>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let cart_id = parse_i64(&cart_item.cart_id, "cart id")?;
    let variant_id = parse_i64(&cart_item.variant_id, "variant id")?;
    let quantity = parse_i64(&cart_item.quantity, "quantity")?;
    validation::validate_quantity(quantity, "quantity")?;
    let user_id = to_option_i64(Some(cart_item.user_id.clone()));
    let session_id = cart_item.session_id.clone();
    if user_id.is_none() && session_id.is_none() {
        return Err(GqlError::new(
            "Either user_id or session_id must be set for update_cart_item",
            Code::InvalidArgument,
        ));
    }
    let response = client
        .update_cart_item(UpdateCartItemRequest {
            cart_id,
            user_id,
            variant_id,
            quantity,
            session_id,
        })
        .await?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(convert::cart_item_response_to_gql)
        .collect())
}
