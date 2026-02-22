use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::cart;
use proto::proto::core::{CartItemResponse, CartItemsResponse, UpdateCartItemRequest};
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn update_cart_item(
    txn: &DatabaseTransaction,
    request: Request<UpdateCartItemRequest>,
) -> Result<Response<CartItemsResponse>, Status> {
    let req = request.into_inner();

    let owner_filter = match (req.user_id, req.session_id.as_deref()) {
        (Some(uid), _) => cart::Column::UserId.eq(uid),
        (_, Some(sid)) => cart::Column::SessionId.eq(sid),
        (None, None) => {
            return Err(Status::invalid_argument(
                "Either user_id or session_id must be set",
            ));
        }
    };

    let existing = cart::Entity::find_by_id(req.cart_id)
        .filter(owner_filter)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?;

    let Some(existing) = existing else {
        return Err(Status::not_found(format!("Cart item {} not found", req.cart_id)));
    };

    let mut model: cart::ActiveModel = existing.into();
    model.product_id = ActiveValue::Set(req.product_id);
    model.quantity = ActiveValue::Set(req.quantity);

    match model.update(txn).await {
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
