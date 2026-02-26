use crate::handlers::db_errors::map_db_error_to_status;
use crate::handlers::order_events::create_order_event;
use chrono::Utc;
use core_db_entities::entity::{order_details, order_status, orders};
use proto::proto::core::{
    CreateOrderEventRequest, OrderResponse, OrdersResponse, UpdateOrderRequest,
};
use rust_decimal::{
    prelude::{FromPrimitive, ToPrimitive},
    Decimal,
};
use sea_orm::DbBackend;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, DatabaseTransaction, EntityTrait,
    QueryFilter, Statement,
};
use tonic::{Request, Response, Status};

pub async fn update_order(
    txn: &DatabaseTransaction,
    request: Request<UpdateOrderRequest>,
) -> Result<Response<OrdersResponse>, Status> {
    let req = request.into_inner();

    // Fetch existing order to capture previous status for audit event.
    let existing = orders::Entity::find_by_id(req.order_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?;
    let prev_status_id = existing.as_ref().map(|o| o.status_id);

    let orders = orders::ActiveModel {
        order_id: ActiveValue::Set(req.order_id),
        user_id: ActiveValue::Set(req.user_id),
        order_date: ActiveValue::Set(Utc::now()),
        shipping_address_id: ActiveValue::Set(req.shipping_address_id),
        total_amount: ActiveValue::Set(Decimal::from_f64(req.total_amount).unwrap()),
        status_id: ActiveValue::Set(req.status_id),
        order_number: ActiveValue::NotSet,
        payment_status: ActiveValue::NotSet,
        currency: ActiveValue::NotSet,
        updated_at: ActiveValue::NotSet,
    };

    match orders.update(txn).await {
        Ok(model) => {
            // When transitioning to cancelled, restore inventory from order line items.
            let cancelled = order_status::Entity::find()
                .filter(order_status::Column::StatusName.eq("cancelled"))
                .one(txn)
                .await
                .map_err(map_db_error_to_status)?;
            if let Some(ref c) = cancelled {
                if model.status_id == c.status_id {
                    let details = order_details::Entity::find()
                        .filter(order_details::Column::OrderId.eq(model.order_id))
                        .all(txn)
                        .await
                        .map_err(map_db_error_to_status)?;
                    for d in &details {
                        let _ = txn
                            .execute(Statement::from_sql_and_values(
                                DbBackend::MySql,
                                r#"UPDATE Inventory SET QuantityAvailable = QuantityAvailable + ? WHERE ProductID = ?"#,
                                [d.quantity.into(), d.product_id.into()],
                            ))
                            .await
                            .map_err(map_db_error_to_status)?;
                    }
                }
            }

            // Emit a status-change event if status_id changed.
            if prev_status_id.map(|p| p != model.status_id).unwrap_or(true) {
                let _ = create_order_event(
                    txn,
                    tonic::Request::new(CreateOrderEventRequest {
                        order_id: model.order_id,
                        event_type: "status_changed".to_string(),
                        from_status: prev_status_id.map(|s| s.to_string()),
                        to_status: Some(model.status_id.to_string()),
                        actor_type: "system".to_string(),
                        message: Some(format!(
                            "Order {} status changed to {}",
                            model.order_id, model.status_id
                        )),
                    }),
                )
                .await;
            }

            let response = OrdersResponse {
                items: vec![OrderResponse {
                    order_id: model.order_id,
                    user_id: model.user_id,
                    order_date: model.order_date.to_string(),
                    shipping_address_id: model.shipping_address_id,
                    total_amount: Decimal::to_f64(&model.total_amount).unwrap(),
                    status_id: model.status_id,
                }],
            };
            Ok(Response::new(response))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
