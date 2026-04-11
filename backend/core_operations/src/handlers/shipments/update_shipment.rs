use crate::handlers::db_errors::map_db_error_to_status;
use crate::handlers::shipments::create_shipment::model_to_response;
use crate::handlers::shipments::shipment_status_parse::parse_shipment_status_str;
use crate::integrations::shiprocket_status::{
    map_shiprocket_id_to_shipment_status, shiprocket_status_label_for_id,
};
use chrono::Utc;
use core_db_entities::entity::sea_orm_active_enums::ShipmentStatus;
use core_db_entities::entity::shipments;
use proto::proto::core::{ShipmentsResponse, UpdateShipmentRequest};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction, EntityTrait, IntoActiveModel};
use tonic::{Request, Response, Status as TonicStatus};

pub async fn update_shipment(
    txn: &DatabaseTransaction,
    request: Request<UpdateShipmentRequest>,
) -> Result<Response<ShipmentsResponse>, TonicStatus> {
    let req = request.into_inner();

    let existing = shipments::Entity::find_by_id(req.shipment_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| TonicStatus::not_found(format!("Shipment {} not found", req.shipment_id)))?;

    let mut model = existing.into_active_model();

    if let Some(v) = req.shiprocket_order_id {
        model.shiprocket_order_id = ActiveValue::Set(Some(v));
    }
    if let Some(v) = req.awb_code {
        model.awb_code = ActiveValue::Set(Some(v));
    }
    if let Some(v) = req.carrier {
        model.carrier = ActiveValue::Set(Some(v));
    }

    if let Some(id) = req.shiprocket_status_id {
        model.shiprocket_status_id = ActiveValue::Set(Some(id));
        let lbl = req
            .shiprocket_status_label
            .clone()
            .unwrap_or_else(|| shiprocket_status_label_for_id(id));
        model.shiprocket_status_label = ActiveValue::Set(Some(lbl));
        let next = map_shiprocket_id_to_shipment_status(id);
        if matches!(
            next,
            ShipmentStatus::Delivered | ShipmentStatus::RtoDelivered
        ) {
            model.delivered_at = ActiveValue::Set(Some(Utc::now()));
        }
        model.status = ActiveValue::Set(next);
    } else if let Some(lbl) = req.shiprocket_status_label {
        model.shiprocket_status_label = ActiveValue::Set(Some(lbl));
    }

    if let Some(status_str) = req.status {
        if let Some(st) = parse_shipment_status_str(&status_str) {
            if matches!(st, ShipmentStatus::Delivered | ShipmentStatus::RtoDelivered) {
                model.delivered_at = ActiveValue::Set(Some(Utc::now()));
            }
            model.status = ActiveValue::Set(st);
        }
    }

    if let Some(raw) = req.tracking_events_json {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            model.tracking_events = ActiveValue::Set(None);
        } else {
            let v: serde_json::Value = serde_json::from_str(trimmed).map_err(|_| {
                TonicStatus::invalid_argument("Invalid tracking_events JSON (must be a JSON array)")
            })?;
            if !v.is_array() {
                return Err(TonicStatus::invalid_argument(
                    "tracking_events JSON must be a JSON array",
                ));
            }
            model.tracking_events = ActiveValue::Set(Some(v));
        }
    }

    match model.update(txn).await {
        Ok(updated) => Ok(Response::new(ShipmentsResponse {
            items: vec![model_to_response(updated)],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
