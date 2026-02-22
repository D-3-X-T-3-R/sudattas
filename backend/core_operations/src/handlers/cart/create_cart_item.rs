use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::cart;
use proto::proto::core::{CartItemResponse, CartItemsResponse, CreateCartItemRequest};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_cart_item(
    txn: &DatabaseTransaction,
    request: Request<CreateCartItemRequest>,
) -> Result<Response<CartItemsResponse>, Status> {
    let req = request.into_inner();

    let (user_id, session_id) = match (req.user_id, req.session_id) {
        (Some(uid), sid) => (ActiveValue::Set(Some(uid)), ActiveValue::Set(sid)),
        (None, Some(sid)) => (ActiveValue::Set(None), ActiveValue::Set(Some(sid))),
        (None, None) => {
            return Err(Status::invalid_argument(
                "Either user_id or session_id must be set",
            ));
        }
    };

    let cart = cart::ActiveModel {
        cart_id: ActiveValue::NotSet,
        user_id,
        session_id,
        product_id: ActiveValue::Set(req.product_id),
        quantity: ActiveValue::Set(req.quantity),
        created_at: ActiveValue::NotSet,
        updated_at: ActiveValue::NotSet,
    };
    match cart.insert(txn).await {
        Ok(cart_model) => {
            let response = CartItemsResponse {
                items: vec![CartItemResponse {
                    cart_id: cart_model.cart_id,
                    product_id: cart_model.product_id,
                    quantity: cart_model.quantity,
                    user_id: cart_model.user_id.unwrap_or(0),
                }],
            };
            Ok(Response::new(response))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
