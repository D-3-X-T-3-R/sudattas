use juniper::{graphql_object, GraphQLInputObject};

#[derive(Default, Debug, Clone)]
pub struct ShippingAddress {
    pub shipping_address_id: String,
    pub country_id: String,
    pub state_id: String,
    pub city_id: String,
    pub road: String,
    pub apartment_no_or_name: String,
}

#[graphql_object]
#[graphql(description = "Shipping address")]
impl ShippingAddress {
    async fn shipping_address_id(&self) -> &String {
        &self.shipping_address_id
    }
    async fn country_id(&self) -> &String {
        &self.country_id
    }
    async fn state_id(&self) -> &String {
        &self.state_id
    }
    async fn city_id(&self) -> &String {
        &self.city_id
    }
    async fn road(&self) -> &String {
        &self.road
    }
    async fn apartment_no_or_name(&self) -> &String {
        &self.apartment_no_or_name
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Create a shipping address")]
pub struct NewShippingAddress {
    pub country_id: String,
    pub state_id: String,
    pub city_id: String,
    pub road: String,
    pub apartment_no_or_name: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Update a shipping address")]
pub struct ShippingAddressMutation {
    pub shipping_address_id: String,
    pub country_id: String,
    pub state_id: String,
    pub city_id: String,
    pub road: String,
    pub apartment_no_or_name: String,
}
