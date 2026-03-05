use juniper::{graphql_object, GraphQLInputObject};

#[derive(Default, Debug, Clone)]
pub struct UserActivity {
    pub activity_id: String,
    pub user_id: String,
    pub activity_type: String,
    pub activity_time: String,
    pub activity_details: String,
}

#[graphql_object]
#[graphql(description = "User activity entry")]
impl UserActivity {
    async fn activity_id(&self) -> &String {
        &self.activity_id
    }

    async fn user_id(&self) -> &String {
        &self.user_id
    }

    async fn activity_type(&self) -> &String {
        &self.activity_type
    }

    async fn activity_time(&self) -> &String {
        &self.activity_time
    }

    async fn activity_details(&self) -> &String {
        &self.activity_details
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Create a user activity entry")]
pub struct NewUserActivity {
    pub user_id: String,
    pub activity_type: String,
    pub activity_details: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Search user activity by ID")]
pub struct SearchUserActivityInput {
    pub activity_id: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Update a user activity entry")]
pub struct UserActivityMutation {
    pub activity_id: String,
    pub user_id: Option<String>,
    pub activity_type: Option<String>,
    pub activity_details: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Delete a user activity entry")]
pub struct DeleteUserActivityInput {
    pub activity_id: String,
}
