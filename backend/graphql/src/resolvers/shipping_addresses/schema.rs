use juniper::{graphql_object, GraphQLInputObject};

#[derive(Default, Debug, Clone)]
pub struct ShippingAddress {
    pub shipping_address_id: String,
    pub user_id: Option<String>,
    pub country: String,
    pub state_region: String,
    pub city: String,
    pub postal_code: String,
    pub road: Option<String>,
    pub apartment_no_or_name: Option<String>,
}

#[graphql_object]
#[graphql(description = "Shipping address")]
impl ShippingAddress {
    async fn shipping_address_id(&self) -> &String {
        &self.shipping_address_id
    }
    async fn user_id(&self) -> &Option<String> {
        &self.user_id
    }
    async fn country(&self) -> &String {
        &self.country
    }
    async fn state_region(&self) -> &String {
        &self.state_region
    }
    async fn city(&self) -> &String {
        &self.city
    }
    async fn postal_code(&self) -> &String {
        &self.postal_code
    }
    async fn road(&self) -> &Option<String> {
        &self.road
    }
    async fn apartment_no_or_name(&self) -> &Option<String> {
        &self.apartment_no_or_name
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Create a shipping address")]
pub struct NewShippingAddress {
    pub user_id: Option<String>,
    pub country: String,
    pub state_region: String,
    pub city: String,
    pub postal_code: String,
    pub road: Option<String>,
    pub apartment_no_or_name: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Update a shipping address")]
pub struct ShippingAddressMutation {
    pub shipping_address_id: String,
    pub user_id: Option<String>,
    pub country: String,
    pub state_region: String,
    pub city: String,
    pub postal_code: String,
    pub road: Option<String>,
    pub apartment_no_or_name: Option<String>,
}
