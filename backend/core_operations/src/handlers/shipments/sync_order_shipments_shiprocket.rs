//! Poll Shiprocket track-by-AWB for all shipments on an order (customer refresh / cron).

use crate::handlers::db_errors::map_db_error_to_status;
use crate::handlers::shipments::apply_shiprocket_scan::apply_shiprocket_scan_to_shipment;
use crate::handlers::shipments::create_shipment::model_to_response;
use crate::integrations::shiprocket::{track_shipment_by_awb, ShiprocketError};
use core_db_entities::entity::shipments;
use proto::proto::core::{ShipmentResponse, SyncOrderShipmentsFromShiprocketResponse};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, TransactionTrait};
use tonic::{Response, Status};

pub async fn sync_order_shipments_from_shiprocket(
    db: &DatabaseConnection,
    order_id: i64,
) -> Result<Response<SyncOrderShipmentsFromShiprocketResponse>, Status> {
    let rows = shipments::Entity::find()
        .filter(shipments::Column::OrderId.eq(order_id))
        .all(db)
        .await
        .map_err(map_db_error_to_status)?;

    let mut items: Vec<ShipmentResponse> = Vec::new();
    for s in rows {
        let awb = match s.awb_code.as_deref().map(str::trim).filter(|x| !x.is_empty()) {
            Some(a) => a,
            None => {
                items.push(model_to_response(s));
                continue;
            }
        };
        let snap = match track_shipment_by_awb(awb).await {
            Ok(snap) => snap,
            Err(ShiprocketError::NotConfigured) => {
                items.push(model_to_response(s));
                continue;
            }
            Err(e) => return Err(Status::failed_precondition(e.to_string())),
        };
        let txn = db.begin().await.map_err(map_db_error_to_status)?;
        let fresh = shipments::Entity::find_by_id(s.shipment_id)
            .one(&txn)
            .await
            .map_err(map_db_error_to_status)?
            .ok_or_else(|| Status::not_found("shipment disappeared during sync"))?;
        let fallback_sr_id = fresh.shiprocket_status_id;
        let fallback_lbl = fresh.shiprocket_status_label.clone();
        let updated = apply_shiprocket_scan_to_shipment(
            &txn,
            fresh,
            snap.status_id.or(fallback_sr_id),
            snap.status_label.or(fallback_lbl),
            snap.scan_events,
        )
        .await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        items.push(model_to_response(updated));
    }

    Ok(Response::new(SyncOrderShipmentsFromShiprocketResponse { items }))
}
