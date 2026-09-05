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
}

#[graphql_object]
#[graphql(description = "Category-scoped exchange request (same product, different size/colour, same price) — distinct from the refund-only ReturnRequest")]
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
