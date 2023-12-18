use core_operations::{check_auth, MyGRPCServices};
use tonic::{transport::Server, Request, Response, Status};
use dotenv::dotenv;

pub mod handlers;

use proto::proto::core::grpc_services_server::GrpcServicesServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let addr = std::env::var("GRPC_SERVER").unwrap().parse()?;
    let mut service = MyGRPCServices::default();
    service.init().await;

    Server::builder()
        .add_service(GrpcServicesServer::with_interceptor(service, check_auth))
        .serve(addr)
        .await?;

    Ok(())
}
