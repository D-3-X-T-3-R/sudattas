use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::wishlist;
use proto::proto::core::{SearchWishlistItemRequest, WishlistItemResponse, WishlistItemsResponse};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryTrait};
use tonic::{Request, Response, Status};

pub async fn search_wishlist_item(
    txn: &DatabaseTransaction,
    request: Request<SearchWishlistItemRequest>,
) -> Result<Response<WishlistItemsResponse>, Status> {
    let req = request.into_inner();

    match wishlist::Entity::find()
        .filter(wishlist::Column::UserId.eq(req.user_id))
        .apply_if(req.wishlist_id, |query, v| {
            query.filter(wishlist::Column::WishlistId.eq(v))
        })
        .apply_if(req.product_id, |query, v| {
            query.filter(wishlist::Column::ProductId.eq(v))
        })
        .all(txn)
        .await
    {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|model| WishlistItemResponse {
                    wishlist_id: model.wishlist_id,
                    product_id: model.product_id.unwrap(),
                    user_id: model.user_id.unwrap(),
                    date_added: model.date_added.to_string(),
                })
                .collect();

            let response = WishlistItemsResponse { items };
            Ok(Response::new(response))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
