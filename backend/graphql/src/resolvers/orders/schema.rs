use juniper::{graphql_object, FieldResult, GraphQLInputObject};

use crate::resolvers::money::{money_from_major_string, Money};
use crate::resolvers::order_details::schema::{OrderDetails, SearchOrderDetails};

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Order {
    pub user_id: String,
    pub order_date: String,
    pub shipping_address_id: String,
    pub total_amount: String,
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

    /// Legacy string (major units); prefer total_amount_money for integer paise + formatted.
    async fn total_amount(&self) -> &String {
        &self.total_amount
    }

    /// Money type: amount_paise (integer), currency, formatted string (avoids float).
    async fn total_amount_money(&self) -> Money {
        money_from_major_string(&self.total_amount)
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
            product_id: None,
            quantity: None,
            price_start: None,
            price_end: None,
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
    pub total_amount: String,
    pub status_id: String,
    pub order_id: String,
}
