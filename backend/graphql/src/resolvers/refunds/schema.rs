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
    pub line_items_refunded_json: Option<String>,
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

    async fn line_items_refunded_json(&self) -> &Option<String> {
        &self.line_items_refunded_json
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

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Resolve a RefundAttempt stuck in needs_review")]
pub struct ResolveRefundAttemptNeedsReviewInput {
    pub attempt_id: String,
    /// "retry" | "mark_settled"
    pub resolution: String,
    /// admin identifier (e.g. user_id or "admin")
    pub actor_id: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Get refunds")]
pub struct GetRefund {
    pub refund_id: Option<String>,
    pub order_id: Option<String>,
    pub gateway_refund_id: Option<String>,
}

/// An in-flight refund attempt against a real gateway payment — distinct from `Refund`, which
/// is a settled/recorded refund. This is the row `resolveRefundAttemptNeedsReview` acts on.
#[derive(Default, Debug, Clone)]
pub struct RefundAttempt {
    pub attempt_id: String,
    pub order_id: String,
    pub payment_intent_id: Option<String>,
    pub razorpay_payment_id: Option<String>,
    pub amount_requested_paise: String,
    pub amount_sent_to_gateway_paise: String,
    pub gateway_refund_id: Option<String>,
    pub status: String,
    pub provider_error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub attempt_count: i32,
}

#[graphql_object]
#[graphql(description = "In-flight refund attempt against a gateway payment")]
impl RefundAttempt {
    async fn attempt_id(&self) -> &String {
        &self.attempt_id
    }
    async fn order_id(&self) -> &String {
        &self.order_id
    }
    async fn payment_intent_id(&self) -> &Option<String> {
        &self.payment_intent_id
    }
    async fn razorpay_payment_id(&self) -> &Option<String> {
        &self.razorpay_payment_id
    }
    async fn amount_requested_paise(&self) -> &String {
        &self.amount_requested_paise
    }
    async fn amount_sent_to_gateway_paise(&self) -> &String {
        &self.amount_sent_to_gateway_paise
    }
    async fn gateway_refund_id(&self) -> &Option<String> {
        &self.gateway_refund_id
    }
    async fn status(&self) -> &String {
        &self.status
    }
    async fn provider_error(&self) -> &Option<String> {
        &self.provider_error
    }
    async fn created_at(&self) -> &String {
        &self.created_at
    }
    async fn updated_at(&self) -> &String {
        &self.updated_at
    }
    async fn attempt_count(&self) -> i32 {
        self.attempt_count
    }
}

#[derive(GraphQLInputObject, Default, Debug, Clone)]
#[graphql(description = "Search refund attempts")]
pub struct SearchRefundAttemptsInput {
    pub attempt_id: Option<String>,
    pub order_id: Option<String>,
    /// e.g. "needs_review", "pending_external", "resolved"
    pub status: Option<String>,
}
