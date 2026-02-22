pub use sea_orm;
use sea_orm::{Database, DatabaseConnection, DbErr};

use tracing::info;

pub type CoreDatabaseConnection = sea_orm::DatabaseConnection;

pub mod entity;

pub async fn get_db() -> Result<DatabaseConnection, DbErr> {
    let conn_str = std::env::var("DATABASE_URL")
        .map_err(|_| DbErr::Custom("DATABASE_URL not set".to_string()))?;

    info!("Connecting to database");

    Database::connect(&conn_str).await
}
