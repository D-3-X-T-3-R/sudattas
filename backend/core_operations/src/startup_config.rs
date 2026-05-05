use std::net::SocketAddr;

const STRICT_REQUIRED_ENV_KEYS: &[&str] = &[
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
];

#[derive(Debug, Clone)]
pub struct StartupConfig {
    pub grpc_server_addr: SocketAddr,
    pub grpc_metrics_addr: SocketAddr,
}

impl StartupConfig {
    pub fn from_env() -> Result<Self, String> {
        let grpc_server_addr = parse_socket_addr_or_default("GRPC_SERVER", "0.0.0.0:50051")?;
        let grpc_metrics_addr = parse_socket_addr_or_default("GRPC_METRICS_ADDR", "0.0.0.0:9090")?;
        let production_env = is_production_env();
        let strict_startup_validation =
            parse_bool_or_default("STRICT_STARTUP_VALIDATION", production_env)?;
        let grpc_auth_required = production_env || strict_startup_validation;

        if strict_startup_validation {
            for key in STRICT_REQUIRED_ENV_KEYS {
                require_non_empty_env(key)?;
            }
        }
        if grpc_auth_required {
            require_grpc_auth_token()?;
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

fn require_grpc_auth_token() -> Result<(), String> {
    match std::env::var("GRPC_AUTH_TOKEN") {
        Ok(raw) if !raw.trim().is_empty() => Ok(()),
        _ => Err(
            "GRPC_AUTH_TOKEN is required in production or when STRICT_STARTUP_VALIDATION=true"
                .to_string(),
        ),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, MutexGuard, OnceLock};

    fn startup_env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn tracked_env_keys() -> Vec<&'static str> {
        let mut keys = vec![
            "APP_ENV",
            "RUST_ENV",
            "NODE_ENV",
            "STRICT_STARTUP_VALIDATION",
            "GRPC_SERVER",
            "GRPC_METRICS_ADDR",
        ];
        keys.extend(STRICT_REQUIRED_ENV_KEYS.iter().copied());
        keys
    }

    struct StartupEnvGuard {
        _lock: MutexGuard<'static, ()>,
        originals: Vec<(&'static str, Option<String>)>,
    }

    impl StartupEnvGuard {
        fn new() -> Self {
            let lock = startup_env_lock()
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let originals = tracked_env_keys()
                .into_iter()
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

    impl Drop for StartupEnvGuard {
        fn drop(&mut self) {
            for (key, previous) in &self.originals {
                match previous {
                    Some(value) => std::env::set_var(key, value),
                    None => std::env::remove_var(key),
                }
            }
        }
    }

    fn set_all_strict_required_envs(env: &StartupEnvGuard) {
        for key in STRICT_REQUIRED_ENV_KEYS {
            env.set(key, &format!("{key}_value"));
        }
    }

    #[test]
    fn production_mode_requires_grpc_auth_token() {
        let env = StartupEnvGuard::new();
        set_all_strict_required_envs(&env);
        env.remove("GRPC_AUTH_TOKEN");
        env.set("APP_ENV", "production");
        env.remove("RUST_ENV");
        env.remove("NODE_ENV");
        env.remove("STRICT_STARTUP_VALIDATION");

        let err = StartupConfig::from_env()
            .expect_err("startup should fail in production when GRPC_AUTH_TOKEN is missing");
        assert!(
            err.contains(
                "GRPC_AUTH_TOKEN is required in production or when STRICT_STARTUP_VALIDATION=true"
            ),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn production_mode_still_requires_grpc_auth_token_when_strict_override_is_false() {
        let env = StartupEnvGuard::new();
        set_all_strict_required_envs(&env);
        env.remove("GRPC_AUTH_TOKEN");
        env.set("APP_ENV", "production");
        env.set("STRICT_STARTUP_VALIDATION", "false");

        let err = StartupConfig::from_env().expect_err(
            "startup should fail in production when GRPC_AUTH_TOKEN is missing, even with strict override disabled",
        );
        assert!(
            err.contains(
                "GRPC_AUTH_TOKEN is required in production or when STRICT_STARTUP_VALIDATION=true"
            ),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn strict_validation_rejects_empty_grpc_auth_token() {
        let env = StartupEnvGuard::new();
        set_all_strict_required_envs(&env);
        env.set("STRICT_STARTUP_VALIDATION", "true");
        env.set("APP_ENV", "development");
        env.set("GRPC_AUTH_TOKEN", "   ");

        let err = StartupConfig::from_env()
            .expect_err("startup should fail when GRPC_AUTH_TOKEN is empty/whitespace");
        assert!(
            err.contains(
                "GRPC_AUTH_TOKEN is required in production or when STRICT_STARTUP_VALIDATION=true"
            ),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn production_mode_allows_startup_when_grpc_auth_token_is_set() {
        let env = StartupEnvGuard::new();
        set_all_strict_required_envs(&env);
        env.set("APP_ENV", "production");
        env.remove("RUST_ENV");
        env.remove("NODE_ENV");
        env.remove("STRICT_STARTUP_VALIDATION");
        env.set("GRPC_AUTH_TOKEN", "expected_token");

        let config = StartupConfig::from_env()
            .expect("startup should pass in production when GRPC_AUTH_TOKEN is configured");
        assert_eq!(
            config.grpc_server_addr,
            "0.0.0.0:50051".parse().expect("valid grpc addr")
        );
        assert_eq!(
            config.grpc_metrics_addr,
            "0.0.0.0:9090".parse().expect("valid metrics addr")
        );
    }

    #[test]
    fn non_production_without_strict_validation_allows_missing_grpc_auth_token() {
        let env = StartupEnvGuard::new();
        env.set("APP_ENV", "development");
        env.set("STRICT_STARTUP_VALIDATION", "false");
        env.remove("GRPC_AUTH_TOKEN");

        let config = StartupConfig::from_env().expect(
            "non-production startup should allow missing GRPC_AUTH_TOKEN when strict validation is disabled",
        );
        assert_eq!(
            config.grpc_server_addr,
            "0.0.0.0:50051".parse().expect("valid grpc addr")
        );
    }
}
