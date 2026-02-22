use crate::handlers::db_errors::map_db_error_to_status;
use chrono::Utc;
use core_db_entities::entity::sea_orm_active_enums::Status;
use core_db_entities::entity::shipments;
use proto::proto::core::{CreateShipmentRequest, ShipmentResponse, ShipmentsResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status as TonicStatus};

pub async fn create_shipment(
    txn: &DatabaseTransaction,
    request: Request<CreateShipmentRequest>,
) -> Result<Response<ShipmentsResponse>, TonicStatus> {
    let req = request.into_inner();

    let shipment = shipments::ActiveModel {
        shipment_id: ActiveValue::NotSet,
        order_id: ActiveValue::Set(req.order_id),
        shiprocket_order_id: ActiveValue::Set(req.shiprocket_order_id),
        awb_code: ActiveValue::Set(req.awb_code),
        carrier: ActiveValue::Set(req.carrier),
        status: ActiveValue::Set(Some(Status::Pending)),
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
    ShipmentResponse {
        shipment_id: model.shipment_id,
        order_id: model.order_id,
        shiprocket_order_id: model.shiprocket_order_id,
        awb_code: model.awb_code,
        carrier: model.carrier,
        status: model
            .status
            .map(|s| format!("{:?}", s).to_lowercase())
            .unwrap_or_else(|| "pending".to_string()),
        created_at: model.created_at.map(|t| t.to_string()).unwrap_or_default(),
        delivered_at: model.delivered_at.map(|t| t.to_string()),
    }
}
