use juniper::{graphql_object, GraphQLInputObject};

#[derive(Default, Debug, Clone)]
pub struct ShippingMethod {
    pub method_id: String,
    pub method_name: String,
    pub cost: f64,
    pub estimated_delivery_time: String,
}

#[graphql_object]
#[graphql(description = "Shipping method")]
impl ShippingMethod {
    async fn method_id(&self) -> &String {
        &self.method_id
    }
    async fn method_name(&self) -> &String {
        &self.method_name
    }
    async fn cost(&self) -> f64 {
        self.cost
    }
    async fn estimated_delivery_time(&self) -> &String {
        &self.estimated_delivery_time
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Create a shipping method")]
pub struct NewShippingMethod {
    pub method_name: String,
    pub cost: f64,
    pub estimated_delivery_time: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Update a shipping method")]
pub struct ShippingMethodMutation {
    pub method_id: String,
    pub method_name: Option<String>,
    pub cost: Option<f64>,
    pub estimated_delivery_time: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Search shipping methods")]
pub struct SearchShippingMethod {
    pub method_id: Option<String>,
}
