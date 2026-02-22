use juniper::{graphql_object, FieldResult, GraphQLInputObject};

use crate::resolvers::order_details::schema::{OrderDetails, SearchOrderDetails};

#[derive(Default, Debug, Clone)]
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

    async fn total_amount(&self) -> &String {
        &self.total_amount
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
    pub user_id: String,
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
