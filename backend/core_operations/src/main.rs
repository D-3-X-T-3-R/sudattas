use core_operations::{check_auth, MyGRPCServices};
use dotenvy::dotenv;
use tonic::transport::Server;

pub mod handlers;
pub mod order_state_machine {
    pub use core_operations::order_state_machine::*;
}
pub use core_operations::auth;

use proto::proto::core::grpc_services_server::GrpcServicesServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let addr = std::env::var("GRPC_SERVER")
        .unwrap_or_else(|_| "0.0.0.0:50051".to_string())
        .parse()?;
    let mut service = MyGRPCServices::default();
    service.init().await;

    Server::builder()
        .add_service(GrpcServicesServer::with_interceptor(service, check_auth))
        .serve(addr)
        .await?;

    Ok(())
}
