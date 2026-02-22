use juniper::{graphql_object, FieldResult, GraphQLInputObject};

use crate::resolvers::product::schema::{Product, SearchProduct};

#[derive(Default, Debug, Clone)]
pub struct ProductImage {
    pub image_id: String,
    pub product_id: String,
    pub image_base64: String,
    pub alt_text: Option<String>,
    pub url: Option<String>,
    pub cdn_path: Option<String>,
    pub thumbnail_url: Option<String>,
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

    async fn url(&self) -> &Option<String> {
        &self.url
    }

    async fn cdn_path(&self) -> &Option<String> {
        &self.cdn_path
    }

    async fn thumbnail_url(&self) -> &Option<String> {
        &self.thumbnail_url
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

#[derive(Default, Debug, Clone)]
pub struct PresignedUploadUrl {
    pub upload_url: String,
    pub key: String,
    pub cdn_url: String,
}

#[graphql_object]
#[graphql(description = "Presigned R2 upload URL")]
impl PresignedUploadUrl {
    async fn upload_url(&self) -> &String {
        &self.upload_url
    }
    async fn key(&self) -> &String {
        &self.key
    }
    async fn cdn_url(&self) -> &String {
        &self.cdn_url
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Request a presigned upload URL")]
pub struct GetPresignedUploadUrl {
    pub product_id: String,
    pub filename: String,
    pub content_type: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Confirm an image upload to R2")]
pub struct ConfirmImageUpload {
    pub product_id: String,
    pub key: String,
    pub alt_text: Option<String>,
    pub display_order: Option<i32>,
}
