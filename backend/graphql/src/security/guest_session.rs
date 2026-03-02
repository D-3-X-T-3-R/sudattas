//! Create a guest session in Redis for storefront cart (no login).
//! The session ID can be sent as X-Session-Id; session_validator expects value = user_id string (we use "0" for guest).

use redis::AsyncCommands;
use uuid::Uuid;

const GUEST_USER_ID: &str = "0";
const SESSION_TTL_SECS: u64 = 30 * 24 * 60 * 60; // 30 days

/// Create a guest session in Redis. Returns the session ID to send as X-Session-Id.
pub async fn create_guest_session(redis_url: &str) -> Result<String, String> {
    let client = redis::Client::open(redis_url).map_err(|e| format!("Redis client error: {e}"))?;
    let mut conn: redis::aio::MultiplexedConnection = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| format!("Redis connection error: {e}"))?;

    let session_id = Uuid::new_v4().to_string();
    let key = format!("session:{session_id}");

    conn.set_ex::<_, _, ()>(&key, GUEST_USER_ID, SESSION_TTL_SECS)
        .await
        .map_err(|e| format!("Redis SETEX error: {e}"))?;

    Ok(session_id)
}
