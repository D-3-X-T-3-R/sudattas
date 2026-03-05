use juniper::{graphql_object, GraphQLInputObject};

#[derive(Default, Debug, Clone)]
pub struct ProductAttributeMapping {
    pub product_id: String,
    pub attribute_id: String,
}

#[graphql_object]
#[graphql(description = "Mapping between product and attribute")]
impl ProductAttributeMapping {
    async fn product_id(&self) -> &String {
        &self.product_id
    }

    async fn attribute_id(&self) -> &String {
        &self.attribute_id
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Create a product-attribute mapping")]
pub struct NewProductAttributeMapping {
    pub product_id: String,
    pub attribute_id: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Search product-attribute mapping")]
pub struct SearchProductAttributeMappingInput {
    pub product_id: String,
    pub attribute_id: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Delete a product-attribute mapping")]
pub struct DeleteProductAttributeMappingInput {
    pub product_id: String,
    pub attribute_id: String,
}
