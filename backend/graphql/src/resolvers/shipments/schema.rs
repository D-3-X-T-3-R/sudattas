use juniper::{graphql_object, GraphQLInputObject};

#[derive(Default, Debug, Clone)]
pub struct Shipment {
    pub shipment_id: String,
    pub order_id: String,
    pub shiprocket_order_id: Option<String>,
    pub awb_code: Option<String>,
    pub carrier: Option<String>,
    pub status: String,
    pub created_at: String,
    pub delivered_at: Option<String>,
    /// JSON array of courier milestones for the storefront, e.g. [{"label":"Picked up","at":"...","location":"..."}].
    pub tracking_events_json: Option<String>,
    /// Shiprocket `shipment_status_id` when known (webhook / API sync).
    pub shiprocket_status_id: Option<String>,
    /// Human-readable Shiprocket status (API or mapped).
    pub shiprocket_status_label: Option<String>,
    /// Customer-facing tracking line (e.g. "In transit", "Out for delivery") derived from Shiprocket.
    pub customer_tracking_status: String,
}

#[graphql_object]
#[graphql(description = "Shipment")]
impl Shipment {
    async fn shipment_id(&self) -> &String {
        &self.shipment_id
    }
    async fn order_id(&self) -> &String {
        &self.order_id
    }
    async fn shiprocket_order_id(&self) -> &Option<String> {
        &self.shiprocket_order_id
    }
    async fn awb_code(&self) -> &Option<String> {
        &self.awb_code
    }
    async fn carrier(&self) -> &Option<String> {
        &self.carrier
    }
    async fn status(&self) -> &String {
        &self.status
    }
    async fn created_at(&self) -> &String {
        &self.created_at
    }
    async fn delivered_at(&self) -> &Option<String> {
        &self.delivered_at
    }
    async fn tracking_events_json(&self) -> &Option<String> {
        &self.tracking_events_json
    }
    async fn shiprocket_status_id(&self) -> &Option<String> {
        &self.shiprocket_status_id
    }
    async fn shiprocket_status_label(&self) -> &Option<String> {
        &self.shiprocket_status_label
    }
    async fn customer_tracking_status(&self) -> &String {
        &self.customer_tracking_status
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Create Shipment")]
pub struct NewShipment {
    pub order_id: String,
    pub shiprocket_order_id: Option<String>,
    pub awb_code: Option<String>,
    pub carrier: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Update Shipment")]
pub struct UpdateShipment {
    pub shipment_id: String,
    pub shiprocket_order_id: Option<String>,
    pub awb_code: Option<String>,
    pub carrier: Option<String>,
    /// pending | processed | failed
    pub status: Option<String>,
    /// JSON array of customer-visible courier steps (replaces stored timeline when set).
    pub tracking_events_json: Option<String>,
    pub shiprocket_status_id: Option<String>,
    pub shiprocket_status_label: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Get Shipment")]
pub struct GetShipment {
    pub shipment_id: Option<String>,
    pub order_id: Option<String>,
}
