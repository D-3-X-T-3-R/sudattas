use async_graphql::{Context, Object, Result as GraphQLResult};

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn example_query(&self, ctx: &Context<'_>) -> GraphQLResult<String> {
        // Call your service layer or business logic here
        Ok("Example response".to_string())
    }

    // Add other queries...
}
