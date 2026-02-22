use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::cart;
use proto::proto::core::{CartItemResponse, CartItemsResponse, GetCartItemsRequest};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn get_cart_items(
    txn: &DatabaseTransaction,
    request: Request<GetCartItemsRequest>,
) -> Result<Response<CartItemsResponse>, Status> {
    let req = request.into_inner();

    let filter = match (req.user_id, req.session_id.as_deref()) {
        (Some(uid), _) => cart::Column::UserId.eq(uid),
        (_, Some(sid)) => cart::Column::SessionId.eq(sid),
        (None, None) => {
            return Err(Status::invalid_argument(
                "Either user_id or session_id must be set",
            ));
        }
    };

    match cart::Entity::find()
        .filter(filter)
        .all(txn)
        .await
    {
        Ok(cart_models) => {
            let items = cart_models
                .into_iter()
                .map(|model| CartItemResponse {
                    cart_id: model.cart_id,
                    user_id: model.user_id.unwrap_or(0),
                    product_id: model.product_id,
                    quantity: model.quantity,
                })
                .collect();

            let response = CartItemsResponse { items };
            Ok(Response::new(response))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
