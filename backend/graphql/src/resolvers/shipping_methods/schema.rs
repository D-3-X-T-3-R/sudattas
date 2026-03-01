use juniper::{graphql_object, GraphQLInputObject};

#[derive(Default, Debug, Clone)]
pub struct ShippingMethod {
    pub method_id: String,
    pub method_name: String,
    pub cost_paise: i64,
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
    /// Cost in paise (integer minor units)
    async fn cost_paise(&self) -> String {
        self.cost_paise.to_string()
    }
    async fn estimated_delivery_time(&self) -> &String {
        &self.estimated_delivery_time
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Create a shipping method")]
pub struct NewShippingMethod {
    pub method_name: String,
    pub cost_paise: String,
    pub estimated_delivery_time: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Update a shipping method")]
pub struct ShippingMethodMutation {
    pub method_id: String,
    pub method_name: Option<String>,
    pub cost_paise: Option<String>,
    pub estimated_delivery_time: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Search shipping methods")]
pub struct SearchShippingMethod {
    pub method_id: Option<String>,
}
