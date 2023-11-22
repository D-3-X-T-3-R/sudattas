use services::{check_auth, MyGRPCServices};
use tonic::{transport::Server, Request, Response, Status};

// use core_db_entities::*;
mod db_errors;
mod handlers;
mod services;

use proto::proto::core::grpc_services_server::GrpcServicesServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50051".parse()?;
    let service = MyGRPCServices::default();

    Server::builder()
        .add_service(GrpcServicesServer::with_interceptor(service, check_auth))
        .serve(addr)
        .await?;

    Ok(())
}
