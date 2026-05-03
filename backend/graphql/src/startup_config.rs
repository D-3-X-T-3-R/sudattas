use std::net::SocketAddr;
use warp::http::Uri;

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
        let production_env = is_production_env();
        let allowed_origins = parse_allowed_origins(
            std::env::var("ALLOWED_ORIGINS")
                .unwrap_or_default()
                .as_str(),
        )?;

        let rate_limit_per_minute = parse_u32_or_default("RATE_LIMIT_PER_MINUTE", 240)?;
        let webhook_rate_limit_per_minute =
            parse_u32_or_default("RATE_LIMIT_WEBHOOK_PER_MINUTE", 120)?;
        let trust_proxy_headers = parse_bool_or_default("RATE_LIMIT_TRUST_PROXY_HEADERS", false)?;
        let enforce_webhook_secrets =
            parse_bool_or_default("REQUIRE_WEBHOOK_SECRETS", production_env)?;
        let strict_startup_validation =
            parse_bool_or_default("STRICT_STARTUP_VALIDATION", production_env)?;
        let cors_allowlist_required = production_env || strict_startup_validation;

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
        }
        if cors_allowlist_required && allowed_origins.is_none() {
            return Err(
                "ALLOWED_ORIGINS is required in production or when STRICT_STARTUP_VALIDATION=true"
                    .to_string(),
            );
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

fn parse_allowed_origins(raw: &str) -> Result<Option<Vec<String>>, String> {
    let mut values = Vec::new();
    for part in raw.split(',') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == "*" {
            return Err(
                "ALLOWED_ORIGINS wildcard '*' is not allowed when credentials are enabled"
                    .to_string(),
            );
        }
        values.push(normalize_origin(trimmed)?);
    }

    if values.is_empty() {
        return Ok(None);
    }

    values.sort();
    values.dedup();
    Ok(Some(values))
}

fn normalize_origin(origin: &str) -> Result<String, String> {
    let parsed: Uri = origin
        .parse()
        .map_err(|_| format!("ALLOWED_ORIGINS contains invalid origin '{origin}'"))?;
    let scheme = parsed
        .scheme_str()
        .ok_or_else(|| format!("ALLOWED_ORIGINS origin '{origin}' is missing scheme"))?
        .to_ascii_lowercase();
    if scheme != "http" && scheme != "https" {
        return Err(format!(
            "ALLOWED_ORIGINS origin '{origin}' must use http or https"
        ));
    }
    let authority = parsed
        .authority()
        .ok_or_else(|| format!("ALLOWED_ORIGINS origin '{origin}' is missing host"))?
        .as_str()
        .to_ascii_lowercase();
    if authority.contains('@') {
        return Err(format!(
            "ALLOWED_ORIGINS origin '{origin}' must not include userinfo"
        ));
    }
    if authority.contains('*') {
        return Err(format!(
            "ALLOWED_ORIGINS origin '{origin}' must not contain wildcard host"
        ));
    }
    let path = parsed.path();
    if path != "/" && !path.is_empty() {
        return Err(format!(
            "ALLOWED_ORIGINS origin '{origin}' must not include a path"
        ));
    }
    if parsed.query().is_some() {
        return Err(format!(
            "ALLOWED_ORIGINS origin '{origin}' must not include query parameters"
        ));
    }
    Ok(format!("{scheme}://{authority}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, MutexGuard, OnceLock};

    const KEYS: &[&str] = &[
        "APP_ENV",
        "RUST_ENV",
        "NODE_ENV",
        "STRICT_STARTUP_VALIDATION",
        "ALLOWED_ORIGINS",
        "REQUIRE_WEBHOOK_SECRETS",
        "RAZORPAY_WEBHOOK_SECRET",
        "SHIPROCKET_WEBHOOK_SECRET",
        "REDIS_URL",
        "INTERNAL_API_SECRET",
        "OAUTH_DOMAIN",
        "OAUTH_AUDIENCE",
        "GOOGLE_CLIENT_ID",
        "GOOGLE_CLIENT_SECRET",
    ];

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct EnvGuard {
        _lock: MutexGuard<'static, ()>,
        originals: Vec<(&'static str, Option<String>)>,
    }

    impl EnvGuard {
        fn new() -> Self {
            let lock = env_lock()
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let originals = KEYS
                .iter()
                .copied()
                .map(|key| (key, std::env::var(key).ok()))
                .collect();
            Self {
                _lock: lock,
                originals,
            }
        }

        fn set(&self, key: &str, value: &str) {
            std::env::set_var(key, value);
        }

        fn remove(&self, key: &str) {
            std::env::remove_var(key);
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (key, previous) in &self.originals {
                match previous {
                    Some(value) => std::env::set_var(key, value),
                    None => std::env::remove_var(key),
                }
            }
        }
    }

    fn configure_required_strict_envs(env: &EnvGuard) {
        env.set("REDIS_URL", "redis://127.0.0.1:6379");
        env.set("INTERNAL_API_SECRET", "test_internal_secret");
        env.set("OAUTH_DOMAIN", "https://accounts.google.com");
        env.set("OAUTH_AUDIENCE", "client-id.apps.googleusercontent.com");
        env.set("GOOGLE_CLIENT_ID", "client-id.apps.googleusercontent.com");
        env.set("GOOGLE_CLIENT_SECRET", "test-google-secret");
        env.set("REQUIRE_WEBHOOK_SECRETS", "false");
    }

    #[test]
    fn production_requires_webhook_secrets() {
        let env = EnvGuard::new();
        env.set("REQUIRE_WEBHOOK_SECRETS", "true");
        env.remove("RAZORPAY_WEBHOOK_SECRET");
        env.remove("SHIPROCKET_WEBHOOK_SECRET");
        env.set("ALLOWED_ORIGINS", "https://app.example.com");
        env.set("APP_ENV", "development");
        env.set("STRICT_STARTUP_VALIDATION", "false");

        let err = StartupConfig::from_env()
            .expect_err("explicit enforcement should fail without webhook secrets");
        assert!(
            err.contains("RAZORPAY_WEBHOOK_SECRET") || err.contains("SHIPROCKET_WEBHOOK_SECRET")
        );
    }

    #[test]
    fn production_requires_allowed_origins() {
        let env = EnvGuard::new();
        configure_required_strict_envs(&env);
        env.set("APP_ENV", "production");
        env.set("STRICT_STARTUP_VALIDATION", "false");
        env.remove("ALLOWED_ORIGINS");

        let err = StartupConfig::from_env()
            .expect_err("production startup should fail when ALLOWED_ORIGINS is missing");
        assert!(err.contains("ALLOWED_ORIGINS is required in production"));
    }

    #[test]
    fn strict_validation_requires_non_empty_allowed_origins() {
        let env = EnvGuard::new();
        configure_required_strict_envs(&env);
        env.set("APP_ENV", "development");
        env.set("STRICT_STARTUP_VALIDATION", "true");
        env.set("ALLOWED_ORIGINS", "   ,   ");

        let err = StartupConfig::from_env()
            .expect_err("strict startup should fail when ALLOWED_ORIGINS is empty");
        assert!(err.contains("ALLOWED_ORIGINS is required in production"));
    }

    #[test]
    fn strict_validation_rejects_wildcard_allowed_origins() {
        let env = EnvGuard::new();
        configure_required_strict_envs(&env);
        env.set("APP_ENV", "development");
        env.set("STRICT_STARTUP_VALIDATION", "true");
        env.set("ALLOWED_ORIGINS", "*");

        let err = StartupConfig::from_env()
            .expect_err("strict startup should reject wildcard ALLOWED_ORIGINS");
        assert!(err.contains("wildcard '*' is not allowed"));
    }

    #[test]
    fn production_allows_valid_allowed_origins() {
        let env = EnvGuard::new();
        configure_required_strict_envs(&env);
        env.set("APP_ENV", "production");
        env.remove("STRICT_STARTUP_VALIDATION");
        env.set(
            "ALLOWED_ORIGINS",
            "HTTPS://APP.EXAMPLE.COM,https://www.example.com/",
        );

        let cfg = StartupConfig::from_env()
            .expect("production startup should pass with valid ALLOWED_ORIGINS");
        assert_eq!(
            cfg.allowed_origins,
            Some(vec![
                "https://app.example.com".to_string(),
                "https://www.example.com".to_string()
            ])
        );
    }

    #[test]
    fn non_production_without_strict_allows_missing_allowed_origins() {
        let env = EnvGuard::new();
        env.set("APP_ENV", "development");
        env.set("STRICT_STARTUP_VALIDATION", "false");
        env.set("REQUIRE_WEBHOOK_SECRETS", "false");
        env.remove("ALLOWED_ORIGINS");

        let cfg = StartupConfig::from_env().expect(
            "non-production without strict validation should allow missing ALLOWED_ORIGINS",
        );
        assert!(cfg.allowed_origins.is_none());
    }

    #[test]
    fn invalid_origin_format_is_rejected() {
        let err =
            parse_allowed_origins("not-an-origin").expect_err("invalid origin should be rejected");
        assert!(
            err.contains("missing scheme")
                || err.contains("invalid origin")
                || err.contains("must use http or https")
        );
    }
}
