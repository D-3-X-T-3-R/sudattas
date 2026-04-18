pub use sea_orm;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Duration;
use tracing::info;

pub type CoreDatabaseConnection = sea_orm::DatabaseConnection;

pub mod entity;

static DOTENV_LOADED: OnceLock<()> = OnceLock::new();

fn load_env_once() {
    DOTENV_LOADED.get_or_init(|| {
        let candidates = [
            PathBuf::from("..").join(".env"),
            PathBuf::from(".env"),
            PathBuf::from("backend").join(".env"),
        ];
        for candidate in candidates {
            if candidate.exists() {
                let _ = dotenvy::from_path(candidate);
                break;
            }
        }
    });
}

/// Create a database connection pool using `DATABASE_URL` and optional pool settings.
///
/// All pool parameters are read from environment variables so they can be tuned
/// per-environment without recompiling:
///
/// | Variable                | Default | Description                          |
/// |-------------------------|---------|--------------------------------------|
/// | `DATABASE_URL`          | —       | MySQL connection string (required)   |
/// | `DB_MAX_CONNECTIONS`    | 10      | Maximum pool connections             |
/// | `DB_MIN_CONNECTIONS`    | 1       | Minimum idle connections kept warm   |
/// | `DB_CONNECT_TIMEOUT_SEC`| 30      | Max wait to acquire a connection (s) |
/// | `DB_IDLE_TIMEOUT_SEC`   | 600     | Idle connection TTL (s)              |
/// | `DB_MAX_LIFETIME_SEC`   | 1800    | Max connection lifetime (s)          |
pub async fn get_db() -> Result<DatabaseConnection, DbErr> {
    load_env_once();
    let conn_str = std::env::var("DATABASE_URL")
        .map_err(|_| DbErr::Custom("DATABASE_URL not set".to_string()))?;

    let max_conn: u32 = env_u32("DB_MAX_CONNECTIONS", 10);
    let min_conn: u32 = env_u32("DB_MIN_CONNECTIONS", 1);
    let connect_timeout = Duration::from_secs(env_u64("DB_CONNECT_TIMEOUT_SEC", 30));
    let idle_timeout = Duration::from_secs(env_u64("DB_IDLE_TIMEOUT_SEC", 600));
    let max_lifetime = Duration::from_secs(env_u64("DB_MAX_LIFETIME_SEC", 1800));

    info!(
        max_connections = max_conn,
        min_connections = min_conn,
        connect_timeout_sec = env_u64("DB_CONNECT_TIMEOUT_SEC", 30),
        "Connecting to database"
    );

    let mut opts = ConnectOptions::new(conn_str);
    opts.max_connections(max_conn)
        .min_connections(min_conn)
        .connect_timeout(connect_timeout)
        .idle_timeout(idle_timeout)
        .max_lifetime(max_lifetime)
        .sqlx_logging(false); // use RUST_LOG=sqlx=trace to enable SQL logging

    Database::connect(opts).await
}

fn env_u32(key: &str, default: u32) -> u32 {
    load_env_once();
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn env_u64(key: &str, default: u64) -> u64 {
    load_env_once();
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}
