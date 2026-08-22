use juniper::{graphql_object, GraphQLInputObject};

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
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
#[graphql(description = "Preview a coupon code for the current amount without redeeming it")]
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

#[derive(Default, Debug, Clone)]
pub struct CouponAdmin {
    pub coupon_id: String,
    pub code: String,
    pub discount_type: String,
    pub discount_value: i32,
    pub min_order_value_paise: Option<i32>,
    pub usage_limit: Option<i32>,
    pub usage_count: Option<i32>,
    pub max_uses_per_customer: Option<i32>,
    /// "active" | "inactive"
    pub status: String,
    /// RFC3339
    pub starts_at: String,
    /// RFC3339
    pub ends_at: Option<String>,
}

#[graphql_object]
#[graphql(description = "Admin view of a coupon, including usage and status")]
impl CouponAdmin {
    async fn coupon_id(&self) -> &String {
        &self.coupon_id
    }
    async fn code(&self) -> &String {
        &self.code
    }
    async fn discount_type(&self) -> &String {
        &self.discount_type
    }
    async fn discount_value(&self) -> i32 {
        self.discount_value
    }
    async fn min_order_value_paise(&self) -> Option<i32> {
        self.min_order_value_paise
    }
    async fn usage_limit(&self) -> Option<i32> {
        self.usage_limit
    }
    async fn usage_count(&self) -> Option<i32> {
        self.usage_count
    }
    async fn max_uses_per_customer(&self) -> Option<i32> {
        self.max_uses_per_customer
    }
    async fn status(&self) -> &String {
        &self.status
    }
    async fn starts_at(&self) -> &String {
        &self.starts_at
    }
    async fn ends_at(&self) -> &Option<String> {
        &self.ends_at
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Admin: search coupons")]
pub struct SearchCouponAdminInput {
    /// Filter by specific coupon ID; omit to return all
    pub coupon_id: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Admin: delete a coupon")]
pub struct DeleteCouponAdminInput {
    pub coupon_id: String,
}

/// Customer-safe view of a currently-usable coupon — no usage-count/limit internals, just what a
/// customer needs to decide whether to apply it. Does not account for per-cart scope/per-customer
/// eligibility; that's still checked for real when a code is actually applied at checkout.
#[derive(Default, Debug, Clone)]
pub struct PublicCoupon {
    pub coupon_id: String,
    pub code: String,
    /// "percentage" | "fixed_amount"
    pub discount_type: String,
    pub discount_value: i32,
    pub min_order_value_paise: Option<i32>,
    /// RFC3339; absent if the coupon never expires.
    pub ends_at: Option<String>,
}

#[graphql_object]
#[graphql(description = "A currently-active, customer-visible coupon")]
impl PublicCoupon {
    async fn coupon_id(&self) -> &String {
        &self.coupon_id
    }
    async fn code(&self) -> &String {
        &self.code
    }
    async fn discount_type(&self) -> &String {
        &self.discount_type
    }
    async fn discount_value(&self) -> i32 {
        self.discount_value
    }
    async fn min_order_value_paise(&self) -> Option<i32> {
        self.min_order_value_paise
    }
    async fn ends_at(&self) -> &Option<String> {
        &self.ends_at
    }
}
