use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct StartupConfig {
    pub redis_url: Option<String>,
    pub allowed_origins: Option<Vec<String>>,
    pub rate_limit_per_minute: u32,
    pub webhook_rate_limit_per_minute: u32,
    pub trust_proxy_headers: bool,
    pub listen_addr: SocketAddr,
}

impl StartupConfig {
    pub fn from_env() -> Result<Self, String> {
        let redis_url = std::env::var("REDIS_URL").ok();

        let allowed_origins = {
            let values = std::env::var("ALLOWED_ORIGINS")
                .unwrap_or_default()
                .split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>();
            if values.is_empty() {
                None
            } else {
                Some(values)
            }
        };

        let rate_limit_per_minute = parse_u32_or_default("RATE_LIMIT_PER_MINUTE", 240)?;
        let webhook_rate_limit_per_minute =
            parse_u32_or_default("RATE_LIMIT_WEBHOOK_PER_MINUTE", 120)?;
        let trust_proxy_headers = parse_bool_or_default("RATE_LIMIT_TRUST_PROXY_HEADERS", false)?;

        let listen_addr = parse_socket_addr_or_default("GRAPHQL_LISTEN_ADDR", "0.0.0.0:8080")?;

        Ok(Self {
            redis_url,
            allowed_origins,
            rate_limit_per_minute,
            webhook_rate_limit_per_minute,
            trust_proxy_headers,
            listen_addr,
        })
    }
}

fn parse_u32_or_default(key: &str, default: u32) -> Result<u32, String> {
    match std::env::var(key) {
        Ok(raw) => raw
            .trim()
            .parse::<u32>()
            .map_err(|_| format!("{key} must be a non-negative integer, got '{raw}'")),
        Err(_) => Ok(default),
    }
}

fn parse_socket_addr_or_default(key: &str, default: &str) -> Result<SocketAddr, String> {
    let raw = std::env::var(key).unwrap_or_else(|_| default.to_string());
    raw.parse::<SocketAddr>()
        .map_err(|_| format!("{key} must be a valid socket address, got '{raw}'"))
}

fn parse_bool_or_default(key: &str, default: bool) -> Result<bool, String> {
    match std::env::var(key) {
        Ok(raw) => {
            let value = raw.trim().to_ascii_lowercase();
            match value.as_str() {
                "1" | "true" | "yes" | "on" => Ok(true),
                "0" | "false" | "no" | "off" => Ok(false),
                _ => Err(format!(
                    "{key} must be one of 1/0,true/false,yes/no,on/off, got '{raw}'"
                )),
            }
        }
        Err(_) => Ok(default),
    }
}
