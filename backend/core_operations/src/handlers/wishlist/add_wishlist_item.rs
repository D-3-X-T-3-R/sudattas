use crate::handlers::db_errors::map_db_error_to_status;
use chrono::Utc;
use core_db_entities::entity::wishlist;
use proto::proto::core::{AddWishlistItemRequest, WishlistItemResponse, WishlistItemsResponse};

use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn add_wishlist_item(
    txn: &DatabaseTransaction,
    request: Request<AddWishlistItemRequest>,
) -> Result<Response<WishlistItemsResponse>, Status> {
    let req = request.into_inner();
    let product = wishlist::ActiveModel {
        wishlist_id: ActiveValue::NotSet,
        user_id: ActiveValue::Set(Some(req.user_id)),
        product_id: ActiveValue::Set(Some(req.product_id)),
        date_added: ActiveValue::Set(Utc::now()),
    };
    match product.insert(txn).await {
        Ok(model) => {
            let response = WishlistItemsResponse {
                items: vec![WishlistItemResponse {
                    wishlist_id: model.wishlist_id,
                    product_id: model.product_id.unwrap(),
                    user_id: model.user_id.unwrap(),
                    date_added: model.date_added.to_string(),
                }],
            };
            Ok(Response::new(response))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
