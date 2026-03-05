use juniper::{graphql_object, GraphQLInputObject};

#[derive(Default, Debug, Clone)]
pub struct User {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub auth_provider: String,
    pub full_name: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub create_date: String,
}

#[graphql_object]
#[graphql(description = "User account")]
impl User {
    async fn user_id(&self) -> &String {
        &self.user_id
    }

    async fn username(&self) -> &String {
        &self.username
    }

    async fn email(&self) -> &String {
        &self.email
    }

    /// \"email\" | \"google\"
    async fn auth_provider(&self) -> &String {
        &self.auth_provider
    }

    async fn full_name(&self) -> &Option<String> {
        &self.full_name
    }

    async fn address(&self) -> &Option<String> {
        &self.address
    }

    async fn phone(&self) -> &Option<String> {
        &self.phone
    }

    async fn create_date(&self) -> &String {
        &self.create_date
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Create a new user account")]
pub struct NewUser {
    pub username: String,
    pub email: String,
    /// \"email\" | \"google\"
    pub auth_provider: String,
    /// Required when auth_provider = \"email\"
    pub password_plain: Option<String>,
    /// Required when auth_provider = \"google\"
    pub google_sub: Option<String>,
    pub full_name: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub role_id: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Update an existing user account")]
pub struct UpdateUserInput {
    pub user_id: String,
    pub username: Option<String>,
    pub email: Option<String>,
    pub password_plain: Option<String>,
    pub google_sub: Option<String>,
    pub full_name: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub role_id: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Search for a user by ID")]
pub struct SearchUserInput {
    pub user_id: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Delete a user account by ID")]
pub struct DeleteUserInput {
    pub user_id: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Record a security audit event (P2)")]
pub struct RecordSecurityAuditEventInput {
    /// e.g. \"secrets_rotation\", \"config_reload\"
    pub event_type: String,
    pub details: Option<String>,
}
