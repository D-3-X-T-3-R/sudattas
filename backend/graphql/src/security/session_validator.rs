//! Redis-backed session validation for the GraphQL gateway.
//!
//! When a client presents an `X-Session-Id` header instead of (or alongside)
//! a JWT, this module validates that the session ID exists and has not expired
//! in Redis.  It returns the stored `user_id` string so callers can build a
//! minimal identity for logging / authorization purposes.

use redis::AsyncCommands;
use tracing::{debug, warn};

/// Validate a session ID against Redis.
///
/// Returns the `user_id` stored in the session on success, or an error string
/// describing why validation failed (never includes the raw session ID to
/// avoid leaking tokens in logs).
pub async fn validate_session(session_id: &str, redis_url: &str) -> Result<String, String> {
    let client = redis::Client::open(redis_url).map_err(|e| format!("Redis client error: {e}"))?;

    let mut conn: redis::aio::MultiplexedConnection = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| format!("Redis connection error: {e}"))?;

    let key = format!("session:{session_id}");
    let value: Option<String> = conn
        .get(key)
        .await
        .map_err(|e| format!("Redis GET error: {e}"))?;

    match value {
        Some(ref user_id) if !user_id.is_empty() => {
            debug!("Session validated for user_id={}", user_id);
            Ok(value.unwrap())
        }
        Some(_) => {
            warn!("Session found but user_id is empty");
            Err("Session has no associated user".to_string())
        }
        None => Err("Session not found or expired".to_string()),
    }
}
