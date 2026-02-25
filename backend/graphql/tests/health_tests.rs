//! Tests for health::check_ready (readiness logic used by GET /ready).
//! Run in a single test to avoid env races when tests run in parallel.

use graphql::health;

struct EnvRestore {
    grpc: Option<String>,
    redis: Option<String>,
}

impl Drop for EnvRestore {
    fn drop(&mut self) {
        if let Some(ref u) = self.grpc {
            std::env::set_var("GRPC_URL", u);
        } else {
            std::env::remove_var("GRPC_URL");
        }
        if let Some(ref u) = self.redis {
            std::env::set_var("REDIS_URL", u);
        } else {
            std::env::remove_var("REDIS_URL");
        }
    }
}

#[tokio::test]
async fn test_check_ready_env_scenarios() {
    let _restore = EnvRestore {
        grpc: std::env::var("GRPC_URL").ok(),
        redis: std::env::var("REDIS_URL").ok(),
    };

    // 1. With neither GRPC_URL nor REDIS_URL set, check_ready skips both checks and returns Ok.
    std::env::remove_var("GRPC_URL");
    std::env::remove_var("REDIS_URL");
    let result = health::check_ready().await;
    assert!(
        result.is_ok(),
        "check_ready should succeed when no deps configured: {:?}",
        result.err()
    );

    // 2. When GRPC_URL is set but unreachable, check_ready returns Err.
    std::env::set_var("GRPC_URL", "http://127.0.0.1:19999");
    std::env::remove_var("REDIS_URL");
    let result = health::check_ready().await;
    assert!(
        result.is_err(),
        "check_ready should fail when GRPC_URL points to unreachable server"
    );
}
