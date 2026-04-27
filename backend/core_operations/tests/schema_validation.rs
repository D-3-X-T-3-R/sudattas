//! Schema bootstrap validation tests.

use core_db_entities::get_db;
use core_operations::schema_guard::{
    validate_required_schema, validate_required_tables_from_found, REQUIRED_TABLES,
};

#[tokio::test]
async fn fresh_bootstrap_validation_passes_with_required_tables() {
    let test_db = std::env::var("TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .expect("TEST_DATABASE_URL or DATABASE_URL must be set for schema validation test");
    std::env::set_var("DATABASE_URL", test_db);

    let db = get_db()
        .await
        .expect("database connection should succeed for schema validation test");
    validate_required_schema(&db)
        .await
        .expect("required migration tables must exist after bootstrap");
}

#[test]
fn missing_required_table_is_a_hard_failure() {
    let found = REQUIRED_TABLES
        .iter()
        .copied()
        .filter(|name| *name != "ReturnRequestItems")
        .collect::<Vec<_>>();
    let err =
        validate_required_tables_from_found(found).expect_err("missing required table must fail");
    assert!(err.contains("ReturnRequestItems"));
}
