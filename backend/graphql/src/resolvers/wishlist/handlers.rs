use proto::proto::core::{
    AddWishlistItemRequest, DeleteWishlistItemRequest, SearchWishlistItemRequest,
};

use tracing::instrument;

use super::schema::{DeleteWishlistItem, NewWishlistItem, SearchWishlistItem, WishlistItem};
use crate::resolvers::{
    convert,
    error::GqlError,
    utils::{connect_grpc_client, to_i64, to_option_i64},
};

#[instrument]
pub(crate) async fn add_wishlist_item(
    wishlist: NewWishlistItem,
) -> Result<Vec<WishlistItem>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let response = client
        .add_wishlist_item(AddWishlistItemRequest {
            user_id: to_i64(wishlist.user_id),
            product_id: to_i64(wishlist.product_id),
        })
        .await?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(convert::wishlist_item_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn search_wishlist_item(
    search: SearchWishlistItem,
) -> Result<Vec<WishlistItem>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let response = client
        .search_wishlist_item(SearchWishlistItemRequest {
            wishlist_id: to_option_i64(search.wishlist_id),
            user_id: to_i64(search.user_id),
            product_id: to_option_i64(search.product_id),
        })
        .await?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(convert::wishlist_item_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn delete_wishlist_item(
    delete: DeleteWishlistItem,
) -> Result<Vec<WishlistItem>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let response = client
        .delete_wishlist_item(DeleteWishlistItemRequest {
            wishlist_id: to_i64(delete.wishlist_id),
            user_id: to_i64(delete.user_id),
        })
        .await?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(convert::wishlist_item_response_to_gql)
        .collect())
}
