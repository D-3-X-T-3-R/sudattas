//! E2E tests that hit every GraphQL query and mutation.
//! Requires server running. Run with: cargo test -p graphql --test e2e_all_graphql_operations -- --ignored
//!
//! Each test sends a minimal valid request; we assert 200 and a valid GraphQL response (data or errors).
//! Auth may return 401; operations may return validation/not-found errors — we only check the endpoint is reachable.

use reqwest::Client;

fn base_url() -> String {
    std::env::var("GRAPHQL_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8080".to_string())
        .trim_end_matches('/')
        .to_string()
}

async fn post_gql(
    client: &Client,
    query: &str,
    variables: Option<serde_json::Value>,
) -> (reqwest::StatusCode, serde_json::Value) {
    let mut body = serde_json::json!({ "query": query });
    if let Some(v) = variables {
        body["variables"] = v;
    }
    let res = client
        .post(format!("{}/v2", base_url()))
        .json(&body)
        .send()
        .await
        .expect("POST /v2");
    let status = res.status();
    let body: serde_json::Value = res.json().await.unwrap_or(serde_json::Value::Null);
    (status, body)
}

/// Assert 200 and valid GraphQL response (has "data" or "errors").
fn assert_valid_gql_response(status: reqwest::StatusCode, body: &serde_json::Value) {
    assert!(
        status.is_success(),
        "expected 2xx, got {} body={}",
        status,
        body
    );
    assert!(
        body.get("data").is_some() || body.get("errors").is_some(),
        "response must have data or errors"
    );
}

// ---------- Queries ----------

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_query_api_version() {
    let (status, body) = post_gql(&Client::new(), "{ apiVersion }", None).await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_query_auth_info() {
    let (status, body) = post_gql(
        &Client::new(),
        "{ authInfo { sessionEnabled currentUserId jwksKeyCount } }",
        None,
    )
    .await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_query_get_cart_items() {
    let (status, body) = post_gql(
        &Client::new(),
        "query { getCartItems(sessionId: \"e2e-session\") { cartId productId quantity } }",
        None,
    )
    .await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_query_search_product() {
    let (status, body) = post_gql(
        &Client::new(),
        "query { searchProduct(search: {}) { productId name } }",
        None,
    )
    .await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_query_search_category() {
    let (status, body) = post_gql(
        &Client::new(),
        "query { searchCategory(search: {}) { categoryId name } }",
        None,
    )
    .await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_query_search_order() {
    let (status, body) = post_gql(
        &Client::new(),
        "query { searchOrder(search: { userId: \"1\", limit: \"5\" }) { orderId userId totalAmount } }",
        None,
    ).await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_query_search_wishlist_item() {
    let (status, body) = post_gql(
        &Client::new(),
        "query { searchWishlistItem(search: { userId: \"1\" }) { wishlistId productId } }",
        None,
    )
    .await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_query_search_country() {
    let (status, body) = post_gql(
        &Client::new(),
        "query { searchCountry(search: {}) { countryId countryName } }",
        None,
    )
    .await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_query_search_state() {
    let (status, body) = post_gql(
        &Client::new(),
        "query { searchState(search: {}) { stateId stateName } }",
        None,
    )
    .await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_query_get_payment_intent() {
    let (status, body) = post_gql(
        &Client::new(),
        "query { getPaymentIntent(input: { intentId: \"1\" }) { intentId amountPaise } }",
        None,
    )
    .await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_query_get_shipment() {
    let (status, body) = post_gql(
        &Client::new(),
        "query { getShipment(input: { shipmentId: \"1\" }) { shipmentId orderId status } }",
        None,
    )
    .await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_query_validate_coupon() {
    let (status, body) = post_gql(
        &Client::new(),
        "query { validateCoupon(input: { code: \"TEST10\", orderAmountPaise: \"10000\" }) { code discountType } }",
        None,
    ).await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_query_search_review() {
    let (status, body) = post_gql(
        &Client::new(),
        "query { searchReview(input: { limit: 5 }) { reviewId productId rating comment } }",
        None,
    )
    .await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_query_search_inventory_item() {
    let (status, body) = post_gql(
        &Client::new(),
        "query { searchInventoryItem(input: {}) { inventoryId productId quantity } }",
        None,
    )
    .await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_query_search_product_image() {
    let (status, body) = post_gql(
        &Client::new(),
        "query { searchProductImage(search: {}) { imageId productId objectKey } }",
        None,
    )
    .await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_query_get_presigned_upload_url() {
    let (status, body) = post_gql(
        &Client::new(),
        "query { getPresignedUploadUrl(input: { productId: \"1\", filename: \"key.jpg\", contentType: \"image/jpeg\" }) { uploadUrl key } }",
        None,
    ).await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_query_get_order_events() {
    let (status, body) = post_gql(
        &Client::new(),
        "query { getOrderEvents(orderId: \"1\") { eventId orderId eventType } }",
        None,
    )
    .await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_query_search_discount() {
    let (status, body) = post_gql(
        &Client::new(),
        "query { searchDiscount(input: {}) { discountId productId } }",
        None,
    )
    .await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_query_search_shipping_method() {
    let (status, body) = post_gql(
        &Client::new(),
        "query { searchShippingMethod(input: {}) { methodId name } }",
        None,
    )
    .await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_query_search_shipping_zone() {
    let (status, body) = post_gql(
        &Client::new(),
        "query { searchShippingZone(input: {}) { zoneId name } }",
        None,
    )
    .await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_query_get_shipping_addresses() {
    let (status, body) = post_gql(
        &Client::new(),
        "query { getShippingAddresses { shippingAddressId countryId road } }",
        None,
    )
    .await;
    assert_valid_gql_response(status, &body);
}

// ---------- Mutations ----------

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_add_cart_item() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { addCartItem(cartItem: { productId: \"1\", quantity: \"1\", sessionId: \"e2e-session\" }) { cartId productId quantity } }",
        None,
    ).await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_delete_cart_item() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { deleteCartItem(delete: { userId: \"1\" }) { cartId } }",
        None,
    )
    .await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_update_cart_item() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { updateCartItem(cartItem: { cartId: \"1\", productId: \"1\", quantity: \"2\", userId: \"1\" }) { cartId quantity } }",
        None,
    ).await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_place_order() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { placeOrder(order: { shippingAddressId: \"1\" }) { orderId userId totalAmount } }",
        None,
    ).await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_create_product() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { createProduct(product: { name: \"E2E Product\", description: \"\", price: \"99.99\", stockQuantity: \"10\", categoryId: \"1\" }) { productId name } }",
        None,
    ).await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_create_category() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { createCategory(category: { name: \"E2E Category\" }) { categoryId name } }",
        None,
    )
    .await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_add_wishlist_item() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { addWishlistItem(wishlist: { userId: \"1\", productId: \"1\" }) { wishlistId productId } }",
        None,
    ).await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_create_country() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { createCountry(country: { countryName: \"E2E Country\", countryCode: \"E2\" }) { countryId countryName } }",
        None,
    ).await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_create_state() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { createState(state: { stateName: \"E2E State\", countryId: \"1\" }) { stateId stateName } }",
        None,
    ).await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_create_shipping_address() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { createShippingAddress(input: { countryId: \"1\", stateId: \"1\", cityId: \"1\", road: \"123 E2E St\", apartmentNoOrName: \"\" }) { shippingAddressId road } }",
        None,
    ).await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_create_review() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { createReview(input: { productId: \"1\", userId: \"1\", rating: 5, comment: \"E2E review\" }) { reviewId rating comment } }",
        None,
    ).await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_apply_coupon() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { applyCoupon(input: { orderId: \"1\", code: \"TEST10\" }) { code } }",
        None,
    )
    .await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_create_order_event() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { createOrderEvent(input: { orderId: \"1\", eventType: \"created\" }) { eventId orderId eventType } }",
        None,
    ).await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_delete_product() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { deleteProduct(productId: \"1\") { productId name } }",
        None,
    )
    .await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_update_product() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { updateProduct(product: { productId: \"1\", name: \"Updated\", description: \"\", price: \"10\", stockQuantity: \"5\", categoryId: \"1\" }) { productId name } }",
        None,
    ).await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_delete_category() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { deleteCategory(categoryId: \"1\") { categoryId name } }",
        None,
    )
    .await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_update_category() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { updateCategory(category: { categoryId: \"1\", name: \"Updated\" }) { categoryId name } }",
        None,
    ).await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_delete_wishlist_item() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { deleteWishlistItem(delete: { wishlistId: \"1\" }) { wishlistId productId } }",
        None,
    )
    .await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_create_order_details() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { createOrderDetails(orderDetails: { orderDetails: [{ orderId: \"1\", productId: \"1\", quantity: \"2\", price: \"10.00\" }] }) { orderDetailId orderId productId } }",
        None,
    ).await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_update_order_detail() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { updateOrderDetail(orderDetail: { orderDetailId: \"1\", orderId: \"1\", productId: \"1\", quantity: \"3\", price: \"15.00\" }) { orderDetailId quantity } }",
        None,
    ).await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_delete_order() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { deleteOrder(orderId: \"1\") { orderId userId } }",
        None,
    )
    .await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_update_order() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { updateOrder(order: { orderId: \"1\", userId: \"1\", shippingAddressId: \"1\", totalAmount: \"99\", statusId: \"1\" }) { orderId statusId } }",
        None,
    ).await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_delete_country() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { deleteCountry(countryId: \"1\") { countryId countryName } }",
        None,
    )
    .await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_delete_state() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { deleteState(stateId: \"1\") { stateId stateName } }",
        None,
    )
    .await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_create_payment_intent() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { createPaymentIntent(input: { orderId: \"1\", userId: \"1\", amountPaise: \"10000\", razorpayOrderId: \"rp_1\" }) { intentId amountPaise } }",
        None,
    ).await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_capture_payment() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { capturePayment(input: { intentId: \"1\", razorpayPaymentId: \"pay_1\" }) { intentId status } }",
        None,
    ).await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_delete_product_image() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { deleteProductImage(imageId: \"1\") { imageId productId } }",
        None,
    )
    .await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_update_product_image() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { updateProductImage(productImage: { imageId: \"1\", productId: \"1\", imageBase64: \"\", altText: \"\" }) { imageId productId } }",
        None,
    ).await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_create_shipment() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { createShipment(input: { orderId: \"1\" }) { shipmentId orderId status } }",
        None,
    )
    .await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_update_shipment() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { updateShipment(input: { shipmentId: \"1\", status: \"processed\" }) { shipmentId status } }",
        None,
    ).await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_update_review() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { updateReview(input: { reviewId: \"1\", rating: 4, comment: \"Updated\" }) { reviewId rating } }",
        None,
    ).await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_delete_review() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { deleteReview(reviewId: \"1\") { reviewId rating } }",
        None,
    )
    .await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_create_inventory_item() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { createInventoryItem(input: { productId: \"1\", quantity: \"100\" }) { inventoryId productId quantity } }",
        None,
    ).await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_update_inventory_item() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { updateInventoryItem(input: { inventoryId: \"1\", productId: \"1\", quantity: \"50\" }) { inventoryId quantity } }",
        None,
    ).await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_delete_inventory_item() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { deleteInventoryItem(inventoryId: \"1\") { inventoryId quantity } }",
        None,
    )
    .await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_create_discount() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { createDiscount(input: { productId: \"1\", discountPercentage: 10.0, startDate: \"2025-01-01\", endDate: \"2025-12-31\" }) { discountId productId } }",
        None,
    ).await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_update_discount() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { updateDiscount(input: { discountId: \"1\", discountPercentage: 20.0, startDate: \"2025-01-01\", endDate: \"2025-12-31\" }) { discountId discountPercentage } }",
        None,
    ).await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_delete_discount() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { deleteDiscount(discountId: \"1\") { discountId productId } }",
        None,
    )
    .await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_create_shipping_method() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { createShippingMethod(input: { methodName: \"E2E Express\", cost: 5.0, estimatedDeliveryTime: \"2 days\" }) { methodId methodName } }",
        None,
    ).await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_update_shipping_method() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { updateShippingMethod(input: { methodId: \"1\", methodName: \"Updated\", cost: 6.0, estimatedDeliveryTime: \"3 days\" }) { methodId methodName } }",
        None,
    ).await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_delete_shipping_method() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { deleteShippingMethod(methodId: \"1\") { methodId methodName } }",
        None,
    )
    .await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_create_shipping_zone() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { createShippingZone(input: { zoneName: \"E2E Zone\", description: \"\" }) { zoneId zoneName } }",
        None,
    ).await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_update_shipping_zone() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { updateShippingZone(input: { zoneId: \"1\", zoneName: \"Updated Zone\", description: \"\" }) { zoneId zoneName } }",
        None,
    ).await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_delete_shipping_zone() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { deleteShippingZone(zoneId: \"1\") { zoneId zoneName } }",
        None,
    )
    .await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_update_shipping_address() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { updateShippingAddress(input: { shippingAddressId: \"1\", countryId: \"1\", stateId: \"1\", cityId: \"1\", road: \"456 Updated\", apartmentNoOrName: \"\" }) { shippingAddressId road } }",
        None,
    ).await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_delete_shipping_address() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { deleteShippingAddress(shippingAddressId: \"1\") { shippingAddressId } }",
        None,
    )
    .await;
    assert_valid_gql_response(status, &body);
}

#[tokio::test]
#[ignore = "requires GraphQL server; run with --ignored"]
async fn e2e_mutation_confirm_image_upload() {
    let (status, body) = post_gql(
        &Client::new(),
        "mutation { confirmImageUpload(input: { productId: \"1\", key: \"test/key.jpg\" }) { imageId productId } }",
        None,
    ).await;
    assert_valid_gql_response(status, &body);
}
