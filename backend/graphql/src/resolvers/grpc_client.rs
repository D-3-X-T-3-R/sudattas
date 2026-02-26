//! gRPC client with timeout, connect retry, and connection-level circuit breaker.

use crate::resolvers::error::{Code, GqlError};
use proto::{
    proto::core::grpc_services_client::GrpcServicesClient,
    tonic::{
        metadata::MetadataValue,
        transport::{Channel, Endpoint},
        Request,
    },
};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Connection-level circuit breaker: after this many connect failures, open circuit.
fn circuit_failures() -> u32 {
    std::env::var("GRPC_CIRCUIT_BREAKER_FAILURES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5)
}

/// Cooldown in seconds while circuit is open.
fn circuit_cooldown_sec() -> u64 {
    std::env::var("GRPC_CIRCUIT_BREAKER_COOLDOWN_SEC")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30)
}

fn request_timeout_sec() -> u64 {
    std::env::var("GRPC_REQUEST_TIMEOUT_SEC")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30)
}

fn connect_timeout_sec() -> u64 {
    std::env::var("GRPC_CONNECT_TIMEOUT_SEC")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10)
}

fn connect_retries() -> u32 {
    std::env::var("GRPC_CONNECT_RETRIES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(2)
}

static CONNECT_FAILURES: AtomicU32 = AtomicU32::new(0);
static CIRCUIT_OPEN_UNTIL: AtomicU64 = AtomicU64::new(0);

fn record_connect_success() {
    CONNECT_FAILURES.store(0, Ordering::Relaxed);
}

fn record_connect_failure() {
    let failures = CONNECT_FAILURES.fetch_add(1, Ordering::Relaxed) + 1;
    if failures >= circuit_failures() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        CIRCUIT_OPEN_UNTIL.store(now + circuit_cooldown_sec(), Ordering::Relaxed);
    }
}

fn is_circuit_open() -> bool {
    let open_until = CIRCUIT_OPEN_UNTIL.load(Ordering::Relaxed);
    if open_until == 0 {
        return false;
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    if now >= open_until {
        CIRCUIT_OPEN_UNTIL.store(0, Ordering::Relaxed);
        CONNECT_FAILURES.store(0, Ordering::Relaxed);
        return false;
    }
    true
}

#[cfg(test)]
pub(super) fn reset_circuit_breaker_for_test() {
    CONNECT_FAILURES.store(0, Ordering::Relaxed);
    CIRCUIT_OPEN_UNTIL.store(0, Ordering::Relaxed);
}

/// Build a gRPC channel with timeout and optional connect retry.
async fn connect_channel() -> Result<Channel, GqlError> {
    if is_circuit_open() {
        return Err(GqlError::new(
            "gRPC circuit breaker open; try again later",
            Code::Unavailable,
        ));
    }

    let grpc_url = std::env::var("GRPC_URL")
        .map_err(|_| GqlError::new("GRPC_URL not set in environment", Code::Internal))?;

    let request_timeout = Duration::from_secs(request_timeout_sec());
    let connect_timeout = Duration::from_secs(connect_timeout_sec());
    let retries = connect_retries();

    let mut last_err = None;
    for attempt in 0..=retries {
        if attempt > 0 {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        let endpoint = Endpoint::from_shared(grpc_url.clone())
            .map_err(|e| GqlError::new(&format!("Invalid GRPC_URL: {}", e), Code::Internal))?;
        let endpoint = endpoint
            .timeout(request_timeout)
            .connect_timeout(connect_timeout);
        match endpoint.connect().await {
            Ok(ch) => {
                record_connect_success();
                return Ok(ch);
            }
            Err(e) => {
                record_connect_failure();
                last_err = Some(e);
            }
        }
    }
    let msg = last_err
        .map(|e| format!("Failed to connect to gRPC service: {}", e))
        .unwrap_or_else(|| "Failed to connect to gRPC service".to_string());
    Err(GqlError::new(&msg, Code::Unavailable))
}

/// Connect to the internal gRPC service with timeout, retry, and circuit breaker.
/// Attaches GRPC_AUTH_TOKEN as Bearer and optional request_id in metadata.
pub async fn connect_grpc_client_with_metadata(
    request_id: Option<&str>,
) -> Result<
    GrpcServicesClient<
        proto::tonic::service::interceptor::InterceptedService<
            Channel,
            impl Fn(Request<()>) -> Result<Request<()>, proto::tonic::Status> + Clone,
        >,
    >,
    GqlError,
> {
    let channel = connect_channel().await?;
    let auth_token = std::env::var("GRPC_AUTH_TOKEN").ok();
    let rid = request_id.map(String::from);

    #[allow(clippy::result_large_err)]
    let client = GrpcServicesClient::with_interceptor(channel, move |mut req: Request<()>| {
        if let Some(ref tok) = auth_token {
            if let Ok(val) = MetadataValue::try_from(format!("Bearer {tok}").as_str()) {
                req.metadata_mut().insert("authorization", val);
            }
        }
        if let Some(ref id) = rid {
            if let Ok(val) = MetadataValue::try_from(id.as_str()) {
                req.metadata_mut().insert("x-request-id", val);
            }
        }
        Ok(req)
    });
    Ok(client)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolvers::error::Code;

    struct EnvRestore {
        grpc_url: Option<String>,
        circuit_failures: Option<String>,
        circuit_cooldown: Option<String>,
    }

    impl Drop for EnvRestore {
        fn drop(&mut self) {
            if let Some(ref s) = self.grpc_url {
                std::env::set_var("GRPC_URL", s);
            } else {
                std::env::remove_var("GRPC_URL");
            }
            if let Some(ref s) = self.circuit_failures {
                std::env::set_var("GRPC_CIRCUIT_BREAKER_FAILURES", s);
            } else {
                std::env::remove_var("GRPC_CIRCUIT_BREAKER_FAILURES");
            }
            if let Some(ref s) = self.circuit_cooldown {
                std::env::set_var("GRPC_CIRCUIT_BREAKER_COOLDOWN_SEC", s);
            } else {
                std::env::remove_var("GRPC_CIRCUIT_BREAKER_COOLDOWN_SEC");
            }
        }
    }

    /// Runs both scenarios in one test to avoid env races when tests run in parallel.
    #[tokio::test]
    async fn grpc_client_env_and_circuit_breaker() {
        let _restore = EnvRestore {
            grpc_url: std::env::var("GRPC_URL").ok(),
            circuit_failures: std::env::var("GRPC_CIRCUIT_BREAKER_FAILURES").ok(),
            circuit_cooldown: std::env::var("GRPC_CIRCUIT_BREAKER_COOLDOWN_SEC").ok(),
        };

        // 1. Missing GRPC_URL returns Internal with message mentioning GRPC_URL.
        reset_circuit_breaker_for_test();
        std::env::remove_var("GRPC_URL");
        std::env::remove_var("GRPC_CIRCUIT_BREAKER_FAILURES");
        std::env::remove_var("GRPC_CIRCUIT_BREAKER_COOLDOWN_SEC");

        let err = connect_grpc_client_with_metadata(None)
            .await
            .expect_err("connect should fail when GRPC_URL is unset");
        assert_eq!(err.code, Code::Internal);
        assert!(
            err.message.contains("GRPC_URL not set"),
            "error message should mention GRPC_URL: {}",
            err.message
        );

        // 2. After N connect failures, circuit opens and next call gets Unavailable.
        reset_circuit_breaker_for_test();
        std::env::set_var("GRPC_URL", "http://127.0.0.1:1");
        std::env::set_var("GRPC_CIRCUIT_BREAKER_FAILURES", "2");
        std::env::set_var("GRPC_CIRCUIT_BREAKER_COOLDOWN_SEC", "30");
        std::env::set_var("GRPC_CONNECT_TIMEOUT_SEC", "1");
        std::env::set_var("GRPC_CONNECT_RETRIES", "0");

        let _ = connect_grpc_client_with_metadata(None).await;
        let _ = connect_grpc_client_with_metadata(None).await;

        let err = connect_grpc_client_with_metadata(None)
            .await
            .expect_err("connect should fail with circuit open");
        assert_eq!(err.code, Code::Unavailable);
        assert!(
            err.message.to_lowercase().contains("circuit"),
            "error should mention circuit breaker: {}",
            err.message
        );
    }
}
