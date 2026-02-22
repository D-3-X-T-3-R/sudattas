//! GraphQL schema and resolver tests. Run with: `cargo test -p graphql`

use graphql::{schema, AuthSource, Context, JWKSet};

/// Convert juniper::Value (data root) to serde_json::Value for assertions.
fn to_json(res: &juniper::Value) -> serde_json::Value {
    serde_json::to_value(res).expect("juniper Value is serializable")
}

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
    assert_eq!(data.get("apiVersion").and_then(|v| v.as_str()), Some("2.0.0"));
}

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
    let info = data.get("authInfo").and_then(|v| v.as_object()).expect("authInfo object");
    assert_eq!(info.get("sessionEnabled").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(info.get("jwksKeyCount").and_then(|v| v.as_i64()), Some(0));
    assert_eq!(info.get("currentUserId").and_then(|v| v.as_str()), Some("42"));
}
