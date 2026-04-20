use crate::handlers::db_errors::map_db_error_to_status;
use crate::handlers::refunds::create_refund::model_to_response;
use core_db_entities::entity::refunds;
use proto::proto::core::{GetRefundsRequest, RefundsResponse};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder};
use tonic::{Request, Response, Status};

pub async fn get_refunds(
    txn: &DatabaseTransaction,
    request: Request<GetRefundsRequest>,
) -> Result<Response<RefundsResponse>, Status> {
    let req = request.into_inner();

    let mut query = refunds::Entity::find();
    let mut has_filter = false;

    if let Some(refund_id) = req.refund_id {
        query = query.filter(refunds::Column::RefundId.eq(refund_id));
        has_filter = true;
    }
    if let Some(order_id) = req.order_id {
        query = query.filter(refunds::Column::OrderId.eq(order_id));
        has_filter = true;
    }
    if let Some(gateway_refund_id) = req
        .gateway_refund_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        query = query.filter(refunds::Column::GatewayRefundId.eq(gateway_refund_id));
        has_filter = true;
    }

    if !has_filter {
        return Err(Status::invalid_argument(
            "At least one refund filter must be set",
        ));
    }

    let rows = query
        .order_by_desc(refunds::Column::RefundId)
        .all(txn)
        .await
        .map_err(map_db_error_to_status)?;

    Ok(Response::new(RefundsResponse {
        items: rows.iter().map(model_to_response).collect(),
    }))
}

