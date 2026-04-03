use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct StartupConfig {
    pub grpc_server_addr: SocketAddr,
    pub grpc_metrics_addr: SocketAddr,
}

impl StartupConfig {
    pub fn from_env() -> Result<Self, String> {
        let grpc_server_addr = parse_socket_addr_or_default("GRPC_SERVER", "0.0.0.0:50051")?;
        let grpc_metrics_addr = parse_socket_addr_or_default("GRPC_METRICS_ADDR", "0.0.0.0:9090")?;

        Ok(Self {
            grpc_server_addr,
            grpc_metrics_addr,
        })
    }
}

fn parse_socket_addr_or_default(key: &str, default: &str) -> Result<SocketAddr, String> {
    let raw = std::env::var(key).unwrap_or_else(|_| default.to_string());
    raw.parse::<SocketAddr>()
        .map_err(|_| format!("{key} must be a valid socket address, got '{raw}'"))
}
