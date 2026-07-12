use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::cart;
use proto::proto::core::{CartItemResponse, CartItemsResponse, MergeCartRequest};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseTransaction, EntityTrait, ModelTrait,
    QueryFilter,
};
use tonic::{Request, Response, Status};

/// Merges a guest (session-scoped) cart into an authenticated user's cart —
/// e.g. called by the frontend right after login, passing the guest session id
/// that was in use beforehand. For each guest row: if the user already has that
/// variant in their cart, quantities are summed and the guest row is dropped;
/// otherwise the guest row is reassigned to the user. Returns the user's cart
/// after merging.
pub async fn merge_cart(
    txn: &DatabaseTransaction,
    request: Request<MergeCartRequest>,
) -> Result<Response<CartItemsResponse>, Status> {
    let req = request.into_inner();
    let session_id = req.session_id.trim();
    if session_id.is_empty() {
        return Err(Status::invalid_argument("session_id must not be empty"));
    }

    let guest_rows = cart::Entity::find()
        .filter(cart::Column::SessionId.eq(session_id))
        .all(txn)
        .await
        .map_err(map_db_error_to_status)?;

    for guest_row in guest_rows {
        let existing_user_row = cart::Entity::find()
            .filter(cart::Column::UserId.eq(req.user_id))
            .filter(cart::Column::VariantId.eq(guest_row.variant_id))
            .one(txn)
            .await
            .map_err(map_db_error_to_status)?;

        match existing_user_row {
            Some(existing) => {
                // Variant already in the user's cart: sum quantities into the
                // existing row and drop the now-redundant guest row.
                let mut model: cart::ActiveModel = existing.clone().into();
                model.quantity = ActiveValue::Set(existing.quantity + guest_row.quantity);
                model.update(txn).await.map_err(map_db_error_to_status)?;
                guest_row
                    .delete(txn)
                    .await
                    .map_err(map_db_error_to_status)?;
            }
            None => {
                // New variant for the user: reassign the guest row instead of
                // inserting a new one, clearing session_id so it's user-only.
                let mut model: cart::ActiveModel = guest_row.into();
                model.user_id = ActiveValue::Set(Some(req.user_id));
                model.session_id = ActiveValue::Set(None);
                model.update(txn).await.map_err(map_db_error_to_status)?;
            }
        }
    }

    let merged = cart::Entity::find()
        .filter(cart::Column::UserId.eq(req.user_id))
        .all(txn)
        .await
        .map_err(map_db_error_to_status)?;

    let items = merged
        .into_iter()
        .map(|model| CartItemResponse {
            cart_id: model.cart_id,
            user_id: model.user_id.unwrap_or(0),
            variant_id: model.variant_id,
            quantity: model.quantity,
        })
        .collect();
    Ok(Response::new(CartItemsResponse { items }))
}
