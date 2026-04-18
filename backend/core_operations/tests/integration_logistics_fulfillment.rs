//! DB-backed logistics fulfillment tests with mocked Shiprocket + Razorpay APIs.

mod integration_common;

use chrono::Utc;
use core_db_entities::entity::sea_orm_active_enums::AuthProvider;
use core_db_entities::entity::{
    inventory, order_status, orders, payment_intents, product_categories, product_variants,
    products, shipping_addresses, user_roles, users,
};
use core_operations::handlers::orders::delete_order;
use core_operations::handlers::payment_intents::verify_razorpay_payment;
use core_operations::handlers::webhooks::ingest_webhook;
use core_operations::procedures::orders::place_order;
use hmac::{Hmac, Mac};
use integration_common::test_db_url;
use proto::proto::core::{
    CreateCartItemRequest, DeleteOrderRequest, IngestWebhookRequest, PlaceOrderRequest,
    VerifyRazorpayPaymentRequest,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, Database, EntityTrait,
    QueryFilter, Statement, TransactionTrait,
};
use serde_json::json;
use sha2::Sha256;
use std::path::PathBuf;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tonic::Request;
use warp::Filter;

type HmacSha256 = Hmac<Sha256>;
static UNIQUE_COUNTER: AtomicI64 = AtomicI64::new(0);

#[derive(Default, Clone)]
struct MockState {
    /// Last Shiprocket create-adhoc `order_id` (channel ref) payloads received.
    last_channel_order_ids: Vec<String>,
    create_order_calls: usize,
    razorpay_order_calls: usize,
    pickup_calls: usize,
    cancel_calls: usize,
    refund_calls: usize,
    refund_idempotency_keys: Vec<String>,
    assigned_courier_ids: Vec<i64>,
    scheduled_pickup_dates: Vec<String>,
    cancel_should_fail: bool,
    refund_should_fail: bool,
}

fn compute_signature(order_id: &str, payment_id: &str, secret: &str) -> String {
    let payload = format!("{order_id}|{payment_id}");
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("hmac");
    mac.update(payload.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

fn unique_tag(port: u16) -> i64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let counter = UNIQUE_COUNTER.fetch_add(1, Ordering::SeqCst) as u128;
    let mixed = now
        .saturating_mul(100)
        .saturating_add(counter)
        .saturating_add(u128::from(port));
    (mixed % (i64::MAX as u128)) as i64
}

fn unique_string_id(prefix: &str) -> String {
    format!("{prefix}_{}", unique_tag(0))
}

fn load_test_env_from_repo() {
    let candidates = [
        PathBuf::from("..").join(".env"),
        PathBuf::from(".env"),
        PathBuf::from("backend").join(".env"),
    ];
    for candidate in candidates {
        if candidate.exists() {
            let _ = dotenvy::from_path(candidate);
            break;
        }
    }
}

async fn spawn_mock_server(port: u16, state: Arc<Mutex<MockState>>) -> tokio::task::JoinHandle<()> {
    let with_state = warp::any().map(move || state.clone());

    let login = warp::path!("v1" / "external" / "auth" / "login")
        .and(warp::post())
        .map(|| warp::reply::json(&json!({ "token": "shiprocket-token" })));

    let quote = warp::path!("v1" / "external" / "courier" / "serviceability")
        .and(warp::get())
        .map(|| {
            warp::reply::json(&json!({
                "data": {
                    "available_courier_companies": [{
                        "courier_company_id": 777,
                        "courier_name": "Delhivery Surface",
                        "freight_charge": 149.00,
                        "etd": "3 days"
                    }]
                }
            }))
        });

    let create_shiprocket_order = warp::path!("v1" / "external" / "orders" / "create" / "adhoc")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_state.clone())
        .map(|body: serde_json::Value, state: Arc<Mutex<MockState>>| {
            let mut guard = state.lock().expect("lock");
            guard.create_order_calls += 1;
            if let Some(ch) = body.get("order_id").and_then(|x| x.as_str()) {
                guard.last_channel_order_ids.push(ch.to_string());
            }
            let unique = unique_tag(0);
            let shiprocket_shipment_id = 555000_i64 + (unique % 100000);
            let shiprocket_order_id = 444000_i64 + (unique % 100000);
            warp::reply::json(&json!({
                "payload": { "shipment_id": shiprocket_shipment_id, "order_id": shiprocket_order_id }
            }))
        });

    let assign_awb = warp::path!("v1" / "external" / "courier" / "assign" / "awb")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_state.clone())
        .map(|body: serde_json::Value, state: Arc<Mutex<MockState>>| {
            if let Some(id) = body.get("courier_id").and_then(|x| x.as_i64()) {
                state.lock().expect("lock").assigned_courier_ids.push(id);
            }
            warp::reply::json(&json!({
                "response": { "data": {
                    "awb_code": unique_string_id("AWB"),
                    "courier_name": "Delhivery Surface",
                    "shipment_status_id": 3,
                    "shipment_status": "AWB Assigned"
                }}
            }))
        });

    let pickup = warp::path!("v1" / "external" / "courier" / "generate" / "pickup")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_state.clone())
        .map(|body: serde_json::Value, state: Arc<Mutex<MockState>>| {
            let mut guard = state.lock().expect("lock");
            guard.pickup_calls += 1;
            if let Some(date) = body.get("pickup_date").and_then(|x| x.as_str()) {
                guard.scheduled_pickup_dates.push(date.to_string());
            }
            warp::reply::json(&json!({ "pickup_status": "scheduled" }))
        });

    let cancel = warp::path!("v1" / "external" / "orders" / "cancel")
        .and(warp::post())
        .and(with_state.clone())
        .map(|state: Arc<Mutex<MockState>>| {
            let mut guard = state.lock().expect("lock");
            guard.cancel_calls += 1;
            if guard.cancel_should_fail {
                warp::reply::with_status(String::from("fail"), warp::http::StatusCode::BAD_GATEWAY)
            } else {
                warp::reply::with_status(String::new(), warp::http::StatusCode::NO_CONTENT)
            }
        });

    let rzp_order = warp::path!("v1" / "orders")
        .and(warp::post())
        .and(with_state.clone())
        .map(|state: Arc<Mutex<MockState>>| {
            let mut guard = state.lock().expect("lock");
            guard.razorpay_order_calls += 1;
            warp::reply::json(&json!({ "id": unique_string_id("order_rzp_logistics") }))
        });

    let refund = warp::path!("v1" / "payments" / String / "refund")
        .and(warp::post())
        .and(warp::header::optional::<String>("x-refund-idempotency"))
        .and(with_state.clone())
        .map(
            |_payment_id: String, idempotency: Option<String>, state: Arc<Mutex<MockState>>| {
                let mut guard = state.lock().expect("lock");
                guard.refund_calls += 1;
                if let Some(key) = idempotency.filter(|k| !k.trim().is_empty()) {
                    guard.refund_idempotency_keys.push(key);
                }
                if guard.refund_should_fail {
                    return warp::reply::with_status(
                        json!({
                            "error": {
                                "code": "BAD_REQUEST_ERROR",
                                "description": "invalid request sent",
                                "metadata": {},
                                "reason": "NA",
                                "source": "NA",
                                "step": "NA"
                            }
                        })
                        .to_string(),
                        warp::http::StatusCode::BAD_REQUEST,
                    );
                }
                warp::reply::with_status(
                    json!({
                        "id": unique_string_id("rfnd_logistics"),
                        "status": "processed"
                    })
                    .to_string(),
                    warp::http::StatusCode::OK,
                )
            },
        );

    let rzp_payment_get = warp::path!("v1" / "payments" / String)
        .and(warp::get())
        .map(|_payment_id: String| {
            warp::reply::json(&json!({
                "id": "pay_mock",
                "amount": 1_000_000_i64,
                "status": "captured"
            }))
        });

    let routes = login
        .or(quote)
        .or(create_shiprocket_order)
        .or(assign_awb)
        .or(pickup)
        .or(cancel)
        .or(rzp_order)
        .or(refund)
        .or(rzp_payment_get);

    tokio::spawn(warp::serve(routes).run(([127, 0, 0, 1], port)))
}

async fn ensure_order_status(txn: &sea_orm::DatabaseTransaction, name: &str) -> i64 {
    if let Some(existing) = order_status::Entity::find()
        .filter(order_status::Column::StatusName.eq(name))
        .one(txn)
        .await
        .expect("query status")
    {
        return existing.status_id;
    }
    order_status::ActiveModel {
        status_id: ActiveValue::NotSet,
        status_name: ActiveValue::Set(name.to_string()),
    }
    .insert(txn)
    .await
    .expect("insert status")
    .status_id
}

async fn seed_checkout_user(txn: &sea_orm::DatabaseTransaction, tag: i64) -> (i64, i64, i64) {
    let _ = ensure_order_status(txn, "pending").await;
    let _ = ensure_order_status(txn, "confirmed").await;
    let _ = ensure_order_status(txn, "cancel_pending_logistics").await;

    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(format!("itest_logistics_role_{tag}")),
    }
    .insert(txn)
    .await
    .expect("insert role");

    let phone = format!("+91{:010}", (tag % 10_000_000_000).abs());
    let user = users::ActiveModel {
        user_id: ActiveValue::NotSet,
        username: ActiveValue::Set(format!("itest_logistics_{tag}")),
        email: ActiveValue::Set(format!("itest_logistics+{tag}@example.com")),
        auth_provider: ActiveValue::Set(AuthProvider::Email),
        password_hash: ActiveValue::Set(Some("itest-hash".to_string())),
        google_sub: ActiveValue::Set(None),
        full_name: ActiveValue::Set(None),
        address: ActiveValue::Set(None),
        phone: ActiveValue::Set(Some(phone.clone())),
        create_date: ActiveValue::Set(Utc::now()),
        role_id: ActiveValue::Set(Some(role.role_id)),
        email_verified: ActiveValue::NotSet,
        email_verified_at: ActiveValue::NotSet,
        user_status_id: ActiveValue::NotSet,
        last_login_at: ActiveValue::NotSet,
        marketing_opt_out: ActiveValue::NotSet,
        updated_at: ActiveValue::NotSet,
    }
    .insert(txn)
    .await
    .expect("insert user")
    .user_id;

    let shipping = shipping_addresses::ActiveModel {
        shipping_address_id: ActiveValue::NotSet,
        user_id: ActiveValue::Set(Some(user)),
        recipient_name: ActiveValue::Set(Some("Logistics Test".to_string())),
        phone_number: ActiveValue::Set(Some(phone)),
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

    let category = product_categories::ActiveModel {
        category_id: ActiveValue::NotSet,
        name: ActiveValue::Set(format!("itest_logistics_cat_{tag}")),
    }
    .insert(txn)
    .await
    .expect("insert category");

    let product = products::ActiveModel {
        product_id: ActiveValue::NotSet,
        sku: ActiveValue::Set(None),
        name: ActiveValue::Set("Logistics Saree".to_string()),
        slug: ActiveValue::Set(None),
        description: ActiveValue::Set(None),
        price_paise: ActiveValue::Set(2_000),
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

    let inventory = inventory::ActiveModel {
        inventory_id: ActiveValue::NotSet,
        variant_id: ActiveValue::Set(Some(variant.variant_id)),
        quantity_available: ActiveValue::Set(Some(6)),
        quantity_reserved: ActiveValue::Set(Some(0)),
        reorder_level: ActiveValue::Set(None),
        updated_at: ActiveValue::Set(Some(Utc::now())),
    }
    .insert(txn)
    .await
    .expect("insert inventory");

    let _cart_id = core_operations::handlers::cart::create_cart_item(
        txn,
        Request::new(CreateCartItemRequest {
            user_id: Some(user),
            session_id: None,
            variant_id: variant.variant_id,
            quantity: 2,
        }),
    )
    .await
    .expect("create cart")
    .into_inner()
    .items[0]
        .cart_id;

    (user, shipping.shipping_address_id, inventory.inventory_id)
}

async fn place_and_pay_order(
    db: &sea_orm::DatabaseConnection,
    port: u16,
    tag: i64,
) -> (i64, i64, i64) {
    load_test_env_from_repo();
    std::env::set_var("SHIPROCKET_EMAIL", "test@example.com");
    std::env::set_var("SHIPROCKET_PASSWORD", "test-password");
    let _ = std::env::var("SHIPROCKET_PICKUP_LOCATION")
        .expect("SHIPROCKET_PICKUP_LOCATION must be set in env/.env for logistics tests");
    std::env::set_var("SHIPROCKET_PICKUP_POSTCODE", "560001");
    std::env::set_var(
        "SHIPROCKET_API_BASE",
        format!("http://127.0.0.1:{port}/v1/external"),
    );
    std::env::set_var("RAZORPAY_API_BASE", format!("http://127.0.0.1:{port}/v1"));
    std::env::set_var("RAZORPAY_KEY_ID", "test_key");
    std::env::set_var("RAZORPAY_KEY_SECRET", "test_secret");

    let unique_tag = tag ^ i64::from(port);
    let txn = db.begin().await.expect("begin");
    let (user_id, shipping_address_id, inventory_marker) =
        seed_checkout_user(&txn, unique_tag).await;
    let cart_item = core_operations::handlers::cart::get_cart_items(
        &txn,
        Request::new(proto::proto::core::GetCartItemsRequest {
            user_id: Some(user_id),
            session_id: None,
        }),
    )
    .await
    .expect("cart")
    .into_inner()
    .items[0]
        .clone();

    let order = place_order(
        &txn,
        Request::new(PlaceOrderRequest {
            shipping_address_id,
            user_id,
            coupon_code: None,
            selected_cart_ids: vec![cart_item.cart_id],
        }),
    )
    .await
    .expect("place order")
    .into_inner()
    .items[0]
        .clone();

    let intent = payment_intents::Entity::find()
        .filter(payment_intents::Column::OrderId.eq(order.order_id))
        .one(&txn)
        .await
        .expect("query intent")
        .expect("intent");
    txn.commit().await.expect("commit place");

    let payment_id = format!("pay_logistics_{tag}");
    let signature = compute_signature(&intent.razorpay_order_id, &payment_id, "test_secret");
    let verify_txn = db.begin().await.expect("verify txn");
    verify_razorpay_payment(
        &verify_txn,
        Request::new(VerifyRazorpayPaymentRequest {
            order_id: order.order_id,
            razorpay_order_id: intent.razorpay_order_id.clone(),
            razorpay_payment_id: payment_id,
            razorpay_signature: signature,
        }),
    )
    .await
    .expect("verify");
    verify_txn.commit().await.expect("commit verify");

    (order.order_id, user_id, inventory_marker)
}

async fn shipment_meta(db: &sea_orm::DatabaseConnection, order_id: i64) -> sea_orm::QueryResult {
    let txn = db.begin().await.expect("begin shipment meta");
    let row = txn
        .query_one(Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            r#"SELECT shipment_id, shiprocket_order_id, shiprocket_external_order_id, awb_code,
                      selected_courier_id, pickup_scheduled_for, logistics_status,
                      can_customer_cancel, razorpay_refund_id, refund_status
               FROM Shipments WHERE order_id = ? ORDER BY shipment_id DESC LIMIT 1"#,
            [order_id.into()],
        ))
        .await
        .expect("query shipment")
        .expect("shipment row");
    txn.rollback().await.ok();
    row
}

async fn order_status_name(db: &sea_orm::DatabaseConnection, order_id: i64) -> String {
    let txn = db.begin().await.expect("begin status");
    let order = orders::Entity::find_by_id(order_id)
        .one(&txn)
        .await
        .expect("query order")
        .expect("order");
    let status = order_status::Entity::find_by_id(order.status_id)
        .one(&txn)
        .await
        .expect("query status")
        .expect("status");
    txn.rollback().await.ok();
    status.status_name
}

async fn inventory_available(db: &sea_orm::DatabaseConnection, inventory_id: i64) -> i64 {
    let txn = db.begin().await.expect("begin inventory");
    let row = inventory::Entity::find_by_id(inventory_id)
        .one(&txn)
        .await
        .expect("query inventory")
        .expect("inventory");
    txn.rollback().await.ok();
    row.quantity_available.unwrap_or_default()
}

async fn refund_attempt_count(db: &sea_orm::DatabaseConnection, order_id: i64) -> i64 {
    let txn = db.begin().await.expect("begin refund attempt count");
    let row = txn
        .query_one(Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            r#"SELECT COUNT(*) AS count FROM RefundAttempts WHERE order_id = ?"#,
            [order_id.into()],
        ))
        .await
        .expect("query refund attempts")
        .expect("count row");
    txn.rollback().await.ok();
    row.try_get("", "count").expect("count")
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and mocked local APIs"]
async fn integration_payment_success_auto_books_shiprocket_and_is_idempotent() {
    let state = Arc::new(Mutex::new(MockState::default()));
    let _server = spawn_mock_server(18101, state.clone()).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    let tag = unique_tag(18101);
    let (order_id, _user_id, _inventory_id) = place_and_pay_order(&db, 18101, tag).await;

    let verify_replay = db.begin().await.expect("replay txn");
    let intent = payment_intents::Entity::find()
        .filter(payment_intents::Column::OrderId.eq(order_id))
        .one(&verify_replay)
        .await
        .expect("intent")
        .expect("intent");
    let payment_id = format!("pay_logistics_{tag}");
    let signature = compute_signature(&intent.razorpay_order_id, &payment_id, "test_secret");
    verify_razorpay_payment(
        &verify_replay,
        Request::new(VerifyRazorpayPaymentRequest {
            order_id,
            razorpay_order_id: intent.razorpay_order_id.clone(),
            razorpay_payment_id: payment_id,
            razorpay_signature: signature,
        }),
    )
    .await
    .expect("verify replay");
    verify_replay.commit().await.expect("commit replay");

    let shipment = shipment_meta(&db, order_id).await;
    let selected_courier_id: i64 = shipment
        .try_get("", "selected_courier_id")
        .expect("courier");
    let logistics_status: String = shipment.try_get("", "logistics_status").expect("status");
    let awb_code: String = shipment.try_get("", "awb_code").expect("awb");
    let pickup_scheduled_for: chrono::DateTime<Utc> = shipment
        .try_get("", "pickup_scheduled_for")
        .expect("pickup");
    assert_eq!(selected_courier_id, 777);
    assert_eq!(logistics_status, "pickup_scheduled");
    assert!(awb_code.starts_with("AWB_"));
    let delta = pickup_scheduled_for - Utc::now();
    assert!(delta.num_hours() >= 47 && delta.num_hours() <= 48);

    let channel_ref = {
        let guard = state.lock().expect("lock");
        assert_eq!(guard.create_order_calls, 1);
        assert_eq!(guard.pickup_calls, 1);
        assert_eq!(guard.assigned_courier_ids, vec![777]);
        guard
            .last_channel_order_ids
            .last()
            .expect("create adhoc should include order_id")
            .clone()
    };
    let row = orders::Entity::find_by_id(order_id)
        .one(&db)
        .await
        .expect("load order")
        .expect("order row");
    assert_eq!(channel_ref, row.public_order_ref);
    assert!(channel_ref.starts_with("SUD-"));
    let parts: Vec<&str> = channel_ref.split('-').collect();
    assert_eq!(
        parts.len(),
        3,
        "expected SUD-YYYYMMDD-SUFFIX, got {channel_ref}"
    );
    assert_eq!(parts[0], "SUD");
    assert_eq!(parts[1].len(), 8);
    assert!(parts[1].chars().all(|c| c.is_ascii_digit()));
    assert_eq!(parts[2].len(), 10);
    assert!(parts[2]
        .chars()
        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()));
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and mocked local APIs"]
async fn integration_pre_pickup_cancel_restores_stock_and_refunds_once() {
    let state = Arc::new(Mutex::new(MockState::default()));
    let _server = spawn_mock_server(18102, state.clone()).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    let tag = unique_tag(18102);
    let (order_id, user_id, inventory_marker) = place_and_pay_order(&db, 18102, tag).await;

    let cancel_txn = db.begin().await.expect("cancel txn");
    delete_order(
        &cancel_txn,
        Request::new(DeleteOrderRequest {
            order_id,
            acting_user_id: Some(user_id),
        }),
    )
    .await
    .expect("cancel order");
    cancel_txn.commit().await.expect("commit cancel");

    let replay_txn = db.begin().await.expect("replay txn");
    let _ = delete_order(
        &replay_txn,
        Request::new(DeleteOrderRequest {
            order_id,
            acting_user_id: Some(user_id),
        }),
    )
    .await
    .expect("idempotent replay");
    replay_txn.commit().await.expect("commit replay");

    let shipment = shipment_meta(&db, order_id).await;
    let refund_id: String = shipment
        .try_get("", "razorpay_refund_id")
        .expect("refund id");
    assert!(refund_id.starts_with("rfnd_logistics_"));
    assert_eq!(order_status_name(&db, order_id).await, "refunded");
    assert_eq!(inventory_available(&db, inventory_marker).await, 6);
    let guard = state.lock().expect("lock");
    assert_eq!(guard.cancel_calls, 1);
    assert_eq!(guard.refund_calls, 1);
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and mocked local APIs"]
async fn integration_customer_cancel_and_webhook_cancel_race_refunds_once() {
    let state = Arc::new(Mutex::new(MockState::default()));
    let _server = spawn_mock_server(18106, state.clone()).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    let tag = unique_tag(18106);
    let (order_id, user_id, _inventory_marker) = place_and_pay_order(&db, 18106, tag).await;
    let shipment = shipment_meta(&db, order_id).await;
    let shiprocket_order_id: String = shipment
        .try_get("", "shiprocket_order_id")
        .expect("ship id");

    let cancel_fut = async {
        let cancel_txn = db.begin().await.expect("cancel txn");
        let res = delete_order(
            &cancel_txn,
            Request::new(DeleteOrderRequest {
                order_id,
                acting_user_id: Some(user_id),
            }),
        )
        .await;
        cancel_txn.commit().await.expect("commit cancel");
        res
    };

    let webhook_fut = async {
        tokio::time::sleep(Duration::from_millis(10)).await;
        let webhook_txn = db.begin().await.expect("webhook txn");
        let res = ingest_webhook(
            &webhook_txn,
            Request::new(IngestWebhookRequest {
                provider: "shiprocket".to_string(),
                event_type: "shiprocket.update".to_string(),
                webhook_id: format!("race_cancel_{tag}"),
                payload_json: json!({
                    "shipment_id": shiprocket_order_id,
                    "shipment_status_id": 8,
                    "shipment_status": "CANCELLED"
                })
                .to_string(),
                signature_verified: true,
                provider_event_id: None,
            }),
        )
        .await;
        webhook_txn.commit().await.expect("commit webhook");
        res
    };

    let (cancel_res, webhook_res) = tokio::join!(cancel_fut, webhook_fut);
    assert!(
        cancel_res.is_ok(),
        "customer cancel path should converge: {cancel_res:?}"
    );
    assert!(
        webhook_res.is_ok(),
        "webhook cancel path should converge: {webhook_res:?}"
    );
    {
        let guard = state.lock().expect("lock");
        assert_eq!(
            guard.refund_calls, 1,
            "race should produce only one outbound refund call"
        );
    }
    assert_eq!(
        refund_attempt_count(&db, order_id).await,
        1,
        "race should produce only one active refund attempt row"
    );
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and mocked local APIs"]
async fn integration_failed_shiprocket_cancel_moves_to_cancel_pending_without_refund() {
    let state = Arc::new(Mutex::new(MockState {
        cancel_should_fail: true,
        ..MockState::default()
    }));
    let _server = spawn_mock_server(18103, state.clone()).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    let tag = unique_tag(18103);
    let (order_id, user_id, inventory_marker) = place_and_pay_order(&db, 18103, tag).await;

    let cancel_txn = db.begin().await.expect("cancel txn");
    let err = delete_order(
        &cancel_txn,
        Request::new(DeleteOrderRequest {
            order_id,
            acting_user_id: Some(user_id),
        }),
    )
    .await
    .expect_err("cancel should fail");
    assert_eq!(err.code(), tonic::Code::Unavailable);
    cancel_txn.commit().await.expect("commit cancel pending");

    assert_eq!(
        order_status_name(&db, order_id).await,
        "cancel_pending_logistics"
    );
    assert_eq!(inventory_available(&db, inventory_marker).await, 4);
    let guard = state.lock().expect("lock");
    assert_eq!(guard.cancel_calls, 1);
    assert_eq!(guard.refund_calls, 0);
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and mocked local APIs"]
async fn integration_pickup_completed_disables_customer_cancellation() {
    let state = Arc::new(Mutex::new(MockState::default()));
    let _server = spawn_mock_server(18104, state.clone()).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    let tag = unique_tag(18104);
    let (order_id, user_id, _inventory_marker) = place_and_pay_order(&db, 18104, tag).await;
    let shipment = shipment_meta(&db, order_id).await;
    let shiprocket_order_id: String = shipment
        .try_get("", "shiprocket_order_id")
        .expect("ship id");

    let webhook_txn = db.begin().await.expect("webhook txn");
    ingest_webhook(
        &webhook_txn,
        Request::new(IngestWebhookRequest {
            provider: "shiprocket".to_string(),
            event_type: "shiprocket.update".to_string(),
            webhook_id: format!("pickup_{tag}"),
            payload_json: json!({
                "shipment_id": shiprocket_order_id,
                "shipment_status_id": 42,
                "shipment_status": "Pickup Completed"
            })
            .to_string(),
            signature_verified: true,
            provider_event_id: None,
        }),
    )
    .await
    .expect("webhook");
    webhook_txn.commit().await.expect("commit webhook");

    let cancel_txn = db.begin().await.expect("cancel txn");
    let err = delete_order(
        &cancel_txn,
        Request::new(DeleteOrderRequest {
            order_id,
            acting_user_id: Some(user_id),
        }),
    )
    .await
    .expect_err("cancel should be blocked");
    assert_eq!(err.code(), tonic::Code::FailedPrecondition);
    cancel_txn.rollback().await.ok();
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and mocked local APIs"]
async fn integration_rto_terminal_webhook_refunds_once() {
    let state = Arc::new(Mutex::new(MockState::default()));
    let _server = spawn_mock_server(18105, state.clone()).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    let tag = unique_tag(18105);
    let (order_id, _user_id, inventory_marker) = place_and_pay_order(&db, 18105, tag).await;
    let shipment = shipment_meta(&db, order_id).await;
    let shiprocket_order_id: String = shipment
        .try_get("", "shiprocket_order_id")
        .expect("ship id");

    for attempt in 0..2 {
        let webhook_txn = db.begin().await.expect("webhook txn");
        ingest_webhook(
            &webhook_txn,
            Request::new(IngestWebhookRequest {
                provider: "shiprocket".to_string(),
                event_type: "shiprocket.update".to_string(),
                webhook_id: format!("rto_{tag}_{attempt}"),
                payload_json: json!({
                    "shipment_id": shiprocket_order_id,
                    "shipment_status_id": 10,
                    "shipment_status": "RTO Delivered"
                })
                .to_string(),
                signature_verified: true,
                provider_event_id: None,
            }),
        )
        .await
        .expect("webhook");
        webhook_txn.commit().await.expect("commit webhook");
    }

    let shipment = shipment_meta(&db, order_id).await;
    let refund_id: String = shipment
        .try_get("", "razorpay_refund_id")
        .expect("refund id");
    assert!(refund_id.starts_with("rfnd_logistics_"));
    assert_eq!(inventory_available(&db, inventory_marker).await, 6);
    let guard = state.lock().expect("lock");
    assert_eq!(guard.refund_calls, 1);
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and mocked local APIs"]
async fn integration_refund_retry_reuses_same_idempotency_key() {
    let state = Arc::new(Mutex::new(MockState {
        refund_should_fail: true,
        ..MockState::default()
    }));
    let _server = spawn_mock_server(18107, state.clone()).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    let tag = unique_tag(18107);
    let (order_id, user_id, _inventory_marker) = place_and_pay_order(&db, 18107, tag).await;

    let first_cancel_txn = db.begin().await.expect("cancel txn");
    delete_order(
        &first_cancel_txn,
        Request::new(DeleteOrderRequest {
            order_id,
            acting_user_id: Some(user_id),
        }),
    )
    .await
    .expect("first cancel should converge with refund_failed");
    first_cancel_txn.commit().await.expect("commit cancel");

    {
        let mut guard = state.lock().expect("lock");
        guard.refund_should_fail = false;
    }

    let shipment = shipment_meta(&db, order_id).await;
    let shiprocket_order_id: String = shipment
        .try_get("", "shiprocket_order_id")
        .expect("ship id");
    let webhook_txn = db.begin().await.expect("webhook txn");
    ingest_webhook(
        &webhook_txn,
        Request::new(IngestWebhookRequest {
            provider: "shiprocket".to_string(),
            event_type: "shiprocket.update".to_string(),
            webhook_id: format!("retry_cancel_{tag}"),
            payload_json: json!({
                "shipment_id": shiprocket_order_id,
                "shipment_status_id": 8,
                "shipment_status": "CANCELLED"
            })
            .to_string(),
            signature_verified: true,
            provider_event_id: None,
        }),
    )
    .await
    .expect("webhook retry should converge");
    webhook_txn.commit().await.expect("commit webhook");

    let guard = state.lock().expect("lock");
    assert_eq!(
        guard.refund_calls, 1,
        "second trigger should skip when an attempt already exists"
    );
    assert_eq!(
        guard.refund_idempotency_keys.len(),
        1,
        "only one outbound refund call should carry idempotency key"
    );
    assert_eq!(
        guard.refund_idempotency_keys[0],
        format!("refund_{order_id}_pay_logistics_{tag}")
    );
}
