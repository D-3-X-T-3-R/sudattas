//! GraphQL schema and resolver tests. Run with: `cargo test -p graphql`
//!
//! Covers: apiVersion, authInfo, context variants (JWT/session/none), error handling,
//! and query structure.

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

#[test]
fn test_invalid_query_syntax_returns_errors() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::Jwt("u".to_string())),
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
