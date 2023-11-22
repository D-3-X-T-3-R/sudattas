use crate::db_errors::map_db_error_to_status;
use core_db_entities::entity::cart;
use proto::proto::core::{CartItemResponse, CartItemsResponse, CreateCartItemRequest};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait, QueryResult};
use tonic::{Request, Response, Status};

pub async fn create_cart_item(
    db: &DatabaseConnection,
    request: Request<CreateCartItemRequest>,
) -> Result<Response<CartItemsResponse>, Status> {
    let req = request.into_inner();

    let cart = cart::ActiveModel {
        cart_id: ActiveValue::NotSet,
        user_id: ActiveValue::Set(Some(req.user_id)),
        product_id: ActiveValue::Set(Some(req.product_id)),
        quantity: ActiveValue::Set(Some(req.quantity)),
    };

    match cart.insert(db).await {
        Ok(cart_model) => {
            let response = CartItemsResponse {
                items: vec![CartItemResponse {
                    cart_id: cart_model.cart_id,
                    product_id: cart_model.product_id.unwrap(),
                    quantity: cart_model.quantity.unwrap(),
                    user_id: cart_model.user_id.unwrap(),
                }],
            };
            Ok(Response::new(response))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
