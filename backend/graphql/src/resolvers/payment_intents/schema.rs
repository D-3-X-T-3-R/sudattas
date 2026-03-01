use juniper::{graphql_object, GraphQLInputObject};

use crate::resolvers::money::{money_from_paise, Money};

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PaymentIntent {
    pub intent_id: String,
    pub razorpay_order_id: String,
    pub razorpay_key_id: Option<String>,
    pub order_id: Option<String>,
    pub user_id: Option<String>,
    pub amount_paise: String,
    pub currency: Option<String>,
    pub status: String,
    pub razorpay_payment_id: Option<String>,
    pub created_at: String,
    pub expires_at: String,
}

#[graphql_object]
#[graphql(description = "Payment Intent")]
impl PaymentIntent {
    async fn intent_id(&self) -> &String {
        &self.intent_id
    }
    async fn razorpay_order_id(&self) -> &String {
        &self.razorpay_order_id
    }
    async fn razorpay_key_id(&self) -> &Option<String> {
        &self.razorpay_key_id
    }
    async fn order_id(&self) -> &Option<String> {
        &self.order_id
    }
    async fn user_id(&self) -> &Option<String> {
        &self.user_id
    }
    /// Amount in paise (legacy string); prefer amount for Money type.
    async fn amount_paise(&self) -> &String {
        &self.amount_paise
    }
    /// Money type: amount_paise (integer), currency, formatted string.
    async fn amount(&self) -> Money {
        let paise = self.amount_paise.parse().unwrap_or(0);
        money_from_paise(paise, self.currency.as_deref())
    }
    async fn currency(&self) -> &Option<String> {
        &self.currency
    }
    async fn status(&self) -> &String {
        &self.status
    }
    async fn razorpay_payment_id(&self) -> &Option<String> {
        &self.razorpay_payment_id
    }
    async fn created_at(&self) -> &String {
        &self.created_at
    }
    async fn expires_at(&self) -> &String {
        &self.expires_at
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Create Payment Intent")]
pub struct NewPaymentIntent {
    pub order_id: String,
    pub user_id: String,
    pub amount_paise: String,
    pub currency: Option<String>,
    /// When absent, backend creates the Razorpay order via API (server-authoritative).
    pub razorpay_order_id: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Capture Payment")]
pub struct CapturePayment {
    pub intent_id: String,
    pub razorpay_payment_id: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Get Payment Intent")]
pub struct GetPaymentIntent {
    pub intent_id: Option<String>,
    pub order_id: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(
    description = "Verify Razorpay payment (client-returned payment_id, order_id, signature)"
)]
pub struct VerifyRazorpayPaymentInput {
    pub order_id: String,
    pub razorpay_payment_id: String,
    pub razorpay_order_id: String,
    pub razorpay_signature: String,
}

#[derive(Default, Debug)]
pub struct VerifyRazorpayPaymentResult {
    pub verified: bool,
    pub payment_intent: Option<PaymentIntent>,
}

#[graphql_object]
#[graphql(description = "Result of Razorpay payment verification")]
impl VerifyRazorpayPaymentResult {
    async fn verified(&self) -> bool {
        self.verified
    }
    async fn payment_intent(&self) -> &Option<PaymentIntent> {
        &self.payment_intent
    }
}
