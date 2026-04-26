use juniper::{graphql_object, GraphQLInputObject};

#[derive(Default, Debug, Clone)]
pub struct ReturnRequestItem {
    pub return_id: String,
    pub order_detail_id: String,
    pub quantity: String,
    pub refund_amount_minor: String,
    pub status: String,
}

#[graphql_object]
#[graphql(description = "Return request item")]
impl ReturnRequestItem {
    async fn return_id(&self) -> &String {
        &self.return_id
    }
    async fn order_detail_id(&self) -> &String {
        &self.order_detail_id
    }
    async fn quantity(&self) -> &String {
        &self.quantity
    }
    async fn refund_amount_minor(&self) -> &String {
        &self.refund_amount_minor
    }
    async fn status(&self) -> &String {
        &self.status
    }
}

#[derive(Default, Debug, Clone)]
pub struct ReturnRequest {
    pub return_id: String,
    pub order_id: String,
    pub user_id: String,
    pub status: String,
    pub reason: String,
    pub created_at: String,
    pub received_at: Option<String>,
    pub refund_attempt_id: Option<String>,
    pub items: Vec<ReturnRequestItem>,
}

#[graphql_object]
#[graphql(description = "Return request")]
impl ReturnRequest {
    async fn return_id(&self) -> &String {
        &self.return_id
    }
    async fn order_id(&self) -> &String {
        &self.order_id
    }
    async fn user_id(&self) -> &String {
        &self.user_id
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
    async fn refund_attempt_id(&self) -> &Option<String> {
        &self.refund_attempt_id
    }
    async fn items(&self) -> &Vec<ReturnRequestItem> {
        &self.items
    }
}

#[derive(GraphQLInputObject, Default, Debug, Clone)]
#[graphql(description = "Selected order line for return request")]
pub struct ReturnRequestItemInput {
    pub order_detail_id: String,
    pub quantity: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug, Clone)]
#[graphql(description = "Customer return request payload")]
pub struct RequestReturnInput {
    pub order_id: String,
    pub reason: String,
    pub items: Vec<ReturnRequestItemInput>,
}

#[derive(GraphQLInputObject, Default, Debug, Clone)]
#[graphql(description = "Search return requests")]
pub struct SearchReturnRequestsInput {
    pub return_id: Option<String>,
    pub order_id: Option<String>,
    pub user_id: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug, Clone)]
#[graphql(description = "Admin: mark return received at store")]
pub struct AdminMarkReturnReceivedInput {
    pub return_id: String,
}

#[derive(GraphQLInputObject, Default, Debug, Clone)]
#[graphql(description = "Admin: update return status")]
pub struct AdminUpdateReturnStatusInput {
    pub return_id: String,
    pub status: String,
    pub note: Option<String>,
}
