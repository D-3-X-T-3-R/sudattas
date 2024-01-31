use juniper::{graphql_object, FieldResult, GraphQLInputObject};

use crate::resolvers::product::schema::{Product, SearchProduct};

#[derive(Default, Debug, Clone)]
pub struct ProductImage {
    pub image_id: String,
    pub product_id: String,
    pub image_base64: String,
    pub alt_text: Option<String>,
}

#[graphql_object]
#[graphql(description = "ProductImage Data")]
impl ProductImage {
    async fn image_id(&self) -> &String {
        &self.image_id
    }

    async fn product_id(&self) -> &String {
        &self.product_id
    }

    async fn image_base64(&self) -> &String {
        &self.image_base64
    }

    async fn alt_text(&self) -> &Option<String> {
        &self.alt_text
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
#[graphql(description = "New ProductImage Data")]
pub struct NewProductImage {
    pub product_id: String,
    pub product_images: Vec<ProductImageList>,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "New ProductImageList Data")]
pub struct ProductImageList {
    pub image_base64: String,
    pub alt_text: Option<String>,
}

#[derive(Default, Debug, Clone, GraphQLInputObject)]
pub struct SearchProductImage {
    pub image_id: Option<String>,
    pub product_id: Option<String>,
    pub alt_text: Option<String>,
}

#[derive(Default, Debug, Clone, GraphQLInputObject)]
pub struct ProductImageMutation {
    pub image_id: String,
    pub product_id: String,
    pub image_base64: String,
    pub alt_text: Option<String>,
}
