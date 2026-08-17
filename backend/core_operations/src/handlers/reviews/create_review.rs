use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::reviews;
use proto::proto::core::{CreateReviewRequest, ReviewResponse, ReviewsResponse};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ConnectionTrait, DatabaseTransaction, DbBackend, Statement,
};
use tonic::{Request, Response, Status};

/// True if `user_id` has a non-cancelled order line for `product_id` on an order whose
/// fulfillment has reached `delivered`. Reviews are gated on this — see `create_review` below —
/// rather than left as an honor-system UI restriction, since the review endpoint is a normal
/// authenticated GraphQL mutation any client could call directly.
async fn has_delivered_purchase(
    txn: &DatabaseTransaction,
    user_id: i64,
    product_id: i64,
) -> Result<bool, Status> {
    let sql = "SELECT CAST(EXISTS ( \
        SELECT 1 FROM OrderDetails od \
        JOIN Orders o ON o.OrderID = od.OrderID \
        JOIN ProductVariants pv ON pv.VariantID = od.VariantID \
        WHERE o.UserID = ? \
          AND pv.ProductID = ? \
          AND od.item_status = 'active' \
          AND o.fulfillment_status = 'delivered' \
    ) AS SIGNED) AS has_delivered_purchase";
    let row = txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::MySql,
            sql,
            [user_id.into(), product_id.into()],
        ))
        .await
        .map_err(map_db_error_to_status)?;
    Ok(row
        .and_then(|r| r.try_get::<i64>("", "has_delivered_purchase").ok())
        .unwrap_or(0)
        > 0)
}

pub async fn create_review(
    txn: &DatabaseTransaction,
    request: Request<CreateReviewRequest>,
) -> Result<Response<ReviewsResponse>, Status> {
    let req = request.into_inner();

    if !has_delivered_purchase(txn, req.user_id, req.product_id).await? {
        return Err(Status::failed_precondition(
            "You can only review products from an order that has been delivered to you",
        ));
    }

    let model = reviews::ActiveModel {
        review_id: ActiveValue::NotSet,
        product_id: ActiveValue::Set(Some(req.product_id)),
        user_id: ActiveValue::Set(Some(req.user_id)),
        rating: ActiveValue::Set(req.rating as i8),
        comment: ActiveValue::Set(Some(req.comment)),
        review_status: ActiveValue::NotSet,
        // Reachable only after the delivered-purchase check above, so this is always a genuine
        // verified purchase — closes the gap where this column was defined ("Derived from order
        // history; not user-controlled") but never actually set by any code path.
        is_verified_purchase: ActiveValue::Set(Some(1)),
        created_at: ActiveValue::NotSet,
    };

    match model.insert(txn).await {
        Ok(inserted) => Ok(Response::new(ReviewsResponse {
            items: vec![ReviewResponse {
                review_id: inserted.review_id,
                product_id: inserted.product_id.unwrap_or(0),
                user_id: inserted.user_id.unwrap_or(0),
                rating: inserted.rating as i32,
                comment: inserted.comment.unwrap_or_default(),
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
