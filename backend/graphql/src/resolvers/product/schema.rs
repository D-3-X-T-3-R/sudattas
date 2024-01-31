use juniper::{graphql_object, FieldResult, GraphQLInputObject};

use crate::resolvers::{
    category::schema::{Category, SearchCategory},
    product_images::schema::{ProductImage, SearchProductImage},
};

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

    async fn category_details(&self) -> FieldResult<Vec<Category>> {
        crate::resolvers::category::handlers::search_category(SearchCategory {
            category_id: match &self.category_id {
                Some(val) => Some(val.to_string()),
                None => None,
            },
            name: None,
        })
        .await
        .map_err(|e| e.into())
    }

    async fn images(&self) -> FieldResult<Vec<ProductImage>> {
        crate::resolvers::product_images::handlers::search_product_image(SearchProductImage {
            product_id: Some(self.product_id.to_string()),
            image_id: None,
            alt_text: None,
        })
        .await
        .map_err(|e| e.into())
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
