//! Integration tests for payments: place_order creates payment intent, verify_razorpay_payment.
//!
//! **Setup**
//! - Set `TEST_DATABASE_URL` or `DATABASE_URL`.
//! - Schema must be loaded (e.g. migrations or `backend/database/sql_dump/01_schema.sql`).
//! - For P2/P3, `RAZORPAY_KEY_SECRET` is set in the test (no real key needed).
//!
//! **Run**
//! - `cargo test --test integration_payments -- --ignored`

mod integration_common;

use chrono::Utc;
use core_db_entities::entity::sea_orm_active_enums::Status as PaymentIntentStatus;
use core_db_entities::entity::{
    inventory, order_status, payment_intents, product_categories, product_variants, products,
    shipping_addresses, user_roles,
};
use core_operations::procedures::orders::place_order;
use hmac::{Hmac, Mac};
use integration_common::test_db_url;
use proto::proto::core::{
    CreateCartItemRequest, CreateUserRequest, PlaceOrderRequest,
    VerifyRazorpayPaymentRequest,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Database, EntityTrait, QueryFilter, TransactionTrait,
};
use sha2::Sha256;
use tonic::Request;

type HmacSha256 = Hmac<Sha256>;

/// Compute Razorpay-style signature: HMAC-SHA256(secret, "razorpay_order_id|razorpay_payment_id") hex-encoded.
fn compute_razorpay_signature(order_id: &str, payment_id: &str, secret: &str) -> String {
    let payload = format!("{}|{}", order_id, payment_id);
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC key");
    mac.update(payload.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

async fn place_order_setup(
    txn: &sea_orm::DatabaseTransaction,
    now_tag: i64,
    cart_total_paise: i64,
) -> (i64, i64) {
    let pending = order_status::Entity::find()
        .filter(order_status::Column::StatusName.eq("pending"))
        .one(txn)
        .await
        .expect("query OrderStatus");
    if pending.is_none() {
        let status = order_status::ActiveModel {
            status_id: ActiveValue::NotSet,
            status_name: ActiveValue::Set("pending".to_string()),
        };
        let _ = status.insert(txn).await.expect("insert pending OrderStatus");
    }

    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(format!("itest_pay_{}", now_tag)),
    }
    .insert(txn)
    .await
    .expect("insert UserRoles");

    let user_res = core_operations::handlers::users::create_user(
        txn,
        Request::new(CreateUserRequest {
            username: format!("itest_pay_{}", now_tag),
            email: format!("itest_pay+{}@example.com", now_tag),
            full_name: None,
            address: None,
            phone: None,
            password_plain: "StrongPass123!".to_string(),
            role_id: Some(role.role_id),
        }),
    )
    .await
    .expect("create_user");
    let user_id = user_res.into_inner().items[0].user_id;

    let ship = shipping_addresses::ActiveModel {
        shipping_address_id: ActiveValue::NotSet,
        user_id: ActiveValue::Set(Some(user_id)),
        country: ActiveValue::Set("IN".to_string()),
        state_region: ActiveValue::Set("KA".to_string()),
        city: ActiveValue::Set("City".to_string()),
        postal_code: ActiveValue::Set("100001".to_string()),
        road: ActiveValue::Set(None),
        apartment_no_or_name: ActiveValue::Set(None),
    }
    .insert(txn)
    .await
    .expect("insert ShippingAddresses");
    let shipping_id = ship.shipping_address_id;

    let cat = product_categories::ActiveModel {
        category_id: ActiveValue::NotSet,
        name: ActiveValue::Set(format!("itest_cat_pay_{}", now_tag)),
    }
    .insert(txn)
    .await
    .expect("insert ProductCategories");

    let prod = products::ActiveModel {
        product_id: ActiveValue::NotSet,
        sku: ActiveValue::Set(None),
        name: ActiveValue::Set("Payment Test Product".to_string()),
        slug: ActiveValue::Set(None),
        description: ActiveValue::Set(None),
        price: ActiveValue::Set(None),
        price_paise: ActiveValue::Set(cart_total_paise as i32),
        category_id: ActiveValue::Set(cat.category_id),
        fabric: ActiveValue::Set(None),
        weave: ActiveValue::Set(None),
        occasion: ActiveValue::Set(None),
        length_meters: ActiveValue::Set(None),
        has_blouse_piece: ActiveValue::Set(None),
        care_instructions: ActiveValue::Set(None),
        product_status_id: ActiveValue::Set(None),
        created_at: ActiveValue::Set(Some(Utc::now())),
        updated_at: ActiveValue::Set(None),
    }
    .insert(txn)
    .await
    .expect("insert Products");

    let variant = product_variants::ActiveModel {
        variant_id: ActiveValue::NotSet,
        product_id: ActiveValue::Set(prod.product_id),
        size_id: ActiveValue::Set(None),
        color_id: ActiveValue::Set(None),
        additional_price: ActiveValue::Set(Some(0)),
    }
    .insert(txn)
    .await
    .expect("insert ProductVariants");

    let _ = inventory::ActiveModel {
        inventory_id: ActiveValue::NotSet,
        variant_id: ActiveValue::Set(Some(variant.variant_id)),
        quantity_available: ActiveValue::Set(Some(10)),
        quantity_reserved: ActiveValue::Set(Some(0)),
        reorder_level: ActiveValue::Set(None),
        updated_at: ActiveValue::Set(Some(Utc::now())),
    }
    .insert(txn)
    .await
    .expect("insert Inventory");

    let _ = core_operations::handlers::cart::create_cart_item(
        txn,
        Request::new(CreateCartItemRequest {
            user_id: Some(user_id),
            session_id: None,
            variant_id: variant.variant_id,
            quantity: 1,
        }),
    )
    .await
    .expect("create_cart_item");

    let place_res = place_order(
        txn,
        Request::new(PlaceOrderRequest {
            shipping_address_id: shipping_id,
            user_id,
            coupon_code: None,
        }),
    )
    .await
    .expect("place_order");
    let order_id = place_res.into_inner().items[0].order_id;
    (user_id, order_id)
}

/// P1 – place_order creates a payment_intents row with correct order_id, amount, and pending status.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_place_order_creates_payment_intent() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let cart_total = 3_000_i64;
    let (_user_id, order_id) = place_order_setup(&txn, now_tag, cart_total).await;

    let intents = payment_intents::Entity::find()
        .filter(payment_intents::Column::OrderId.eq(order_id))
        .all(&txn)
        .await
        .expect("query payment_intents");
    assert_eq!(intents.len(), 1, "place_order should create exactly one payment intent");
    let intent = &intents[0];
    assert_eq!(intent.order_id, Some(order_id));
    assert_eq!(intent.amount_paise, cart_total as i32);
    assert_eq!(intent.status, PaymentIntentStatus::Pending);
    assert!(intent.razorpay_payment_id.is_none());

    txn.rollback().await.ok();
}

/// P2 – Happy-path verify_razorpay_payment marks intent as ClientVerified and sets razorpay_payment_id.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_verify_razorpay_payment_success_updates_intent() {
    const TEST_SECRET: &str = "itest_razorpay_secret";

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let (_user_id, order_id) = place_order_setup(&txn, now_tag, 2_000).await;

    let intents = payment_intents::Entity::find()
        .filter(payment_intents::Column::OrderId.eq(order_id))
        .all(&txn)
        .await
        .expect("query payment_intents");
    let intent = intents.into_iter().next().expect("one intent");
    let razorpay_order_id = intent.razorpay_order_id.clone();
    let razorpay_payment_id = "pay_test_verified_123".to_string();
    let signature = compute_razorpay_signature(&razorpay_order_id, &razorpay_payment_id, TEST_SECRET);

    std::env::set_var("RAZORPAY_KEY_SECRET", TEST_SECRET);
    let verify_res = core_operations::handlers::payment_intents::verify_razorpay_payment(
        &txn,
        Request::new(VerifyRazorpayPaymentRequest {
            order_id,
            razorpay_order_id: razorpay_order_id.clone(),
            razorpay_payment_id: razorpay_payment_id.clone(),
            razorpay_signature: signature,
        }),
    )
    .await
    .expect("verify_razorpay_payment should not error");
    let inner = verify_res.into_inner();
    assert!(inner.verified, "signature valid => verified true");
    assert!(inner.payment_intent.is_some());

    let updated = payment_intents::Entity::find()
        .filter(payment_intents::Column::OrderId.eq(order_id))
        .filter(payment_intents::Column::RazorpayOrderId.eq(&razorpay_order_id))
        .one(&txn)
        .await
        .expect("query intent")
        .expect("intent exists");
    assert_eq!(updated.status, PaymentIntentStatus::ClientVerified);
    assert_eq!(updated.razorpay_payment_id.as_deref(), Some(razorpay_payment_id.as_str()));

    txn.rollback().await.ok();
}

/// P3 – verify_razorpay_payment with invalid signature returns verification failure and does not update DB.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_verify_razorpay_payment_invalid_signature_no_update() {
    const TEST_SECRET: &str = "itest_razorpay_secret_p3";

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let (_user_id, order_id) = place_order_setup(&txn, now_tag, 1_500).await;

    let intents = payment_intents::Entity::find()
        .filter(payment_intents::Column::OrderId.eq(order_id))
        .all(&txn)
        .await
        .expect("query payment_intents");
    let intent = intents.into_iter().next().expect("one intent");
    let razorpay_order_id = intent.razorpay_order_id.clone();
    let razorpay_payment_id = "pay_test_invalid_456".to_string();

    std::env::set_var("RAZORPAY_KEY_SECRET", TEST_SECRET);
    let verify_res = core_operations::handlers::payment_intents::verify_razorpay_payment(
        &txn,
        Request::new(VerifyRazorpayPaymentRequest {
            order_id,
            razorpay_order_id: razorpay_order_id.clone(),
            razorpay_payment_id: razorpay_payment_id.clone(),
            razorpay_signature: "invalid_signature_hex".to_string(),
        }),
    )
    .await
    .expect("verify_razorpay_payment returns Ok even when signature invalid");
    let inner = verify_res.into_inner();
    assert!(!inner.verified);
    assert!(inner.payment_intent.is_none());

    let unchanged = payment_intents::Entity::find()
        .filter(payment_intents::Column::OrderId.eq(order_id))
        .filter(payment_intents::Column::RazorpayOrderId.eq(&razorpay_order_id))
        .one(&txn)
        .await
        .expect("query intent")
        .expect("intent exists");
    assert_eq!(unchanged.status, PaymentIntentStatus::Pending);
    assert!(unchanged.razorpay_payment_id.is_none());

    txn.rollback().await.ok();
}
