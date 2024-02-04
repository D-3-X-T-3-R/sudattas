use juniper::{graphql_object, GraphQLInputObject};

#[derive(Default, Debug, Clone)]
pub struct State {
    pub state_id: String,
    pub state_name: String,
}

#[graphql_object]
#[graphql(description = "State Data")]
impl State {
    async fn state_id(&self) -> &String {
        &self.state_id
    }

    async fn state_name(&self) -> &String {
        &self.state_name
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "New State Data")]
pub struct NewState {
    pub state_name: String,
}

#[derive(Default, Debug, Clone, GraphQLInputObject)]
pub struct SearchState {
    pub state_id: Option<String>,
    pub state_name: Option<String>,
}

#[derive(Default, Debug, Clone, GraphQLInputObject)]
pub struct StateMutation {
    pub state_id: String,
    pub state_name: String,
}
