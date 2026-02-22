// Session management module
// Integrates with Redis for fast session storage

use redis::{AsyncCommands, Client as RedisClient};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum SessionError {
    #[error("Redis connection failed: {0}")]
    RedisError(#[from] redis::RedisError),

    #[error("Session not found")]
    SessionNotFound,

    #[error("Session expired")]
    SessionExpired,

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Session data stored in Redis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub user_id: Option<i64>,
    pub email: Option<String>,
    pub cart_id: Option<i64>,
    pub created_at: i64, // Unix timestamp
    pub last_activity: i64,
    pub ip_address: Option<String>,
}

impl Default for SessionData {
    fn default() -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            user_id: None,
            email: None,
            cart_id: None,
            created_at: now,
            last_activity: now,
            ip_address: None,
        }
    }
}

/// Session manager
#[derive(Debug)]
pub struct SessionManager {
    redis: RedisClient,
    ttl: Duration,
}

impl SessionManager {
    /// Create a new session manager
    ///
    /// Example (requires Redis; use `no_run` so doctest compiles without connecting):
    /// ```no_run
    /// use core_operations::auth::session::SessionManager;
    /// use std::time::Duration;
    /// let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1".to_string());
    /// let session_manager = SessionManager::new(&redis_url, Duration::from_secs(86400))?;
    /// # Ok::<(), core_operations::auth::session::SessionError>(())
    /// ```
    pub fn new(redis_url: &str, ttl: Duration) -> Result<Self, SessionError> {
        let redis = RedisClient::open(redis_url)?;
        Ok(Self { redis, ttl })
    }

    /// Create a new session
    pub async fn create_session(&self, data: SessionData) -> Result<String, SessionError> {
        let session_id = Uuid::new_v4().to_string();
        let key = format!("session:{}", session_id);

        let mut conn = self.redis.get_multiplexed_async_connection().await?;

        let serialized = serde_json::to_string(&data)
            .map_err(|e| SessionError::SerializationError(e.to_string()))?;

        conn.set_ex::<_, _, ()>(&key, serialized, self.ttl.as_secs())
            .await?;

        Ok(session_id)
    }

    /// Get session data
    pub async fn get_session(&self, session_id: &str) -> Result<SessionData, SessionError> {
        let key = format!("session:{}", session_id);
        let mut conn = self.redis.get_multiplexed_async_connection().await?;

        let data: Option<String> = conn.get(&key).await?;

        match data {
            Some(json) => {
                let session: SessionData = serde_json::from_str(&json)
                    .map_err(|e| SessionError::SerializationError(e.to_string()))?;

                // Update last activity
                let mut updated = session.clone();
                updated.last_activity = chrono::Utc::now().timestamp();
                self.update_session(session_id, updated).await?;

                Ok(session)
            }
            None => Err(SessionError::SessionNotFound),
        }
    }

    /// Update session data
    pub async fn update_session(
        &self,
        session_id: &str,
        data: SessionData,
    ) -> Result<(), SessionError> {
        let key = format!("session:{}", session_id);
        let mut conn = self.redis.get_multiplexed_async_connection().await?;

        let serialized = serde_json::to_string(&data)
            .map_err(|e| SessionError::SerializationError(e.to_string()))?;

        conn.set_ex::<_, _, ()>(&key, serialized, self.ttl.as_secs())
            .await?;

        Ok(())
    }

    /// Delete session
    pub async fn delete_session(&self, session_id: &str) -> Result<(), SessionError> {
        let key = format!("session:{}", session_id);
        let mut conn = self.redis.get_multiplexed_async_connection().await?;
        conn.del::<_, ()>(&key).await?;
        Ok(())
    }

    /// Associate user with session
    pub async fn login_session(
        &self,
        session_id: &str,
        user_id: i64,
        email: String,
    ) -> Result<(), SessionError> {
        let mut session = self.get_session(session_id).await?;
        session.user_id = Some(user_id);
        session.email = Some(email);
        self.update_session(session_id, session).await
    }

    /// Remove user from session (logout)
    pub async fn logout_session(&self, session_id: &str) -> Result<(), SessionError> {
        let mut session = self.get_session(session_id).await?;
        session.user_id = None;
        session.email = None;
        self.update_session(session_id, session).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Requires Redis running. Run with: `cargo test -p core_operations --lib -- --ignored`
    #[tokio::test]
    #[ignore = "requires Redis; run with --ignored"]
    async fn test_session_lifecycle() {
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1".to_string());

        let manager =
            SessionManager::new(&redis_url, Duration::from_secs(60)).expect("SessionManager::new");
        let data = SessionData::default();
        let session_id = manager.create_session(data).await.expect("create_session");
        assert!(!session_id.is_empty());

        let retrieved = manager.get_session(&session_id).await.expect("get_session");
        assert_eq!(retrieved.user_id, None);

        manager
            .login_session(&session_id, 123, "test@example.com".to_string())
            .await
            .expect("login_session");
        let logged_in = manager.get_session(&session_id).await.expect("get_session");
        assert_eq!(logged_in.user_id, Some(123));

        manager
            .logout_session(&session_id)
            .await
            .expect("logout_session");
        let logged_out = manager.get_session(&session_id).await.expect("get_session");
        assert_eq!(logged_out.user_id, None);

        manager
            .delete_session(&session_id)
            .await
            .expect("delete_session");
        assert!(manager.get_session(&session_id).await.is_err());
    }
}
