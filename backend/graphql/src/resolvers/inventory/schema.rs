use juniper::{graphql_object, GraphQLInputObject};

#[derive(Default, Debug, Clone)]
pub struct InventoryItem {
    pub inventory_id: String,
    pub product_id: String,
    pub quantity_available: String,
    pub reorder_level: String,
    pub supplier_id: String,
}

#[graphql_object]
#[graphql(description = "Inventory item")]
impl InventoryItem {
    async fn inventory_id(&self) -> &String {
        &self.inventory_id
    }
    async fn product_id(&self) -> &String {
        &self.product_id
    }
    async fn quantity_available(&self) -> &String {
        &self.quantity_available
    }
    async fn reorder_level(&self) -> &String {
        &self.reorder_level
    }
    async fn supplier_id(&self) -> &String {
        &self.supplier_id
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Create inventory item")]
pub struct NewInventoryItem {
    pub product_id: String,
    pub quantity_available: String,
    pub reorder_level: Option<String>,
    pub supplier_id: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Update inventory item")]
pub struct InventoryItemMutation {
    pub inventory_id: String,
    pub product_id: Option<String>,
    pub quantity_available: Option<String>,
    pub reorder_level: Option<String>,
    pub supplier_id: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Search inventory")]
pub struct SearchInventoryItem {
    /// Filter by inventory record ID
    pub inventory_id: Option<String>,
    /// Filter by product ID
    pub product_id: Option<String>,
}
