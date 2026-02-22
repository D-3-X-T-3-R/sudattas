use crate::handlers::db_errors::map_db_error_to_status;
use crate::handlers::shipments::create_shipment::model_to_response;
use chrono::Utc;
use core_db_entities::entity::sea_orm_active_enums::Status;
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
    if let Some(status_str) = req.status {
        let status = match status_str.as_str() {
            "processed" => Some(Status::Processed),
            "failed" => Some(Status::Failed),
            _ => Some(Status::Pending),
        };
        if matches!(status, Some(Status::Processed)) {
            model.delivered_at = ActiveValue::Set(Some(Utc::now()));
        }
        model.status = ActiveValue::Set(status);
    }

    match model.update(txn).await {
        Ok(updated) => Ok(Response::new(ShipmentsResponse {
            items: vec![model_to_response(updated)],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
