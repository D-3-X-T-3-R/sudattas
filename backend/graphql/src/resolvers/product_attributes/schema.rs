use juniper::{graphql_object, GraphQLInputObject};

#[derive(Default, Debug, Clone)]
pub struct ProductAttribute {
    pub attribute_id: String,
    pub attribute_name: String,
    pub attribute_value: String,
}

#[graphql_object]
#[graphql(description = "Product attribute (name/value pair)")]
impl ProductAttribute {
    async fn attribute_id(&self) -> &String {
        &self.attribute_id
    }

    async fn attribute_name(&self) -> &String {
        &self.attribute_name
    }

    async fn attribute_value(&self) -> &String {
        &self.attribute_value
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Create a product attribute")]
pub struct NewProductAttribute {
    pub attribute_name: String,
    pub attribute_value: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Search product attributes")]
pub struct SearchProductAttributeInput {
    pub attribute_id: Option<String>,
    pub attribute_name: Option<String>,
    pub attribute_value: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Update a product attribute")]
pub struct ProductAttributeMutation {
    pub attribute_id: String,
    pub attribute_name: Option<String>,
    pub attribute_value: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Delete a product attribute")]
pub struct DeleteProductAttributeInput {
    pub attribute_id: String,
}
