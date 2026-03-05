use juniper::{graphql_object, GraphQLInputObject};

#[derive(Default, Debug, Clone)]
pub struct InventoryLog {
    pub log_id: String,
    pub variant_id: String,
    pub change_quantity: String,
    pub log_time: String,
    pub reason: String,
}

#[graphql_object]
#[graphql(description = "Inventory log entry")]
impl InventoryLog {
    async fn log_id(&self) -> &String {
        &self.log_id
    }

    async fn variant_id(&self) -> &String {
        &self.variant_id
    }

    async fn change_quantity(&self) -> &String {
        &self.change_quantity
    }

    async fn log_time(&self) -> &String {
        &self.log_time
    }

    async fn reason(&self) -> &String {
        &self.reason
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Create an inventory log entry")]
pub struct NewInventoryLog {
    pub variant_id: String,
    pub change_quantity: String,
    pub reason: String,
    pub actor_id: Option<String>,
    pub quantity_before: Option<String>,
    pub quantity_after: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Search inventory logs")]
pub struct SearchInventoryLogInput {
    pub log_id: Option<String>,
    pub variant_id: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Update an inventory log entry")]
pub struct InventoryLogMutation {
    pub log_id: String,
    pub variant_id: Option<String>,
    pub change_quantity: Option<String>,
    pub reason: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Delete an inventory log entry")]
pub struct DeleteInventoryLogInput {
    pub log_id: String,
}
