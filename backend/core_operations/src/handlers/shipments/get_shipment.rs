use crate::handlers::db_errors::map_db_error_to_status;
use crate::handlers::shipments::create_shipment::model_to_response;
use core_db_entities::entity::shipments;
use proto::proto::core::{GetShipmentRequest, ShipmentsResponse};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn get_shipment(
    txn: &DatabaseTransaction,
    request: Request<GetShipmentRequest>,
) -> Result<Response<ShipmentsResponse>, Status> {
    let req = request.into_inner();

    let query = shipments::Entity::find();

    let query = if let Some(shipment_id) = req.shipment_id {
        query.filter(shipments::Column::ShipmentId.eq(shipment_id))
    } else if let Some(order_id) = req.order_id {
        query.filter(shipments::Column::OrderId.eq(order_id))
    } else {
        return Err(Status::invalid_argument("Either shipment_id or order_id must be set"));
    };

    let results = query.all(txn).await.map_err(map_db_error_to_status)?;
    let items = results.into_iter().map(model_to_response).collect();
    Ok(Response::new(ShipmentsResponse { items }))
}
