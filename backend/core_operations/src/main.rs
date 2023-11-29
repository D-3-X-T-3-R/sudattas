use core_operations::{check_auth, MyGRPCServices};
use tonic::{transport::Server, Request, Response, Status};

pub mod handlers;

use proto::proto::core::grpc_services_server::GrpcServicesServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50051".parse()?;
    let mut service = MyGRPCServices::default();
    service.init().await;
    
    Server::builder()
        .add_service(GrpcServicesServer::with_interceptor(service, check_auth))
        .serve(addr)
        .await?;

    Ok(())
}

