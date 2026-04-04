use juniper::{graphql_object, GraphQLInputObject};

#[derive(Default, Debug, Clone)]
pub struct Fabric {
    pub fabric_id: String,
    pub fabric_name: String,
}

#[graphql_object]
#[graphql(description = "Fabric")]
impl Fabric {
    async fn fabric_id(&self) -> &String {
        &self.fabric_id
    }

    async fn fabric_name(&self) -> &String {
        &self.fabric_name
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Create a fabric")]
pub struct NewFabric {
    pub fabric_name: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Search fabric by ID")]
pub struct SearchFabricInput {
    pub fabric_id: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Update a fabric")]
pub struct FabricMutation {
    pub fabric_id: String,
    pub fabric_name: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Delete a fabric")]
pub struct DeleteFabricInput {
    pub fabric_id: String,
}
