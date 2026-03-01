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
    /// Price in minor units (paise); use formatted for display.
    pub amount_paise: i64,
    pub formatted: String,
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

    /// Price in paise (integer minor units). Use formatted for display.
    async fn amount_paise(&self) -> String {
        self.amount_paise.to_string()
    }

    async fn formatted(&self) -> &String {
        &self.formatted
    }

    async fn stock_quantity(&self) -> &Option<String> {
        &self.stock_quantity
    }

    async fn category_id(&self) -> &Option<String> {
        &self.category_id
    }

    async fn category_details(&self) -> FieldResult<Vec<Category>> {
        crate::resolvers::category::handlers::search_category(SearchCategory {
            category_id: self.category_id.as_ref().map(|val| val.to_string()),
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
    /// Price in paise (e.g. 49900 = ₹499.00)
    pub price_paise: String,
    pub stock_quantity: String,
    pub category_id: String,
}

#[derive(Default, Debug, Clone, GraphQLInputObject)]
pub struct SearchProduct {
    pub product_id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    /// Min price in paise (inclusive)
    pub starting_price_paise: Option<String>,
    /// Max price in paise (inclusive)
    pub ending_price_paise: Option<String>,
    pub stock_quantity: Option<String>,
    pub category_id: Option<String>,
    /// Maximum number of results to return (default: all)
    pub limit: Option<String>,
    /// Number of results to skip for pagination
    pub offset: Option<String>,
}

#[derive(Default, Debug, Clone, GraphQLInputObject)]
pub struct ProductMutation {
    pub product_id: String,
    pub name: String,
    pub description: String,
    pub price_paise: String,
    pub stock_quantity: String,
    pub category_id: String,
}
