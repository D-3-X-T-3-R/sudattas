use core_db_entities::entity::orders;
use proto::proto::core::OrderResponse;

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
    }
}
