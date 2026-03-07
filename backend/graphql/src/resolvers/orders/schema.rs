use juniper::{graphql_object, FieldResult, GraphQLInputObject};

use crate::resolvers::money::{money_from_paise, Money};
use crate::resolvers::order_details::schema::{OrderDetails, SearchOrderDetails};

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Order {
    pub user_id: String,
    pub order_date: String,
    pub shipping_address_id: String,
    pub total_amount_paise: i64,
    pub total_amount_formatted: String,
    pub status_id: String,
    pub order_id: String,
}

#[graphql_object]
#[graphql(description = "Order Data")]
impl Order {
    async fn user_id(&self) -> &String {
        &self.user_id
    }

    async fn order_date(&self) -> &String {
        &self.order_date
    }

    async fn shipping_address_id(&self) -> &String {
        &self.shipping_address_id
    }

    /// Total in paise (integer minor units).
    async fn total_amount_paise(&self) -> String {
        self.total_amount_paise.to_string()
    }

    /// Formatted display string (e.g. "₹499.00").
    async fn total_amount_formatted(&self) -> &String {
        &self.total_amount_formatted
    }

    /// Money type: amount_paise, currency, formatted (avoids float).
    async fn total_amount_money(&self) -> Money {
        money_from_paise(self.total_amount_paise, Some("INR"))
    }

    async fn status_id(&self) -> &String {
        &self.status_id
    }

    async fn order_id(&self) -> &String {
        &self.order_id
    }

    async fn order_details(&self) -> FieldResult<Vec<OrderDetails>> {
        crate::resolvers::order_details::handlers::search_order_detail(SearchOrderDetails {
            order_id: Some(self.order_id.to_string()),
            order_detail_id: None,
            variant_id: None,
            quantity: None,
            price_start_paise: None,
            price_end_paise: None,
        })
        .await
        .map_err(|e| e.into())
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "New Order Data")]
pub struct NewOrder {
    /// `user_id` is intentionally absent — it is taken from the authenticated JWT claim.
    /// Clients cannot impersonate another user at checkout.
    pub shipping_address_id: String,
    /// Optional coupon code to apply a discount at checkout
    pub coupon_code: Option<String>,
}

/// Order status row (from OrderStatus table) for admin dropdowns.
#[derive(Default, Debug, Clone)]
pub struct OrderStatus {
    pub status_id: String,
    pub status_name: String,
}

#[graphql_object]
#[graphql(description = "Order status (for admin filters)")]
impl OrderStatus {
    async fn status_id(&self) -> &String {
        &self.status_id
    }
    async fn status_name(&self) -> &String {
        &self.status_name
    }
}

#[derive(Default, Debug, Clone, GraphQLInputObject)]
pub struct SearchOrder {
    pub user_id: String,
    pub order_date_start: Option<String>,
    pub order_date_end: Option<String>,
    pub status_id: Option<String>,
    pub order_id: Option<String>,
    /// Maximum number of results to return
    pub limit: Option<String>,
    /// Number of results to skip for pagination
    pub offset: Option<String>,
}

#[derive(Default, Debug, Clone, GraphQLInputObject)]
pub struct OrderMutation {
    pub user_id: String,
    pub order_date: String,
    pub shipping_address_id: String,
    /// Total in paise
    pub total_amount_paise: String,
    pub status_id: String,
    pub order_id: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Create an order (admin/low-level)")]
pub struct CreateOrderInput {
    pub user_id: String,
    pub shipping_address_id: String,
    pub status_id: String,
    /// Legacy total in paise
    pub total_amount_paise: String,
    pub subtotal_minor: Option<String>,
    pub shipping_minor: Option<String>,
    pub tax_total_minor: Option<String>,
    pub discount_total_minor: Option<String>,
    pub grand_total_minor: Option<String>,
    pub applied_coupon_id: Option<String>,
    pub applied_coupon_code: Option<String>,
    pub applied_discount_paise: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Admin: mark order shipped")]
pub struct AdminMarkOrderShippedInput {
    pub order_id: String,
    pub awb_code: Option<String>,
    pub carrier: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Admin: mark order delivered")]
pub struct AdminMarkOrderDeliveredInput {
    pub order_id: String,
}
