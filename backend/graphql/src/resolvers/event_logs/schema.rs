use juniper::{graphql_object, GraphQLInputObject};

#[derive(Default, Debug, Clone)]
pub struct EventLog {
    pub log_id: String,
    pub event_type: String,
    pub event_description: String,
    pub user_id: String,
    pub event_time: String,
}

#[graphql_object]
#[graphql(description = "Event log entry")]
impl EventLog {
    async fn log_id(&self) -> &String {
        &self.log_id
    }

    async fn event_type(&self) -> &String {
        &self.event_type
    }

    async fn event_description(&self) -> &String {
        &self.event_description
    }

    async fn user_id(&self) -> &String {
        &self.user_id
    }

    async fn event_time(&self) -> &String {
        &self.event_time
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Create an event log entry")]
pub struct NewEventLog {
    pub event_type: String,
    pub event_description: String,
    pub user_id: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Search an event log by ID")]
pub struct SearchEventLogInput {
    pub log_id: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Update an event log entry")]
pub struct EventLogMutation {
    pub log_id: String,
    pub event_type: Option<String>,
    pub event_description: Option<String>,
    pub user_id: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Delete an event log entry")]
pub struct DeleteEventLogInput {
    pub log_id: String,
}
