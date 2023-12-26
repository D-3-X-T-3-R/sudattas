use juniper::{graphql_object, GraphQLInputObject};

#[derive(Default, Debug, Clone)]
pub struct Category {
    pub name: String,
    pub category_id: String,
}

#[graphql_object]
#[graphql(description = "Category Data")]
impl Category {
    async fn name(&self) -> &String {
        &self.name
    }
    async fn category_id(&self) -> &String {
        &self.category_id
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "New Category Data")]
pub struct NewCategory {
    pub name: String,
}

#[derive(Default, Debug, Clone, GraphQLInputObject)]
pub struct SearchCategory {
    pub name: Option<String>,
    pub category_id: Option<String>,
}

#[derive(Default, Debug, Clone, GraphQLInputObject)]
pub struct CategoryMutation {
    pub name: String,
    pub category_id: String,
}
