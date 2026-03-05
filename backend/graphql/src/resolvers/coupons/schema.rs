use juniper::{graphql_object, GraphQLInputObject};

#[derive(Default, Debug, Clone)]
pub struct Coupon {
    pub coupon_id: String,
    pub code: String,
    pub discount_type: String,
    pub discount_value: i32,
    pub discount_amount_paise: String,
    pub final_amount_paise: String,
    pub is_valid: bool,
    pub reason: String,
}

#[graphql_object]
#[graphql(description = "Coupon validation/application result")]
impl Coupon {
    async fn coupon_id(&self) -> &String {
        &self.coupon_id
    }
    async fn code(&self) -> &String {
        &self.code
    }
    /// "percentage" or "fixed_amount"
    async fn discount_type(&self) -> &String {
        &self.discount_type
    }
    async fn discount_value(&self) -> i32 {
        self.discount_value
    }
    async fn discount_amount_paise(&self) -> &String {
        &self.discount_amount_paise
    }
    async fn final_amount_paise(&self) -> &String {
        &self.final_amount_paise
    }
    async fn is_valid(&self) -> bool {
        self.is_valid
    }
    async fn reason(&self) -> &String {
        &self.reason
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Validate a coupon code")]
pub struct ValidateCoupon {
    pub code: String,
    /// Cart/order gross total in paise (1 INR = 100 paise)
    pub order_amount_paise: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Apply a coupon code (increments usage count)")]
pub struct ApplyCoupon {
    pub code: String,
    pub order_amount_paise: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Admin: create a coupon")]
pub struct CreateCouponInput {
    pub code: String,
    /// \"percentage\" | \"fixed_amount\"
    pub discount_type: String,
    pub discount_value: i32,
    pub min_order_value_paise: Option<i32>,
    pub usage_limit: Option<i32>,
    pub max_uses_per_customer: Option<i32>,
    /// RFC3339 timestamp
    pub starts_at: String,
    /// RFC3339 timestamp
    pub ends_at: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Admin: update a coupon")]
pub struct UpdateCouponInput {
    pub coupon_id: String,
    /// \"active\" | \"inactive\"
    pub status: Option<String>,
    pub usage_limit: Option<i32>,
    /// RFC3339 timestamp
    pub ends_at: Option<String>,
}
