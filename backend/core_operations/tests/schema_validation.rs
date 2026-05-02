//! Schema bootstrap validation tests.

use core_db_entities::get_db;
use core_operations::schema_guard::{
    validate_required_schema, validate_required_tables_from_found, REQUIRED_TABLES,
};
mod integration_common;
use integration_common::test_db_url;

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn fresh_bootstrap_validation_passes_with_required_tables() {
    std::env::set_var("TEST_DATABASE_URL", test_db_url());

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
