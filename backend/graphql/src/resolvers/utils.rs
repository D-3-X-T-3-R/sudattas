//! Shared utilities for GraphQL resolvers.
//!
//! Use `connect_grpc_client()` then call gRPC methods and use `.await?` to map
//! `tonic::Status` to `GqlError` via `From` (avoids repeating `.map_err(|e| GqlError::new(...))`).
//!
//! Use `parse_i64(s, label)` and `parse_f64(s, label)` to parse String fields from
//! GraphQL input into numeric types with a consistent InvalidArgument error.

use crate::resolvers::error::{Code, GqlError};
use proto::{
    proto::core::grpc_services_client::GrpcServicesClient,
    tonic::{
        metadata::MetadataValue,
        service::interceptor::InterceptedService,
        transport::Channel,
        Request,
    },
};

/// Connect to the internal gRPC service and attach `GRPC_AUTH_TOKEN` as a
/// `Bearer` token on every outgoing request (if the env var is set).
///
/// In dev mode (no `GRPC_AUTH_TOKEN`), requests are forwarded without auth.
pub async fn connect_grpc_client()
-> Result<GrpcServicesClient<InterceptedService<Channel, impl Fn(Request<()>) -> Result<Request<()>, proto::tonic::Status> + Clone>>, GqlError>
{
    let grpc_url = std::env::var("GRPC_URL")
        .map_err(|_| GqlError::new("GRPC_URL not set in environment", Code::Internal))?;

    let channel = Channel::from_shared(grpc_url)
        .map_err(|e| GqlError::new(&format!("Invalid GRPC_URL: {}", e), Code::Internal))?
        .connect()
        .await
        .map_err(|e| {
            GqlError::new(
                &format!("Failed to connect to gRPC service: {}", e),
                Code::Unavailable,
            )
        })?;

    let auth_token = std::env::var("GRPC_AUTH_TOKEN").ok();

    let client = GrpcServicesClient::with_interceptor(channel, move |mut req: Request<()>| {
        if let Some(ref tok) = auth_token {
            if let Ok(val) = MetadataValue::try_from(format!("Bearer {tok}").as_str()) {
                req.metadata_mut().insert("authorization", val);
            }
        }
        Ok(req)
    });

    Ok(client)
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
