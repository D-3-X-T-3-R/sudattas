//! P2 Data retention: export current user's PII (no password).

use juniper::GraphQLObject;

#[derive(GraphQLObject)]
#[graphql(description = "Current user PII export (data retention / GDPR)")]
pub struct UserPiiExport {
    pub user_id: i32,
    pub email: String,
    pub full_name: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub create_date: String,
}
