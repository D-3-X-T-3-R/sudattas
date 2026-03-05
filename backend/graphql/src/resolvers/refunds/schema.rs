use juniper::{graphql_object, GraphQLInputObject};

#[derive(Default, Debug, Clone)]
pub struct Refund {
    pub refund_id: String,
    pub order_id: String,
    pub gateway_refund_id: String,
    pub amount_paise: String,
    pub currency: String,
    pub status: String,
    pub created_at: String,
}

#[graphql_object]
#[graphql(description = "Refund")]
impl Refund {
    async fn refund_id(&self) -> &String {
        &self.refund_id
    }

    async fn order_id(&self) -> &String {
        &self.order_id
    }

    async fn gateway_refund_id(&self) -> &String {
        &self.gateway_refund_id
    }

    async fn amount_paise(&self) -> &String {
        &self.amount_paise
    }

    async fn currency(&self) -> &String {
        &self.currency
    }

    async fn status(&self) -> &String {
        &self.status
    }

    async fn created_at(&self) -> &String {
        &self.created_at
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Create a refund")]
pub struct NewRefund {
    pub order_id: String,
    pub gateway_refund_id: String,
    pub amount_paise: String,
    pub currency: Option<String>,
    /// JSON array of `{order_detail_id, quantity_refunded, amount_paise}`
    pub line_items_refunded_json: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Resolve a NeedsReview payment manually")]
pub struct ResolveNeedsReviewInput {
    pub order_id: String,
    /// "paid" | "cancelled" | "refunded"
    pub resolution: String,
    /// admin identifier (e.g. user_id or "admin")
    pub actor_id: String,
}
