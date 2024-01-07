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

    let cart_item = cart::Entity::find()
        .filter(cart::Column::UserId.eq(req.user_id))
        .all(db)
        .await;

    match cart_item {
        Ok(item) => {
            match cart::Entity::delete_many()
                .filter(cart::Column::UserId.eq(req.user_id))
                .exec(db)
                .await
            {
                Ok(delete_result) => {
                    if delete_result.rows_affected > 0 {
                        let response = CartItemsResponse {
                            items: item
                                .into_iter()
                                .map(|model| CartItemResponse {
                                    cart_id: model.cart_id,
                                    product_id: model.product_id,
                                    quantity: model.quantity,
                                    user_id: model.user_id,
                                })
                                .collect(),
                        };
                        Ok(Response::new(response))
                    } else {
                        Err(Status::not_found(format!(
                            "Cart item with ID {} not found.",
                            req.user_id
                        )))
                    }
                }
                Err(e) => Err(map_db_error_to_status(e)),
            }
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
