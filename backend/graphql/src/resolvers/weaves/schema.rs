use juniper::{graphql_object, GraphQLInputObject};

#[derive(Default, Debug, Clone)]
pub struct Weave {
    pub weave_id: String,
    pub weave_name: String,
}

#[graphql_object]
#[graphql(description = "Weave")]
impl Weave {
    async fn weave_id(&self) -> &String {
        &self.weave_id
    }

    async fn weave_name(&self) -> &String {
        &self.weave_name
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Create a weave")]
pub struct NewWeave {
    pub weave_name: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Search weave by ID")]
pub struct SearchWeaveInput {
    pub weave_id: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Update a weave")]
pub struct WeaveMutation {
    pub weave_id: String,
    pub weave_name: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Delete a weave")]
pub struct DeleteWeaveInput {
    pub weave_id: String,
}

