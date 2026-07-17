//! Targeted DB-backed concurrency proofs for launch-hardening without provider calls.
//!
//! **Setup**
//! - Set `TEST_DATABASE_URL` or `DATABASE_URL`.
//! - Schema must be loaded first.
//!
//! **Run**
//! - `cargo test --test integration_launch_concurrency -- --ignored --test-threads=1`

mod integration_common;
mod provider_test_gate;

use chrono::{Duration, Utc};
use core_db_entities::entity::sea_orm_active_enums::{
    FulfillmentStatus, PaymentStatus, Status as PaymentIntentStatus,
};
use core_db_entities::entity::{
    cart, coupon_redemptions, coupons, inventory, order_status, orders, payment_intents,
    product_categories, product_variants, products, shipping_addresses, user_roles,
};
use core_operations::handlers::orders::public_order_ref::generate_public_order_ref_candidate;
use core_operations::handlers::payment_intents::{create_payment_intent, finalize_order_paid};
use core_operations::handlers::webhooks::ingest_webhook;
use core_operations::procedures::stale_order_expiry::expire_stale_pending_orders;
use hmac::{Hmac, Mac};
use integration_common::test_db_url;
use proto::proto::core::{
    CreateCartItemRequest, CreateCouponRequest, CreateOrderDetailRequest,
    CreateOrderDetailsRequest, CreatePaymentIntentRequest, CreateUserRequest, IngestWebhookRequest,
    VerifyRazorpayPaymentRequest,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, Database, EntityTrait,
    QueryFilter, Statement, TransactionTrait,
};
use sha2::Sha256;
use tokio::time::{sleep, Duration as TokioDuration};
use tonic::Request;

type HmacSha256 = Hmac<Sha256>;

async fn ensure_order_status_row(
    txn: &sea_orm::DatabaseTransaction,
    name: &str,
) -> order_status::Model {
    if let Some(existing) = order_status::Entity::find()
        .filter(order_status::Column::StatusName.eq(name))
        .one(txn)
        .await
        .expect("query order status")
    {
        return existing;
    }
    order_status::ActiveModel {
        status_id: ActiveValue::NotSet,
        status_name: ActiveValue::Set(name.to_string()),
    }
    .insert(txn)
    .await
    .expect("insert order status")
}

async fn seed_user(txn: &sea_orm::DatabaseTransaction, tag: &str) -> i64 {
    let unique = format!("{tag}_{}", Utc::now().timestamp_micros());
    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(format!("launch_concurrency_role_{unique}")),
    }
    .insert(txn)
    .await
    .expect("insert role");

    core_operations::handlers::users::create_user(
        txn,
        Request::new(CreateUserRequest {
            username: format!("launch_concurrency_{unique}"),
            email: format!("launch_concurrency_{unique}@example.com"),
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
    .expect("create user")
    .into_inner()
    .items[0]
        .user_id
}

async fn seed_variant(
    txn: &sea_orm::DatabaseTransaction,
    tag: &str,
    price_paise: i32,
    inventory_qty: i64,
) -> i64 {
    let category = product_categories::ActiveModel {
        category_id: ActiveValue::NotSet,
        name: ActiveValue::Set(format!(
            "launch_concurrency_cat_{}_{}",
            tag,
            Utc::now().timestamp_micros()
        )),
    }
    .insert(txn)
    .await
    .expect("insert category");

    let product = products::ActiveModel {
        product_id: ActiveValue::NotSet,
        sku: ActiveValue::Set(None),
        name: ActiveValue::Set(format!("Launch Concurrency Product {tag}")),
        slug: ActiveValue::Set(None),
        description: ActiveValue::Set(None),
        price_paise: ActiveValue::Set(price_paise),
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
    .expect("insert product");

    let variant = product_variants::ActiveModel {
        variant_id: ActiveValue::NotSet,
        product_id: ActiveValue::Set(product.product_id),
        size_id: ActiveValue::Set(None),
        color_id: ActiveValue::Set(None),
        additional_price: ActiveValue::Set(Some(0)),
    }
    .insert(txn)
    .await
    .expect("insert variant");

    let _ = inventory::ActiveModel {
        inventory_id: ActiveValue::NotSet,
        variant_id: ActiveValue::Set(Some(variant.variant_id)),
        quantity_available: ActiveValue::Set(Some(inventory_qty)),
        quantity_reserved: ActiveValue::Set(Some(0)),
        reorder_level: ActiveValue::Set(None),
        updated_at: ActiveValue::Set(Some(Utc::now())),
    }
    .insert(txn)
    .await
    .expect("insert inventory");

    variant.variant_id
}

async fn seed_address(txn: &sea_orm::DatabaseTransaction, user_id: i64, tag: &str) -> i64 {
    let row = shipping_addresses::ActiveModel {
        shipping_address_id: ActiveValue::NotSet,
        user_id: ActiveValue::Set(Some(user_id)),
        recipient_name: ActiveValue::Set(Some(format!("Recipient {tag}"))),
        phone_number: ActiveValue::Set(Some("+919999999999".to_string())),
        is_default: ActiveValue::Set(1),
        country: ActiveValue::Set("IN".to_string()),
        state_region: ActiveValue::Set("KA".to_string()),
        city: ActiveValue::Set("Bengaluru".to_string()),
        postal_code: ActiveValue::Set("560001".to_string()),
        road: ActiveValue::Set(Some("MG Road".to_string())),
        apartment_no_or_name: ActiveValue::Set(None),
    }
    .insert(txn)
    .await
    .expect("insert shipping");
    row.shipping_address_id
}

async fn seed_order_with_line(
    txn: &sea_orm::DatabaseTransaction,
    tag: &str,
    user_id: i64,
    variant_id: i64,
    quantity: i64,
    coupon_id: Option<i64>,
) -> (i64, i64) {
    let pending = ensure_order_status_row(txn, "pending").await;
    let order = orders::ActiveModel {
        order_id: ActiveValue::NotSet,
        user_id: ActiveValue::Set(user_id),
        order_date: ActiveValue::Set(Utc::now()),
        created_at: ActiveValue::Set(Utc::now()),
        cancel_window_ends_at: ActiveValue::NotSet,
        earliest_booking_at: ActiveValue::NotSet,
        pickup_target_at: ActiveValue::NotSet,
        pickup_target_reason: ActiveValue::NotSet,
        pickup_target_set_by: ActiveValue::NotSet,
        pickup_target_updated_at: ActiveValue::NotSet,
        shipping_address_id: ActiveValue::Set(seed_address(txn, user_id, tag).await),
        total_amount: ActiveValue::Set(Some(rust_decimal::Decimal::new(1000, 2))),
        status_id: ActiveValue::Set(pending.status_id),
        order_number: ActiveValue::Set(None),
        public_order_ref: ActiveValue::Set(generate_public_order_ref_candidate(Utc::now())),
        payment_status: ActiveValue::Set(Some(PaymentStatus::Pending)),
        payment_method: ActiveValue::Set(Some("razorpay".to_string())),
        currency: ActiveValue::Set(Some("INR".to_string())),
        updated_at: ActiveValue::Set(Some(Utc::now())),
        subtotal_minor: ActiveValue::Set(1_000),
        items_total_minor_before_discount: ActiveValue::Set(Some(1_000)),
        shipping_minor: ActiveValue::Set(Some(0)),
        shipping_charge_minor: ActiveValue::Set(Some(0)),
        tax_total_minor: ActiveValue::Set(Some(0)),
        discount_total_minor: ActiveValue::Set(Some(if coupon_id.is_some() { 100 } else { 0 })),
        items_total_minor_after_discount: ActiveValue::Set(Some(if coupon_id.is_some() {
            900
        } else {
            1_000
        })),
        grand_total_minor: ActiveValue::Set(if coupon_id.is_some() { 900 } else { 1_000 }),
        invoice_id: ActiveValue::Set(None),
        invoice_number: ActiveValue::Set(None),
        invoice_generated_at: ActiveValue::Set(None),
        invoice_storage_path: ActiveValue::Set(None),
        applied_coupon_id: ActiveValue::Set(coupon_id),
        applied_coupon_code: ActiveValue::Set(coupon_id.map(|_| format!("LASTSLOT_{tag}"))),
        applied_discount_paise: ActiveValue::Set(coupon_id.map(|_| 100)),
        refund_settlement_status: ActiveValue::NotSet,
        fulfillment_status: ActiveValue::Set(FulfillmentStatus::NotCreated),
    }
    .insert(txn)
    .await
    .expect("insert order");

    let line = core_operations::handlers::order_details::create_order_details(
        txn,
        Request::new(CreateOrderDetailsRequest {
            order_details: vec![CreateOrderDetailRequest {
                order_id: order.order_id,
                variant_id,
                quantity,
                price_paise: 1_000,
                unit_price_minor: Some(1_000),
                discount_minor: None,
                tax_minor: None,
                sku: None,
                title: Some(format!("Order line {tag}")),
            }],
        }),
    )
    .await
    .expect("create order detail")
    .into_inner()
    .items
    .into_iter()
    .next()
    .expect("order detail");

    (order.order_id, line.order_detail_id)
}

fn compute_signature(order_id: &str, payment_id: &str, secret: &str) -> String {
    let payload = format!("{order_id}|{payment_id}");
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("hmac");
    mac.update(payload.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_stale_expiry_two_workers_claim_rows_once() {
    let db = Database::connect(&test_db_url()).await.expect("connect");
    let setup = db.begin().await.expect("begin");
    let unique = Utc::now().timestamp_micros();
    let user_id = seed_user(&setup, "expiry_claim").await;
    let active_sale = ensure_order_status_row(&setup, "active_sale").await;
    let cancelled = ensure_order_status_row(&setup, "cancelled").await;
    let variant_id = seed_variant(&setup, "expiry_claim", 1_000, 4).await;
    let inventory_row = inventory::Entity::find()
        .filter(inventory::Column::VariantId.eq(variant_id))
        .one(&setup)
        .await
        .expect("query inventory")
        .expect("inventory exists");

    let (order_id, _) =
        seed_order_with_line(&setup, "expiry_claim", user_id, variant_id, 2, None).await;
    orders::ActiveModel {
        order_id: ActiveValue::Set(order_id),
        status_id: ActiveValue::Set(active_sale.status_id),
        ..Default::default()
    }
    .update(&setup)
    .await
    .expect("set order active_sale");
    let intent = payment_intents::ActiveModel {
        intent_id: ActiveValue::NotSet,
        razorpay_order_id: ActiveValue::Set(format!("order_expiry_claim_{unique}")),
        order_id: ActiveValue::Set(Some(order_id)),
        active_order_id: ActiveValue::NotSet,
        user_id: ActiveValue::Set(Some(user_id)),
        amount_paise: ActiveValue::Set(1_000),
        currency: ActiveValue::Set(Some("INR".to_string())),
        status: ActiveValue::Set(PaymentIntentStatus::Pending),
        razorpay_payment_id: ActiveValue::Set(None),
        metadata: ActiveValue::Set(None),
        gateway_fee_paise: ActiveValue::Set(None),
        gateway_tax_paise: ActiveValue::Set(None),
        created_at: ActiveValue::Set(Some(Utc::now() - Duration::hours(2))),
        expires_at: ActiveValue::Set(Utc::now() - Duration::minutes(30)),
    }
    .insert(&setup)
    .await
    .expect("insert intent");

    // Keep this concurrency proof deterministic in a shared integration DB:
    // prevent unrelated historical pending intents from being claimed by worker B.
    let _ = setup
        .execute(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::MySql,
            r#"UPDATE PaymentIntents
               SET expires_at = DATE_ADD(UTC_TIMESTAMP(), INTERVAL 1 DAY)
               WHERE status = 'pending'
                 AND intent_id <> ?"#,
            [intent.intent_id.into()],
        ))
        .await
        .expect("defer unrelated pending intents");
    setup.commit().await.expect("commit");

    let db_a = Database::connect(&test_db_url()).await.expect("connect a");
    let db_b = Database::connect(&test_db_url()).await.expect("connect b");
    let (first, second) = tokio::join!(
        expire_stale_pending_orders(&db_a, 10),
        expire_stale_pending_orders(&db_b, 10)
    );
    let first = first.expect("worker a");
    let second = second.expect("worker b");

    let verify = db.begin().await.expect("verify begin");
    let refreshed_order = orders::Entity::find_by_id(order_id)
        .one(&verify)
        .await
        .expect("query order")
        .expect("order exists");
    let refreshed_intent = payment_intents::Entity::find_by_id(intent.intent_id)
        .one(&verify)
        .await
        .expect("query intent")
        .expect("intent exists");
    let refreshed_inventory = inventory::Entity::find_by_id(inventory_row.inventory_id)
        .one(&verify)
        .await
        .expect("query inventory")
        .expect("inventory exists");

    assert_eq!(refreshed_order.status_id, cancelled.status_id);
    assert_eq!(refreshed_intent.status, PaymentIntentStatus::Failed);
    assert_eq!(refreshed_inventory.quantity_available, Some(6));
    assert!(
        first + second >= 1,
        "at least one worker should process expired rows while our claimed row is restored exactly once"
    );

    let repeat = expire_stale_pending_orders(&db, 10)
        .await
        .expect("repeat worker should succeed");
    assert_eq!(repeat, 0, "repeated worker execution should be idempotent");
    verify.rollback().await.ok();
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_duplicate_payment_intent_race_returns_single_active_intent() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_duplicate_payment_intent_race_returns_single_active_intent",
    ) {
        return;
    }

    let db = Database::connect(&test_db_url()).await.expect("connect");
    let setup = db.begin().await.expect("begin");
    let unique = Utc::now().timestamp_micros();
    let user_id = seed_user(&setup, "pi_race").await;
    let _ = ensure_order_status_row(&setup, "pending").await;
    let order = orders::ActiveModel {
        order_id: ActiveValue::NotSet,
        user_id: ActiveValue::Set(user_id),
        order_date: ActiveValue::Set(Utc::now()),
        created_at: ActiveValue::Set(Utc::now()),
        cancel_window_ends_at: ActiveValue::NotSet,
        earliest_booking_at: ActiveValue::NotSet,
        pickup_target_at: ActiveValue::NotSet,
        pickup_target_reason: ActiveValue::NotSet,
        pickup_target_set_by: ActiveValue::NotSet,
        pickup_target_updated_at: ActiveValue::NotSet,
        shipping_address_id: ActiveValue::Set(seed_address(&setup, user_id, "pi_race").await),
        total_amount: ActiveValue::Set(Some(rust_decimal::Decimal::new(1000, 2))),
        status_id: ActiveValue::Set(ensure_order_status_row(&setup, "pending").await.status_id),
        order_number: ActiveValue::Set(None),
        public_order_ref: ActiveValue::Set(generate_public_order_ref_candidate(Utc::now())),
        payment_status: ActiveValue::Set(Some(PaymentStatus::Pending)),
        payment_method: ActiveValue::Set(Some("razorpay".to_string())),
        currency: ActiveValue::Set(Some("INR".to_string())),
        updated_at: ActiveValue::Set(Some(Utc::now())),
        subtotal_minor: ActiveValue::Set(1_000),
        items_total_minor_before_discount: ActiveValue::Set(Some(1_000)),
        shipping_minor: ActiveValue::Set(Some(0)),
        shipping_charge_minor: ActiveValue::Set(Some(0)),
        tax_total_minor: ActiveValue::Set(Some(0)),
        discount_total_minor: ActiveValue::Set(Some(0)),
        items_total_minor_after_discount: ActiveValue::Set(Some(1_000)),
        grand_total_minor: ActiveValue::Set(1_000),
        invoice_id: ActiveValue::Set(None),
        invoice_number: ActiveValue::Set(None),
        invoice_generated_at: ActiveValue::Set(None),
        invoice_storage_path: ActiveValue::Set(None),
        applied_coupon_id: ActiveValue::Set(None),
        applied_coupon_code: ActiveValue::Set(None),
        applied_discount_paise: ActiveValue::Set(None),
        refund_settlement_status: ActiveValue::NotSet,
        fulfillment_status: ActiveValue::Set(FulfillmentStatus::NotCreated),
    }
    .insert(&setup)
    .await
    .expect("insert order");
    setup.commit().await.expect("commit");

    let db_a = Database::connect(&test_db_url()).await.expect("connect a");
    let db_b = Database::connect(&test_db_url()).await.expect("connect b");
    let req_a = Request::new(CreatePaymentIntentRequest {
        order_id: order.order_id,
        user_id,
        amount_paise: 1_000,
        currency: Some("INR".to_string()),
        razorpay_order_id: Some(format!("order_pi_race_a_{unique}")),
    });
    let req_b = Request::new(CreatePaymentIntentRequest {
        order_id: order.order_id,
        user_id,
        amount_paise: 1_000,
        currency: Some("INR".to_string()),
        razorpay_order_id: Some(format!("order_pi_race_b_{unique}")),
    });

    let (res_a, res_b) = tokio::join!(
        async {
            let txn = db_a.begin().await.expect("txn a");
            let res = create_payment_intent(&txn, req_a).await;
            if res.is_ok() {
                txn.commit().await.expect("commit a");
            } else {
                txn.rollback().await.ok();
            }
            res
        },
        async {
            let txn = db_b.begin().await.expect("txn b");
            let res = create_payment_intent(&txn, req_b).await;
            if res.is_ok() {
                txn.commit().await.expect("commit b");
            } else {
                txn.rollback().await.ok();
            }
            res
        }
    );
    assert!(res_a.is_ok(), "worker a should converge: {res_a:?}");
    assert!(res_b.is_ok(), "worker b should converge: {res_b:?}");

    let verify = db.begin().await.expect("verify");
    let intents = payment_intents::Entity::find()
        .filter(payment_intents::Column::OrderId.eq(order.order_id))
        .all(&verify)
        .await
        .expect("query intents");
    assert_eq!(
        intents.len(),
        1,
        "only one active payment intent row should exist"
    );
    verify.rollback().await.ok();
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_same_low_stock_sku_race_allows_only_one_reservation() {
    let db = Database::connect(&test_db_url()).await.expect("connect");
    let setup = db.begin().await.expect("begin");
    let variant_id = seed_variant(&setup, "low_stock", 1_000, 1).await;
    let inventory_row = inventory::Entity::find()
        .filter(inventory::Column::VariantId.eq(variant_id))
        .one(&setup)
        .await
        .expect("query inventory")
        .expect("inventory exists");
    setup.commit().await.expect("commit");

    let db_a = Database::connect(&test_db_url()).await.expect("connect a");
    let db_b = Database::connect(&test_db_url()).await.expect("connect b");
    let (first, second) = tokio::join!(
        async {
            let txn = db_a.begin().await.expect("txn a");
            let result = txn
                .execute(Statement::from_sql_and_values(
                    sea_orm::DbBackend::MySql,
                    r#"UPDATE Inventory
                       SET QuantityAvailable = QuantityAvailable - 1
                       WHERE VariantID = ? AND QuantityAvailable >= 1"#,
                    [variant_id.into()],
                ))
                .await
                .expect("reserve a");
            txn.commit().await.expect("commit a");
            result.rows_affected()
        },
        async {
            let txn = db_b.begin().await.expect("txn b");
            let result = txn
                .execute(Statement::from_sql_and_values(
                    sea_orm::DbBackend::MySql,
                    r#"UPDATE Inventory
                       SET QuantityAvailable = QuantityAvailable - 1
                       WHERE VariantID = ? AND QuantityAvailable >= 1"#,
                    [variant_id.into()],
                ))
                .await
                .expect("reserve b");
            txn.commit().await.expect("commit b");
            result.rows_affected()
        }
    );
    assert_eq!(
        first + second,
        1,
        "only one checkout can reserve the last unit"
    );

    let verify = db.begin().await.expect("verify");
    let refreshed = inventory::Entity::find_by_id(inventory_row.inventory_id)
        .one(&verify)
        .await
        .expect("query inventory")
        .expect("inventory exists");
    assert_eq!(refreshed.quantity_available, Some(0));
    verify.rollback().await.ok();
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_coupon_final_slot_race_sets_second_order_needs_review() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_coupon_final_slot_race_sets_second_order_needs_review",
    ) {
        return;
    }

    let db = Database::connect(&test_db_url()).await.expect("connect");
    let setup = db.begin().await.expect("begin");
    let unique = Utc::now().timestamp_micros();
    let user_a = seed_user(&setup, "coupon_a").await;
    let user_b = seed_user(&setup, "coupon_b").await;
    let confirmed = ensure_order_status_row(&setup, "confirmed").await;
    let needs_review = ensure_order_status_row(&setup, "needs_review").await;
    let coupon = core_operations::handlers::coupons::create_coupon(
        &setup,
        Request::new(CreateCouponRequest {
            code: format!("LAST_SLOT_RACE_{unique}"),
            discount_type: "fixed_amount".to_string(),
            discount_value: 100,
            min_order_value_paise: Some(100),
            usage_limit: Some(1),
            max_uses_per_customer: None,
            starts_at: (Utc::now() - Duration::hours(1)).to_rfc3339(),
            ends_at: Some((Utc::now() + Duration::hours(1)).to_rfc3339()),
        }),
    )
    .await
    .expect("create coupon")
    .into_inner()
    .items
    .into_iter()
    .next()
    .expect("coupon");

    let variant_a = seed_variant(&setup, "coupon_a", 1_000, 5).await;
    let variant_b = seed_variant(&setup, "coupon_b", 1_000, 5).await;
    let (order_a, _) = seed_order_with_line(
        &setup,
        "coupon_a",
        user_a,
        variant_a,
        1,
        Some(coupon.coupon_id),
    )
    .await;
    let (order_b, _) = seed_order_with_line(
        &setup,
        "coupon_b",
        user_b,
        variant_b,
        1,
        Some(coupon.coupon_id),
    )
    .await;
    setup.commit().await.expect("commit");

    let db_a = Database::connect(&test_db_url()).await.expect("connect a");
    let db_b = Database::connect(&test_db_url()).await.expect("connect b");
    let (res_a, res_b) = tokio::join!(
        async {
            let txn = db_a.begin().await.expect("txn a");
            let res = finalize_order_paid(&txn, order_a, "verify", "customer", "paid").await;
            if res.is_ok() {
                txn.commit().await.expect("commit a");
            } else {
                txn.rollback().await.ok();
            }
            res
        },
        async {
            let txn = db_b.begin().await.expect("txn b");
            let res = finalize_order_paid(&txn, order_b, "webhook", "system", "paid").await;
            if res.is_ok() {
                txn.commit().await.expect("commit b");
            } else {
                txn.rollback().await.ok();
            }
            res
        }
    );
    assert!(res_a.is_ok(), "first finalizer should converge: {res_a:?}");
    assert!(res_b.is_ok(), "second finalizer should converge: {res_b:?}");

    let verify = db.begin().await.expect("verify");
    let refreshed_coupon = coupons::Entity::find_by_id(coupon.coupon_id)
        .one(&verify)
        .await
        .expect("query coupon")
        .expect("coupon exists");
    let redemptions = coupon_redemptions::Entity::find()
        .filter(coupon_redemptions::Column::CouponId.eq(coupon.coupon_id))
        .all(&verify)
        .await
        .expect("query redemptions");
    let order_a_row = orders::Entity::find_by_id(order_a)
        .one(&verify)
        .await
        .expect("query a")
        .expect("order a");
    let order_b_row = orders::Entity::find_by_id(order_b)
        .one(&verify)
        .await
        .expect("query b")
        .expect("order b");

    assert_eq!(refreshed_coupon.usage_count, Some(1));
    assert_eq!(
        redemptions.len(),
        1,
        "exactly one redemption should survive the contention"
    );
    let statuses = [order_a_row.status_id, order_b_row.status_id];
    assert!(statuses.contains(&confirmed.status_id));
    assert!(statuses.contains(&needs_review.status_id));
    verify.rollback().await.ok();
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_verify_and_webhook_overlap_finalize_once() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_verify_and_webhook_overlap_finalize_once",
    ) {
        return;
    }

    const SECRET: &str = "launch_concurrency_secret";

    let db = Database::connect(&test_db_url()).await.expect("connect");
    let setup = db.begin().await.expect("begin");
    let unique = Utc::now().timestamp_micros();
    let user_id = seed_user(&setup, "verify_webhook").await;
    let confirmed = ensure_order_status_row(&setup, "confirmed").await;
    let _ = ensure_order_status_row(&setup, "pending").await;
    let variant_id = seed_variant(&setup, "verify_webhook", 1_000, 5).await;
    let (order_id, _) =
        seed_order_with_line(&setup, "verify_webhook", user_id, variant_id, 1, None).await;
    let intent = payment_intents::ActiveModel {
        intent_id: ActiveValue::NotSet,
        razorpay_order_id: ActiveValue::Set(format!("order_verify_webhook_{unique}")),
        order_id: ActiveValue::Set(Some(order_id)),
        active_order_id: ActiveValue::NotSet,
        user_id: ActiveValue::Set(Some(user_id)),
        amount_paise: ActiveValue::Set(1_000),
        currency: ActiveValue::Set(Some("INR".to_string())),
        status: ActiveValue::Set(PaymentIntentStatus::Pending),
        razorpay_payment_id: ActiveValue::Set(None),
        metadata: ActiveValue::Set(None),
        gateway_fee_paise: ActiveValue::Set(None),
        gateway_tax_paise: ActiveValue::Set(None),
        created_at: ActiveValue::Set(Some(Utc::now())),
        expires_at: ActiveValue::Set(Utc::now() + Duration::hours(1)),
    }
    .insert(&setup)
    .await
    .expect("insert intent");
    setup.commit().await.expect("commit");

    std::env::set_var("RAZORPAY_KEY_SECRET", SECRET);
    let payment_id = format!("pay_verify_webhook_overlap_{unique}");
    let signature = compute_signature(&intent.razorpay_order_id, &payment_id, SECRET);
    let payload = serde_json::json!({
        "event": "payment.captured",
        "payload": {
            "payment": {
                "entity": {
                    "id": payment_id,
                    "order_id": intent.razorpay_order_id,
                    "amount": 1_000,
                    "currency": "INR"
                }
            }
        }
    });

    let db_a = Database::connect(&test_db_url()).await.expect("connect a");
    let db_b = Database::connect(&test_db_url()).await.expect("connect b");
    let (verify_res, webhook_res) = tokio::join!(
        async {
            let txn = db_a.begin().await.expect("txn a");
            let res = core_operations::handlers::payment_intents::verify_razorpay_payment(
                &txn,
                Request::new(VerifyRazorpayPaymentRequest {
                    order_id,
                    razorpay_order_id: intent.razorpay_order_id.clone(),
                    razorpay_payment_id: payment_id.clone(),
                    razorpay_signature: signature,
                }),
            )
            .await;
            if res.is_ok() {
                txn.commit().await.expect("commit a");
            } else {
                txn.rollback().await.ok();
            }
            res
        },
        async {
            let txn = db_b.begin().await.expect("txn b");
            let res = ingest_webhook(
                &txn,
                Request::new(IngestWebhookRequest {
                    provider: "razorpay".to_string(),
                    event_type: "payment.captured".to_string(),
                    webhook_id: format!("razorpay:{payment_id}:overlap"),
                    payload_json: payload.to_string(),
                    signature_verified: true,
                    provider_event_id: None,
                    raw_signature: None,
                }),
            )
            .await;
            if res.is_ok() {
                txn.commit().await.expect("commit b");
            } else {
                txn.rollback().await.ok();
            }
            res
        }
    );
    assert!(verify_res.is_ok(), "verify should converge: {verify_res:?}");
    assert!(
        webhook_res.is_ok(),
        "webhook should converge: {webhook_res:?}"
    );

    let verify = db.begin().await.expect("verify");
    let refreshed_intent = payment_intents::Entity::find_by_id(intent.intent_id)
        .one(&verify)
        .await
        .expect("query intent")
        .expect("intent exists");
    let refreshed_order = orders::Entity::find_by_id(order_id)
        .one(&verify)
        .await
        .expect("query order")
        .expect("order exists");
    assert_eq!(
        refreshed_intent.razorpay_payment_id.as_deref(),
        Some(payment_id.as_str())
    );
    assert_eq!(refreshed_intent.status, PaymentIntentStatus::Processed);
    assert_eq!(refreshed_order.status_id, confirmed.status_id);
    verify.rollback().await.ok();
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_merge_checkout_snapshot_guard_rejects_stale_delete() {
    let db = Database::connect(&test_db_url()).await.expect("connect");
    let setup = db.begin().await.expect("begin");
    let user_id = seed_user(&setup, "merge_guard").await;
    let variant_id = seed_variant(&setup, "merge_guard", 1_000, 5).await;
    let row = core_operations::handlers::cart::create_cart_item(
        &setup,
        Request::new(CreateCartItemRequest {
            user_id: Some(user_id),
            session_id: None,
            variant_id,
            quantity: 1,
        }),
    )
    .await
    .expect("create cart row")
    .into_inner()
    .items
    .into_iter()
    .next()
    .expect("cart row");
    setup.commit().await.expect("commit");

    let db_a = Database::connect(&test_db_url()).await.expect("connect a");
    let db_b = Database::connect(&test_db_url()).await.expect("connect b");
    let (update_rows, delete_rows) = tokio::join!(
        async {
            let txn = db_a.begin().await.expect("txn a");
            let result = txn
                .execute(Statement::from_sql_and_values(
                    sea_orm::DbBackend::MySql,
                    "UPDATE Cart SET Quantity = 2 WHERE CartID = ?",
                    [row.cart_id.into()],
                ))
                .await
                .expect("update cart");
            sleep(TokioDuration::from_millis(200)).await;
            txn.commit().await.expect("commit a");
            result.rows_affected()
        },
        async {
            sleep(TokioDuration::from_millis(50)).await;
            let txn = db_b.begin().await.expect("txn b");
            let result = txn
                .execute(Statement::from_sql_and_values(
                    sea_orm::DbBackend::MySql,
                    r#"DELETE FROM Cart
                       WHERE CartID = ?
                         AND UserID = ?
                         AND VariantID = ?
                         AND Quantity = ?"#,
                    [
                        row.cart_id.into(),
                        user_id.into(),
                        variant_id.into(),
                        1_i64.into(),
                    ],
                ))
                .await
                .expect("delete cart with stale snapshot");
            txn.commit().await.expect("commit b");
            result.rows_affected()
        }
    );
    assert_eq!(update_rows, 1);
    assert_eq!(
        delete_rows, 0,
        "stale snapshot delete must not remove the changed cart row"
    );

    let verify = db.begin().await.expect("verify");
    let refreshed = cart::Entity::find_by_id(row.cart_id)
        .one(&verify)
        .await
        .expect("query cart")
        .expect("cart exists");
    assert_eq!(refreshed.quantity, 2);
    verify.rollback().await.ok();
}
