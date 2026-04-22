//! Integration tests proving coupon redemption side effects remain exactly-once
//! across client verify, webhook capture, and retry/reconciliation flows.
//!
//! **Setup**
//! - Set `TEST_DATABASE_URL` or `DATABASE_URL`.
//! - Schema must be loaded (e.g. migrations or `backend/database/sql_dump/01_schema.sql`).
//!
//! **Run**
//! - `cargo test --test integration_coupon_redemption_consistency -- --ignored`

mod integration_common;
mod provider_test_gate;

use chrono::{Duration, Utc};
use core_db_entities::entity::sea_orm_active_enums::Status as PaymentIntentStatus;
use core_db_entities::entity::{
    coupon_redemptions, coupons, inventory, order_status, orders, payment_intents,
    product_categories, product_variants, products, shipping_addresses, user_roles,
};
use core_operations::handlers::payment_intents::{create_payment_intent, verify_razorpay_payment};
use core_operations::handlers::webhooks::ingest_webhook;
use core_operations::procedures::orders::place_order;
use hmac::{Hmac, Mac};
use integration_common::test_db_url;
use proto::proto::core::{
    CreateCartItemRequest, CreateCouponRequest, CreatePaymentIntentRequest, CreateUserRequest,
    IngestWebhookRequest, PlaceOrderRequest, VerifyRazorpayPaymentRequest,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Database, EntityTrait, QueryFilter,
    TransactionTrait,
};
use sha2::Sha256;
use tonic::Request;

type HmacSha256 = Hmac<Sha256>;

struct CouponOrderSetup {
    order_id: i64,
    coupon_id: i64,
    coupon_code: String,
    payment_intent_id: i64,
    razorpay_order_id: String,
    amount_paise: i64,
}

async fn ensure_order_status(txn: &sea_orm::DatabaseTransaction, name: &str) -> i64 {
    if let Ok(Some(id)) = core_operations::order_state_machine::get_status_id(txn, name).await {
        return id;
    }
    let row = order_status::ActiveModel {
        status_id: ActiveValue::NotSet,
        status_name: ActiveValue::Set(name.to_string()),
    }
    .insert(txn)
    .await
    .expect("insert OrderStatus");
    row.status_id
}

fn compute_razorpay_signature(order_id: &str, payment_id: &str, secret: &str) -> String {
    let payload = format!("{}|{}", order_id, payment_id);
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC key");
    mac.update(payload.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

async fn seed_coupon_checkout_order(
    txn: &sea_orm::DatabaseTransaction,
    now_tag: i64,
) -> CouponOrderSetup {
    let _ = ensure_order_status(txn, "pending").await;

    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(format!("itest_coupon_consistency_role_{}", now_tag)),
    }
    .insert(txn)
    .await
    .expect("insert UserRoles");

    let user = core_operations::handlers::users::create_user(
        txn,
        Request::new(CreateUserRequest {
            username: format!("itest_coupon_consistency_{}", now_tag),
            email: format!("itest_coupon_consistency+{}@example.com", now_tag),
            full_name: None,
            address: None,
            phone: None,
            auth_provider: "email".to_string(),
            password_plain: Some("StrongPass123!".to_string()),
            google_sub: None,
            role_id: Some(role.role_id),
        }),
    )
    .await
    .expect("create_user")
    .into_inner()
    .items
    .into_iter()
    .next()
    .expect("created user");
    let user_id = user.user_id;

    let shipping = shipping_addresses::ActiveModel {
        shipping_address_id: ActiveValue::NotSet,
        user_id: ActiveValue::Set(Some(user_id)),
        recipient_name: ActiveValue::Set(Some("Coupon Consistency".to_string())),
        phone_number: ActiveValue::Set(Some("+919999999999".to_string())),
        is_default: ActiveValue::Set(1),
        country: ActiveValue::Set("IN".to_string()),
        state_region: ActiveValue::Set("KA".to_string()),
        city: ActiveValue::Set("Bengaluru".to_string()),
        postal_code: ActiveValue::Set("560001".to_string()),
        road: ActiveValue::Set(None),
        apartment_no_or_name: ActiveValue::Set(None),
    }
    .insert(txn)
    .await
    .expect("insert ShippingAddresses");

    let category = product_categories::ActiveModel {
        category_id: ActiveValue::NotSet,
        name: ActiveValue::Set(format!("itest_coupon_consistency_cat_{}", now_tag)),
    }
    .insert(txn)
    .await
    .expect("insert ProductCategories");

    let product = products::ActiveModel {
        product_id: ActiveValue::NotSet,
        sku: ActiveValue::Set(None),
        name: ActiveValue::Set("Coupon Consistency Saree".to_string()),
        slug: ActiveValue::Set(None),
        description: ActiveValue::Set(None),
        // Keep subtotal above FREE_SHIPPING_THRESHOLD_MINOR to avoid live quote dependency.
        price_paise: ActiveValue::Set(150_000),
        category_id: ActiveValue::Set(category.category_id),
        fabric: ActiveValue::Set(None),
        weave: ActiveValue::Set(None),
        occasion: ActiveValue::Set(None),
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
        product_id: ActiveValue::Set(product.product_id),
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

    let cart_id = core_operations::handlers::cart::create_cart_item(
        txn,
        Request::new(CreateCartItemRequest {
            user_id: Some(user_id),
            session_id: None,
            variant_id: variant.variant_id,
            quantity: 1,
        }),
    )
    .await
    .expect("create_cart_item")
    .into_inner()
    .items
    .into_iter()
    .next()
    .expect("created cart item")
    .cart_id;

    let coupon_code = format!("CONSIST_{}", now_tag);
    let coupon = core_operations::handlers::coupons::create_coupon(
        txn,
        Request::new(CreateCouponRequest {
            code: coupon_code.clone(),
            discount_type: "fixed_amount".to_string(),
            discount_value: 500,
            min_order_value_paise: Some(1_000),
            usage_limit: Some(10),
            max_uses_per_customer: None,
            starts_at: (Utc::now() - Duration::hours(1)).to_rfc3339(),
            ends_at: Some((Utc::now() + Duration::days(1)).to_rfc3339()),
        }),
    )
    .await
    .expect("create_coupon")
    .into_inner()
    .items
    .into_iter()
    .next()
    .expect("created coupon");

    let placed = place_order(
        txn,
        Request::new(PlaceOrderRequest {
            shipping_address_id: shipping.shipping_address_id,
            user_id,
            coupon_code: Some(coupon_code.clone()),
            selected_cart_ids: vec![cart_id],
            // Keep this fixture independent from live Razorpay order creation.
            payment_mode: Some("cod".to_string()),
        }),
    )
    .await
    .expect("place_order")
    .into_inner()
    .items
    .into_iter()
    .next()
    .expect("placed order");

    let order = orders::Entity::find_by_id(placed.order_id)
        .one(txn)
        .await
        .expect("query order")
        .expect("order exists");
    let seeded_razorpay_order_id = format!("order_coupon_consistency_{}", placed.order_id);
    let _ = create_payment_intent(
        txn,
        Request::new(CreatePaymentIntentRequest {
            order_id: placed.order_id,
            user_id,
            amount_paise: order.grand_total_minor,
            currency: Some(order.currency.clone().unwrap_or_else(|| "INR".to_string())),
            razorpay_order_id: Some(seeded_razorpay_order_id.clone()),
        }),
    )
    .await
    .expect("seed payment intent");

    let payment_intent = payment_intents::Entity::find()
        .filter(payment_intents::Column::OrderId.eq(placed.order_id))
        .filter(payment_intents::Column::RazorpayOrderId.eq(&seeded_razorpay_order_id))
        .one(txn)
        .await
        .expect("query payment_intents")
        .expect("payment intent exists");

    CouponOrderSetup {
        order_id: placed.order_id,
        coupon_id: coupon.coupon_id,
        coupon_code,
        payment_intent_id: payment_intent.intent_id,
        razorpay_order_id: payment_intent.razorpay_order_id.clone(),
        amount_paise: i64::from(payment_intent.amount_paise),
    }
}

async fn assert_coupon_side_effects(
    txn: &sea_orm::DatabaseTransaction,
    setup: &CouponOrderSetup,
    expected_usage_count: i32,
    expected_redemptions: usize,
) {
    let coupon = coupons::Entity::find_by_id(setup.coupon_id)
        .one(txn)
        .await
        .expect("query coupon")
        .expect("coupon exists");
    assert_eq!(
        coupon.code, setup.coupon_code,
        "expected to keep asserting against the same coupon row"
    );
    assert_eq!(coupon.usage_count, Some(expected_usage_count));

    let redemptions = coupon_redemptions::Entity::find()
        .filter(coupon_redemptions::Column::CouponId.eq(setup.coupon_id))
        .filter(coupon_redemptions::Column::OrderId.eq(setup.order_id))
        .all(txn)
        .await
        .expect("query coupon_redemptions");
    assert_eq!(redemptions.len(), expected_redemptions);

    let order = orders::Entity::find_by_id(setup.order_id)
        .one(txn)
        .await
        .expect("query order")
        .expect("order exists");
    assert_eq!(order.applied_coupon_id, Some(setup.coupon_id));
    assert_eq!(
        order.applied_coupon_code.as_deref(),
        Some(setup.coupon_code.as_str())
    );
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_coupon_redemption_recorded_once_on_client_verify() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_coupon_redemption_recorded_once_on_client_verify",
    ) {
        return;
    }

    const TEST_SECRET: &str = "itest_coupon_verify_secret";

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let setup = seed_coupon_checkout_order(&txn, Utc::now().timestamp_millis()).await;
    let payment_id = "pay_coupon_verify_once";
    let signature = compute_razorpay_signature(&setup.razorpay_order_id, payment_id, TEST_SECRET);

    assert_coupon_side_effects(&txn, &setup, 0, 0).await;

    std::env::set_var("RAZORPAY_KEY_SECRET", TEST_SECRET);
    let verify = verify_razorpay_payment(
        &txn,
        Request::new(VerifyRazorpayPaymentRequest {
            order_id: setup.order_id,
            razorpay_order_id: setup.razorpay_order_id.clone(),
            razorpay_payment_id: payment_id.to_string(),
            razorpay_signature: signature,
        }),
    )
    .await
    .expect("verify_razorpay_payment should succeed")
    .into_inner();

    assert!(verify.verified);
    assert_coupon_side_effects(&txn, &setup, 1, 1).await;

    let intent = payment_intents::Entity::find_by_id(setup.payment_intent_id)
        .one(&txn)
        .await
        .expect("query intent")
        .expect("intent exists");
    assert_eq!(intent.status, PaymentIntentStatus::Processed);
    assert_eq!(intent.razorpay_payment_id.as_deref(), Some(payment_id));

    txn.rollback().await.ok();
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_coupon_redemption_recorded_once_on_webhook_capture_and_replay() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_coupon_redemption_recorded_once_on_webhook_capture_and_replay",
    ) {
        return;
    }

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let setup = seed_coupon_checkout_order(&txn, Utc::now().timestamp_millis()).await;
    let payment_id = format!("pay_coupon_webhook_{}", Utc::now().timestamp_millis());
    let payload = serde_json::json!({
        "event": "payment.captured",
        "payload": {
            "payment": {
                "entity": {
                    "id": payment_id,
                    "order_id": setup.razorpay_order_id,
                    "amount": setup.amount_paise,
                    "currency": "INR"
                }
            }
        }
    });

    assert_coupon_side_effects(&txn, &setup, 0, 0).await;

    let first = ingest_webhook(
        &txn,
        Request::new(IngestWebhookRequest {
            provider: "razorpay".to_string(),
            event_type: "payment.captured".to_string(),
            webhook_id: format!("razorpay:first:{}", payment_id),
            payload_json: payload.to_string(),
            signature_verified: true,
            provider_event_id: Some(format!("evt:first:{}", payment_id)),
        }),
    )
    .await;
    assert!(
        first.is_ok(),
        "first webhook should succeed: {:?}",
        first.err()
    );
    assert_coupon_side_effects(&txn, &setup, 1, 1).await;

    let second = ingest_webhook(
        &txn,
        Request::new(IngestWebhookRequest {
            provider: "razorpay".to_string(),
            event_type: "payment.captured".to_string(),
            webhook_id: format!("razorpay:second:{}", payment_id),
            payload_json: payload.to_string(),
            signature_verified: true,
            provider_event_id: Some(format!("evt:second:{}", payment_id)),
        }),
    )
    .await;
    assert!(
        second.is_ok(),
        "replayed webhook with a new delivery id should still be idempotent: {:?}",
        second.err()
    );
    assert_coupon_side_effects(&txn, &setup, 1, 1).await;

    txn.rollback().await.ok();
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_coupon_redemption_stays_exactly_once_across_verify_retry_and_webhook_reconciliation(
) {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_coupon_redemption_stays_exactly_once_across_verify_retry_and_webhook_reconciliation",
    ) {
        return;
    }

    const TEST_SECRET: &str = "itest_coupon_reconcile_secret";

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let setup = seed_coupon_checkout_order(&txn, Utc::now().timestamp_millis()).await;
    let payment_id = format!("pay_coupon_reconcile_{}", Utc::now().timestamp_millis());
    let signature = compute_razorpay_signature(&setup.razorpay_order_id, &payment_id, TEST_SECRET);

    std::env::set_var("RAZORPAY_KEY_SECRET", TEST_SECRET);
    let first_verify = verify_razorpay_payment(
        &txn,
        Request::new(VerifyRazorpayPaymentRequest {
            order_id: setup.order_id,
            razorpay_order_id: setup.razorpay_order_id.clone(),
            razorpay_payment_id: payment_id.clone(),
            razorpay_signature: signature.clone(),
        }),
    )
    .await;
    assert!(
        first_verify.is_ok(),
        "first verify should succeed: {:?}",
        first_verify.err()
    );
    assert_coupon_side_effects(&txn, &setup, 1, 1).await;

    let second_verify = verify_razorpay_payment(
        &txn,
        Request::new(VerifyRazorpayPaymentRequest {
            order_id: setup.order_id,
            razorpay_order_id: setup.razorpay_order_id.clone(),
            razorpay_payment_id: payment_id.clone(),
            razorpay_signature: signature,
        }),
    )
    .await;
    assert!(
        second_verify.is_ok(),
        "client retry should be idempotent: {:?}",
        second_verify.err()
    );
    assert_coupon_side_effects(&txn, &setup, 1, 1).await;

    let payload = serde_json::json!({
        "event": "payment.captured",
        "payload": {
            "payment": {
                "entity": {
                    "id": payment_id,
                    "order_id": setup.razorpay_order_id,
                    "amount": setup.amount_paise,
                    "currency": "INR"
                }
            }
        }
    });
    let webhook = ingest_webhook(
        &txn,
        Request::new(IngestWebhookRequest {
            provider: "razorpay".to_string(),
            event_type: "payment.captured".to_string(),
            webhook_id: format!("razorpay:reconcile:{}", setup.order_id),
            payload_json: payload.to_string(),
            signature_verified: true,
            provider_event_id: Some(format!("evt:reconcile:{}", setup.order_id)),
        }),
    )
    .await;
    assert!(
        webhook.is_ok(),
        "webhook reconciliation after client verify should remain idempotent: {:?}",
        webhook.err()
    );
    assert_coupon_side_effects(&txn, &setup, 1, 1).await;

    txn.rollback().await.ok();
}
