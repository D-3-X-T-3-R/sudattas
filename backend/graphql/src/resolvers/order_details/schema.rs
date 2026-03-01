use juniper::{graphql_object, FieldResult, GraphQLInputObject};

use crate::resolvers::money::{money_from_paise, Money};
use crate::resolvers::product::schema::{Product, SearchProduct};

#[derive(Default, Debug, Clone)]
pub struct OrderDetails {
    pub order_id: String,
    pub product_id: String,
    pub quantity: String,
    pub price_paise: i64,
    pub price_formatted: String,
    pub order_detail_id: String,
}

#[graphql_object]
#[graphql(description = "Order Details Data")]
impl OrderDetails {
    async fn order_id(&self) -> &String {
        &self.order_id
    }

    async fn product_id(&self) -> &String {
        &self.product_id
    }

    async fn quantity(&self) -> &String {
        &self.quantity
    }

    /// Line price in paise (integer minor units).
    async fn price_paise(&self) -> String {
        self.price_paise.to_string()
    }

    async fn price_formatted(&self) -> &String {
        &self.price_formatted
    }

    /// Money type: amount_paise, currency, formatted.
    async fn price_money(&self) -> Money {
        money_from_paise(self.price_paise, Some("INR"))
    }

    async fn order_detail_id(&self) -> &String {
        &self.order_detail_id
    }

    async fn product_details(&self) -> FieldResult<Vec<Product>> {
        crate::resolvers::product::handlers::search_product(SearchProduct {
            product_id: Some(self.product_id.to_string()),
            name: None,
            description: None,
            starting_price_paise: None,
            ending_price_paise: None,
            stock_quantity: None,
            category_id: None,
            limit: None,
            offset: None,
        })
        .await
        .map_err(|e| e.into())
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "New Order Details Data")]
pub struct NewOrderDetail {
    pub order_id: String,
    pub product_id: String,
    pub quantity: String,
    /// Line total in paise
    pub price_paise: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "New Order Details Data")]
pub struct NewOrderDetails {
    pub order_details: Vec<NewOrderDetail>,
}

#[derive(Default, Debug, Clone, GraphQLInputObject)]
pub struct SearchOrderDetails {
    pub order_detail_id: Option<String>,
    pub order_id: Option<String>,
    pub product_id: Option<String>,
    pub quantity: Option<String>,
    pub price_start_paise: Option<String>,
    pub price_end_paise: Option<String>,
}

#[derive(Default, Debug, Clone, GraphQLInputObject)]
pub struct OrderDetailsMutation {
    pub order_id: String,
    pub product_id: String,
    pub quantity: String,
    pub price_paise: String,
    pub order_detail_id: String,
}
