//! P1 Admin: mark order as shipped with enforced state machine; optional tracking.

use crate::handlers::shipments::{create_shipment, update_shipment};
use crate::order_state_machine;
use core_db_entities::entity::shipments;
use proto::proto::core::{
    AdminMarkOrderShippedRequest, AdminMarkOrderShippedResponse, CreateShipmentRequest,
    UpdateShipmentRequest,
};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn admin_mark_order_shipped(
    txn: &DatabaseTransaction,
    request: Request<AdminMarkOrderShippedRequest>,
) -> Result<Response<AdminMarkOrderShippedResponse>, Status> {
    let req = request.into_inner();

    order_state_machine::transition_order_status(
        txn,
        req.order_id,
        order_state_machine::OrderState::Shipped,
        "admin_mark_shipped",
        "admin",
        None,
        None,
    )
    .await?;

    let mut shipment_id: i64 = 0;
    if req.awb_code.is_some() || req.carrier.is_some() {
        let existing = shipments::Entity::find()
            .filter(shipments::Column::OrderId.eq(req.order_id))
            .one(txn)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        if let Some(ship) = existing {
            let _ = update_shipment(
                txn,
                Request::new(UpdateShipmentRequest {
                    shipment_id: ship.shipment_id,
                    shiprocket_order_id: None,
                    awb_code: req.awb_code,
                    carrier: req.carrier,
                    status: None,
                }),
            )
            .await?;
            shipment_id = ship.shipment_id;
        } else {
            let create_res = create_shipment(
                txn,
                Request::new(CreateShipmentRequest {
                    order_id: req.order_id,
                    shiprocket_order_id: None,
                    awb_code: req.awb_code,
                    carrier: req.carrier,
                }),
            )
            .await?;
            shipment_id = create_res
                .into_inner()
                .items
                .first()
                .map(|s| s.shipment_id)
                .unwrap_or(0);
        }
    }

    Ok(Response::new(AdminMarkOrderShippedResponse {
        order_id: req.order_id,
        shipment_id,
    }))
}
