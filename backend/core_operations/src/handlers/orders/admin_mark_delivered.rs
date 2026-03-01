//! P1 Admin: mark order as delivered with enforced state machine.

use crate::order_state_machine;
use proto::proto::core::{AdminMarkOrderDeliveredRequest, AdminMarkOrderDeliveredResponse};
use sea_orm::DatabaseTransaction;
use tonic::{Request, Response, Status};

pub async fn admin_mark_order_delivered(
    txn: &DatabaseTransaction,
    request: Request<AdminMarkOrderDeliveredRequest>,
) -> Result<Response<AdminMarkOrderDeliveredResponse>, Status> {
    let req = request.into_inner();

    order_state_machine::transition_order_status(
        txn,
        req.order_id,
        order_state_machine::OrderState::Delivered,
        "admin_mark_delivered",
        "admin",
        None,
        None,
    )
    .await?;

    Ok(Response::new(AdminMarkOrderDeliveredResponse {
        order_id: req.order_id,
    }))
}
