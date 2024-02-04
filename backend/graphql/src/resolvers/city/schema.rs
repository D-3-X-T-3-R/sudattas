use juniper::{graphql_object, FieldResult, GraphQLInputObject};

use crate::resolvers::category::schema::{Category, SearchCategory};

#[derive(Default, Debug, Clone)]
pub struct City {
    pub city_id: String,
    pub city_name: String,
}

#[graphql_object]
#[graphql(description = "City Data")]
impl City {
    async fn city_id(&self) -> &String {
        &self.city_id
    }

    async fn city_name(&self) -> &String {
        &self.city_name
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "New City Data")]
pub struct NewCity {
    pub city_name: String,
}

#[derive(Default, Debug, Clone, GraphQLInputObject)]
pub struct SearchCity {
    pub city_id: Option<String>,
    pub city_name: Option<String>,
}

#[derive(Default, Debug, Clone, GraphQLInputObject)]
pub struct CityMutation {
    pub city_id: String,
    pub city_name: String,
}
