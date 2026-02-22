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

    let owner_filter = match (req.user_id, req.session_id.as_deref()) {
        (Some(uid), _) => cart::Column::UserId.eq(uid),
        (_, Some(sid)) => cart::Column::SessionId.eq(sid),
        (None, None) => {
            return Err(Status::invalid_argument(
                "Either user_id or session_id must be set",
            ));
        }
    };

    let items_before = cart::Entity::find()
        .filter(owner_filter.clone())
        .all(txn)
        .await
        .map_err(map_db_error_to_status)?;

    let mut query = cart::Entity::delete_many().filter(owner_filter);
    if let Some(cart_id) = req.cart_id {
        query = query.filter(cart::Column::CartId.eq(cart_id));
    }

    let result = query.exec(txn).await.map_err(map_db_error_to_status)?;

    if result.rows_affected > 0 {
        let remaining: Vec<CartItemResponse> = if let Some(cid) = req.cart_id {
            items_before
                .into_iter()
                .filter(|m| m.cart_id != cid)
                .map(|model| CartItemResponse {
                    cart_id: model.cart_id,
                    product_id: model.product_id,
                    quantity: model.quantity,
                    user_id: model.user_id.unwrap_or(0),
                })
                .collect()
        } else {
            vec![]
        };
        Ok(Response::new(CartItemsResponse { items: remaining }))
    } else {
        Err(Status::not_found("Cart item(s) not found"))
    }
}
