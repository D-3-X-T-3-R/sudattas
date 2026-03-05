use juniper::{graphql_object, GraphQLInputObject};

#[derive(Default, Debug, Clone)]
pub struct Color {
    pub color_id: String,
    pub color_name: String,
}

#[graphql_object]
#[graphql(description = "Color")]
impl Color {
    async fn color_id(&self) -> &String {
        &self.color_id
    }

    async fn color_name(&self) -> &String {
        &self.color_name
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Create a color")]
pub struct NewColor {
    pub color_name: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Search color by ID")]
pub struct SearchColorInput {
    pub color_id: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Update a color")]
pub struct ColorMutation {
    pub color_id: String,
    pub color_name: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Delete a color")]
pub struct DeleteColorInput {
    pub color_id: String,
}
