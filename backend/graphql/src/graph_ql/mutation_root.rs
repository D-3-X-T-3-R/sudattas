use async_graphql::{Context, Object, Result as GraphQLResult};

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn example_mutation(&self, ctx: &Context<'_>) -> GraphQLResult<String> {
        // Call your service layer or business logic here
        Ok("Mutation response".to_string())
    }

    // Add other mutations...
}
