use crate::handlers::db_errors::map_db_error_to_status;
use crate::handlers::order_events::create_order_event;
use crate::money::{decimal_to_paise, paise_to_decimal};
use crate::order_state_machine;
use chrono::Utc;
use core_db_entities::entity::{order_details, order_status, orders};
use proto::proto::core::{
    CreateOrderEventRequest, OrderResponse, OrdersResponse, UpdateOrderRequest,
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

    let existing = orders::Entity::find_by_id(req.order_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?;
    let existing = existing.ok_or_else(|| Status::not_found("Order not found"))?;
    let prev_status_id = existing.status_id;

    let allowed = order_state_machine::can_transition_by_ids(txn, prev_status_id, req.status_id)
        .await
        .map_err(|e: tonic::Status| Status::internal(e.message().to_string()))?;
    if !allowed {
        return Err(Status::invalid_argument(format!(
            "Illegal order state transition from status_id {} to {}",
            prev_status_id, req.status_id
        )));
    }

    let orders = orders::ActiveModel {
        order_id: ActiveValue::Set(req.order_id),
        user_id: ActiveValue::Set(req.user_id),
        order_date: ActiveValue::Set(Utc::now()),
        shipping_address_id: ActiveValue::Set(req.shipping_address_id),
        total_amount: ActiveValue::Set(paise_to_decimal(req.total_amount_paise)),
        status_id: ActiveValue::Set(req.status_id),
        order_number: ActiveValue::NotSet,
        payment_status: ActiveValue::NotSet,
        currency: ActiveValue::NotSet,
        updated_at: ActiveValue::NotSet,
        subtotal_minor: ActiveValue::NotSet,
        shipping_minor: ActiveValue::NotSet,
        tax_total_minor: ActiveValue::NotSet,
        discount_total_minor: ActiveValue::NotSet,
        grand_total_minor: ActiveValue::NotSet,
        applied_coupon_id: ActiveValue::NotSet,
        applied_coupon_code: ActiveValue::NotSet,
        applied_discount_paise: ActiveValue::NotSet,
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

            if prev_status_id != model.status_id {
                let from_name = order_state_machine::get_status_name(txn, prev_status_id)
                    .await
                    .ok()
                    .flatten()
                    .unwrap_or_else(|| prev_status_id.to_string());
                let to_name = order_state_machine::get_status_name(txn, model.status_id)
                    .await
                    .ok()
                    .flatten()
                    .unwrap_or_else(|| model.status_id.to_string());
                let _ = create_order_event(
                    txn,
                    tonic::Request::new(CreateOrderEventRequest {
                        order_id: model.order_id,
                        event_type: "status_changed".to_string(),
                        from_status: Some(from_name),
                        to_status: Some(to_name),
                        actor_type: "system".to_string(),
                        message: Some(format!(
                            "Order {} status changed to {}",
                            model.order_id, model.status_id
                        )),
                    }),
                )
                .await;
            }

            let total_amount_paise = model
                .grand_total_minor
                .unwrap_or_else(|| decimal_to_paise(&model.total_amount));
            let response = OrdersResponse {
                items: vec![OrderResponse {
                    order_id: model.order_id,
                    user_id: model.user_id,
                    order_date: model.order_date.to_string(),
                    shipping_address_id: model.shipping_address_id,
                    total_amount_paise,
                    status_id: model.status_id,
                }],
            };
            Ok(Response::new(response))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
