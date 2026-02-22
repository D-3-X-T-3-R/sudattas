use juniper::{graphql_object, GraphQLInputObject};

#[derive(Default, Debug, Clone)]
pub struct ShippingZone {
    pub zone_id: String,
    pub zone_name: String,
    pub description: String,
}

#[graphql_object]
#[graphql(description = "Shipping zone")]
impl ShippingZone {
    async fn zone_id(&self) -> &String {
        &self.zone_id
    }
    async fn zone_name(&self) -> &String {
        &self.zone_name
    }
    async fn description(&self) -> &String {
        &self.description
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Create a shipping zone")]
pub struct NewShippingZone {
    pub zone_name: String,
    pub description: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Update a shipping zone")]
pub struct ShippingZoneMutation {
    pub zone_id: String,
    pub zone_name: Option<String>,
    pub description: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Search shipping zones")]
pub struct SearchShippingZone {
    pub zone_id: Option<String>,
}
