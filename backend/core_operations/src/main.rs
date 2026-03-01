use core_operations::{check_auth, MyGRPCServices};
use dotenvy::dotenv;
use proto::proto::core::grpc_services_server::GrpcServicesServer;
use tonic::transport::Server;
use warp::Filter;

pub mod order_state_machine {
    pub use core_operations::order_state_machine::*;
}
pub use core_operations::auth;
pub use core_operations::observability;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // P1 Observability: install Prometheus recorder so core_operations::metrics record to it.
    let prom_handle = metrics_exporter_prometheus::PrometheusBuilder::new()
        .install_recorder()
        .expect("Prometheus metrics recorder");

    let metrics_addr: std::net::SocketAddr = std::env::var("GRPC_METRICS_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:9090".to_string())
        .parse()?;
    let metrics_route = warp::get()
        .and(warp::path("metrics"))
        .and(warp::path::end())
        .map(move || {
            let body = prom_handle.render();
            warp::reply::with_header(body, "content-type", "text/plain; charset=utf-8")
        });
    tokio::spawn(async move {
        warp::serve(metrics_route).run(metrics_addr).await;
    });

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
