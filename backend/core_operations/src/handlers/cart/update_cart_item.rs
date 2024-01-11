use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::cart;
use proto::proto::core::{CartItemResponse, CartItemsResponse, UpdateCartItemRequest};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn update_cart_item(
    txn: &DatabaseTransaction,
    request: Request<UpdateCartItemRequest>,
) -> Result<Response<CartItemsResponse>, Status> {
    let req = request.into_inner();

    let cart = cart::ActiveModel {
        cart_id: ActiveValue::Set(req.cart_id),
        user_id: ActiveValue::Set(req.user_id),
        product_id: ActiveValue::Set(req.product_id),
        quantity: ActiveValue::Set(req.quantity),
    };
    match cart.update(txn).await {
        Ok(cart_model) => {
            let response = CartItemsResponse {
                items: vec![CartItemResponse {
                    cart_id: cart_model.cart_id,
                    product_id: cart_model.product_id,
                    quantity: cart_model.quantity,
                    user_id: cart_model.user_id,
                }],
            };
            Ok(Response::new(response))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
