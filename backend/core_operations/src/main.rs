use core_db_entities::get_db;
use core_operations::{
    check_auth,
    procedures::{
        cancel_pending_logistics::process_cancel_pending_logistics,
        create_shipments_after_cancel_window::process_create_shipments_after_cancel_window,
        outbox_worker::process_pending_outbox_events,
        refund_attempts_worker::process_refund_attempts,
        stale_order_expiry::expire_stale_pending_orders,
    },
    MyGRPCServices,
};
use dotenvy::dotenv;
use proto::proto::core::grpc_services_server::GrpcServicesServer;
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use std::time::Duration;
use tonic::transport::Server;
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};
use warp::Filter;
mod startup_config;

pub mod order_state_machine {
    pub use core_operations::order_state_machine::*;
}
pub use core_operations::auth;
pub use core_operations::observability;

async fn query_count(
    db: &core_db_entities::CoreDatabaseConnection,
    sql: &str,
) -> Result<f64, sea_orm::DbErr> {
    let row = db
        .query_one(Statement::from_string(DbBackend::MySql, sql.to_string()))
        .await?;
    Ok(row
        .and_then(|r| r.try_get::<i64>("", "count").ok())
        .unwrap_or_default() as f64)
}

async fn refresh_backlog_metrics(db: &core_db_entities::CoreDatabaseConnection) {
    let stuck_pending = query_count(
        db,
        "SELECT COUNT(*) AS count FROM Orders o JOIN OrderStatus s ON s.StatusID = o.StatusID WHERE s.StatusName IN ('active_sale','pending') AND o.updated_at < (UTC_TIMESTAMP() - INTERVAL 15 MINUTE)",
    )
    .await
    .unwrap_or(0.0);
    observability::record_stuck_pending_orders_gauge(stuck_pending);

    let cancel_pending = query_count(
        db,
        "SELECT COUNT(*) AS count FROM Orders o JOIN OrderStatus s ON s.StatusID = o.StatusID WHERE s.StatusName = 'cancel_pending_logistics'",
    )
    .await
    .unwrap_or(0.0);
    observability::record_cancel_pending_logistics_backlog_gauge(cancel_pending);

    let refund_failed_orders = query_count(
        db,
        "SELECT COUNT(*) AS count FROM Orders WHERE refund_settlement_status = 'refund_failed'",
    )
    .await
    .unwrap_or(0.0);
    observability::record_refund_failed_orders_gauge(refund_failed_orders);

    let outbox_backlog = query_count(
        db,
        "SELECT COUNT(*) AS count FROM OutboxEvents WHERE status = 'pending'",
    )
    .await
    .unwrap_or(0.0);
    observability::record_outbox_backlog_gauge(outbox_backlog);
    let outbox_pending_max_age = query_count(
        db,
        "SELECT COALESCE(MAX(TIMESTAMPDIFF(SECOND, created_at, UTC_TIMESTAMP())), 0) AS count FROM OutboxEvents WHERE status = 'pending'",
    )
    .await
    .unwrap_or(0.0);
    observability::record_outbox_pending_max_age_seconds_gauge(outbox_pending_max_age);
    let outbox_retry_backlog = query_count(
        db,
        "SELECT COUNT(*) AS count FROM OutboxEvents WHERE status = 'pending' AND published_at IS NOT NULL",
    )
    .await
    .unwrap_or(0.0);
    observability::record_outbox_retry_backlog_gauge(outbox_retry_backlog);

    let webhook_failed_backlog = query_count(
        db,
        "SELECT COUNT(*) AS count FROM WebhookEvents WHERE status = 'failed'",
    )
    .await
    .unwrap_or(0.0);
    observability::record_webhook_failed_backlog_gauge(webhook_failed_backlog);
    let webhook_pending_max_age = query_count(
        db,
        "SELECT COALESCE(MAX(TIMESTAMPDIFF(SECOND, received_at, UTC_TIMESTAMP())), 0) AS count FROM WebhookEvents WHERE status = 'pending'",
    )
    .await
    .unwrap_or(0.0);
    observability::record_webhook_pending_max_age_seconds_gauge(webhook_pending_max_age);

    let refund_attempts_stuck = query_count(
        db,
        "SELECT COUNT(*) AS count FROM RefundAttempts WHERE status IN ('pending_external','submitted','submitting') AND created_at < (UTC_TIMESTAMP() - INTERVAL 15 MINUTE)",
    )
    .await
    .unwrap_or(0.0);
    observability::record_refund_attempts_stuck_gauge(refund_attempts_stuck);

    let shipments_retry_backlog = query_count(
        db,
        "SELECT COUNT(*) AS count FROM Shipments WHERE logistics_status IN ('booking_failed','booking_persist_pending','cancel_pending_logistics','cancel_persist_pending')",
    )
    .await
    .unwrap_or(0.0);
    observability::record_shipments_retry_backlog_gauge(shipments_retry_backlog);

    let stuck_intents = query_count(
        db,
        "SELECT COUNT(*) AS count FROM PaymentIntents WHERE status IN ('pending','client_verified','needs_review') AND expires_at < UTC_TIMESTAMP()",
    )
    .await
    .unwrap_or(0.0);
    observability::record_stuck_payment_intents_gauge(stuck_intents);

    let stale_idempotency_pending = query_count(
        db,
        "SELECT COUNT(*) AS count FROM IdempotencyKeys WHERE status = 'pending' AND created_at < (UTC_TIMESTAMP() - INTERVAL 15 MINUTE)",
    )
    .await
    .unwrap_or(0.0);
    observability::record_stale_idempotency_pending_gauge(stale_idempotency_pending);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_level(true)
        .with_env_filter(EnvFilter::from_default_env())
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .json()
        .init();

    let startup = startup_config::StartupConfig::from_env()
        .map_err(|e| format!("invalid startup environment: {e}"))?;

    // P1 Observability: install Prometheus recorder so core_operations::metrics record to it.
    let prom_handle = metrics_exporter_prometheus::PrometheusBuilder::new()
        .install_recorder()
        .expect("Prometheus metrics recorder");

    // Readiness pings the DB rather than returning 200 unconditionally, so an
    // orchestrator stops routing traffic to an instance whose DB pool has died.
    // `DatabaseConnection` isn't `Clone`, so it's wrapped in an `Arc` to be shared
    // across every invocation of the readiness filter below.
    let readiness_db = std::sync::Arc::new(
        get_db()
            .await
            .map_err(|e| format!("readiness route: database connect failed: {e}"))?,
    );

    let metrics_addr = startup.grpc_metrics_addr;
    let metrics_route = warp::get()
        .and(warp::path("metrics"))
        .and(warp::path::end())
        .map(move || {
            let body = prom_handle.render();
            warp::reply::with_header(body, "content-type", "text/plain; charset=utf-8")
        });
    let health_route = warp::get()
        .and(warp::path("healthz"))
        .and(warp::path::end())
        .map(|| warp::reply::with_status("ok", warp::http::StatusCode::OK));
    let readiness_route = warp::get()
        .and(warp::path("ready"))
        .and(warp::path::end())
        .and_then(move || {
            let db = readiness_db.clone();
            async move {
                match db.ping().await {
                    Ok(()) => Ok(warp::reply::with_status(
                        "ready",
                        warp::http::StatusCode::OK,
                    )),
                    Err(e) => {
                        log::warn!("readiness check failed: DB ping error: {e}");
                        Ok::<_, std::convert::Infallible>(warp::reply::with_status(
                            "database unavailable",
                            warp::http::StatusCode::SERVICE_UNAVAILABLE,
                        ))
                    }
                }
            }
        });
    tokio::spawn(async move {
        warp::serve(metrics_route.or(health_route).or(readiness_route))
            .run(metrics_addr)
            .await;
    });

    let addr = startup.grpc_server_addr;
    let mut service = MyGRPCServices::default();
    service
        .init()
        .await
        .map_err(|e| format!("service initialization failed: {}", e.message()))?;

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
                        observability::record_outbox_worker_failure_total();
                        log::warn!(
                            "outbox worker: batch failed (events stay Pending for retry): {}",
                            e.message()
                        );
                    }
                }
                refresh_backlog_metrics(&db).await;
            }
        });
    } else {
        log::info!("outbox worker: disabled via OUTBOX_DISABLE_WORKER");
    }

    let stale_order_expiry_disabled = std::env::var("STALE_ORDER_EXPIRY_DISABLE_WORKER")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if !stale_order_expiry_disabled {
        let db = get_db()
            .await
            .map_err(|e| format!("stale order expiry worker: database connect failed: {e}"))?;
        let poll_sec = std::env::var("STALE_ORDER_EXPIRY_POLL_INTERVAL_SEC")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .filter(|&n| n > 0)
            .unwrap_or(60);
        let batch_limit = std::env::var("STALE_ORDER_EXPIRY_BATCH_LIMIT")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .filter(|&n| n > 0)
            .unwrap_or(25);

        log::info!(
            "stale order expiry worker: background task started (poll_interval_sec={poll_sec}, batch_limit={batch_limit})"
        );

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(poll_sec));
            loop {
                interval.tick().await;
                match expire_stale_pending_orders(&db, batch_limit).await {
                    Ok(n) if n > 0 => {
                        log::info!("stale order expiry worker: expired {n} stale pending order(s)");
                    }
                    Ok(_) => {}
                    Err(e) => {
                        observability::record_stale_order_expiry_failure_total();
                        log::warn!("stale order expiry worker: batch failed: {}", e.message());
                    }
                }
                refresh_backlog_metrics(&db).await;
            }
        });
    } else {
        log::info!("stale order expiry worker: disabled via STALE_ORDER_EXPIRY_DISABLE_WORKER");
    }

    let cancel_pending_disabled = std::env::var("CANCEL_PENDING_LOGISTICS_DISABLE_WORKER")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if !cancel_pending_disabled {
        let db = get_db().await.map_err(|e| {
            format!("cancel pending logistics worker: database connect failed: {e}")
        })?;
        let poll_sec = std::env::var("CANCEL_PENDING_LOGISTICS_POLL_INTERVAL_SEC")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .filter(|&n| n > 0)
            .unwrap_or(90);
        let batch_limit = std::env::var("CANCEL_PENDING_LOGISTICS_BATCH_LIMIT")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .filter(|&n| n > 0)
            .unwrap_or(25);
        log::info!(
            "cancel pending logistics worker: background task started (poll_interval_sec={poll_sec}, batch_limit={batch_limit})"
        );
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(poll_sec));
            loop {
                interval.tick().await;
                match process_cancel_pending_logistics(&db, batch_limit).await {
                    Ok(n) if n > 0 => {
                        log::info!(
                            "cancel pending logistics worker: processed {n} retry candidate(s)"
                        );
                    }
                    Ok(_) => {}
                    Err(e) => {
                        observability::record_cancel_pending_logistics_failure_total();
                        log::warn!(
                            "cancel pending logistics worker: batch failed: {}",
                            e.message()
                        );
                    }
                }
                refresh_backlog_metrics(&db).await;
            }
        });
    } else {
        log::info!(
            "cancel pending logistics worker: disabled via CANCEL_PENDING_LOGISTICS_DISABLE_WORKER"
        );
    }

    let delayed_shipment_worker_disabled =
        std::env::var("DELAYED_SHIPMENT_CREATION_DISABLE_WORKER")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
    if !delayed_shipment_worker_disabled {
        let db = get_db().await.map_err(|e| {
            format!("delayed shipment creation worker: database connect failed: {e}")
        })?;
        let poll_sec = std::env::var("DELAYED_SHIPMENT_CREATION_POLL_INTERVAL_SEC")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .filter(|&n| n > 0)
            .unwrap_or(60);
        let batch_limit = std::env::var("DELAYED_SHIPMENT_CREATION_BATCH_LIMIT")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .filter(|&n| n > 0)
            .unwrap_or(25);
        log::info!(
            "delayed shipment creation worker: background task started (poll_interval_sec={poll_sec}, batch_limit={batch_limit})"
        );
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(poll_sec));
            loop {
                interval.tick().await;
                match process_create_shipments_after_cancel_window(&db, batch_limit).await {
                    Ok(n) if n > 0 => {
                        log::info!("delayed shipment creation worker: created {n} shipment(s)");
                    }
                    Ok(_) => {}
                    Err(e) => {
                        observability::record_shiprocket_booking_failure_total(
                            "delayed_shipment_worker",
                        );
                        log::warn!(
                            "delayed shipment creation worker: batch failed: {}",
                            e.message()
                        );
                    }
                }
                refresh_backlog_metrics(&db).await;
            }
        });
    } else {
        log::info!(
            "delayed shipment creation worker: disabled via DELAYED_SHIPMENT_CREATION_DISABLE_WORKER"
        );
    }

    let refund_worker_disabled = std::env::var("REFUND_ATTEMPTS_DISABLE_WORKER")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if !refund_worker_disabled {
        let db = get_db()
            .await
            .map_err(|e| format!("refund attempts worker: database connect failed: {e}"))?;
        let poll_sec = std::env::var("REFUND_ATTEMPTS_POLL_INTERVAL_SEC")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .filter(|&n| n > 0)
            .unwrap_or(30);
        let batch_limit = std::env::var("REFUND_ATTEMPTS_BATCH_LIMIT")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .filter(|&n| n > 0)
            .unwrap_or(25);
        log::info!(
            "refund attempts worker: background task started (poll_interval_sec={poll_sec}, batch_limit={batch_limit})"
        );
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(poll_sec));
            loop {
                interval.tick().await;
                match process_refund_attempts(&db, batch_limit).await {
                    Ok(n) if n > 0 => {
                        log::info!("refund attempts worker: processed {n} refund attempt(s)");
                    }
                    Ok(_) => {}
                    Err(e) => {
                        observability::record_refund_failure_total("worker_error");
                        log::warn!("refund attempts worker: batch failed: {}", e.message());
                    }
                }
                refresh_backlog_metrics(&db).await;
            }
        });
    } else {
        log::info!("refund attempts worker: disabled via REFUND_ATTEMPTS_DISABLE_WORKER");
    }

    Server::builder()
        .add_service(GrpcServicesServer::with_interceptor(service, check_auth))
        .serve_with_shutdown(addr, async {
            let _ = tokio::signal::ctrl_c().await;
            log::info!("core_operations shutdown signal received");
        })
        .await?;

    Ok(())
}
