pub use sea_orm;
use sea_orm::{Database, DatabaseConnection, DbErr};
use std::env;
use tracing::info;

pub type CoreDatabaseConnection = sea_orm::DatabaseConnection;

pub mod entity;

// #[tokio::main]
pub async fn get_db() -> Result<DatabaseConnection, DbErr> {
    let conn_str = "mysql://root:12345678@localhost:3306/SUDATTAS";

    info!("Connecting to database: {conn_str}");

    Database::connect(conn_str).await
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
