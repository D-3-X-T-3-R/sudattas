/// Resolve the test database URL from `TEST_DATABASE_URL` or `DATABASE_URL`.
#[allow(dead_code)]
pub fn test_db_url() -> String {
    core_operations::load_env_once();
    test_db_url_optional().expect("TEST_DATABASE_URL must be set for integration tests")
}

/// Same as `test_db_url()` but returns `None` when neither env var is set (allows tests to skip).
#[allow(dead_code)]
pub fn test_db_url_optional() -> Option<String> {
    core_operations::load_env_once();
    std::env::var("TEST_DATABASE_URL").ok()
}
