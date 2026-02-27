//! Integration tests for `place_order` idempotency semantics against a real DB.
//!
//! **Setup**
//! - Set `TEST_DATABASE_URL` or `DATABASE_URL` (e.g. `mysql://root:test_password@127.0.0.1:3306/sudattas_test`).
//! - Schema must be loaded first (e.g. migrations or `backend/database/sql_dump/01_schema.sql`).
//!
//! **Run**
//! - `cargo test --test integration_place_order_idempotency -- --ignored`

mod integration_common;

use chrono::Utc;
use core_db_entities::entity::{
    idempotency_keys, sea_orm_active_enums::Status as IdempotencyStatus,
};
use integration_common::test_db_url;
use proto::proto::core::{
    CreateCartItemRequest, CreateCategoryRequest, CreateCityRequest, CreateCountryRequest,
    CreateInventoryItemRequest, CreateProductRequest, CreateShippingAddressRequest,
    CreateStateRequest, CreateSupplierRequest, CreateUserRequest, PlaceOrderRequest,
};
use sea_orm::{ActiveModelTrait, ActiveValue, Database, TransactionTrait};
use serde_json::json;
use tonic::Request;
use uuid::Uuid;

fn build_place_order_payload_json(
    user_id: i64,
    shipping_address_id: i64,
    product_id: i64,
    quantity: i64,
) -> String {
    let cart_snapshot = vec![json!({
        "product_id": product_id,
        "quantity": quantity,
    })];

    let coupon_code: Option<String> = None;
    let payload_json = json!({
        "user_id": user_id,
        "shipping_address_id": shipping_address_id,
        "coupon_code": coupon_code,
        "cart": cart_snapshot,
    });

    payload_json.to_string()
}

async fn seed_user_and_address(txn: &sea_orm::DatabaseTransaction) -> (i64, i64) {
    let unique = Uuid::new_v4().to_string();
    let country = core_operations::handlers::country::create_country(
        txn,
        Request::new(CreateCountryRequest {
            country_name: format!("Idem Country {}", unique),
        }),
    )
    .await
    .expect("create country");
    let country_id = country.into_inner().items[0].country_id;

    let state = core_operations::handlers::state::create_state(
        txn,
        Request::new(CreateStateRequest {
            state_name: format!("Idem State {}", unique),
        }),
    )
    .await
    .expect("create state");
    let state_id = state.into_inner().items[0].state_id;

    let city = core_operations::handlers::city::create_city(
        txn,
        Request::new(CreateCityRequest {
            city_name: format!("Idem City {}", unique),
        }),
    )
    .await
    .expect("create city");
    let city_id = city.into_inner().items[0].city_id;

    let addr = core_operations::handlers::shipping_address::create_shipping_address(
        txn,
        Request::new(CreateShippingAddressRequest {
            country_id,
            state_id,
            city_id,
            road: format!("123 Idem St {}", unique),
            apartment_no_or_name: "".to_string(),
        }),
    )
    .await
    .expect("create shipping address");
    let shipping_address_id = addr.into_inner().items[0].shipping_address_id;

    let user = core_operations::handlers::users::create_user(
        txn,
        Request::new(CreateUserRequest {
            username: format!("idem_user_{}", unique),
            email: format!("idem_{}@test.local", unique),
            password: "SecurePass123!".to_string(),
            full_name: Some("Idem User".to_string()),
            address: None,
            phone: None,
        }),
    )
    .await
    .expect("create user");
    let user_id = user.into_inner().items[0].user_id;

    (user_id, shipping_address_id)
}

async fn seed_product_inventory_and_cart(
    txn: &sea_orm::DatabaseTransaction,
    user_id: i64,
) -> (i64, i64) {
    let supplier = core_operations::handlers::suppliers::create_supplier(
        txn,
        Request::new(CreateSupplierRequest {
            name: "Idem Supplier".to_string(),
            contact_info: "supplier@example.test".to_string(),
            address: "123 Supplier St".to_string(),
        }),
    )
    .await
    .expect("create supplier");
    let supplier_id = supplier.into_inner().items[0].supplier_id;

    let cat = core_operations::handlers::categories::create_category(
        txn,
        Request::new(CreateCategoryRequest {
            name: "Idem Category".to_string(),
        }),
    )
    .await
    .expect("create category");
    let category_id = cat.into_inner().items[0].category_id;

    let prod = core_operations::handlers::products::create_product(
        txn,
        Request::new(CreateProductRequest {
            name: "Idem Product".to_string(),
            description: None,
            price: 10.0,
            stock_quantity: Some(10),
            category_id: Some(category_id),
        }),
    )
    .await
    .expect("create product");
    let product_id = prod.into_inner().items[0].product_id;

    let _inv = core_operations::handlers::inventory::create_inventory_item(
        txn,
        Request::new(CreateInventoryItemRequest {
            product_id,
            quantity_available: 10,
            reorder_level: 0,
            supplier_id,
        }),
    )
    .await
    .expect("create inventory item");

    let quantity = 2_i64;
    let _cart = core_operations::handlers::cart::create_cart_item(
        txn,
        Request::new(CreateCartItemRequest {
            user_id: Some(user_id),
            session_id: None,
            product_id,
            quantity,
        }),
    )
    .await
    .expect("create cart item for user");

    (product_id, quantity)
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_place_order_idempotent_replay_returns_same_order() {
    use core_operations::procedures::orders::place_order;

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let (user_id, shipping_address_id) = seed_user_and_address(&txn).await;
    let (_product_id, _quantity) = seed_product_inventory_and_cart(&txn, user_id).await;

    let key = "idem-replay";
    let mut req1 = Request::new(PlaceOrderRequest {
        user_id,
        shipping_address_id,
        coupon_code: None,
    });
    req1.metadata_mut()
        .insert("idempotency-key", key.parse().unwrap());

    let first = place_order(&txn, req1).await.expect("first place_order");
    let orders1 = first.into_inner().items;
    assert!(
        !orders1.is_empty(),
        "first place_order should return an order"
    );
    let order1 = &orders1[0];

    let mut req2 = Request::new(PlaceOrderRequest {
        user_id,
        shipping_address_id,
        coupon_code: None,
    });
    req2.metadata_mut()
        .insert("idempotency-key", key.parse().unwrap());

    let second = place_order(&txn, req2).await.expect("second place_order");
    let orders2 = second.into_inner().items;
    assert!(
        !orders2.is_empty(),
        "second place_order should return an order"
    );
    let order2 = &orders2[0];

    assert_eq!(
        order1.order_id, order2.order_id,
        "idempotent replay should return the same order_id"
    );

    txn.rollback().await.ok();
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_place_order_pending_idempotency_row_returns_unavailable() {
    use core_operations::handlers::idempotency::compute_request_hash;
    use core_operations::procedures::orders::place_order;

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let (user_id, shipping_address_id) = seed_user_and_address(&txn).await;
    let (product_id, quantity) = seed_product_inventory_and_cart(&txn, user_id).await;

    let key = "idem-pending";
    let payload_json =
        build_place_order_payload_json(user_id, shipping_address_id, product_id, quantity);
    let request_hash = compute_request_hash(&payload_json);

    let active = idempotency_keys::ActiveModel {
        id: ActiveValue::NotSet,
        scope: ActiveValue::Set("place_order".to_string()),
        key: ActiveValue::Set(key.to_string()),
        request_hash: ActiveValue::Set(request_hash),
        response_ref: ActiveValue::Set(None),
        status: ActiveValue::Set(IdempotencyStatus::Pending),
        created_at: ActiveValue::Set(Utc::now()),
        expires_at: ActiveValue::Set(Utc::now() + chrono::Duration::hours(1)),
    };
    active
        .insert(&txn)
        .await
        .expect("insert pending idempotency row");

    let mut req = Request::new(PlaceOrderRequest {
        user_id,
        shipping_address_id,
        coupon_code: None,
    });
    req.metadata_mut()
        .insert("idempotency-key", key.parse().unwrap());

    let result = place_order(&txn, req).await;
    assert!(
        result.is_err(),
        "place_order with pending idempotency row should be Unavailable"
    );
    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::Unavailable);

    txn.rollback().await.ok();
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_place_order_conflicting_payload_reuses_key_is_conflict() {
    use core_operations::handlers::idempotency::compute_request_hash;
    use core_operations::procedures::orders::place_order;

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let (user_id, shipping_address_id) = seed_user_and_address(&txn).await;
    let (product_id, quantity) = seed_product_inventory_and_cart(&txn, user_id).await;

    let key = "idem-conflict";

    // Existing entry with a different request_hash than the one computed below.
    let existing = idempotency_keys::ActiveModel {
        id: ActiveValue::NotSet,
        scope: ActiveValue::Set("place_order".to_string()),
        key: ActiveValue::Set(key.to_string()),
        request_hash: ActiveValue::Set(compute_request_hash("different-payload")),
        response_ref: ActiveValue::Set(Some("12345".to_string())),
        status: ActiveValue::Set(IdempotencyStatus::Processed),
        created_at: ActiveValue::Set(Utc::now()),
        expires_at: ActiveValue::Set(Utc::now() + chrono::Duration::hours(1)),
    };
    existing
        .insert(&txn)
        .await
        .expect("insert conflicting idempotency row");

    // This payload produces a different hash than "different-payload".
    let _payload_json =
        build_place_order_payload_json(user_id, shipping_address_id, product_id, quantity);

    let mut req = Request::new(PlaceOrderRequest {
        user_id,
        shipping_address_id,
        coupon_code: None,
    });
    req.metadata_mut()
        .insert("idempotency-key", key.parse().unwrap());

    let result = place_order(&txn, req).await;
    assert!(
        result.is_err(),
        "place_order should reject reused idempotency key with different payload"
    );
    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::AlreadyExists);

    txn.rollback().await.ok();
}
