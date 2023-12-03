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
