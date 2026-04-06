use core_db_entities::get_db;
use core_operations::{check_auth, procedures::outbox_worker::process_pending_outbox_events, MyGRPCServices};
use dotenvy::dotenv;
use proto::proto::core::grpc_services_server::GrpcServicesServer;
use std::time::Duration;
use tonic::transport::Server;
use warp::Filter;
mod startup_config;

pub mod order_state_machine {
    pub use core_operations::order_state_machine::*;
}
pub use core_operations::auth;
pub use core_operations::observability;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let startup = startup_config::StartupConfig::from_env()
        .map_err(|e| format!("invalid startup environment: {e}"))?;

    // P1 Observability: install Prometheus recorder so core_operations::metrics record to it.
    let prom_handle = metrics_exporter_prometheus::PrometheusBuilder::new()
        .install_recorder()
        .expect("Prometheus metrics recorder");

    let metrics_addr = startup.grpc_metrics_addr;
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

    let addr = startup.grpc_server_addr;
    let mut service = MyGRPCServices::default();
    service.init().await;

    let outbox_disabled = std::env::var("OUTBOX_DISABLE_WORKER")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if !outbox_disabled {
        // Second pool: `sea_orm::DatabaseConnection` is not `Clone` here; gRPC holds the first pool on `service`.
        let db = get_db()
            .await
            .map_err(|e| format!("outbox worker: database connect failed: {e}"))?;
        let poll_sec = std::env::var("OUTBOX_POLL_INTERVAL_SEC")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .filter(|&n| n > 0)
            .unwrap_or(30);
        let batch_limit: u64 = std::env::var("OUTBOX_BATCH_LIMIT")
            .ok()
            .and_then(|s| s.parse().ok())
            .filter(|&n| n > 0)
            .unwrap_or(50);

        log::info!(
            "outbox worker: background task started (poll_interval_sec={poll_sec}, batch_limit={batch_limit})"
        );

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(poll_sec));
            loop {
                interval.tick().await;
                match process_pending_outbox_events(&db, batch_limit).await {
                    Ok(n) if n > 0 => {
                        log::info!("outbox worker: processed {n} pending event(s)");
                    }
                    Ok(_) => {}
                    Err(e) => {
                        log::warn!(
                            "outbox worker: batch failed (events stay Pending for retry): {}",
                            e.message()
                        );
                    }
                }
            }
        });
    } else {
        log::info!("outbox worker: disabled via OUTBOX_DISABLE_WORKER");
    }

    Server::builder()
        .add_service(GrpcServicesServer::with_interceptor(service, check_auth))
        .serve(addr)
        .await?;

    Ok(())
}
