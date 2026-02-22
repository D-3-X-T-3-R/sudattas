//! E2E test: requires GraphQL server running. Set GRAPHQL_URL (e.g. http://127.0.0.1:8080).
//! Run with: cargo test -p graphql --test e2e_tests -- --ignored

use reqwest::Client;

fn graphql_url() -> String {
    std::env::var("GRAPHQL_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".to_string())
}

#[tokio::test]
#[ignore = "requires GraphQL server running; run with --ignored"]
async fn e2e_health_then_api_version() {
    let base = graphql_url();
    let client = Client::new();

    let health = client
        .get(format!("{}/", base.trim_end_matches('/')))
        .send()
        .await
        .expect("GET / (health)");
    assert!(health.status().is_success(), "health check should return 200");

    let gql = client
        .post(format!("{}/v2", base.trim_end_matches('/')))
        .json(&serde_json::json!({
            "query": "{ apiVersion }"
        }))
        .send()
        .await
        .expect("POST /v2 GraphQL");
    assert!(gql.status().is_success(), "GraphQL should return 200");
    let body: serde_json::Value = gql.json().await.expect("JSON body");
    assert_eq!(
        body.get("data").and_then(|d| d.get("apiVersion")).and_then(|v| v.as_str()),
        Some("2.0.0")
    );
}
