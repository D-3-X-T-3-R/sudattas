//! Library exports for the GraphQL service. Used by the binary and by integration tests.

pub mod health;
pub mod idempotency;
pub mod query_handler;
pub mod resolvers;
pub mod security;
pub mod webhooks;

pub use juniper::{EmptySubscription, RootNode};
pub use query_handler::{AuthSource, Context};
pub use security::jwks_loader::JWKSet;

use query_handler::mutation_root::MutationRoot;
use query_handler::query_root::QueryRoot;

pub type Schema = RootNode<'static, QueryRoot, MutationRoot, EmptySubscription<Context>>;

/// Build the GraphQL schema (queries + mutations). Used by the server and by tests.
pub fn schema() -> Schema {
    RootNode::new(
        QueryRoot {},
        MutationRoot {},
        EmptySubscription::<Context>::new(),
    )
}
