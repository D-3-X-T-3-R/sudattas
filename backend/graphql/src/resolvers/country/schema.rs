use juniper::{graphql_object, GraphQLInputObject};



#[derive(Default, Debug, Clone)]
pub struct Country {
    pub country_id: String,
    pub country_name: String,
}

#[graphql_object]
#[graphql(description = "Country Data")]
impl Country {
    async fn country_id(&self) -> &String {
        &self.country_id
    }

    async fn country_name(&self) -> &String {
        &self.country_name
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "New Country Data")]
pub struct NewCountry {
    pub country_name: String,
}

#[derive(Default, Debug, Clone, GraphQLInputObject)]
pub struct SearchCountry {
    pub country_id: Option<String>,
    pub country_name: Option<String>,
}

#[derive(Default, Debug, Clone, GraphQLInputObject)]
pub struct CountryMutation {
    pub country_id: String,
    pub country_name: String,
}
