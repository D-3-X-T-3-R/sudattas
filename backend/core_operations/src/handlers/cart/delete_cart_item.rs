use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::cart;
use proto::proto::core::{CartItemResponse, CartItemsResponse, DeleteCartItemRequest};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn delete_cart_item(
    db: &DatabaseConnection,
    request: Request<DeleteCartItemRequest>,
) -> Result<Response<CartItemsResponse>, Status> {
    let req = request.into_inner();

    let cart_item = cart::Entity::find_by_id(req.cart_id).one(db).await;

    match cart_item {
        Ok(Some(item)) => {
            match cart::Entity::delete_many()
                .filter(cart::Column::CartId.eq(req.cart_id))
                .exec(db)
                .await
            {
                Ok(delete_result) => {
                    if delete_result.rows_affected > 0 {
                        let response = CartItemsResponse {
                            items: vec![CartItemResponse {
                                cart_id: item.cart_id,
                                product_id: item.product_id,
                                quantity: item.quantity,
                                user_id: item.user_id,
                            }],
                        };
                        Ok(Response::new(response))
                    } else {
                        Err(Status::not_found(format!(
                            "Cart item with ID {} not found.",
                            req.cart_id
                        )))
                    }
                }
                Err(e) => Err(map_db_error_to_status(e)),
            }
        }
        Ok(None) => Err(Status::not_found(format!(
            "Cart item with ID {} not found.",
            req.cart_id
        ))),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
