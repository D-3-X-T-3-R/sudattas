//! GraphQL schema and resolver tests. Run with: `cargo test -p graphql`
//!
//! Covers: apiVersion, authInfo, context variants (JWT/session/none), error handling,
//! query structure, and Phase 8 depth/complexity limits.

use graphql::graphql_limits;
use graphql::security::jwks_loader::JWKey;
use graphql::{schema, AuthSource, Context, JWKSet};

/// Convert juniper::Value (data root) to serde_json::Value for assertions.
fn to_json(res: &juniper::Value) -> serde_json::Value {
    serde_json::to_value(res).expect("juniper Value is serializable")
}

// =============================================================================
// apiVersion
// =============================================================================

#[test]
fn test_api_version_query() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::Jwt("user_123".to_string())),
        request_id: None,
        idempotency_key: None,
    };

    let (res, _errors) = juniper::execute_sync(
        r#"{ apiVersion }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .unwrap();

    let data = to_json(&res);
    assert!(data.get("apiVersion").is_some());
    assert_eq!(
        data.get("apiVersion").and_then(|v| v.as_str()),
        Some("2.0.0")
    );
}

#[test]
fn test_api_version_format_semver_like() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::Jwt("any".to_string())),
        request_id: None,
        idempotency_key: None,
    };

    let (res, _) = juniper::execute_sync(
        r#"{ apiVersion }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .unwrap();

    let data = to_json(&res);
    let version = data.get("apiVersion").and_then(|v| v.as_str()).unwrap();
    // Expect major.minor.patch
    let parts: Vec<&str> = version.split('.').collect();
    assert_eq!(
        parts.len(),
        3,
        "apiVersion should be semver-like (e.g. 2.0.0)"
    );
    assert!(parts[0].parse::<u32>().is_ok());
    assert!(parts[1].parse::<u32>().is_ok());
    assert!(parts[2].parse::<u32>().is_ok());
}

#[test]
fn test_api_version_with_session_context() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: Some("redis://127.0.0.1".to_string()),
        auth: Some(AuthSource::Session("guest_99".to_string())),
        request_id: None,
        idempotency_key: None,
    };

    let (res, errors) = juniper::execute_sync(
        r#"{ apiVersion }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .unwrap();

    assert!(errors.is_empty());
    let data = to_json(&res);
    assert_eq!(
        data.get("apiVersion").and_then(|v| v.as_str()),
        Some("2.0.0")
    );
}

// =============================================================================
// authInfo
// =============================================================================

#[test]
fn test_auth_info_query() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: Some("redis://127.0.0.1:6379".to_string()),
        auth: Some(AuthSource::Session("42".to_string())),
        request_id: None,
        idempotency_key: None,
    };

    let (res, errors) = juniper::execute_sync(
        r#"{ authInfo { sessionEnabled jwksKeyCount currentUserId } }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .unwrap();

    assert!(errors.is_empty(), "{:?}", errors);
    let data = to_json(&res);
    let info = data
        .get("authInfo")
        .and_then(|v| v.as_object())
        .expect("authInfo object");
    assert_eq!(
        info.get("sessionEnabled").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(info.get("jwksKeyCount").and_then(|v| v.as_i64()), Some(0));
    assert_eq!(
        info.get("currentUserId").and_then(|v| v.as_str()),
        Some("42")
    );
}

fn dummy_jwk() -> JWKey {
    JWKey {
        e: "AQAB".to_string(),
        n: "n".to_string(),
        kty: "RSA".to_string(),
        r#use: "sig".to_string(),
        alg: "RS256".to_string(),
        kid: "k1".to_string(),
    }
}

#[test]
fn test_auth_info_jwt_context() {
    let ctx = Context {
        jwks: JWKSet {
            keys: vec![dummy_jwk(), dummy_jwk()],
        },
        redis_url: None,
        auth: Some(AuthSource::Jwt("jwt_user_456".to_string())),
        request_id: None,
        idempotency_key: None,
    };

    let (res, errors) = juniper::execute_sync(
        r#"{ authInfo { sessionEnabled jwksKeyCount currentUserId } }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .unwrap();

    assert!(errors.is_empty());
    let data = to_json(&res);
    let info = data
        .get("authInfo")
        .and_then(|v| v.as_object())
        .expect("authInfo");
    assert_eq!(
        info.get("sessionEnabled").and_then(|v| v.as_bool()),
        Some(false)
    );
    assert_eq!(info.get("jwksKeyCount").and_then(|v| v.as_i64()), Some(2));
    assert_eq!(
        info.get("currentUserId").and_then(|v| v.as_str()),
        Some("jwt_user_456")
    );
}

#[test]
fn test_auth_info_session_disabled_when_no_redis() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::Session("sid".to_string())),
        request_id: None,
        idempotency_key: None,
    };

    let (res, errors) = juniper::execute_sync(
        r#"{ authInfo { sessionEnabled jwksKeyCount currentUserId } }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .unwrap();

    assert!(errors.is_empty());
    let data = to_json(&res);
    let info = data
        .get("authInfo")
        .and_then(|v| v.as_object())
        .expect("authInfo");
    assert_eq!(
        info.get("sessionEnabled").and_then(|v| v.as_bool()),
        Some(false)
    );
    assert_eq!(
        info.get("currentUserId").and_then(|v| v.as_str()),
        Some("sid")
    );
}

// =============================================================================
// Combined queries and structure
// =============================================================================

#[test]
fn test_multiple_root_fields_in_one_query() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::Jwt("u".to_string())),
        request_id: None,
        idempotency_key: None,
    };

    let (res, errors) = juniper::execute_sync(
        r#"{ apiVersion authInfo { sessionEnabled currentUserId } }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .unwrap();

    assert!(errors.is_empty());
    let data = to_json(&res);
    assert_eq!(
        data.get("apiVersion").and_then(|v| v.as_str()),
        Some("2.0.0")
    );
    let info = data
        .get("authInfo")
        .and_then(|v| v.as_object())
        .expect("authInfo");
    assert!(info.contains_key("sessionEnabled"));
    assert!(info.contains_key("currentUserId"));
}

// =============================================================================
// Error handling
// =============================================================================
// place_order auth (JWT required)
// =============================================================================

#[tokio::test]
async fn test_place_order_requires_jwt_rejects_session_only() {
    // place_order requires full login (JWT). Session-only (guest) must get an error.
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::Session("guest_99".to_string())),
        request_id: None,
        idempotency_key: None,
    };

    let (res, errors) = juniper::execute(
        r#"mutation { placeOrder(order: { shippingAddressId: "1" }) { orderId } }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .await
    .unwrap();

    assert!(
        !errors.is_empty(),
        "place_order with session-only auth should return error, got: {:?}",
        (res, errors)
    );
    let err_str = format!("{:?}", errors[0]);
    assert!(
        err_str.to_lowercase().contains("login"),
        "error should mention login: {}",
        err_str
    );
}

#[tokio::test]
async fn test_place_order_with_jwt_accepts_request() {
    // With JWT context, place_order mutation is accepted (gRPC may fail if server down).
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::Jwt("user_42".to_string())),
        request_id: None,
        idempotency_key: None,
    };

    let schema = schema();
    let result = juniper::execute(
        r#"mutation { placeOrder(order: { shippingAddressId: "1" }) { orderId } }"#,
        None,
        &schema,
        &juniper::Variables::new(),
        &ctx,
    )
    .await;

    // Either success (if gRPC up) or error from gRPC/unavailable - not "Login required"
    if let Ok((_, errors)) = result {
        if !errors.is_empty() {
            let err_str = format!("{:?}", errors[0]);
            assert!(
                !err_str.to_lowercase().contains("login required"),
                "with JWT we should not get login required: {}",
                err_str
            );
        }
    }
}

// =============================================================================

#[test]
fn test_invalid_query_syntax_returns_errors() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::Jwt("u".to_string())),
        request_id: None,
        idempotency_key: None,
    };

    let schema = schema();
    let result = juniper::execute_sync(
        r#"{ apiVersion "#, // unclosed string
        None,
        &schema,
        &juniper::Variables::new(),
        &ctx,
    );

    match &result {
        Err(_) => {}
        Ok((_, errors)) => assert!(!errors.is_empty(), "Invalid query should yield errors"),
    }
}

#[test]
fn test_unknown_field_returns_errors() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::Jwt("u".to_string())),
        request_id: None,
        idempotency_key: None,
    };

    let schema = schema();
    let result = juniper::execute_sync(
        r#"{ apiVersion nonExistentField }"#,
        None,
        &schema,
        &juniper::Variables::new(),
        &ctx,
    );

    match &result {
        Err(_) => {} // validation error (e.g. unknown field) returns Err
        Ok((_, errors)) => assert!(!errors.is_empty(), "unknown field should yield errors"),
    }
}

// =============================================================================
// Money type (GraphQL schema: amount_paise, currency, formatted)
// =============================================================================

#[test]
fn test_money_type_in_schema() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::Jwt("u".to_string())),
        request_id: None,
        idempotency_key: None,
    };

    let (res, errors) = juniper::execute_sync(
        r#"{ __type(name: "Money") { name kind fields { name } } }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .unwrap();

    assert!(
        errors.is_empty(),
        "introspection should not error: {:?}",
        errors
    );
    let data = to_json(&res);
    let typ = data.get("__type").expect("__type(Money) should be present");
    assert_eq!(typ.get("name").and_then(|v| v.as_str()), Some("Money"));
    let fields: Vec<String> = typ
        .get("fields")
        .and_then(|f| f.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|f| f.get("name").and_then(|n| n.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default();
    assert!(
        fields.contains(&"amountPaise".to_string()),
        "Money.amountPaise"
    );
    assert!(fields.contains(&"currency".to_string()), "Money.currency");
    assert!(fields.contains(&"formatted".to_string()), "Money.formatted");
}

// =============================================================================
// Phase 8: Query depth limit
// =============================================================================

#[test]
fn test_query_depth_limit_rejects_deep_query() {
    // Depth 11: root + 10 nested levels
    let query = "{ a { b { c { d { e { f { g { h { i { j { x } } } } } } } } } } }";
    let err = graphql_limits::check_query_depth(query, 10).unwrap_err();
    assert!(
        err.contains("exceeds maximum"),
        "depth limit should return clear error: {}",
        err
    );
}

/// Integration test (no server): handler returns 400 when query exceeds depth limit.
#[tokio::test]
async fn integration_handler_rejects_deep_query_with_400() {
    use graphql::graphql_handler;
    use std::sync::Arc;

    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::Jwt("u".to_string())),
        request_id: None,
        idempotency_key: None,
    };
    let deep_query = "{ a { b { c { d { e { f { g { h { i { j { x } } } } } } } } } } }";
    let body =
        warp::hyper::body::Bytes::from(serde_json::json!({ "query": deep_query }).to_string());
    let response = graphql_handler::handle_graphql_request(ctx, body, Arc::new(schema()))
        .await
        .expect("handler should not reject");
    assert_eq!(
        response.status(),
        warp::http::StatusCode::BAD_REQUEST,
        "deep query should return 400"
    );
}

// =============================================================================
// P1 Observability: metrics recorded by handler
// =============================================================================

/// Integration test (no server): GraphQL handler runs and records request metrics; when we have
/// the Prometheus handle, rendered output should contain our metric names.
#[tokio::test]
async fn integration_handler_records_graphql_metrics() {
    use graphql::graphql_handler;
    use std::sync::Arc;

    let handle = metrics_exporter_prometheus::PrometheusBuilder::new()
        .install_recorder()
        .ok();

    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::Jwt("u".to_string())),
        request_id: None,
        idempotency_key: None,
    };
    let body = warp::hyper::body::Bytes::from(
        serde_json::json!({ "query": "{ apiVersion }" }).to_string(),
    );
    let response = graphql_handler::handle_graphql_request(ctx, body, Arc::new(schema()))
        .await
        .expect("handler should not reject");

    assert!(
        response.status().is_success(),
        "handler should return 2xx, got {}",
        response.status()
    );

    if let Some(h) = handle {
        let out = h.render();
        if !out.is_empty() {
            assert!(
                out.contains("graphql_requests_total"),
                "metrics output should contain graphql_requests_total: {}",
                out
            );
            assert!(
                out.contains("graphql_request_duration_seconds"),
                "metrics output should contain graphql_request_duration_seconds: {}",
                out
            );
        }
    }
}

// =============================================================================
// Phase 8: Query depth limit
// =============================================================================

/// Integration test (no server): when GRAPHQL_MAX_QUERY_COMPLEXITY is set, handler returns 400 for high-complexity query.
#[tokio::test]
async fn integration_handler_rejects_high_complexity_with_400_when_limit_set() {
    use graphql::graphql_handler;
    use std::sync::Arc;

    // Query with complexity 1+2+3+4+5 = 15 (five nesting levels)
    let complex_query = "{ a { b { c { d { e } } } } }";
    assert_eq!(
        graphql::graphql_limits::compute_query_complexity(complex_query),
        15
    );

    std::env::set_var("GRAPHQL_MAX_QUERY_COMPLEXITY", "10");
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::Jwt("u".to_string())),
        request_id: None,
        idempotency_key: None,
    };
    let body =
        warp::hyper::body::Bytes::from(serde_json::json!({ "query": complex_query }).to_string());
    let response = graphql_handler::handle_graphql_request(ctx, body, Arc::new(schema()))
        .await
        .expect("handler should not reject");
    std::env::remove_var("GRAPHQL_MAX_QUERY_COMPLEXITY");

    assert_eq!(
        response.status(),
        warp::http::StatusCode::BAD_REQUEST,
        "high-complexity query should return 400 when GRAPHQL_MAX_QUERY_COMPLEXITY is set"
    );
}
