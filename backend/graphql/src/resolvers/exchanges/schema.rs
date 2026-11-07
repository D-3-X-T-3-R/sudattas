use juniper::{graphql_object, GraphQLInputObject};

#[derive(Default, Debug, Clone)]
pub struct ExchangeRequest {
    pub exchange_id: String,
    pub order_id: String,
    pub user_id: String,
    pub order_detail_id: String,
    pub desired_variant_id: String,
    pub quantity: String,
    pub status: String,
    pub reason: String,
    pub created_at: String,
    pub received_at: Option<String>,
    pub replacement_order_id: Option<String>,
    pub pickup_shiprocket_order_id: Option<String>,
    pub pickup_shiprocket_shipment_id: Option<String>,
    pub pickup_awb_code: Option<String>,
    pub pickup_courier_name: Option<String>,
    pub pickup_status: Option<String>,
    pub pickup_scheduled_at: Option<String>,
    pub pickup_tracking_events_json: Option<String>,
}

#[graphql_object]
#[graphql(
    description = "Category-scoped exchange request (same product, different size/colour, same price) — distinct from the refund-only ReturnRequest"
)]
impl ExchangeRequest {
    async fn exchange_id(&self) -> &String {
        &self.exchange_id
    }
    async fn order_id(&self) -> &String {
        &self.order_id
    }
    async fn user_id(&self) -> &String {
        &self.user_id
    }
    async fn order_detail_id(&self) -> &String {
        &self.order_detail_id
    }
    async fn desired_variant_id(&self) -> &String {
        &self.desired_variant_id
    }
    async fn quantity(&self) -> &String {
        &self.quantity
    }
    async fn status(&self) -> &String {
        &self.status
    }
    async fn reason(&self) -> &String {
        &self.reason
    }
    async fn created_at(&self) -> &String {
        &self.created_at
    }
    async fn received_at(&self) -> &Option<String> {
        &self.received_at
    }
    async fn replacement_order_id(&self) -> &Option<String> {
        &self.replacement_order_id
    }
    async fn pickup_shiprocket_order_id(&self) -> &Option<String> {
        &self.pickup_shiprocket_order_id
    }
    async fn pickup_shiprocket_shipment_id(&self) -> &Option<String> {
        &self.pickup_shiprocket_shipment_id
    }
    async fn pickup_awb_code(&self) -> &Option<String> {
        &self.pickup_awb_code
    }
    async fn pickup_courier_name(&self) -> &Option<String> {
        &self.pickup_courier_name
    }
    async fn pickup_status(&self) -> &Option<String> {
        &self.pickup_status
    }
    async fn pickup_scheduled_at(&self) -> &Option<String> {
        &self.pickup_scheduled_at
    }
    async fn pickup_tracking_events_json(&self) -> &Option<String> {
        &self.pickup_tracking_events_json
    }
}

#[derive(GraphQLInputObject, Default, Debug, Clone)]
#[graphql(description = "Customer exchange request payload")]
pub struct RequestExchangeInput {
    pub order_id: String,
    pub order_detail_id: String,
    pub desired_variant_id: String,
    pub quantity: Option<String>,
    pub reason: String,
}

#[derive(GraphQLInputObject, Default, Debug, Clone)]
#[graphql(description = "Search exchange requests")]
pub struct SearchExchangeRequestsInput {
    pub exchange_id: Option<String>,
    pub order_id: Option<String>,
    pub user_id: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug, Clone)]
#[graphql(description = "Admin: mark exchange item received and create the replacement order")]
pub struct AdminMarkExchangeReceivedInput {
    pub exchange_id: String,
}

#[derive(GraphQLInputObject, Default, Debug, Clone)]
#[graphql(description = "Admin: update exchange status")]
pub struct AdminUpdateExchangeStatusInput {
    pub exchange_id: String,
    pub status: String,
    pub note: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug, Clone)]
#[graphql(
    description = "Admin: book the reverse-pickup shipment (courier collects from the customer, delivers to the warehouse) via Shiprocket"
)]
pub struct ScheduleExchangePickupInput {
    pub exchange_id: String,
}

#[derive(GraphQLInputObject, Default, Debug, Clone)]
#[graphql(
    description = "Admin: refresh the reverse-pickup shipment's tracking; auto-completes the exchange once Shiprocket reports it delivered to the warehouse"
)]
pub struct SyncExchangePickupInput {
    pub exchange_id: String,
}
