use juniper::{graphql_object, GraphQLInputObject};

#[derive(Default, Debug, Clone)]
pub struct ProductVariant {
    pub variant_id: String,
    pub product_id: String,
    pub size_id: Option<String>,
    pub color_id: Option<String>,
    pub additional_price_paise: Option<String>,
}

#[graphql_object]
#[graphql(description = "Product variant")]
impl ProductVariant {
    async fn variant_id(&self) -> &String {
        &self.variant_id
    }

    async fn product_id(&self) -> &String {
        &self.product_id
    }

    async fn size_id(&self) -> &Option<String> {
        &self.size_id
    }

    async fn color_id(&self) -> &Option<String> {
        &self.color_id
    }

    async fn additional_price_paise(&self) -> &Option<String> {
        &self.additional_price_paise
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Create a product variant")]
pub struct NewProductVariant {
    pub product_id: String,
    pub size_id: Option<String>,
    pub color_id: Option<String>,
    pub additional_price_paise: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Update a product variant")]
pub struct ProductVariantMutation {
    pub variant_id: String,
    pub product_id: Option<String>,
    pub size_id: Option<String>,
    pub color_id: Option<String>,
    pub additional_price_paise: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Delete a product variant")]
pub struct DeleteProductVariantInput {
    pub variant_id: String,
}
