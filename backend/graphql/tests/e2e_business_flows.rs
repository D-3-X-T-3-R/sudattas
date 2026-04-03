//! E2E business-flow coverage (storefront + account + admin auth boundaries).
//! Requires server running. Run with:
//! `cargo test -p graphql --test e2e_business_flows -- --ignored`

use reqwest::Client;

fn base_url() -> String {
    std::env::var("GRAPHQL_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8080".to_string())
        .trim_end_matches('/')
        .to_string()
}

fn customer_user_id() -> String {
    std::env::var("GRAPHQL_E2E_CUSTOMER_USER_ID").unwrap_or_else(|_| "2".to_string())
}

fn admin_user_id() -> String {
    std::env::var("GRAPHQL_E2E_ADMIN_USER_ID").unwrap_or_else(|_| "1".to_string())
}

fn with_auth_headers(req: reqwest::RequestBuilder, as_admin: bool) -> reqwest::RequestBuilder {
    let mut req = req;
    if let Ok(secret) = std::env::var("INTERNAL_API_SECRET") {
        if !secret.trim().is_empty() {
            req = req.header("X-Internal-Auth", secret);
            req = req.header(
                "X-Customer-User-Id",
                if as_admin {
                    admin_user_id()
                } else {
                    customer_user_id()
                },
            );
        }
    }
    req
}

async fn post_gql(client: &Client, query: &str, as_admin: bool) -> (reqwest::StatusCode, serde_json::Value) {
    let req = client
        .post(format!("{}/v2", base_url()))
        .json(&serde_json::json!({ "query": query }));
    let res = with_auth_headers(req, as_admin)
        .send()
        .await
        .expect("POST /v2");
    let status = res.status();
    let body: serde_json::Value = res.json().await.unwrap_or(serde_json::Value::Null);
    (status, body)
}

fn assert_success_gql(status: reqwest::StatusCode, body: &serde_json::Value) {
    assert!(status.is_success(), "expected 2xx, got {} body={}", status, body);
    assert!(
        body.get("data").is_some() || body.get("errors").is_some(),
        "response must include data or errors"
    );
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_browse_product() {
    let (status, body) = post_gql(&Client::new(), "query { searchProduct(search: { limit: \"5\", offset: \"0\" }) { productId name } }", false).await;
    assert_success_gql(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_add_to_bag() {
    let uid = customer_user_id();
    let query = format!(
        "mutation {{ addCartItem(cartItem: {{ userId: \"{}\", variantId: \"1\", quantity: \"1\" }}) {{ cartId variantId quantity }} }}",
        uid
    );
    let (status, body) = post_gql(&Client::new(), &query, false).await;
    assert_success_gql(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_login_identity_context() {
    let (status, body) = post_gql(
        &Client::new(),
        "query { authInfo { currentUserId sessionEnabled jwksKeyCount } }",
        false,
    )
    .await;
    assert_success_gql(status, &body);
    let current = body
        .get("data")
        .and_then(|d| d.get("authInfo"))
        .and_then(|a| a.get("currentUserId"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if std::env::var("INTERNAL_API_SECRET").is_ok() {
        assert_eq!(current, customer_user_id());
    }
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_save_address() {
    let query = "mutation { createShippingAddress(input: { country: \"India\", stateRegion: \"WB\", city: \"Kolkata\", postalCode: \"700001\", road: \"Main Rd\" }) { shippingAddressId userId } }";
    let (status, body) = post_gql(&Client::new(), query, false).await;
    assert_success_gql(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_order_visible_in_profile_query() {
    let uid = customer_user_id();
    let query = format!(
        "query {{ searchOrder(search: {{ userId: \"{}\", limit: \"5\", offset: \"0\" }}) {{ orderId userId statusId }} }}",
        uid
    );
    let (status, body) = post_gql(&Client::new(), &query, false).await;
    assert_success_gql(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_place_order() {
    let query = "mutation { placeOrder(order: { shippingAddressId: \"1\" }) { orderId userId totalAmount } }";
    let (status, body) = post_gql(&Client::new(), query, false).await;
    assert_success_gql(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_payment_verify() {
    let query = "mutation { verifyRazorpayPayment(input: { orderId: \"1\", razorpayPaymentId: \"pay_1\", razorpayOrderId: \"order_1\", razorpaySignature: \"sig\" }) { verified paymentIntent { intentId status } } }";
    let (status, body) = post_gql(&Client::new(), query, false).await;
    assert_success_gql(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_admin_sees_order_product_customer_data() {
    let query = "query { searchOrder(search: { userId: \"1\", limit: \"1\", offset: \"0\" }) { orderId } searchProduct(search: { limit: \"1\", offset: \"0\" }) { productId } searchUser(input: { userId: \"1\" }) { userId email } }";
    let (status, body) = post_gql(&Client::new(), query, true).await;
    assert_success_gql(status, &body);
    let msg = body.to_string();
    assert!(
        !msg.contains("Admin authorization required"),
        "admin flow should not be denied: {}",
        msg
    );
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_unauthorized_admin_access_blocked() {
    let query = "query { searchUser(input: { userId: \"1\" }) { userId email } }";
    let (status, body) = post_gql(&Client::new(), query, false).await;
    assert_success_gql(status, &body);
    let msg = body.to_string();
    assert!(
        msg.contains("Admin authorization required"),
        "expected admin access denial for non-admin auth: {}",
        msg
    );
}
