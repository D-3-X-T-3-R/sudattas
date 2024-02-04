use juniper::{graphql_object, GraphQLInputObject};

#[derive(Default, Debug, Clone)]
pub struct ShippingZone {
    pub zone_id: String,
    pub zip_code: String,
    pub description: Option<String>,
}

#[graphql_object]
#[graphql(description = "ShippingZone Data")]
impl ShippingZone {
    async fn zone_id(&self) -> &String {
        &self.zone_id
    }

    async fn zip_code(&self) -> &String {
        &self.zip_code
    }

    async fn description(&self) -> &Option<String> {
        &self.description
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "New ShippingZone Data")]
pub struct NewShippingZone {
    pub zip_code: String,
    pub description: Option<String>,
}

#[derive(Default, Debug, Clone, GraphQLInputObject)]
pub struct SearchShippingZone {
    pub zone_id: Option<String>,
    pub zip_code: Option<String>,
}

#[derive(Default, Debug, Clone, GraphQLInputObject)]
pub struct ShippingZoneMutation {
    pub zone_id: String,
    pub zip_code: String,
    pub description: Option<String>,
}
