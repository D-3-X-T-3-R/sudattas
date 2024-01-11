use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::wishlist;
use proto::proto::core::{DeleteWishlistItemRequest, WishlistItemResponse, WishlistItemsResponse};
use rust_decimal::{prelude::ToPrimitive, Decimal};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn delete_wishlist_item(
    txn: &DatabaseTransaction,
    request: Request<DeleteWishlistItemRequest>,
) -> Result<Response<WishlistItemsResponse>, Status> {
    let req = request.into_inner();

    let wishlist_item = wishlist::Entity::find_by_id(req.wishlist_id).one(txn).await;

    match wishlist_item {
        Ok(Some(model)) => {
            match wishlist::Entity::delete_many()
                .filter(wishlist::Column::WishlistId.eq(req.wishlist_id))
                .filter(wishlist::Column::UserId.eq(req.user_id))
                .exec(txn)
                .await
            {
                Ok(delete_result) => {
                    if delete_result.rows_affected > 0 {
                        let response = WishlistItemsResponse {
                            items: vec![WishlistItemResponse {
                                wishlist_id: model.wishlist_id,
                                product_id: model.product_id.unwrap(),
                                user_id: model.user_id.unwrap(),
                                date_added: model.date_added.to_string(),
                            }],
                        };
                        Ok(Response::new(response))
                    } else {
                        Err(Status::not_found(format!(
                            "Wishlist item with ID {} not found.",
                            req.wishlist_id
                        )))
                    }
                }
                Err(e) => Err(map_db_error_to_status(e)),
            }
        }
        Ok(None) => Err(Status::not_found(format!(
            "Wishlist item with ID {} not found.",
            req.wishlist_id
        ))),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
