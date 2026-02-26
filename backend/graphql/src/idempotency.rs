//! Idempotency for critical mutations (place_order, capture_payment).
//!
//! When `Idempotency-Key` header is set and REDIS_URL is configured, the mutation result
//! is cached in Redis; duplicate requests with the same key within the window return the cached result.

use crate::resolvers::error::{Code, GqlError};
use serde::{de::DeserializeOwned, Serialize};

const KEY_PREFIX: &str = "idempotency";

fn idempotency_window_hours() -> u32 {
    std::env::var("IDEMPOTENCY_WINDOW_HOURS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(24)
}

/// If redis_url and idempotency_key are set, try to return cached result for this operation.
/// On cache miss (or when Redis is not configured), run `f` and cache a successful result.
pub async fn with_idempotency<T, F, Fut>(
    redis_url: Option<&str>,
    operation: &str,
    idempotency_key: Option<&str>,
    f: F,
) -> Result<T, GqlError>
where
    T: Serialize + DeserializeOwned + Send,
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, GqlError>> + Send,
{
    let (redis_url, key) = match (redis_url, idempotency_key) {
        (Some(url), Some(k)) if !k.is_empty() => (url, k),
        _ => return f().await,
    };

    let redis_key = format!("{}:{}:{}", KEY_PREFIX, operation, key);
    let window_hours = idempotency_window_hours();
    let ttl_secs = window_hours * 3600;

    let client = redis::Client::open(redis_url)
        .map_err(|e| GqlError::new(&format!("Redis client error: {}", e), Code::Internal))?;
    let mut conn = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| GqlError::new(&format!("Redis connection error: {}", e), Code::Unavailable))?;

    // Try cache hit
    let cached: Result<String, redis::RedisError> = redis::cmd("GET")
        .arg(&redis_key)
        .query_async(&mut conn)
        .await;
    if let Ok(json) = cached {
        if !json.is_empty() {
            let value: T = serde_json::from_str(&json).map_err(|e| {
                GqlError::new(
                    &format!("Idempotency cache deserialize error: {}", e),
                    Code::Internal,
                )
            })?;
            return Ok(value);
        }
    }

    // Cache miss: run the mutation
    let result = f().await?;
    let json = serde_json::to_string(&result).map_err(|e| {
        GqlError::new(
            &format!("Idempotency cache serialize error: {}", e),
            Code::Internal,
        )
    })?;

    let _: Result<(), redis::RedisError> = redis::cmd("SETEX")
        .arg(&redis_key)
        .arg(ttl_secs)
        .arg(&json)
        .query_async(&mut conn)
        .await;

    Ok(result)
}
