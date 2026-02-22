use juniper::{graphql_object, GraphQLInputObject};

#[derive(Default, Debug, Clone)]
pub struct OrderEvent {
    pub event_id: String,
    pub order_id: String,
    pub event_type: String,
    pub from_status: String,
    pub to_status: String,
    pub actor_type: String,
    pub message: String,
    pub created_at: String,
}

#[graphql_object]
#[graphql(description = "Order audit event")]
impl OrderEvent {
    async fn event_id(&self) -> &String {
        &self.event_id
    }
    async fn order_id(&self) -> &String {
        &self.order_id
    }
    async fn event_type(&self) -> &String {
        &self.event_type
    }
    async fn from_status(&self) -> &String {
        &self.from_status
    }
    async fn to_status(&self) -> &String {
        &self.to_status
    }
    /// customer | admin | system
    async fn actor_type(&self) -> &String {
        &self.actor_type
    }
    async fn message(&self) -> &String {
        &self.message
    }
    async fn created_at(&self) -> &String {
        &self.created_at
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Create an order event manually")]
pub struct NewOrderEvent {
    pub order_id: String,
    pub event_type: String,
    pub from_status: Option<String>,
    pub to_status: Option<String>,
    /// customer | admin | system
    pub actor_type: String,
    pub message: Option<String>,
}
