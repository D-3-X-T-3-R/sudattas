use juniper::{graphql_object, GraphQLInputObject};

#[derive(Default, Debug, Clone)]
pub struct Size {
    pub size_id: String,
    pub size_name: String,
}

#[graphql_object]
#[graphql(description = "Size")]
impl Size {
    async fn size_id(&self) -> &String {
        &self.size_id
    }

    async fn size_name(&self) -> &String {
        &self.size_name
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Create a size")]
pub struct NewSize {
    pub size_name: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Search size by ID")]
pub struct SearchSizeInput {
    pub size_id: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Update a size")]
pub struct SizeMutation {
    pub size_id: String,
    pub size_name: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Delete a size")]
pub struct DeleteSizeInput {
    pub size_id: String,
}
