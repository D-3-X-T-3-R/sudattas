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
        let strict_startup_validation =
            parse_bool_or_default("STRICT_STARTUP_VALIDATION", is_production_env())?;

        if strict_startup_validation {
            for key in [
                "DATABASE_URL",
                "REDIS_URL",
                "RAZORPAY_KEY_ID",
                "RAZORPAY_KEY_SECRET",
                "SHIPROCKET_EMAIL",
                "SHIPROCKET_PASSWORD",
                "SHIPROCKET_PICKUP_LOCATION",
                "CANCEL_WINDOW_HOURS",
                "PICKUP_DELAY_HOURS",
                "FREE_SHIPPING_THRESHOLD_MINOR",
                "RETURN_WINDOW_DAYS",
                "R2_ACCOUNT_ID",
                "R2_ACCESS_KEY_ID",
                "R2_SECRET_ACCESS_KEY",
                "R2_ENDPOINT",
                "R2_BUCKET_NAME",
                "R2_PUBLIC_URL",
            ] {
                require_non_empty_env(key)?;
            }
        }

        Ok(Self {
            grpc_server_addr,
            grpc_metrics_addr,
        })
    }
}

fn is_production_env() -> bool {
    ["APP_ENV", "RUST_ENV", "NODE_ENV"]
        .into_iter()
        .filter_map(|key| std::env::var(key).ok())
        .map(|value| value.trim().to_ascii_lowercase())
        .any(|value| value == "production")
}

fn require_non_empty_env(key: &str) -> Result<(), String> {
    match std::env::var(key) {
        Ok(raw) if !raw.trim().is_empty() => Ok(()),
        _ => Err(format!(
            "{key} is required when strict startup validation is enabled"
        )),
    }
}

fn parse_socket_addr_or_default(key: &str, default: &str) -> Result<SocketAddr, String> {
    let raw = std::env::var(key).unwrap_or_else(|_| default.to_string());
    raw.parse::<SocketAddr>()
        .map_err(|_| format!("{key} must be a valid socket address, got '{raw}'"))
}

fn parse_bool_or_default(key: &str, default: bool) -> Result<bool, String> {
    match std::env::var(key) {
        Ok(raw) => match raw.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => Ok(true),
            "0" | "false" | "no" | "off" => Ok(false),
            _ => Err(format!(
                "{key} must be one of 1/0,true/false,yes/no,on/off, got '{raw}'"
            )),
        },
        Err(_) => Ok(default),
    }
}
