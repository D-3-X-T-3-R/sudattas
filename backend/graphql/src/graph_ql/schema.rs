use super::mutation_root::MutationRoot;
use super::query_root::QueryRoot;
use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema};

pub type MyGraphQLSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub fn create_schema() -> MyGraphQLSchema {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription).finish()
}
