use juniper::{graphql_object, GraphQLInputObject};

#[derive(Default, Debug, Clone)]
pub struct Occasion {
    pub occasion_id: String,
    pub occasion_name: String,
}

#[graphql_object]
#[graphql(description = "Occasion")]
impl Occasion {
    async fn occasion_id(&self) -> &String {
        &self.occasion_id
    }

    async fn occasion_name(&self) -> &String {
        &self.occasion_name
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Create an occasion")]
pub struct NewOccasion {
    pub occasion_name: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Search occasion by ID")]
pub struct SearchOccasionInput {
    pub occasion_id: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Update an occasion")]
pub struct OccasionMutation {
    pub occasion_id: String,
    pub occasion_name: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Delete an occasion")]
pub struct DeleteOccasionInput {
    pub occasion_id: String,
}

