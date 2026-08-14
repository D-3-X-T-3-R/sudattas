//! P1 Admin: mark order as shipped with enforced state machine; optional tracking.

use crate::handlers::shipments::{book_order_after_validation, create_shipment, update_shipment};
use crate::order_state_machine;
use core_db_entities::entity::shipments;
use proto::proto::core::{
    AdminMarkOrderShippedRequest, AdminMarkOrderShippedResponse, CreateShipmentRequest,
    UpdateShipmentRequest,
};
use sea_orm::{ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn admin_mark_order_shipped(
    txn: &DatabaseTransaction,
    db: &DatabaseConnection,
    request: Request<AdminMarkOrderShippedRequest>,
) -> Result<Response<AdminMarkOrderShippedResponse>, Status> {
    let req = request.into_inner();

    let existing = shipments::Entity::find()
        .filter(shipments::Column::OrderId.eq(req.order_id))
        .one(txn)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    let has_tracking =
        req.awb_code.is_some() || req.carrier.is_some() || req.shiprocket_order_id.is_some();
    let shipment_id: i64 = if req.shiprocket_book == Some(true) {
        if let Some(ship) = existing {
            ship.shipment_id
        } else {
            // book_order_after_validation manages its own short transactions internally (and
            // makes an outbound Shiprocket call for COD orders with no transaction open) rather
            // than running inside `txn`, so its writes commit independently of `txn`. `txn`'s
            // own read snapshot was already fixed by the `existing = ...` query above (MySQL
            // REPEATABLE READ fixes a transaction's consistent-read view at its first query),
            // so re-reading the just-created row through `txn` would not see it — read through
            // `db` instead, which always sees the latest committed state.
            book_order_after_validation(db, req.order_id, chrono::Utc::now(), "shipment_booked_admin")
                .await?;
            shipments::Entity::find()
                .filter(shipments::Column::OrderId.eq(req.order_id))
                .one(db)
                .await
                .map_err(|e| Status::internal(e.to_string()))?
                .map(|ship| ship.shipment_id)
                .ok_or_else(|| {
                    Status::internal("Shipment booking completed but no shipment row found")
                })?
        }
    } else if let Some(ship) = existing {
        ship.shipment_id
    } else if has_tracking {
        let created = create_shipment(
            txn,
            Request::new(CreateShipmentRequest {
                order_id: req.order_id,
                shiprocket_order_id: req.shiprocket_order_id.clone(),
                awb_code: req.awb_code.clone(),
                carrier: req.carrier.clone(),
                shiprocket_status_id: req.shiprocket_status_id,
                shiprocket_status_label: req.shiprocket_status_label.clone(),
            }),
        )
        .await?
        .into_inner()
        .items
        .into_iter()
        .next()
        .ok_or_else(|| Status::internal("create_shipment returned no shipment"))?;
        created.shipment_id
    } else {
        return Err(Status::failed_precondition(
            "Shipment is not booked yet. Use delayed booking flow or shiprocket_book=true after eligibility.",
        ));
    };
    if has_tracking {
        let _ = update_shipment(
            txn,
            Request::new(UpdateShipmentRequest {
                shipment_id,
                shiprocket_order_id: req.shiprocket_order_id.clone(),
                awb_code: req.awb_code,
                carrier: req.carrier,
                status: None,
                tracking_events_json: None,
                shiprocket_status_id: req.shiprocket_status_id,
                shiprocket_status_label: req.shiprocket_status_label.clone(),
            }),
        )
        .await?;
    }

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

    Ok(Response::new(AdminMarkOrderShippedResponse {
        order_id: req.order_id,
        shipment_id,
    }))
}
