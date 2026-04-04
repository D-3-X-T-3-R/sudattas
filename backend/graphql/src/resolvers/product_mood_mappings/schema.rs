use juniper::{graphql_object, GraphQLInputObject};

#[derive(Default, Debug, Clone)]
pub struct ProductMoodMapping {
    pub product_id: String,
    pub mood_id: String,
}

#[graphql_object]
#[graphql(description = "Mapping between product and mood")]
impl ProductMoodMapping {
    async fn product_id(&self) -> &String {
        &self.product_id
    }

    async fn mood_id(&self) -> &String {
        &self.mood_id
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Create a product-mood mapping")]
pub struct NewProductMoodMapping {
    pub product_id: String,
    pub mood_id: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(
    description = "Search product-mood mapping (omit mood_id to list all moods for a product)"
)]
pub struct SearchProductMoodMappingInput {
    pub product_id: String,
    pub mood_id: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Delete a product-mood mapping")]
pub struct DeleteProductMoodMappingInput {
    pub product_id: String,
    pub mood_id: String,
}
