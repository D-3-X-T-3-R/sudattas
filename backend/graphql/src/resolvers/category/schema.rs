use juniper::{graphql_object, GraphQLInputObject};

#[derive(Default, Debug, Clone)]
pub struct Category {
    pub name: String,
    pub category_id: String,
    pub exchange_eligible: bool,
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
    /// Whether products in this category can be exchanged (same product, different
    /// size/colour, same price) instead of only refunded via a return.
    async fn exchange_eligible(&self) -> bool {
        self.exchange_eligible
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "New Category Data")]
pub struct NewCategory {
    pub name: String,
    pub exchange_eligible: Option<bool>,
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
    /// Unset leaves the existing value unchanged.
    pub exchange_eligible: Option<bool>,
}
