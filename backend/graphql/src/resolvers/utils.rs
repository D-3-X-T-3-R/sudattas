//! Shared utilities for GraphQL resolvers.
//!
//! Use `connect_grpc_client()` then call gRPC methods and use `.await?` to map
//! `tonic::Status` to `GqlError` via `From` (avoids repeating `.map_err(|e| GqlError::new(...))`).
//!
//! Use `parse_i64(s, label)` and `parse_f64(s, label)` to parse String fields from
//! GraphQL input into numeric types with a consistent InvalidArgument error.

use crate::resolvers::error::{Code, GqlError};
use crate::resolvers::grpc_client;

/// Connect to the internal gRPC service (timeout, retry, circuit breaker).
/// Attaches `GRPC_AUTH_TOKEN` as Bearer. For request_id propagation use `grpc_client::connect_grpc_client_with_metadata(Some(id))`.
pub async fn connect_grpc_client() -> Result<
    proto::proto::core::grpc_services_client::GrpcServicesClient<
        proto::tonic::service::interceptor::InterceptedService<
            proto::tonic::transport::Channel,
            impl Fn(
                    proto::tonic::Request<()>,
                ) -> Result<proto::tonic::Request<()>, proto::tonic::Status>
                + Clone,
        >,
    >,
    GqlError,
> {
    grpc_client::connect_grpc_client_with_metadata(None).await
}

pub fn to_option_i64<T: Into<Option<String>>>(input: T) -> Option<i64> {
    input.into().and_then(|val| val.parse::<i64>().ok())
}

pub fn to_option_f64<T: Into<Option<String>>>(input: T) -> Option<f64> {
    input.into().and_then(|val| val.parse::<f64>().ok())
}

pub fn to_i64<T: Into<Option<String>>>(input: T) -> i64 {
    input
        .into()
        .and_then(|val| val.parse::<i64>().ok())
        .unwrap_or(0)
}

pub fn to_f64<T: Into<Option<String>>>(input: T) -> f64 {
    input
        .into()
        .and_then(|val| val.parse::<f64>().ok())
        .unwrap_or(0.0)
}

/// Parse a required String field to i64, returning InvalidArgument on failure.
/// Use instead of `.parse::<i64>().map_err(|_| GqlError::new("Failed to parse X", ...))`.
pub fn parse_i64(s: &str, label: &str) -> Result<i64, GqlError> {
    s.parse::<i64>()
        .map_err(|_| GqlError::new(&format!("Failed to parse {label}"), Code::InvalidArgument))
}

/// Parse a required String field to f64, returning InvalidArgument on failure.
pub fn parse_f64(s: &str, label: &str) -> Result<f64, GqlError> {
    s.parse::<f64>()
        .map_err(|_| GqlError::new(&format!("Failed to parse {label}"), Code::InvalidArgument))
}
