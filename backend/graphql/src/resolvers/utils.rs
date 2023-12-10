use crate::resolvers::error::{Code, GqlError};
use proto::{proto::core::grpc_services_client::GrpcServicesClient, tonic::transport::Channel};

pub async fn connect_grpc_client() -> Result<GrpcServicesClient<Channel>, GqlError> {
    let grpc_url = std::env::var("GRPC_URL")
        .map_err(|_| GqlError::new("GRPC_URL not set in environment", Code::Internal))?;

    GrpcServicesClient::connect(grpc_url).await.map_err(|e| {
        GqlError::new(
            &format!("Failed to connect to gRPC client: {}", e),
            Code::Unavailable,
        )
    })
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
