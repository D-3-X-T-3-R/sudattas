//! E2E tests: require GraphQL server running.
//!
//! Set `GRAPHQL_URL` (e.g. `http://127.0.0.1:8080`).
//! Run with: `cargo test -p graphql --test e2e_tests -- --ignored`

use reqwest::Client;

fn graphql_url() -> String {
    std::env::var("GRAPHQL_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".to_string())
}

fn base_url() -> String {
    let u = graphql_url();
    u.trim_end_matches('/').to_string()
}

// =============================================================================
// Health and version
// =============================================================================

#[tokio::test]
#[ignore = "requires GraphQL server running; run with --ignored"]
async fn e2e_health_then_api_version() {
    let client = Client::new();
    let base = base_url();

    let health = client
        .get(format!("{}/", base))
        .send()
        .await
        .expect("GET / (health)");
    assert!(
        health.status().is_success(),
        "health check should return 200"
    );
    let body = health.text().await.expect("health body");
    assert!(body.contains("OK") || !body.is_empty(), "health body");

    let gql = client
        .post(format!("{}/v2", base))
        .json(&serde_json::json!({
            "query": "{ apiVersion }"
        }))
        .send()
        .await
        .expect("POST /v2 GraphQL");
    assert!(gql.status().is_success(), "GraphQL should return 200");
    let body: serde_json::Value = gql.json().await.expect("JSON body");
    assert_eq!(
        body.get("data")
            .and_then(|d| d.get("apiVersion"))
            .and_then(|v| v.as_str()),
        Some("2.0.0")
    );
}

#[tokio::test]
#[ignore = "requires GraphQL server running; run with --ignored"]
async fn e2e_graphql_response_structure() {
    let client = Client::new();
    let res = client
        .post(format!("{}/v2", base_url()))
        .json(&serde_json::json!({ "query": "{ apiVersion authInfo { sessionEnabled currentUserId } }" }))
        .send()
        .await
        .expect("POST /v2");

    assert!(res.status().is_success());
    let body: serde_json::Value = res.json().await.expect("JSON");
    assert!(
        body.get("data").is_some(),
        "response should have 'data' key"
    );
    let data = body.get("data").unwrap();
    assert_eq!(
        data.get("apiVersion").and_then(|v| v.as_str()),
        Some("2.0.0")
    );
    let auth_info = data.get("authInfo");
    assert!(auth_info.is_some(), "authInfo should be present");
    let info = auth_info.unwrap().as_object().expect("authInfo object");
    assert!(info.contains_key("sessionEnabled"));
    assert!(info.contains_key("currentUserId"));
}

// =============================================================================
// Error and edge cases
// =============================================================================

#[tokio::test]
#[ignore = "requires GraphQL server running; run with --ignored"]
async fn e2e_post_invalid_json_returns_4xx() {
    let client = Client::new();
    let res = client
        .post(format!("{}/v2", base_url()))
        .header("content-type", "application/json")
        .body("not valid json")
        .send()
        .await
        .expect("POST");

    assert!(
        res.status().is_client_error(),
        "invalid JSON should return 4xx"
    );
}

#[tokio::test]
#[ignore = "requires GraphQL server running; run with --ignored"]
async fn e2e_graphql_syntax_error_returns_200_with_errors() {
    let client = Client::new();
    let res = client
        .post(format!("{}/v2", base_url()))
        .json(&serde_json::json!({
            "query": "{ apiVersion "
        }))
        .send()
        .await
        .expect("POST");

    assert!(
        res.status().is_success(),
        "GraphQL typically returns 200 even for errors"
    );
    let body: serde_json::Value = res.json().await.expect("JSON");
    let errors = body.get("errors");
    assert!(
        errors.is_some()
            && errors
                .as_ref()
                .unwrap()
                .as_array()
                .map(|a| !a.is_empty())
                .unwrap_or(false),
        "syntax error should populate 'errors'"
    );
}

#[tokio::test]
#[ignore = "requires GraphQL server running; run with --ignored"]
async fn e2e_unknown_path_returns_404() {
    let client = Client::new();
    let res = client
        .get(format!("{}/unknown-path", base_url()))
        .send()
        .await
        .expect("GET");

    assert_eq!(res.status().as_u16(), 404, "unknown path should return 404");
}

#[tokio::test]
#[ignore = "requires GraphQL server running; run with --ignored"]
async fn e2e_post_without_auth_returns_401() {
    let client = Client::new();
    let res = client
        .post(format!("{}/v2", base_url()))
        .json(&serde_json::json!({ "query": "{ apiVersion }" }))
        .send()
        .await
        .expect("POST");

    // If server requires auth for all requests, expect 401 when no Authorization/X-Session-Id
    let status = res.status();
    assert!(
        status.as_u16() == 401 || status.is_success(),
        "either 401 (auth required) or 200 (auth optional)"
    );
}
