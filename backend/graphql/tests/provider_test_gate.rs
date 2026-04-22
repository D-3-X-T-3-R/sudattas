pub fn should_run_provider_dependent_test(test_name: &str) -> bool {
    let flag = std::env::var("RUN_LIVE_LOGISTICS_TESTS").ok();
    if flag.as_deref() == Some("1") {
        return true;
    }

    let current = flag.unwrap_or_else(|| "<unset>".to_string());
    eprintln!(
        "skipping provider-dependent test `{}`: RUN_LIVE_LOGISTICS_TESTS must be exactly '1' (current: {})",
        test_name, current
    );
    false
}
