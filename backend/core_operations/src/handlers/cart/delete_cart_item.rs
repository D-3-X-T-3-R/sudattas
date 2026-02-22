use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::cart;
use proto::proto::core::{CartItemResponse, CartItemsResponse, DeleteCartItemRequest};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryTrait};
use tonic::{Request, Response, Status};

pub async fn delete_cart_item(
    txn: &DatabaseTransaction,
    request: Request<DeleteCartItemRequest>,
) -> Result<Response<CartItemsResponse>, Status> {
    let req = request.into_inner();

    let cart_item = cart::Entity::find()
        .filter(cart::Column::UserId.eq(req.user_id))
        .all(txn)
        .await;

    match cart_item {
        Ok(item) => {
            match cart::Entity::delete_many()
                .filter(cart::Column::UserId.eq(req.user_id))
                .apply_if(req.cart_id, |query, _| {
                    query.filter(cart::Column::CartId.eq(req.cart_id))
                })
                .exec(txn)
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
                                    user_id: model.user_id.unwrap_or(0),
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
