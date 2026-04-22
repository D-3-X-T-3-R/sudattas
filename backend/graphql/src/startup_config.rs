use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct StartupConfig {
    pub redis_url: Option<String>,
    pub allowed_origins: Option<Vec<String>>,
    pub rate_limit_per_minute: u32,
    pub webhook_rate_limit_per_minute: u32,
    pub trust_proxy_headers: bool,
    pub listen_addr: SocketAddr,
    pub enforce_webhook_secrets: bool,
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
        let enforce_webhook_secrets =
            parse_bool_or_default("REQUIRE_WEBHOOK_SECRETS", is_production_env())?;
        let strict_startup_validation =
            parse_bool_or_default("STRICT_STARTUP_VALIDATION", is_production_env())?;

        let listen_addr = parse_socket_addr_or_default("GRAPHQL_LISTEN_ADDR", "0.0.0.0:8080")?;

        if enforce_webhook_secrets {
            require_non_empty_env("RAZORPAY_WEBHOOK_SECRET")?;
            require_non_empty_env("SHIPROCKET_WEBHOOK_SECRET")?;
        }
        if strict_startup_validation {
            for key in [
                "REDIS_URL",
                "INTERNAL_API_SECRET",
                "OAUTH_DOMAIN",
                "OAUTH_AUDIENCE",
                "GOOGLE_CLIENT_ID",
                "GOOGLE_CLIENT_SECRET",
            ] {
                require_non_empty_env(key)?;
            }
            if allowed_origins.is_none() {
                return Err(
                    "ALLOWED_ORIGINS is required when strict startup validation is enabled"
                        .to_string(),
                );
            }
        }

        Ok(Self {
            redis_url,
            allowed_origins,
            rate_limit_per_minute,
            webhook_rate_limit_per_minute,
            trust_proxy_headers,
            listen_addr,
            enforce_webhook_secrets,
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
            "{key} is required when webhook secret enforcement is enabled"
        )),
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

#[cfg(test)]
mod tests {
    use super::StartupConfig;

    #[test]
    fn production_requires_webhook_secrets() {
        std::env::set_var("REQUIRE_WEBHOOK_SECRETS", "true");
        std::env::remove_var("RAZORPAY_WEBHOOK_SECRET");
        std::env::remove_var("SHIPROCKET_WEBHOOK_SECRET");

        let err = StartupConfig::from_env()
            .expect_err("explicit enforcement should fail without webhook secrets");
        assert!(
            err.contains("RAZORPAY_WEBHOOK_SECRET") || err.contains("SHIPROCKET_WEBHOOK_SECRET")
        );

        std::env::remove_var("REQUIRE_WEBHOOK_SECRETS");
    }
}
