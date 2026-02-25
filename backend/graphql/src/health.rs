//! Readiness check for orchestrators (Kubernetes, etc.).
//!
//! GET /ready verifies that dependencies required to serve traffic are up:
//! - gRPC backend (and thus DB, which gRPC checks via its Readiness RPC)
//! - Redis, if REDIS_URL is set

use crate::resolvers::utils::connect_grpc_client;
use proto::proto::core::ReadinessRequest;

/// Run readiness checks: gRPC (which pings DB) and optionally Redis.
/// Returns Ok(()) if ready, Err(message) otherwise.
pub async fn check_ready() -> Result<(), String> {
    // 1. gRPC (and thus DB) — when GRPC_URL is set we require it to be ready
    if std::env::var("GRPC_URL").is_ok() {
        let mut client = connect_grpc_client()
            .await
            .map_err(|e| format!("gRPC unreachable or not ready: {}", e.message))?;
        let req = proto::tonic::Request::new(ReadinessRequest {});
        let res = client
            .readiness(req)
            .await
            .map_err(|e| format!("gRPC readiness failed: {}", e))?;
        if !res.get_ref().ok {
            return Err(res
                .get_ref()
                .error
                .clone()
                .unwrap_or_else(|| "gRPC reported not ready".to_string()));
        }
    }

    // 2. Redis — optional; only check if REDIS_URL is set
    if let Ok(redis_url) = std::env::var("REDIS_URL") {
        let client =
            redis::Client::open(redis_url).map_err(|e| format!("Redis client error: {e}"))?;
        let mut conn: redis::aio::MultiplexedConnection = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| format!("Redis connection error: {e}"))?;
        redis::cmd("PING")
            .query_async::<String>(&mut conn)
            .await
            .map_err(|e| format!("Redis ping failed: {e}"))?;
    }

    Ok(())
}
