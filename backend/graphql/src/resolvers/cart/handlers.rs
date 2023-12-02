use chrono::{DateTime, Utc};
use proto::proto::core::CreateCartItemRequest;

use tracing::instrument;

use super::schema::{Cart, NewCart};
use crate::resolvers::error::GqlError;

#[instrument]
pub(crate) async fn add_cart_item(cart_item: NewCart) -> Result<Vec<Cart>, GqlError> {
    let mut client = proto::proto::core::grpc_services_client::GrpcServicesClient::connect(
        "grpc://localhost:50051",
    )
    .await?;
    let response = client
        .create_cart_item(CreateCartItemRequest {
            product_id: cart_item.product_id.parse::<i64>().expect("err"),
            quantity: cart_item.quantity.parse::<i64>().expect("err"),
            user_id: cart_item.user_id.parse::<i64>().expect("err"),
        })
        .await?;

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
