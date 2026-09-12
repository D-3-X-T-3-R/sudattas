use juniper::{GraphQLInputObject, GraphQLObject};

#[derive(GraphQLObject, Default, Debug, Clone)]
#[graphql(
    description = "Admin-configurable schedule for the abandoned-cart recovery worker — DB-backed, so a change here takes effect on the worker's next poll with no restart"
)]
pub struct AbandonedCartSettings {
    pub enabled: bool,
    pub delay_hours: String,
    pub poll_interval_sec: String,
}

#[derive(GraphQLInputObject, Default, Debug, Clone)]
#[graphql(description = "Update one or more abandoned-cart worker settings; omitted fields are left unchanged")]
pub struct UpdateAbandonedCartSettingsInput {
    pub enabled: Option<bool>,
    pub delay_hours: Option<String>,
    pub poll_interval_sec: Option<String>,
}
