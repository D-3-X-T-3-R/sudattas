use juniper::{graphql_object, GraphQLInputObject};

#[derive(Default, Debug, Clone)]
pub struct UserRole {
    pub role_id: String,
    pub role_name: String,
}

#[graphql_object]
#[graphql(description = "User role")]
impl UserRole {
    async fn role_id(&self) -> &String {
        &self.role_id
    }

    async fn role_name(&self) -> &String {
        &self.role_name
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Create a user role")]
pub struct NewUserRole {
    pub role_name: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Search a user role by ID")]
pub struct SearchUserRoleInput {
    pub role_id: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Update a user role")]
pub struct UserRoleMutation {
    pub role_id: String,
    pub role_name: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Delete a user role")]
pub struct DeleteUserRoleInput {
    pub role_id: String,
}
