pub use sea_orm;
use sea_orm::{Database, DatabaseConnection, DbErr};
use tracing::info;

pub type CoreDatabaseConnection = sea_orm::DatabaseConnection;

pub mod entity;

pub async fn get_db() -> Result<DatabaseConnection, DbErr> {
    let db_user = std::env::var("DB_USER").expect("DB_USER must be set");
    let db_password = std::env::var("DB_PASSWORD").expect("DB_PASSWORD must be set");
    let db_host = std::env::var("DB_HOST").expect("DB_HOST must be set");
    let db_port = std::env::var("DB_PORT").expect("DB_PORT must be set");
    let db_name = std::env::var("DB_NAME").expect("DB_NAME must be set");

    let conn_str = format!(
        "mysql://{}:{}@{}:{}/{}",
        db_user, db_password, db_host, db_port, db_name
    );

    info!("Connecting to database: {conn_str}");

    Database::connect(conn_str).await
}
