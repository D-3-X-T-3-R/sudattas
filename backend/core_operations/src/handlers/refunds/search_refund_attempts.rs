//! Admin visibility into in-flight `RefundAttempts` — the row `resolve_refund_attempt_needs_review`
//! acts on. Without this, nothing exposed which attempt_ids exist or which are stuck in
//! `needs_review`, so that mutation had no way to be driven from a UI.

use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::refund_attempts;
use proto::proto::core::{
    RefundAttemptResponse, RefundAttemptsResponse, SearchRefundAttemptsRequest,
};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder, QueryTrait};
use tonic::{Request, Response, Status};

fn to_response(row: &refund_attempts::Model) -> RefundAttemptResponse {
    RefundAttemptResponse {
        attempt_id: row.attempt_id,
        order_id: row.order_id,
        payment_intent_id: row.payment_intent_id,
        razorpay_payment_id: row.razorpay_payment_id.clone(),
        amount_requested_paise: row.amount_requested_paise,
        amount_sent_to_gateway_paise: row.amount_sent_to_gateway_paise,
        gateway_refund_id: row.gateway_refund_id.clone(),
        status: row.status.clone(),
        provider_error: row.provider_error.clone(),
        created_at: row.created_at.to_rfc3339(),
        updated_at: row.updated_at.to_rfc3339(),
        attempt_count: row.attempt_count,
    }
}

pub async fn search_refund_attempts(
    txn: &DatabaseTransaction,
    request: Request<SearchRefundAttemptsRequest>,
) -> Result<Response<RefundAttemptsResponse>, Status> {
    let req = request.into_inner();
    let rows = refund_attempts::Entity::find()
        .apply_if(req.attempt_id, |query, v| {
            query.filter(refund_attempts::Column::AttemptId.eq(v))
        })
        .apply_if(req.order_id, |query, v| {
            query.filter(refund_attempts::Column::OrderId.eq(v))
        })
        .apply_if(req.status, |query, v| {
            query.filter(refund_attempts::Column::Status.eq(v))
        })
        .order_by_desc(refund_attempts::Column::AttemptId)
        .all(txn)
        .await
        .map_err(map_db_error_to_status)?;

    Ok(Response::new(RefundAttemptsResponse {
        items: rows.iter().map(to_response).collect(),
    }))
}
