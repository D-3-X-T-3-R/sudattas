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
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: None,
        admin_resolution_source: None,
        account_status: None,
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
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: None,
        admin_resolution_source: None,
        account_status: None,
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
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: None,
        admin_resolution_source: None,
        account_status: None,
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
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: None,
        admin_resolution_source: None,
        account_status: None,
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
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: None,
        admin_resolution_source: None,
        account_status: None,
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
fn test_auth_info_reports_account_deactivated_ungated() {
    // This field is deliberately readable even when the account is deactivated — every other
    // JWT-gated query/mutation would reject this same context outright (require_jwt), so
    // authInfo is the one place the frontend can actually learn "you're deactivated" in order
    // to show that message instead of a wall of "Login required"/deactivation errors.
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::Jwt("jwt_user_789".to_string())),
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: None,
        admin_resolution_source: None,
        account_status: Some("suspended".to_string()),
    };

    let (res, errors) = juniper::execute_sync(
        r#"{ authInfo { accountDeactivated currentUserId } }"#,
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
        info.get("accountDeactivated").and_then(|v| v.as_bool()),
        Some(true)
    );
    // Still resolves the raw JWT subject even though the account is deactivated — this query
    // is the ungated exception, not `jwt_user_id()`-based like everything else.
    assert_eq!(
        info.get("currentUserId").and_then(|v| v.as_str()),
        Some("jwt_user_789")
    );
}

#[test]
fn test_auth_info_reports_active_account_as_not_deactivated() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::Jwt("jwt_user_790".to_string())),
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: None,
        admin_resolution_source: None,
        account_status: Some("active".to_string()),
    };

    let (res, errors) = juniper::execute_sync(
        r#"{ authInfo { accountDeactivated } }"#,
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
        info.get("accountDeactivated").and_then(|v| v.as_bool()),
        Some(false)
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
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: None,
        admin_resolution_source: None,
        account_status: None,
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
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: None,
        admin_resolution_source: None,
        account_status: None,
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
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: None,
        admin_resolution_source: None,
        account_status: None,
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
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: None,
        admin_resolution_source: None,
        account_status: None,
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

#[tokio::test]
async fn test_admin_mutation_requires_admin_authorization() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::Session("guest_1".to_string())),
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: None,
        admin_resolution_source: None,
        account_status: None,
    };

    let (res, errors) = juniper::execute(
        r#"mutation { createCategory(category: { name: "Test" }) { categoryId } }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .await
    .unwrap();

    assert!(
        !errors.is_empty(),
        "admin mutation should reject non-admin auth, got: {:?}",
        (res, errors)
    );
    let err = format!("{:?}", errors[0]).to_lowercase();
    assert!(
        err.contains("admin authorization required"),
        "expected admin authorization error, got: {}",
        err
    );
}

#[tokio::test]
async fn test_capture_payment_requires_privileged_authorization() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::Jwt("regular_user_123".to_string())),
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: Some(false),
        admin_resolution_source: Some("db".to_string()),
        account_status: None,
    };

    let (_res, errors) = juniper::execute(
        r#"mutation { capturePayment(input: { intentId: "1", razorpayPaymentId: "pay_1" }) { intentId } }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .await
    .unwrap();

    assert!(
        !errors.is_empty(),
        "capturePayment should reject customer auth"
    );
    let err = format!("{:?}", errors[0]).to_lowercase();
    assert!(err.contains("privileged authorization required"));
}

#[tokio::test]
async fn test_apply_coupon_requires_customer_login() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: Some("redis://127.0.0.1".to_string()),
        auth: Some(AuthSource::Session("guest_1".to_string())),
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: Some("guest-session".to_string()),
        jwt_subject: None,
        admin_authorized: Some(false),
        admin_resolution_source: Some("none".to_string()),
        account_status: None,
    };

    let (_res, errors) = juniper::execute(
        r#"mutation { applyCoupon(input: { code: "TEST10", orderAmountPaise: "10000" }) { code } }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .await
    .unwrap();

    assert!(
        !errors.is_empty(),
        "applyCoupon should reject guest session auth"
    );
    let err = format!("{:?}", errors[0]).to_lowercase();
    assert!(err.contains("login required"));
}

#[tokio::test]
async fn test_validate_coupon_requires_customer_login() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: Some("redis://127.0.0.1".to_string()),
        auth: Some(AuthSource::Session("guest_1".to_string())),
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: Some("guest-session".to_string()),
        jwt_subject: None,
        admin_authorized: Some(false),
        admin_resolution_source: Some("none".to_string()),
        account_status: None,
    };

    let (_res, errors) = juniper::execute(
        r#"{ validateCoupon(input: { code: "TEST10", orderAmountPaise: "10000" }) { code } }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .await
    .unwrap();

    assert!(
        !errors.is_empty(),
        "validateCoupon should reject guest session auth"
    );
    let err = format!("{:?}", errors[0]).to_lowercase();
    assert!(err.contains("login required"));
}

#[tokio::test]
async fn test_order_internal_mutations_require_privileged_authorization() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::Jwt("regular_user_123".to_string())),
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: Some(false),
        admin_resolution_source: Some("db".to_string()),
        account_status: None,
    };

    let cases = [
        r#"mutation { createOrderEvent(input: { orderId: "1", eventType: "created", actorType: "system" }) { eventId } }"#,
        r#"mutation { createOrderDetails(orderDetails: { orderDetails: [{ orderId: "1", variantId: "1", quantity: "1", pricePaise: "1000" }] }) { orderDetailId } }"#,
        r#"mutation { updateOrderDetail(orderDetail: { orderDetailId: "1", orderId: "1", variantId: "1", quantity: "2", pricePaise: "1000" }) { orderDetailId } }"#,
    ];

    for query in cases {
        let (_res, errors) =
            juniper::execute(query, None, &schema(), &juniper::Variables::new(), &ctx)
                .await
                .unwrap();

        assert!(
            !errors.is_empty(),
            "privileged mutation should reject customer auth"
        );
        let err = format!("{:?}", errors[0]).to_lowercase();
        assert!(err.contains("privileged authorization required"));
    }
}

#[tokio::test]
async fn test_search_inventory_item_requires_admin() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::Jwt("regular_user_123".to_string())),
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: Some(false),
        admin_resolution_source: Some("db".to_string()),
        account_status: None,
    };

    let (_res, errors) = juniper::execute(
        r#"{ searchInventoryItem(input: {}) { inventoryId } }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .await
    .unwrap();

    assert!(
        !errors.is_empty(),
        "searchInventoryItem should reject customer auth"
    );
    let err = format!("{:?}", errors[0]).to_lowercase();
    assert!(err.contains("admin authorization required"));
}

#[tokio::test]
async fn test_search_inventory_log_requires_admin_for_customer() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::Jwt("regular_user_123".to_string())),
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: Some(false),
        admin_resolution_source: Some("db".to_string()),
        account_status: None,
    };

    let (_res, errors) = juniper::execute(
        r#"{ searchInventoryLog(input: {}) { logId } }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .await
    .unwrap();

    assert!(
        !errors.is_empty(),
        "searchInventoryLog should reject customer auth"
    );
    let err = format!("{:?}", errors[0]).to_lowercase();
    assert!(err.contains("admin authorization required"));
}

#[tokio::test]
async fn test_search_inventory_log_requires_admin_for_guest_session() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: Some("redis://127.0.0.1".to_string()),
        auth: Some(AuthSource::Session("guest_123".to_string())),
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: Some("guest-session".to_string()),
        jwt_subject: None,
        admin_authorized: Some(false),
        admin_resolution_source: Some("none".to_string()),
        account_status: None,
    };

    let (_res, errors) = juniper::execute(
        r#"{ searchInventoryLog(input: {}) { logId } }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .await
    .unwrap();

    assert!(
        !errors.is_empty(),
        "searchInventoryLog should reject guest auth"
    );
    let err = format!("{:?}", errors[0]).to_lowercase();
    assert!(err.contains("admin authorization required"));
}

#[tokio::test]
async fn test_search_inventory_log_allows_admin_context() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::Jwt("google_sub_admin".to_string())),
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: None,
        jwt_subject: Some("google_sub_admin".to_string()),
        admin_authorized: Some(true),
        admin_resolution_source: Some("db".to_string()),
        account_status: None,
    };

    let (_res, errors) = juniper::execute(
        r#"{ searchInventoryLog(input: {}) { logId } }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .await
    .unwrap();

    if !errors.is_empty() {
        let err = format!("{:?}", errors[0]).to_lowercase();
        assert!(
            !err.contains("admin authorization required"),
            "admin context should not fail authz: {}",
            err
        );
    }
}

#[tokio::test]
async fn test_search_user_requires_admin_authorization() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::Jwt("regular_user_123".to_string())),
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: Some(false),
        admin_resolution_source: Some("db".to_string()),
        account_status: None,
    };

    let (res, errors) = juniper::execute(
        r#"{ searchUser(input: { userId: "1" }) { userId } }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .await
    .unwrap();

    assert!(
        !errors.is_empty(),
        "searchUser should reject non-admin user, got: {:?}",
        (res, errors)
    );
    let err = format!("{:?}", errors[0]).to_lowercase();
    assert!(err.contains("admin authorization required"));
}

#[tokio::test]
async fn test_record_security_audit_event_rejects_customer_auth() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::Jwt("regular_user_123".to_string())),
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: Some(false),
        admin_resolution_source: Some("db".to_string()),
        account_status: None,
    };

    let (_res, errors) = juniper::execute(
        r#"mutation { recordSecurityAuditEvent(input: { eventType: "secrets_rotation" }) }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .await
    .unwrap();

    assert!(
        !errors.is_empty(),
        "recordSecurityAuditEvent should reject customer auth"
    );
    let err = format!("{:?}", errors[0]).to_lowercase();
    assert!(err.contains("privileged authorization required"));
}

#[tokio::test]
async fn test_record_security_audit_event_rejects_guest_session() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: Some("redis://127.0.0.1".to_string()),
        auth: Some(AuthSource::Session("guest_123".to_string())),
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: Some("guest-session".to_string()),
        jwt_subject: None,
        admin_authorized: Some(false),
        admin_resolution_source: Some("none".to_string()),
        account_status: None,
    };

    let (_res, errors) = juniper::execute(
        r#"mutation { recordSecurityAuditEvent(input: { eventType: "secrets_rotation" }) }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .await
    .unwrap();

    assert!(
        !errors.is_empty(),
        "recordSecurityAuditEvent should reject guest auth"
    );
    let err = format!("{:?}", errors[0]).to_lowercase();
    assert!(err.contains("privileged authorization required"));
}

#[tokio::test]
async fn test_record_security_audit_event_allows_admin_context() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::Jwt("google_sub_admin".to_string())),
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: None,
        jwt_subject: Some("google_sub_admin".to_string()),
        admin_authorized: Some(true),
        admin_resolution_source: Some("db".to_string()),
        account_status: None,
    };

    let (_res, errors) = juniper::execute(
        r#"mutation { recordSecurityAuditEvent(input: { eventType: "secrets_rotation" }) }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .await
    .unwrap();

    if !errors.is_empty() {
        let err = format!("{:?}", errors[0]).to_lowercase();
        assert!(
            !err.contains("privileged authorization required"),
            "admin context should not fail authz: {}",
            err
        );
    }
}

#[tokio::test]
async fn test_record_security_audit_event_allows_internal_service() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::InternalService),
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: Some(false),
        admin_resolution_source: Some("internal".to_string()),
        account_status: None,
    };

    let (_res, errors) = juniper::execute(
        r#"mutation { recordSecurityAuditEvent(input: { eventType: "secrets_rotation" }) }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .await
    .unwrap();

    if !errors.is_empty() {
        let err = format!("{:?}", errors[0]).to_lowercase();
        assert!(
            !err.contains("privileged authorization required"),
            "internal service context should not fail authz: {}",
            err
        );
    }
}

#[tokio::test]
async fn test_search_user_allows_admin_context() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::Jwt("google_sub_admin".to_string())),
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: None,
        jwt_subject: Some("google_sub_admin".to_string()),
        admin_authorized: Some(true),
        admin_resolution_source: Some("db".to_string()),
        account_status: None,
    };

    let (_res, errors) = juniper::execute(
        r#"{ searchUser(input: { userId: "1" }) { userId } }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .await
    .unwrap();

    if !errors.is_empty() {
        let err = format!("{:?}", errors[0]).to_lowercase();
        assert!(
            !err.contains("admin authorization required"),
            "admin context should not fail authz: {}",
            err
        );
    }
}

#[tokio::test]
async fn test_admin_export_user_pii_requires_admin_authorization() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::Jwt("regular_user_123".to_string())),
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: Some(false),
        admin_resolution_source: Some("db".to_string()),
        account_status: None,
    };

    let (res, errors) = juniper::execute(
        r#"{ adminExportUserPii(userId: "1") { userId } }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .await
    .unwrap();

    assert!(
        !errors.is_empty(),
        "adminExportUserPii should reject non-admin user, got: {:?}",
        (res, errors)
    );
    let err = format!("{:?}", errors[0]).to_lowercase();
    assert!(
        err.contains("admin authorization required"),
        "expected admin authz rejection, got: {}",
        err
    );
}

#[tokio::test]
async fn test_admin_export_user_pii_allows_admin_context() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::Jwt("google_sub_admin".to_string())),
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: None,
        jwt_subject: Some("google_sub_admin".to_string()),
        admin_authorized: Some(true),
        admin_resolution_source: Some("db".to_string()),
        account_status: None,
    };

    let (_res, errors) = juniper::execute(
        r#"{ adminExportUserPii(userId: "1") { userId } }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .await
    .unwrap();

    if !errors.is_empty() {
        let err = format!("{:?}", errors[0]).to_lowercase();
        assert!(
            !err.contains("admin authorization required"),
            "admin context should not fail authz: {}",
            err
        );
    }
}

#[tokio::test]
async fn test_admin_export_user_pii_invalid_user_id_returns_error() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::Jwt("google_sub_admin".to_string())),
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: None,
        jwt_subject: Some("google_sub_admin".to_string()),
        admin_authorized: Some(true),
        admin_resolution_source: Some("db".to_string()),
        account_status: None,
    };

    let (res, errors) = juniper::execute(
        r#"{ adminExportUserPii(userId: "not-a-number") { userId } }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .await
    .unwrap();

    assert!(
        !errors.is_empty(),
        "non-numeric userId should be rejected before any gRPC call, got: {:?}",
        (res, errors)
    );
    let err = format!("{:?}", errors[0]).to_lowercase();
    assert!(
        err.contains("invalid user id"),
        "expected an invalid-user-id error, got: {}",
        err
    );
}

#[tokio::test]
async fn test_request_exchange_requires_login() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: Some("redis://127.0.0.1".to_string()),
        auth: Some(AuthSource::Session("guest_1".to_string())),
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: Some("guest-session".to_string()),
        jwt_subject: None,
        admin_authorized: Some(false),
        admin_resolution_source: Some("none".to_string()),
        account_status: None,
    };

    let (res, errors) = juniper::execute(
        r#"mutation {
            requestExchange(input: {
                orderId: "1",
                orderDetailId: "1",
                desiredVariantId: "2",
                reason: "Wrong size"
            }) { exchangeId }
        }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .await
    .unwrap();

    assert!(
        !errors.is_empty(),
        "requestExchange should reject a guest session, got: {:?}",
        (res, errors)
    );
    let err = format!("{:?}", errors[0]).to_lowercase();
    assert!(err.contains("login required"), "got: {}", err);
}

#[tokio::test]
async fn test_admin_mark_exchange_received_requires_admin_authorization() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::Jwt("regular_user_123".to_string())),
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: Some(false),
        admin_resolution_source: Some("db".to_string()),
        account_status: None,
    };

    let (res, errors) = juniper::execute(
        r#"mutation { adminMarkExchangeReceived(input: { exchangeId: "1" }) { exchangeId } }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .await
    .unwrap();

    assert!(
        !errors.is_empty(),
        "adminMarkExchangeReceived should reject a non-admin user, got: {:?}",
        (res, errors)
    );
    let err = format!("{:?}", errors[0]).to_lowercase();
    assert!(err.contains("admin authorization required"), "got: {}", err);
}

#[tokio::test]
async fn test_admin_update_exchange_status_requires_admin_authorization() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::Jwt("regular_user_123".to_string())),
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: Some(false),
        admin_resolution_source: Some("db".to_string()),
        account_status: None,
    };

    let (res, errors) = juniper::execute(
        r#"mutation {
            adminUpdateExchangeStatus(input: { exchangeId: "1", status: "approved" }) { exchangeId }
        }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .await
    .unwrap();

    assert!(
        !errors.is_empty(),
        "adminUpdateExchangeStatus should reject a non-admin user, got: {:?}",
        (res, errors)
    );
    let err = format!("{:?}", errors[0]).to_lowercase();
    assert!(err.contains("admin authorization required"), "got: {}", err);
}

#[tokio::test]
async fn test_search_exchange_requests_requires_customer_or_admin() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: Some("redis://127.0.0.1".to_string()),
        auth: Some(AuthSource::Session("guest_1".to_string())),
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: Some("guest-session".to_string()),
        jwt_subject: None,
        admin_authorized: Some(false),
        admin_resolution_source: Some("none".to_string()),
        account_status: None,
    };

    let (res, errors) = juniper::execute(
        r#"{ searchExchangeRequests(input: {}) { exchangeId } }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .await
    .unwrap();

    assert!(
        !errors.is_empty(),
        "searchExchangeRequests should reject a guest session, got: {:?}",
        (res, errors)
    );
    let err = format!("{:?}", errors[0]).to_lowercase();
    assert!(err.contains("login required"), "got: {}", err);
}

#[tokio::test]
async fn test_search_order_rejects_cross_user_access_for_customer() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::Jwt("42".to_string())),
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: None,
        admin_resolution_source: None,
        account_status: None,
    };

    let (res, errors) = juniper::execute(
        r#"{ searchOrder(search: { userId: "99" }) { orderId } }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .await
    .unwrap();

    assert!(
        !errors.is_empty(),
        "customer should not query another user's orders, got: {:?}",
        (res, errors)
    );
    let err = format!("{:?}", errors[0]).to_lowercase();
    assert!(err.contains("own orders"));
}

#[tokio::test]
async fn test_update_user_rejects_cross_user_access_for_customer() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::InternalCustomer("42".to_string())),
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: Some(false),
        admin_resolution_source: Some("internal".to_string()),
        account_status: None,
    };

    let (_res, errors) = juniper::execute(
        r#"mutation { updateUser(input: { userId: "99", fullName: "Wrong" }) { userId } }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .await
    .unwrap();

    assert!(!errors.is_empty(), "cross-user updateUser should fail");
    let err = format!("{:?}", errors[0]).to_lowercase();
    assert!(err.contains("own profile"));
}

/// Customer-editable profile fields (firstName/lastName/gender/dateOfBirth) must not
/// bypass the same cross-user ownership guard that plain field updates already enforce.
#[tokio::test]
async fn test_update_user_rejects_cross_user_access_with_profile_fields() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::InternalCustomer("42".to_string())),
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: Some(false),
        admin_resolution_source: Some("internal".to_string()),
        account_status: None,
    };

    let (_res, errors) = juniper::execute(
        r#"mutation {
            updateUser(input: {
                userId: "99"
                firstName: "Wrong"
                lastName: "User"
                gender: "male"
                dateOfBirth: "1990-01-01"
                roleId: "1"
            }) { userId }
        }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .await
    .unwrap();

    assert!(
        !errors.is_empty(),
        "cross-user updateUser with profile fields should fail"
    );
    let err = format!("{:?}", errors[0]).to_lowercase();
    assert!(err.contains("own profile"));
}

#[tokio::test]
async fn test_create_user_rejects_session_auth() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: Some("redis://127.0.0.1".to_string()),
        auth: Some(AuthSource::Session("0".to_string())),
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: Some("guest-session".to_string()),
        jwt_subject: None,
        admin_authorized: Some(false),
        admin_resolution_source: Some("none".to_string()),
        account_status: None,
    };

    let (_res, errors) = juniper::execute(
        r#"mutation { createUser(input: { username: "guest", email: "guest@example.com", authProvider: "google", googleSub: "sub_1" }) { userId } }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .await
    .unwrap();

    assert!(!errors.is_empty(), "guest session createUser should fail");
    let err = format!("{:?}", errors[0]).to_lowercase();
    assert!(err.contains("verified customer") || err.contains("internal auth"));
}

#[tokio::test]
async fn test_get_presigned_upload_url_requires_admin() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::InternalCustomer("42".to_string())),
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: Some(false),
        admin_resolution_source: Some("internal".to_string()),
        account_status: None,
    };

    let (_res, errors) = juniper::execute(
        r#"{ getPresignedUploadUrl(input: { productId: "1", filename: "test.png", contentType: "image/png" }) { uploadUrl } }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .await
    .unwrap();

    assert!(!errors.is_empty(), "non-admin presign should fail");
    let err = format!("{:?}", errors[0]).to_lowercase();
    assert!(err.contains("admin authorization required"));
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
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: None,
        admin_resolution_source: None,
        account_status: None,
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
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: None,
        admin_resolution_source: None,
        account_status: None,
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
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: None,
        admin_resolution_source: None,
        account_status: None,
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
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: None,
        admin_resolution_source: None,
        account_status: None,
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
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: None,
        admin_resolution_source: None,
        account_status: None,
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
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: None,
        admin_resolution_source: None,
        account_status: None,
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

// =============================================================================
// subscribeNewsletter (public, unauthenticated storefront footer signup)
// =============================================================================

#[tokio::test]
async fn test_subscribe_newsletter_rejects_invalid_email_without_auth() {
    // No auth context at all — this mutation must be reachable by a fully anonymous visitor,
    // and the invalid-email guard runs before any gRPC call, so this needs no live backend.
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: None,
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: None,
        admin_resolution_source: None,
        account_status: None,
    };

    let (res, errors) = juniper::execute(
        r#"mutation { subscribeNewsletter(email: "not-an-email") }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .await
    .unwrap();

    assert!(
        !errors.is_empty(),
        "an email with no '@' should be rejected, got: {:?}",
        (res, errors)
    );
    let err_str = format!("{:?}", errors[0]);
    assert!(
        err_str.to_lowercase().contains("valid email"),
        "error should mention a valid email is required: {}",
        err_str
    );
}

#[tokio::test]
async fn test_subscribe_newsletter_rejects_blank_email_without_auth() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: None,
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: None,
        admin_resolution_source: None,
        account_status: None,
    };

    let (_res, errors) = juniper::execute(
        r#"mutation { subscribeNewsletter(email: "   ") }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .await
    .unwrap();

    assert!(!errors.is_empty(), "a blank email should be rejected");
}

#[tokio::test]
async fn test_subscribe_newsletter_with_valid_email_not_rejected_for_auth() {
    // No auth context — a plausible email must not be turned away for lacking admin/JWT
    // credentials. It may still fail if the gRPC backend is unreachable in this test run;
    // what matters is that failure is never an authorization error.
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: None,
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: None,
        admin_resolution_source: None,
        account_status: None,
    };

    let schema = schema();
    let result = juniper::execute(
        r#"mutation { subscribeNewsletter(email: "shopper@example.com") }"#,
        None,
        &schema,
        &juniper::Variables::new(),
        &ctx,
    )
    .await;

    if let Ok((_, errors)) = result {
        if !errors.is_empty() {
            let err_str = format!("{:?}", errors[0]).to_lowercase();
            assert!(
                !err_str.contains("admin") && !err_str.contains("login"),
                "a valid email from an anonymous caller should never fail on authorization: {}",
                err_str
            );
        }
    }
}

// =============================================================================
// Newsletter campaigns (admin) + unsubscribeNewsletter (public)
// =============================================================================

#[tokio::test]
async fn test_send_newsletter_campaign_requires_admin_authorization() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::Jwt("user_1".to_string())),
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: None,
        admin_resolution_source: None,
        account_status: None,
    };

    let (_res, errors) = juniper::execute(
        r#"mutation { sendNewsletterCampaign(input: { subject: "Hi", bodyText: "Body." }) { campaignId } }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .await
    .unwrap();

    assert!(
        !errors.is_empty(),
        "a logged-in but non-admin caller must not be able to send a campaign"
    );
    let err_str = format!("{:?}", errors[0]).to_lowercase();
    assert!(err_str.contains("admin"), "error should mention admin: {}", err_str);
}

#[tokio::test]
async fn test_search_newsletter_campaign_requires_admin_authorization() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: None,
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: None,
        admin_resolution_source: None,
        account_status: None,
    };

    let (_res, errors) = juniper::execute(
        r#"mutation { searchNewsletterCampaign(input: {}) { campaignId } }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .await
    .unwrap();

    assert!(
        !errors.is_empty(),
        "an anonymous caller must not be able to read campaign history"
    );
}

#[tokio::test]
async fn test_unsubscribe_newsletter_not_rejected_for_auth_without_credentials() {
    // No auth context at all — the unsubscribe link in an email must work for a fully
    // anonymous click. A bad/expired token should fail for its own reason, never for lacking
    // admin/login credentials.
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: None,
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: None,
        admin_resolution_source: None,
        account_status: None,
    };

    let schema = schema();
    let result = juniper::execute(
        r#"mutation { unsubscribeNewsletter(input: { subscriberId: "1", token: "bogus" }) }"#,
        None,
        &schema,
        &juniper::Variables::new(),
        &ctx,
    )
    .await;

    if let Ok((_, errors)) = result {
        if !errors.is_empty() {
            let err_str = format!("{:?}", errors[0]).to_lowercase();
            assert!(
                !err_str.contains("admin") && !err_str.contains("login required"),
                "an anonymous unsubscribe click should never fail on authorization: {}",
                err_str
            );
        }
    }
}

// =============================================================================
// Account deactivation (setUserStatus) enforcement at the auth gate
// =============================================================================

#[tokio::test]
async fn test_deactivated_customer_rejected_with_clear_message_via_require_customer_actor() {
    // applyCoupon goes through require_customer_actor, which gives the specific "deactivated"
    // message (see the bypass test below for a mutation that doesn't go through it).
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::Jwt("user_1".to_string())),
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: Some(false),
        admin_resolution_source: Some("db".to_string()),
        account_status: Some("inactive".to_string()),
    };

    let (_res, errors) = juniper::execute(
        r#"mutation { applyCoupon(input: { code: "TEST10", orderAmountPaise: "10000" }) { code } }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .await
    .unwrap();

    assert!(
        !errors.is_empty(),
        "a deactivated customer must be rejected even with a valid JWT"
    );
    let err_str = format!("{:?}", errors[0]).to_lowercase();
    assert!(
        err_str.contains("deactivated"),
        "error should mention deactivation, not just 'login required': {}",
        err_str
    );
}

#[tokio::test]
async fn test_deactivated_customer_blocked_even_on_a_bypass_path() {
    // place_order checks context.jwt_user_id() directly instead of going through
    // require_jwt/require_customer_actor — this is exactly the kind of call site the
    // enforcement must not depend on being routed through those two helpers. The message here
    // is place_order's own ("Login required..."), not the nicer "deactivated" one, but the
    // request must still be blocked — jwt_user_id() itself returns None once deactivated.
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::Jwt("user_1".to_string())),
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: Some(false),
        admin_resolution_source: Some("db".to_string()),
        account_status: Some("inactive".to_string()),
    };

    let (_res, errors) = juniper::execute(
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
        "a deactivated customer must be blocked from place_order even though it doesn't call require_jwt/require_customer_actor"
    );
    let err_str = format!("{:?}", errors[0]).to_lowercase();
    assert!(
        err_str.contains("login required"),
        "should fail as 'not logged in' (jwt_user_id() returns None), not reach the gRPC call: {}",
        err_str
    );
}

#[tokio::test]
async fn test_suspended_admin_rejected_from_admin_action() {
    // Even a user who would otherwise pass is_admin() (admin_authorized: Some(true)) must be
    // blocked once their own account is suspended — role membership doesn't override this.
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::Jwt("admin_1".to_string())),
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: Some(true),
        admin_resolution_source: Some("db".to_string()),
        account_status: Some("suspended".to_string()),
    };

    let (_res, errors) = juniper::execute(
        r#"mutation { searchNewsletterCampaign(input: {}) { campaignId } }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .await
    .unwrap();

    assert!(
        !errors.is_empty(),
        "a suspended admin must be rejected even though is_admin() would say true"
    );
    let err_str = format!("{:?}", errors[0]).to_lowercase();
    assert!(err_str.contains("deactivated"), "error should mention deactivation: {}", err_str);
}

#[tokio::test]
async fn test_never_set_status_is_not_treated_as_deactivated() {
    // account_status: None means "never explicitly set" — must behave exactly like today
    // (active), not be rejected.
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::Jwt("user_2".to_string())),
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: Some(false),
        admin_resolution_source: Some("db".to_string()),
        account_status: None,
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

    if let Ok((_, errors)) = result {
        if !errors.is_empty() {
            let err_str = format!("{:?}", errors[0]).to_lowercase();
            assert!(
                !err_str.contains("deactivated"),
                "a never-set status must never be treated as deactivated: {}",
                err_str
            );
        }
    }
}

#[tokio::test]
async fn test_permanently_delete_product_requires_admin_authorization() {
    let ctx = Context {
        jwks: JWKSet { keys: vec![] },
        redis_url: None,
        auth: Some(AuthSource::Jwt("user_1".to_string())),
        request_id: None,
        idempotency_key: None,
        client_action: None,
        guest_session_id: None,
        jwt_subject: None,
        admin_authorized: Some(false),
        admin_resolution_source: Some("db".to_string()),
        account_status: None,
    };

    let (_res, errors) = juniper::execute(
        r#"mutation { permanentlyDeleteProduct(productId: "1") { productId } }"#,
        None,
        &schema(),
        &juniper::Variables::new(),
        &ctx,
    )
    .await
    .unwrap();

    assert!(
        !errors.is_empty(),
        "a logged-in but non-admin caller must not be able to permanently delete a product"
    );
    let err_str = format!("{:?}", errors[0]).to_lowercase();
    assert!(err_str.contains("admin"), "error should mention admin: {}", err_str);
}
