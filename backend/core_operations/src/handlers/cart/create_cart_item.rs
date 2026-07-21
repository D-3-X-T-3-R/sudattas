use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::cart;
use proto::proto::core::{CartItemResponse, CartItemsResponse, CreateCartItemRequest};
use sea_orm::{
    sea_query::SimpleExpr, ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseTransaction,
    EntityTrait, QueryFilter,
};
use tonic::{Request, Response, Status};

fn to_response(cart_model: cart::Model) -> Response<CartItemsResponse> {
    Response::new(CartItemsResponse {
        items: vec![CartItemResponse {
            cart_id: cart_model.cart_id,
            variant_id: cart_model.variant_id,
            quantity: cart_model.quantity,
            user_id: cart_model.user_id.unwrap_or(0),
        }],
    })
}

#[allow(clippy::result_large_err)]
fn owner_filter(user_id: Option<i64>, session_id: Option<&str>) -> Result<SimpleExpr, Status> {
    match (user_id, session_id) {
        (Some(uid), _) => Ok(cart::Column::UserId.eq(uid)),
        (_, Some(sid)) => Ok(cart::Column::SessionId.eq(sid)),
        (None, None) => Err(Status::invalid_argument(
            "Either user_id or session_id must be set",
        )),
    }
}

async fn increment_existing(
    txn: &DatabaseTransaction,
    existing: cart::Model,
    quantity_delta: i64,
) -> Result<Response<CartItemsResponse>, Status> {
    let mut model: cart::ActiveModel = existing.clone().into();
    model.quantity = ActiveValue::Set(existing.quantity + quantity_delta);
    model
        .update(txn)
        .await
        .map(to_response)
        .map_err(map_db_error_to_status)
}

pub async fn create_cart_item(
    txn: &DatabaseTransaction,
    request: Request<CreateCartItemRequest>,
) -> Result<Response<CartItemsResponse>, Status> {
    let req = request.into_inner();
    let session_id = req.session_id.clone();

    // Adding a variant already in the cart increments its quantity instead of
    // erroring on the (owner, variant) unique constraint — a repeat "Add to Cart"
    // click is a normal action, not a conflict.
    let existing = cart::Entity::find()
        .filter(owner_filter(req.user_id, session_id.as_deref())?)
        .filter(cart::Column::VariantId.eq(req.variant_id))
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?;

    if let Some(existing) = existing {
        return increment_existing(txn, existing, req.quantity).await;
    }

    let (user_id, session_id_value) = match (req.user_id, session_id.clone()) {
        (Some(uid), sid) => (ActiveValue::Set(Some(uid)), ActiveValue::Set(sid)),
        (None, Some(sid)) => (ActiveValue::Set(None), ActiveValue::Set(Some(sid))),
        (None, None) => {
            return Err(Status::invalid_argument(
                "Either user_id or session_id must be set",
            ));
        }
    };

    let cart = cart::ActiveModel {
        cart_id: ActiveValue::NotSet,
        user_id,
        session_id: session_id_value,
        variant_id: ActiveValue::Set(req.variant_id),
        quantity: ActiveValue::Set(req.quantity),
        created_at: ActiveValue::NotSet,
        updated_at: ActiveValue::NotSet,
        abandoned_email_sent_at: ActiveValue::NotSet,
    };
    match cart.insert(txn).await {
        Ok(cart_model) => Ok(to_response(cart_model)),
        // Concurrent add for the same (owner, variant) can still race past the
        // find-above and hit the unique constraint on insert; fall back to the
        // same increment behavior instead of surfacing a raw duplicate-key error.
        Err(e) => {
            let raced_existing = cart::Entity::find()
                .filter(owner_filter(req.user_id, session_id.as_deref())?)
                .filter(cart::Column::VariantId.eq(req.variant_id))
                .one(txn)
                .await
                .map_err(map_db_error_to_status)?;
            match raced_existing {
                Some(existing) => increment_existing(txn, existing, req.quantity).await,
                None => Err(map_db_error_to_status(e)),
            }
        }
    }
}
