use juniper::{graphql_object, GraphQLInputObject};

#[derive(Default, Debug, Clone)]
pub struct Product {
    pub product_id: String,
    pub name: String,
    pub description: Option<String>,
    pub price: String,
    pub stock_quantity: Option<String>,
    pub category_id: Option<String>,
}

#[graphql_object]
#[graphql(description = "Product Data")]
impl Product {
    async fn product_id(&self) -> &String {
        &self.product_id
    }

    async fn name(&self) -> &String {
        &self.name
    }

    async fn description(&self) -> &Option<String> {
        &self.description
    }

    async fn price(&self) -> &String {
        &self.price
    }

    async fn stock_quantity(&self) -> &Option<String> {
        &self.stock_quantity
    }

    async fn category_id(&self) -> &Option<String> {
        &self.category_id
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "New Product Data")]
pub struct NewProduct {
    pub name: String,
    pub description: String,
    pub price: String,
    pub stock_quantity: String,
    pub category_id: String,
}

#[derive(Default, Debug, Clone, GraphQLInputObject)]
pub struct SearchProduct {
    pub product_id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub starting_price: Option<String>,
    pub ending_price: Option<String>,
    pub stock_quantity: Option<String>,
    pub category_id: Option<String>,
}

#[derive(Default, Debug, Clone, GraphQLInputObject)]
pub struct ProductMutation {
    pub product_id: String,
    pub name: String,
    pub description: String,
    pub price: String,
    pub stock_quantity: String,
    pub category_id: String,
}
