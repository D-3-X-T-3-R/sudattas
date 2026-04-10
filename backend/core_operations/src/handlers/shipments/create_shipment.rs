use crate::handlers::db_errors::map_db_error_to_status;
use crate::integrations::shiprocket_status::{
    customer_tracking_label, map_shiprocket_id_to_shipment_status, shiprocket_status_label_for_id,
};
use chrono::Utc;
use core_db_entities::entity::sea_orm_active_enums::Status;
use core_db_entities::entity::shipments;
use proto::proto::core::{CreateShipmentRequest, ShipmentResponse, ShipmentsResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status as TonicStatus};

fn derive_line_status(req: &CreateShipmentRequest) -> Status {
    if let Some(id) = req.shiprocket_status_id {
        return map_shiprocket_id_to_shipment_status(id);
    }
    if req.awb_code.is_some() || req.shiprocket_order_id.is_some() {
        return Status::Processed;
    }
    Status::Pending
}

fn status_to_wire_str(status: &Status) -> &'static str {
    match status {
        Status::Pending => "pending",
        Status::Processed => "processed",
        Status::Failed => "failed",
        Status::NeedsReview => "needs_review",
        Status::ClientVerified => "client_verified",
    }
}

pub async fn create_shipment(
    txn: &DatabaseTransaction,
    request: Request<CreateShipmentRequest>,
) -> Result<Response<ShipmentsResponse>, TonicStatus> {
    let req = request.into_inner();

    let line_status = derive_line_status(&req);
    let sr_id = req.shiprocket_status_id;
    let sr_label = req
        .shiprocket_status_label
        .clone()
        .or_else(|| sr_id.map(shiprocket_status_label_for_id));

    let shipment = shipments::ActiveModel {
        shipment_id: ActiveValue::NotSet,
        order_id: ActiveValue::Set(req.order_id),
        shiprocket_order_id: ActiveValue::Set(req.shiprocket_order_id),
        awb_code: ActiveValue::Set(req.awb_code),
        carrier: ActiveValue::Set(req.carrier),
        shiprocket_status_id: ActiveValue::Set(sr_id),
        shiprocket_status_label: ActiveValue::Set(sr_label),
        status: ActiveValue::Set(line_status),
        tracking_events: ActiveValue::Set(None),
        created_at: ActiveValue::Set(Some(Utc::now())),
        delivered_at: ActiveValue::Set(None),
    };

    match shipment.insert(txn).await {
        Ok(model) => Ok(Response::new(ShipmentsResponse {
            items: vec![model_to_response(model)],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}

pub fn model_to_response(model: shipments::Model) -> ShipmentResponse {
    let tracking_events_json = model
        .tracking_events
        .as_ref()
        .map(std::string::ToString::to_string);
    let status_str = status_to_wire_str(&model.status).to_string();
    let customer_tracking_status =
        customer_tracking_label(model.shiprocket_status_id, Some(&model.status));
    ShipmentResponse {
        shipment_id: model.shipment_id,
        order_id: model.order_id,
        shiprocket_order_id: model.shiprocket_order_id,
        awb_code: model.awb_code,
        carrier: model.carrier,
        status: status_str,
        created_at: model.created_at.map(|t| t.to_string()).unwrap_or_default(),
        delivered_at: model.delivered_at.map(|t| t.to_string()),
        tracking_events_json,
        shiprocket_status_id: model.shiprocket_status_id,
        shiprocket_status_label: model.shiprocket_status_label,
        customer_tracking_status,
    }
}
