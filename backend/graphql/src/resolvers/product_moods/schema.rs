use juniper::{graphql_object, GraphQLInputObject};

#[derive(Default, Debug, Clone)]
pub struct ProductMood {
    pub mood_id: String,
    pub mood_name: String,
}

#[graphql_object]
#[graphql(description = "Product mood (id + name only)")]
impl ProductMood {
    async fn mood_id(&self) -> &String {
        &self.mood_id
    }

    async fn mood_name(&self) -> &String {
        &self.mood_name
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Create a product mood")]
pub struct NewProductMood {
    pub mood_name: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Search product moods")]
pub struct SearchProductMoodInput {
    pub mood_id: Option<String>,
    pub mood_name: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Update a product mood")]
pub struct ProductMoodMutation {
    pub mood_id: String,
    pub mood_name: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Delete a product mood")]
pub struct DeleteProductMoodInput {
    pub mood_id: String,
}
