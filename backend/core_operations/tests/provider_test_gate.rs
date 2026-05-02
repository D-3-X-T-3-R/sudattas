fn env_non_empty(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn db_name_from_url(db_url: &str) -> Option<String> {
    let without_query = db_url.split('?').next().unwrap_or(db_url);
    without_query
        .rsplit('/')
        .next()
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .map(str::to_string)
}

fn db_name_looks_like_test(db_url: &str) -> bool {
    let Some(name) = db_name_from_url(db_url) else {
        return false;
    };
    let upper = name.to_ascii_uppercase();
    upper.contains("TEST") || upper.contains("TESTING") || upper.contains("SUDATTAS_TEST")
}

#[allow(dead_code)]
pub fn should_run_provider_dependent_test(test_name: &str) -> bool {
    let flag = std::env::var("RUN_LIVE_LOGISTICS_TESTS").ok();
    if flag.as_deref() != Some("1") {
        let current = flag.unwrap_or_else(|| "<unset>".to_string());
        eprintln!(
            "skipping provider-dependent test `{}`: RUN_LIVE_LOGISTICS_TESTS must be exactly '1' (current: {})",
            test_name, current
        );
        return false;
    }

    let Some(test_db_url) = env_non_empty("TEST_DATABASE_URL") else {
        eprintln!(
            "skipping provider-dependent test `{}`: TEST_DATABASE_URL must be set explicitly (DATABASE_URL fallback is disallowed for safety)",
            test_name
        );
        return false;
    };
    if !db_name_looks_like_test(test_db_url.as_str()) {
        eprintln!(
            "skipping provider-dependent test `{}`: TEST_DATABASE_URL does not appear to be an isolated test DB (db_name={:?})",
            test_name,
            db_name_from_url(test_db_url.as_str())
        );
        return false;
    }

    true
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LiveShiprocketSafety {
    pub test_database_url: String,
}

#[allow(dead_code)]
pub fn validate_live_shiprocket_test_safety(
    test_name: &str,
) -> Result<LiveShiprocketSafety, String> {
    let run_flag = std::env::var("RUN_LIVE_LOGISTICS_TESTS")
        .ok()
        .map(|value| value.trim().to_string())
        .unwrap_or_default();
    if run_flag != "1" {
        return Err(format!(
            "RUN_LIVE_LOGISTICS_TESTS must be exactly '1' (current: {})",
            if run_flag.is_empty() {
                "<unset>".to_string()
            } else {
                run_flag
            }
        ));
    }

    let test_db_url = env_non_empty("TEST_DATABASE_URL")
        .ok_or_else(|| "TEST_DATABASE_URL must be set for live Shiprocket tests".to_string())?;
    if !db_name_looks_like_test(test_db_url.as_str()) {
        return Err(format!(
            "TEST_DATABASE_URL does not appear to target a dedicated test DB (db_name={:?})",
            db_name_from_url(test_db_url.as_str())
        ));
    }

    let confirm = env_non_empty("SHIPROCKET_LIVE_TEST_CONFIRM").ok_or_else(|| {
        "SHIPROCKET_LIVE_TEST_CONFIRM must be set to acknowledge real-provider traffic".to_string()
    })?;
    if confirm != "I_UNDERSTAND_THIS_HITS_REAL_PROVIDER" {
        return Err(format!(
            "SHIPROCKET_LIVE_TEST_CONFIRM value mismatch for `{test_name}`"
        ));
    }

    for key in [
        "SHIPROCKET_API_BASE",
        "SHIPROCKET_EMAIL",
        "SHIPROCKET_PASSWORD",
        "SHIPROCKET_PICKUP_LOCATION",
        "RAZORPAY_KEY_ID",
        "RAZORPAY_KEY_SECRET",
    ] {
        if env_non_empty(key).is_none() {
            return Err(format!("missing required env: {key}"));
        }
    }

    let razorpay_key_id = env_non_empty("RAZORPAY_KEY_ID")
        .ok_or_else(|| "missing required env: RAZORPAY_KEY_ID".to_string())?;
    if !razorpay_key_id.starts_with("rzp_test_") {
        return Err(format!(
            "RAZORPAY_KEY_ID must be test-mode (expected prefix `rzp_test_`, got masked len={})",
            razorpay_key_id.len()
        ));
    }

    Ok(LiveShiprocketSafety {
        test_database_url: test_db_url,
    })
}
