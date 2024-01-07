use juniper::{graphql_object, FieldResult, GraphQLInputObject};

use crate::resolvers::{
    category::schema::{Category, SearchCategory},
    product::schema::{Product, SearchProduct},
};

#[derive(Default, Debug, Clone)]
pub struct OrderDetails {
    pub order_id: String,
    pub product_id: String,
    pub quantity: String,
    pub price: String,
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

    async fn price(&self) -> &String {
        &self.price
    }

    async fn order_detail_id(&self) -> &String {
        &self.order_detail_id
    }

    async fn product_details(&self) -> FieldResult<Vec<Product>> {
        crate::resolvers::product::handlers::search_product(SearchProduct {
            product_id: Some(self.product_id.to_string()),
            name: None,
            description: None,
            starting_price: None,
            ending_price: None,
            stock_quantity: None,
            category_id: None,
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
    pub price: String,
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
    pub price_start: Option<String>,
    pub price_end: Option<String>,
}

#[derive(Default, Debug, Clone, GraphQLInputObject)]
pub struct OrderDetailsMutation {
    pub order_id: String,
    pub product_id: String,
    pub quantity: String,
    pub price: String,
    pub order_detail_id: String,
}
