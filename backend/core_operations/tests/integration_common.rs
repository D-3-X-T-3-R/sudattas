use sea_orm::Database;

/// Resolve the test database URL from `TEST_DATABASE_URL` or `DATABASE_URL`.
pub fn test_db_url() -> String {
    std::env::var("TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .expect("TEST_DATABASE_URL or DATABASE_URL must be set for integration tests")
}

/// Convenience helper for tests that prefer to get a live connection directly.
pub async fn connect_test_db() -> sea_orm::DatabaseConnection {
    Database::connect(&test_db_url())
        .await
        .expect("connect to test DB")
}

