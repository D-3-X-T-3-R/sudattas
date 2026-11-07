//! No mutation response may be served from an actorless Redis key. Checkout's
//! durable actor/payload-scoped replay is in MySQL. Other mutations execute
//! authorization and domain-level deduplication on every request.
use crate::resolvers::error::GqlError;
use serde::{de::DeserializeOwned, Serialize};

pub async fn with_idempotency<T, F, Fut>(
    _redis_url: Option<&str>,
    _operation: &str,
    _idempotency_key: Option<&str>,
    f: F,
) -> Result<T, GqlError>
where
    T: Serialize + DeserializeOwned + Send,
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, GqlError>> + Send,
{
    // Legacy cached responses do not bind either an actor or a payload.
    f().await
}
