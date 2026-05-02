use core_db_entities::entity::orders;
use core_db_entities::entity::sea_orm_active_enums::FulfillmentStatus;
use proto::proto::core::OrderResponse;

fn fulfillment_status_to_string(status: FulfillmentStatus) -> &'static str {
    match status {
        FulfillmentStatus::NotCreated => "not_created",
        FulfillmentStatus::Booked => "booked",
        FulfillmentStatus::PickupCompleted => "pickup_completed",
        FulfillmentStatus::InTransit => "in_transit",
        FulfillmentStatus::Delivered => "delivered",
        FulfillmentStatus::Rto => "rto",
    }
}

pub fn from_model(model: &orders::Model) -> OrderResponse {
    OrderResponse {
        order_id: model.order_id,
        user_id: model.user_id,
        order_date: model.order_date.to_string(),
        shipping_address_id: model.shipping_address_id,
        total_amount_paise: model.grand_total_minor,
        status_id: model.status_id,
        public_order_ref: model.public_order_ref.clone(),
        refund_settlement_status: model.refund_settlement_status.clone(),
        payment_method: model.payment_method.clone(),
        cancel_window_ends_at: None,
        earliest_booking_at: None,
        pickup_target_at: None,
        fulfillment_status: Some(
            fulfillment_status_to_string(model.fulfillment_status.clone()).to_string(),
        ),
        invoice_id: model.invoice_id,
        invoice_number: model.invoice_number.clone(),
        invoice_generated_at: model.invoice_generated_at.map(|v| v.to_rfc3339()),
        invoice_storage_path: model.invoice_storage_path.clone(),
        invoice_available: Some(model.invoice_id.is_some()),
    }
}
