//! Unit tests for the app_settings handler (abandoned-cart worker schedule).

use chrono::Utc;
use core_db_entities::entity::app_settings;
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, TransactionTrait};

fn make_setting(key: &str, value: &str) -> app_settings::Model {
    app_settings::Model {
        setting_key: key.to_string(),
        setting_value: value.to_string(),
        updated_at: Utc::now(),
    }
}

#[tokio::test]
async fn get_abandoned_cart_settings_falls_back_to_env_defaults_when_no_rows() {
    use core_operations::handlers::app_settings::get_abandoned_cart_settings;

    // No env vars set in this test process, so this exercises the hardcoded defaults
    // (enabled, 24h delay, 3600s poll) at the very bottom of the fallback chain.
    std::env::remove_var("ABANDONED_CART_DISABLE_WORKER");
    std::env::remove_var("ABANDONED_CART_DELAY_HOURS");
    std::env::remove_var("ABANDONED_CART_POLL_INTERVAL_SEC");

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<app_settings::Model>::new()]) // enabled key: absent
        .append_query_results(vec![Vec::<app_settings::Model>::new()]) // delay_hours key: absent
        .append_query_results(vec![Vec::<app_settings::Model>::new()]) // poll_interval_sec key: absent
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let settings = get_abandoned_cart_settings(&txn)
        .await
        .expect("should succeed with defaults");
    assert!(settings.enabled);
    assert_eq!(settings.delay_hours, 24);
    assert_eq!(settings.poll_interval_sec, 3600);
}

#[tokio::test]
async fn get_abandoned_cart_settings_uses_db_values_when_present() {
    use core_operations::handlers::app_settings::get_abandoned_cart_settings;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![make_setting("abandoned_cart.enabled", "0")]])
        .append_query_results(vec![vec![make_setting("abandoned_cart.delay_hours", "48")]])
        .append_query_results(vec![vec![make_setting(
            "abandoned_cart.poll_interval_sec",
            "7200",
        )]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let settings = get_abandoned_cart_settings(&txn)
        .await
        .expect("should succeed reading DB-backed values");
    assert!(!settings.enabled, "admin-set '0' should disable the worker");
    assert_eq!(settings.delay_hours, 48);
    assert_eq!(settings.poll_interval_sec, 7200);
}

#[tokio::test]
async fn update_abandoned_cart_settings_rejects_non_positive_delay_hours() {
    use core_operations::handlers::app_settings::update_abandoned_cart_settings;

    // No mock results appended at all: validation must fail before any DB call, since
    // enabled/poll_interval_sec are both None here and delay_hours is checked up front.
    let db = MockDatabase::new(DatabaseBackend::MySql).into_connection();
    let txn = db.begin().await.expect("begin");

    let result = update_abandoned_cart_settings(&txn, None, Some(0), None).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn update_abandoned_cart_settings_inserts_when_no_existing_row() {
    use core_operations::handlers::app_settings::update_abandoned_cart_settings;

    // Setting only `enabled`: find_by_id (not found) -> insert, then get_abandoned_cart_settings
    // re-reads all three keys (enabled now present, the other two absent -> env/defaults).
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<app_settings::Model>::new()]) // find existing "enabled" row
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }]) // insert
        .append_query_results(vec![vec![make_setting("abandoned_cart.enabled", "0")]]) // insert's MySQL fetch-back
        .append_query_results(vec![vec![make_setting("abandoned_cart.enabled", "0")]]) // re-read: enabled
        .append_query_results(vec![Vec::<app_settings::Model>::new()]) // re-read: delay_hours (absent)
        .append_query_results(vec![Vec::<app_settings::Model>::new()]) // re-read: poll_interval_sec (absent)
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let result = update_abandoned_cart_settings(&txn, Some(false), None, None).await;
    assert!(result.is_ok(), "update should succeed: {:?}", result.err());
    let settings = result.unwrap();
    assert!(!settings.enabled);
}
