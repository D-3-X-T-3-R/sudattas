//! DB-backed logistics fulfillment tests with mocked Shiprocket + Razorpay APIs.

mod integration_common;
mod provider_test_gate;

use chrono::Utc;
use core_db_entities::entity::sea_orm_active_enums::AuthProvider;
use core_db_entities::entity::{
    inventory, order_status, orders, payment_intents, product_categories, product_variants,
    products, shipping_addresses, user_roles, users,
};
use core_operations::handlers::orders::{
    admin_mark_order_shipped, cancel_order_items, delete_order, update_pickup_target,
};
use core_operations::handlers::payment_intents::verify_razorpay_payment;
use core_operations::handlers::shipments::{
    book_order_after_validation, cancel_order_via_logistics, create_shipment,
    process_booking_intents_batch,
};
use core_operations::handlers::webhooks::ingest_webhook;
use core_operations::procedures::cancel_pending_logistics::process_cancel_pending_logistics;
use core_operations::procedures::create_shipments_after_cancel_window::process_create_shipments_after_cancel_window;
use core_operations::procedures::orders::place_order;
use core_operations::procedures::refund_attempts_worker::process_refund_attempts;
use hmac::{Hmac, Mac};
use integration_common::test_db_url;
use proto::proto::core::{
    AdminMarkOrderShippedRequest, CancelOrderItemsRequest, CreateCartItemRequest,
    CreateShipmentRequest, DeleteOrderRequest, IngestWebhookRequest, PlaceOrderRequest,
    UpdatePickupTargetRequest, VerifyRazorpayPaymentRequest,
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
    cancel_order_ids: Vec<i64>,
    refund_calls: usize,
    refund_amounts: Vec<i64>,
    refund_idempotency_keys: Vec<String>,
    assigned_courier_ids: Vec<i64>,
    create_order_item_counts: Vec<usize>,
    scheduled_pickup_dates: Vec<String>,
    cancel_should_fail: bool,
    refund_should_fail: bool,
}

fn refund_idempotency_key(order_id: i64, payment_id: &str, target_minor: i64) -> String {
    format!("refund_{order_id}_{payment_id}_{target_minor}")
}

fn order_scoped_refund_call_count(state: &MockState, order_id: i64) -> usize {
    let prefix = format!("refund_{order_id}_");
    state
        .refund_idempotency_keys
        .iter()
        .filter(|key| key.starts_with(&prefix))
        .count()
}

fn order_scoped_refund_amounts(state: &MockState, order_id: i64) -> Vec<i64> {
    let prefix = format!("refund_{order_id}_");
    state
        .refund_idempotency_keys
        .iter()
        .enumerate()
        .filter_map(|(idx, key)| {
            if key.starts_with(&prefix) {
                state.refund_amounts.get(idx).copied()
            } else {
                None
            }
        })
        .collect()
}

fn order_scoped_refund_idempotency_keys(state: &MockState, order_id: i64) -> Vec<String> {
    let prefix = format!("refund_{order_id}_");
    state
        .refund_idempotency_keys
        .iter()
        .filter(|key| key.starts_with(&prefix))
        .cloned()
        .collect()
}

fn channel_order_scoped_booking_call_count(state: &MockState, channel_order_id: &str) -> usize {
    state
        .last_channel_order_ids
        .iter()
        .filter(|id| id.as_str() == channel_order_id)
        .count()
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
            let item_count = body
                .get("order_items")
                .and_then(|v| v.as_array())
                .map(|v| v.len())
                .unwrap_or(0);
            guard.create_order_item_counts.push(item_count);
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
        .and(warp::body::json())
        .and(with_state.clone())
        .map(|body: serde_json::Value, state: Arc<Mutex<MockState>>| {
            let mut guard = state.lock().expect("lock");
            guard.cancel_calls += 1;
            if let Some(ids) = body.get("ids").and_then(|x| x.as_array()) {
                for id in ids {
                    if let Some(parsed) = id.as_i64() {
                        guard.cancel_order_ids.push(parsed);
                    }
                }
            }
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
        .and(warp::body::json())
        .and(warp::header::optional::<String>("x-refund-idempotency"))
        .and(with_state.clone())
        .map(
            |_payment_id: String,
             body: serde_json::Value,
             idempotency: Option<String>,
             state: Arc<Mutex<MockState>>| {
                let mut guard = state.lock().expect("lock");
                guard.refund_calls += 1;
                if let Some(amount) = body.get("amount").and_then(|v| v.as_i64()) {
                    guard.refund_amounts.push(amount);
                }
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
        first_name: ActiveValue::NotSet,
        last_name: ActiveValue::NotSet,
        gender: ActiveValue::NotSet,
        date_of_birth: ActiveValue::NotSet,
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

fn configure_mock_provider_env(port: u16) {
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
}

async fn place_order_without_payment_verification(
    db: &sea_orm::DatabaseConnection,
    port: u16,
    tag: i64,
    payment_mode: Option<&str>,
) -> (i64, i64, i64) {
    configure_mock_provider_env(port);

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
            payment_mode: payment_mode.map(|m| m.to_string()),
        }),
    )
    .await
    .expect("place order")
    .into_inner()
    .items[0]
        .clone();
    txn.commit().await.expect("commit place");

    (order.order_id, user_id, inventory_marker)
}

async fn place_and_pay_order(
    db: &sea_orm::DatabaseConnection,
    port: u16,
    tag: i64,
) -> (i64, i64, i64) {
    let (order_id, user_id, inventory_marker) =
        place_order_without_payment_verification(db, port, tag, Some("prepaid")).await;
    let txn = db.begin().await.expect("begin intent read");
    let intent = payment_intents::Entity::find()
        .filter(payment_intents::Column::OrderId.eq(order_id))
        .one(&txn)
        .await
        .expect("query intent")
        .expect("intent");
    txn.rollback().await.ok();

    let payment_id = format!("pay_logistics_{tag}");
    let signature = compute_signature(&intent.razorpay_order_id, &payment_id, "test_secret");
    let verify_txn = db.begin().await.expect("verify txn");
    verify_razorpay_payment(
        &verify_txn,
        Request::new(VerifyRazorpayPaymentRequest {
            order_id,
            razorpay_order_id: intent.razorpay_order_id.clone(),
            razorpay_payment_id: payment_id,
            razorpay_signature: signature,
        }),
    )
    .await
    .expect("verify");
    verify_txn.commit().await.expect("commit verify");

    (order_id, user_id, inventory_marker)
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

async fn refund_attempt_status(db: &sea_orm::DatabaseConnection, order_id: i64) -> Option<String> {
    let txn = db.begin().await.expect("begin refund attempt status");
    let row = txn
        .query_one(Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            r#"SELECT status
               FROM RefundAttempts
               WHERE order_id = ?
               ORDER BY attempt_id DESC
               LIMIT 1"#,
            [order_id.into()],
        ))
        .await
        .expect("query refund attempt status");
    txn.rollback().await.ok();
    row.and_then(|r| r.try_get("", "status").ok())
}

async fn refund_row_count(db: &sea_orm::DatabaseConnection, order_id: i64) -> i64 {
    let txn = db.begin().await.expect("begin refund count");
    let row = txn
        .query_one(Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            r#"SELECT COUNT(*) AS count
               FROM Refunds
               WHERE order_id = ?"#,
            [order_id.into()],
        ))
        .await
        .expect("query refund count")
        .expect("count row");
    txn.rollback().await.ok();
    row.try_get("", "count").expect("count")
}

async fn processed_refund_total_minor(db: &sea_orm::DatabaseConnection, order_id: i64) -> i64 {
    let txn = db.begin().await.expect("begin processed refund total");
    let row = txn
        .query_one(Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            r#"SELECT CAST(COALESCE(SUM(amount_paise), 0) AS SIGNED) AS total
               FROM Refunds
               WHERE order_id = ?
                 AND status = 'processed'"#,
            [order_id.into()],
        ))
        .await
        .expect("query processed refund total")
        .expect("total row");
    txn.rollback().await.ok();
    row.try_get("", "total").expect("total")
}

#[derive(Debug, Clone)]
struct RefundAttemptSnapshot {
    status: String,
    amount_requested_paise: i64,
    amount_sent_to_gateway_paise: i64,
}

async fn latest_refund_attempt_snapshot(
    db: &sea_orm::DatabaseConnection,
    order_id: i64,
) -> Option<RefundAttemptSnapshot> {
    let txn = db.begin().await.expect("begin latest refund attempt");
    let row = txn
        .query_one(Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            r#"SELECT status,
                      amount_requested_paise,
                      amount_sent_to_gateway_paise
               FROM RefundAttempts
               WHERE order_id = ?
               ORDER BY attempt_id DESC
               LIMIT 1"#,
            [order_id.into()],
        ))
        .await
        .expect("query latest refund attempt");
    txn.rollback().await.ok();
    row.map(|r| RefundAttemptSnapshot {
        status: r.try_get("", "status").expect("status"),
        amount_requested_paise: r
            .try_get("", "amount_requested_paise")
            .expect("amount_requested_paise"),
        amount_sent_to_gateway_paise: r
            .try_get("", "amount_sent_to_gateway_paise")
            .expect("amount_sent_to_gateway_paise"),
    })
}

async fn neutralize_open_refund_attempts(db: &sea_orm::DatabaseConnection, reason: &str) {
    let txn = db.begin().await.expect("begin refund attempt cleanup");
    txn.execute(Statement::from_sql_and_values(
        sea_orm::DbBackend::MySql,
        r#"UPDATE RefundAttempts
           SET status = 'processed',
               provider_error = COALESCE(provider_error, ?),
               updated_at = UTC_TIMESTAMP()
           WHERE status IN ('pending_external', 'submitting', 'submitted')"#,
        [reason.into()],
    ))
    .await
    .expect("cleanup open refund attempts");
    txn.commit().await.expect("commit refund attempt cleanup");
}

async fn create_trigger(db: &sea_orm::DatabaseConnection, trigger_name: &str, body_sql: &str) {
    db.execute_unprepared(&format!("DROP TRIGGER IF EXISTS `{trigger_name}`"))
        .await
        .expect("drop existing trigger");
    db.execute_unprepared(body_sql)
        .await
        .expect("create trigger");
}

async fn drop_trigger(db: &sea_orm::DatabaseConnection, trigger_name: &str) {
    db.execute_unprepared(&format!("DROP TRIGGER IF EXISTS `{trigger_name}`"))
        .await
        .expect("drop trigger");
}

async fn shipment_count(db: &sea_orm::DatabaseConnection, order_id: i64) -> i64 {
    let txn = db.begin().await.expect("begin shipment count");
    let row = txn
        .query_one(Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            r#"SELECT COUNT(*) AS count FROM Shipments WHERE order_id = ?"#,
            [order_id.into()],
        ))
        .await
        .expect("query shipment count")
        .expect("count row");
    txn.rollback().await.ok();
    row.try_get("", "count").expect("count")
}

async fn order_fulfillment_status(db: &sea_orm::DatabaseConnection, order_id: i64) -> String {
    let txn = db.begin().await.expect("begin fulfillment status");
    let row = txn
        .query_one(Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            r#"SELECT fulfillment_status FROM Orders WHERE OrderID = ?"#,
            [order_id.into()],
        ))
        .await
        .expect("query fulfillment status")
        .expect("row");
    txn.rollback().await.ok();
    row.try_get("", "fulfillment_status")
        .expect("fulfillment_status")
}

async fn order_grand_total_minor(db: &sea_orm::DatabaseConnection, order_id: i64) -> i64 {
    let txn = db.begin().await.expect("begin grand total");
    let row = txn
        .query_one(Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            r#"SELECT grand_total_minor FROM Orders WHERE OrderID = ?"#,
            [order_id.into()],
        ))
        .await
        .expect("query grand total")
        .expect("row");
    txn.rollback().await.ok();
    row.try_get("", "grand_total_minor")
        .expect("grand_total_minor")
}

async fn order_payment_status(db: &sea_orm::DatabaseConnection, order_id: i64) -> String {
    let txn = db.begin().await.expect("begin payment status");
    let row = txn
        .query_one(Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            r#"SELECT payment_status FROM Orders WHERE OrderID = ?"#,
            [order_id.into()],
        ))
        .await
        .expect("query payment status")
        .expect("row");
    txn.rollback().await.ok();
    row.try_get("", "payment_status").expect("payment_status")
}

async fn order_refund_settlement_status(
    db: &sea_orm::DatabaseConnection,
    order_id: i64,
) -> Option<String> {
    let txn = db.begin().await.expect("begin refund settlement status");
    let row = txn
        .query_one(Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            r#"SELECT refund_settlement_status FROM Orders WHERE OrderID = ?"#,
            [order_id.into()],
        ))
        .await
        .expect("query refund settlement status")
        .expect("row");
    txn.rollback().await.ok();
    row.try_get("", "refund_settlement_status").ok()
}

async fn webhook_status_by_webhook_id(
    db: &sea_orm::DatabaseConnection,
    webhook_id: &str,
) -> Option<String> {
    let txn = db.begin().await.expect("begin webhook status");
    let row = txn
        .query_one(Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            r#"SELECT status
               FROM WebhookEvents
               WHERE webhook_id = ?
               ORDER BY event_id DESC
               LIMIT 1"#,
            [webhook_id.into()],
        ))
        .await
        .expect("query webhook status");
    txn.rollback().await.ok();
    row.and_then(|r| r.try_get("", "status").ok())
}

async fn latest_payment_intent_ids_for_order(
    db: &sea_orm::DatabaseConnection,
    order_id: i64,
) -> (String, Option<String>) {
    let txn = db.begin().await.expect("begin payment intent lookup");
    let row = txn
        .query_one(Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            r#"SELECT razorpay_order_id, razorpay_payment_id
               FROM PaymentIntents
               WHERE order_id = ?
               ORDER BY intent_id DESC
               LIMIT 1"#,
            [order_id.into()],
        ))
        .await
        .expect("query payment intent")
        .expect("payment intent row");
    txn.rollback().await.ok();
    (
        row.try_get("", "razorpay_order_id")
            .expect("razorpay_order_id"),
        row.try_get("", "razorpay_payment_id").ok(),
    )
}

async fn shipment_logistics_status(
    db: &sea_orm::DatabaseConnection,
    order_id: i64,
) -> Option<String> {
    let txn = db.begin().await.expect("begin shipment logistics status");
    let row = txn
        .query_one(Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            r#"SELECT logistics_status
               FROM Shipments
               WHERE order_id = ?
               ORDER BY shipment_id DESC
               LIMIT 1"#,
            [order_id.into()],
        ))
        .await
        .expect("query logistics status");
    txn.rollback().await.ok();
    row.and_then(|r| r.try_get("", "logistics_status").ok())
}

async fn make_order_eligible_for_delayed_booking(db: &sea_orm::DatabaseConnection, order_id: i64) {
    let rewind_txn = db.begin().await.expect("rewind txn");
    rewind_txn
        .execute(Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            r#"UPDATE Orders
               SET cancel_window_ends_at = UTC_TIMESTAMP() - INTERVAL 1 MINUTE,
                   earliest_booking_at = UTC_TIMESTAMP() - INTERVAL 1 MINUTE,
                   updated_at = UTC_TIMESTAMP()
               WHERE OrderID = ?"#,
            [order_id.into()],
        ))
        .await
        .expect("set booking eligibility");
    rewind_txn.commit().await.expect("commit rewind");
}

async fn order_fulfillment_timestamps(
    db: &sea_orm::DatabaseConnection,
    order_id: i64,
) -> (
    chrono::DateTime<Utc>,
    chrono::DateTime<Utc>,
    chrono::DateTime<Utc>,
    chrono::DateTime<Utc>,
) {
    let txn = db.begin().await.expect("begin timestamp query");
    let row = txn
        .query_one(Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            r#"SELECT created_at, cancel_window_ends_at, earliest_booking_at, pickup_target_at
               FROM Orders
               WHERE OrderID = ?"#,
            [order_id.into()],
        ))
        .await
        .expect("query order timestamps")
        .expect("order row");
    txn.rollback().await.ok();
    (
        row.try_get("", "created_at").expect("created_at"),
        row.try_get("", "cancel_window_ends_at")
            .expect("cancel_window_ends_at"),
        row.try_get("", "earliest_booking_at")
            .expect("earliest_booking_at"),
        row.try_get("", "pickup_target_at")
            .expect("pickup_target_at"),
    )
}

async fn ensure_delayed_shipment_booked(db: &sea_orm::DatabaseConnection, order_id: i64) {
    make_order_eligible_for_delayed_booking(db, order_id).await;
    for _ in 0..20 {
        process_create_shipments_after_cancel_window(db, 500)
            .await
            .expect("run delayed shipment worker");
        if shipment_count(db, order_id).await == 0 {
            continue;
        }

        let shipment = shipment_meta(db, order_id).await;
        let shiprocket_order_id = shipment
            .try_get::<Option<String>>("", "shiprocket_order_id")
            .ok()
            .flatten()
            .filter(|s| !s.trim().is_empty());
        let shiprocket_external_order_id = shipment
            .try_get::<Option<String>>("", "shiprocket_external_order_id")
            .ok()
            .flatten()
            .filter(|s| !s.trim().is_empty());
        let awb_code = shipment
            .try_get::<Option<String>>("", "awb_code")
            .ok()
            .flatten()
            .filter(|s| !s.trim().is_empty());
        if (shiprocket_external_order_id.is_some() || shiprocket_order_id.is_some())
            && awb_code.is_some()
        {
            return;
        }
    }
    panic!("shipment was not fully booked for order {order_id} after delayed worker retries");
}

async fn split_order_into_two_lines(db: &sea_orm::DatabaseConnection, order_id: i64) -> Vec<i64> {
    let txn = db.begin().await.expect("split txn");
    let row = txn
        .query_one(Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            r#"SELECT OrderDetailID, VariantID
               FROM OrderDetails
               WHERE OrderID = ?
               ORDER BY OrderDetailID ASC
               LIMIT 1"#,
            [order_id.into()],
        ))
        .await
        .expect("query detail")
        .expect("detail row");
    let order_detail_id: i64 = row.try_get("", "OrderDetailID").expect("detail id");
    let variant_id: i64 = row.try_get("", "VariantID").expect("variant id");

    txn.execute(Statement::from_sql_and_values(
        sea_orm::DbBackend::MySql,
        r#"UPDATE OrderDetails
           SET Quantity = 1,
               Price = 20.00,
               line_total_minor = 2000,
               unit_price_minor = 2000
           WHERE OrderDetailID = ?"#,
        [order_detail_id.into()],
    ))
    .await
    .expect("update line");

    txn.execute(Statement::from_sql_and_values(
        sea_orm::DbBackend::MySql,
        r#"INSERT INTO OrderDetails (
               OrderID, VariantID, Quantity, Price, line_total_minor, unit_price_minor,
               discount_minor, tax_minor, sku, title, line_attrs, item_status, cancelled_at
           ) VALUES (?, ?, 1, 20.00, 2000, 2000, NULL, NULL, NULL, 'Split line', NULL, 'active', NULL)"#,
        [order_id.into(), variant_id.into()],
    ))
    .await
    .expect("insert second line");

    let rows = txn
        .query_all(Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            r#"SELECT OrderDetailID
               FROM OrderDetails
               WHERE OrderID = ?
               ORDER BY OrderDetailID ASC"#,
            [order_id.into()],
        ))
        .await
        .expect("list details");
    txn.commit().await.expect("commit split");
    rows.into_iter()
        .map(|r| r.try_get("", "OrderDetailID").expect("id"))
        .collect()
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_inventory_unique_variant_constraint_blocks_duplicate_rows() {
    let db = Database::connect(&test_db_url()).await.expect("connect");
    let txn = db.begin().await.expect("begin");
    let tag = unique_tag(18127);
    let (_user_id, _shipping_address_id, inventory_marker) = seed_checkout_user(&txn, tag).await;

    let inventory_row = inventory::Entity::find_by_id(inventory_marker)
        .one(&txn)
        .await
        .expect("query inventory row")
        .expect("inventory row");
    let variant_id = inventory_row.variant_id.expect("variant_id must be set");

    let duplicate_insert = inventory::ActiveModel {
        inventory_id: ActiveValue::NotSet,
        variant_id: ActiveValue::Set(Some(variant_id)),
        quantity_available: ActiveValue::Set(Some(1)),
        quantity_reserved: ActiveValue::Set(Some(0)),
        reorder_level: ActiveValue::Set(None),
        updated_at: ActiveValue::Set(Some(Utc::now())),
    }
    .insert(&txn)
    .await;
    assert!(
        duplicate_insert.is_err(),
        "unique(VariantID) must reject duplicate inventory rows"
    );

    txn.rollback().await.ok();
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_place_order_fails_when_inventory_row_is_missing() {
    let state = Arc::new(Mutex::new(MockState::default()));
    let _server = spawn_mock_server(18128, state).await;
    configure_mock_provider_env(18128);
    let db = Database::connect(&test_db_url()).await.expect("connect");
    let txn = db.begin().await.expect("begin");
    let tag = unique_tag(18128);
    let (user_id, shipping_address_id, inventory_marker) = seed_checkout_user(&txn, tag).await;
    let cart_item = core_operations::handlers::cart::get_cart_items(
        &txn,
        Request::new(proto::proto::core::GetCartItemsRequest {
            user_id: Some(user_id),
            session_id: None,
        }),
    )
    .await
    .expect("get cart")
    .into_inner()
    .items
    .into_iter()
    .next()
    .expect("cart item");

    txn.execute(Statement::from_sql_and_values(
        sea_orm::DbBackend::MySql,
        "DELETE FROM Inventory WHERE InventoryID = ?",
        [inventory_marker.into()],
    ))
    .await
    .expect("delete inventory row");

    let err = place_order(
        &txn,
        Request::new(PlaceOrderRequest {
            shipping_address_id,
            user_id,
            coupon_code: None,
            selected_cart_ids: vec![cart_item.cart_id],
            payment_mode: Some("cod".to_string()),
        }),
    )
    .await
    .expect_err("reserve must fail when inventory row is missing");
    assert_eq!(err.code(), tonic::Code::FailedPrecondition);
    assert!(
        err.message().contains("No inventory row exists"),
        "expected missing inventory row error, got: {}",
        err.message()
    );

    txn.rollback().await.ok();
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and mocked local APIs"]
async fn integration_cancel_fails_when_restore_inventory_row_is_missing() {
    let state = Arc::new(Mutex::new(MockState::default()));
    let _server = spawn_mock_server(18129, state).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    let tag = unique_tag(18129);
    let (order_id, user_id, inventory_marker) =
        place_order_without_payment_verification(&db, 18129, tag, Some("cod")).await;

    let delete_inv_txn = db.begin().await.expect("delete inventory txn");
    delete_inv_txn
        .execute(Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            "DELETE FROM Inventory WHERE InventoryID = ?",
            [inventory_marker.into()],
        ))
        .await
        .expect("delete inventory row");
    delete_inv_txn
        .commit()
        .await
        .expect("commit inventory delete");

    let cancel_txn = db.begin().await.expect("cancel txn");
    let err = delete_order(
        &cancel_txn,
        Request::new(DeleteOrderRequest {
            order_id,
            acting_user_id: Some(user_id),
        }),
    )
    .await
    .expect_err("cancel must fail when inventory restore row is missing");
    assert_eq!(err.code(), tonic::Code::FailedPrecondition);
    assert!(
        err.message()
            .contains("No inventory row exists for variant"),
        "expected missing inventory restore row error, got: {}",
        err.message()
    );
    cancel_txn.rollback().await.ok();
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and mocked local APIs"]
async fn integration_refund_attempt_persisted_before_external_call_and_processed_post_commit() {
    let state = Arc::new(Mutex::new(MockState::default()));
    let _server = spawn_mock_server(18121, state.clone()).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    neutralize_open_refund_attempts(
        &db,
        "itest cleanup: isolate refund worker before durable-attempt flow",
    )
    .await;
    let tag = unique_tag(18121);
    let (order_id, user_id, _inventory_marker) = place_and_pay_order(&db, 18121, tag).await;
    let frozen_order_total = order_grand_total_minor(&db, order_id).await;

    let cancel_txn = db.begin().await.expect("cancel txn");
    delete_order(
        &cancel_txn,
        Request::new(DeleteOrderRequest {
            order_id,
            acting_user_id: Some(user_id),
        }),
    )
    .await
    .expect("cancellation should queue refund attempt");

    let pending_attempts_row = cancel_txn
        .query_one(Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            "SELECT COUNT(*) AS count FROM RefundAttempts WHERE order_id = ?",
            [order_id.into()],
        ))
        .await
        .expect("query pending attempts in txn")
        .expect("count row");
    let pending_attempts: i64 = pending_attempts_row.try_get("", "count").expect("count");
    assert_eq!(
        pending_attempts, 1,
        "refund attempt must exist before commit"
    );
    assert_eq!(
        state.lock().expect("lock").refund_calls,
        0,
        "no gateway refund call should happen before durable commit"
    );

    cancel_txn.commit().await.expect("commit cancel");

    assert_eq!(refund_attempt_count(&db, order_id).await, 1);
    assert_eq!(
        refund_attempt_status(&db, order_id).await.as_deref(),
        Some("pending_external")
    );
    assert_eq!(
        state.lock().expect("lock").refund_calls,
        0,
        "refund worker not run yet; gateway call count must stay 0"
    );

    process_refund_attempts(&db, 25)
        .await
        .expect("run refund worker");
    let expected_idempotency_key = refund_idempotency_key(
        order_id,
        &format!("pay_logistics_{tag}"),
        frozen_order_total,
    );
    {
        let guard = state.lock().expect("lock");
        assert_eq!(
            order_scoped_refund_call_count(&guard, order_id),
            1,
            "refund worker should issue exactly one gateway refund call for this order"
        );
        assert_eq!(
            order_scoped_refund_idempotency_keys(&guard, order_id),
            vec![expected_idempotency_key],
            "worker should use deterministic order-scoped idempotency key"
        );
    }
    assert_eq!(
        processed_refund_total_minor(&db, order_id).await,
        frozen_order_total,
        "worker should persist full refund amount for a fully cancelled prepaid order"
    );
    assert_eq!(
        refund_attempt_status(&db, order_id).await.as_deref(),
        Some("processed")
    );
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and mocked local APIs"]
async fn integration_cod_cancellation_creates_no_refund_attempt_even_with_worker() {
    let state = Arc::new(Mutex::new(MockState::default()));
    let _server = spawn_mock_server(18122, state.clone()).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    let cleanup_txn = db.begin().await.expect("cleanup txn");
    cleanup_txn
        .execute(Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            r#"UPDATE RefundAttempts
               SET status = 'processed',
                   provider_error = COALESCE(provider_error, 'itest cleanup: already open before COD worker assertion')
               WHERE status IN ('pending_external', 'submitting', 'submitted')"#,
            Vec::<sea_orm::Value>::new(),
        ))
        .await
        .expect("cleanup open refund attempts");
    cleanup_txn.commit().await.expect("commit cleanup");
    let tag = unique_tag(18122);
    let (order_id, user_id, _inventory_marker) =
        place_order_without_payment_verification(&db, 18122, tag, Some("cod")).await;

    let cancel_txn = db.begin().await.expect("cancel txn");
    delete_order(
        &cancel_txn,
        Request::new(DeleteOrderRequest {
            order_id,
            acting_user_id: Some(user_id),
        }),
    )
    .await
    .expect("COD cancellation should succeed");
    cancel_txn.commit().await.expect("commit cancel");

    assert_eq!(refund_attempt_count(&db, order_id).await, 0);
    let refund_calls_before_worker = state.lock().expect("lock").refund_calls;
    process_refund_attempts(&db, 25)
        .await
        .expect("run refund worker");
    assert_eq!(
        state.lock().expect("lock").refund_calls,
        refund_calls_before_worker,
        "COD flow must not issue gateway refunds"
    );
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and mocked local APIs"]
async fn integration_refund_worker_persistence_retry_does_not_duplicate_gateway_call() {
    let state = Arc::new(Mutex::new(MockState::default()));
    let _server = spawn_mock_server(18123, state.clone()).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    neutralize_open_refund_attempts(
        &db,
        "itest cleanup: isolate refund worker before persistence-retry flow",
    )
    .await;
    let tag = unique_tag(18123);
    let (order_id, user_id, _inventory_marker) = place_and_pay_order(&db, 18123, tag).await;
    let frozen_order_total = order_grand_total_minor(&db, order_id).await;

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

    let trigger_name = format!("trg_itest_refund_insert_fail_{}", unique_tag(18123));
    create_trigger(
        &db,
        &trigger_name,
        &format!(
            r#"CREATE TRIGGER `{trigger_name}`
BEFORE INSERT ON `Refunds`
FOR EACH ROW
BEGIN
    IF NEW.order_id = {order_id} THEN
        SIGNAL SQLSTATE '45000'
            SET MESSAGE_TEXT = 'itest forced refund persistence failure';
    END IF;
END"#
        ),
    )
    .await;

    process_refund_attempts(&db, 25)
        .await
        .expect("refund worker run with forced persistence failure");
    let expected_idempotency_key = refund_idempotency_key(
        order_id,
        &format!("pay_logistics_{tag}"),
        frozen_order_total,
    );
    {
        let guard = state.lock().expect("lock");
        assert_eq!(
            order_scoped_refund_call_count(&guard, order_id),
            1,
            "gateway call should have happened exactly once for this order"
        );
        assert_eq!(
            order_scoped_refund_idempotency_keys(&guard, order_id),
            vec![expected_idempotency_key.clone()],
            "first worker run should call gateway with deterministic idempotency key"
        );
    }
    assert_eq!(
        refund_attempt_status(&db, order_id).await.as_deref(),
        Some("submitted"),
        "attempt should remain submitted for persistence retry"
    );
    assert_eq!(refund_row_count(&db, order_id).await, 0);

    drop_trigger(&db, &trigger_name).await;

    process_refund_attempts(&db, 25)
        .await
        .expect("retry refund persistence");
    {
        let guard = state.lock().expect("lock");
        assert_eq!(
            order_scoped_refund_call_count(&guard, order_id),
            1,
            "retry should reconcile persisted result without a second gateway call for this order"
        );
        assert_eq!(
            order_scoped_refund_idempotency_keys(&guard, order_id),
            vec![expected_idempotency_key],
            "retry must preserve the original idempotency key"
        );
    }
    assert_eq!(
        processed_refund_total_minor(&db, order_id).await,
        frozen_order_total,
        "once persistence recovers, full refunded amount should match frozen order total"
    );
    assert_eq!(refund_row_count(&db, order_id).await, 1);
    assert_eq!(
        refund_attempt_status(&db, order_id).await.as_deref(),
        Some("processed")
    );
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and mocked local APIs"]
async fn integration_booking_intent_is_persisted_before_external_call_and_worker_is_idempotent() {
    let state = Arc::new(Mutex::new(MockState::default()));
    let _server = spawn_mock_server(18124, state.clone()).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    let tag = unique_tag(18124);
    let (order_id, _user_id, _inventory_marker) = place_and_pay_order(&db, 18124, tag).await;

    make_order_eligible_for_delayed_booking(&db, order_id).await;

    let txn = db.begin().await.expect("booking intent txn");
    let _shipment_id =
        book_order_after_validation(&txn, order_id, Utc::now(), "itest_booking_intent")
            .await
            .expect("persist booking intent");
    let row = txn
        .query_one(Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            r#"SELECT logistics_status, shiprocket_order_id, awb_code
               FROM Shipments
               WHERE order_id = ?
               ORDER BY shipment_id DESC
               LIMIT 1"#,
            [order_id.into()],
        ))
        .await
        .expect("query shipment intent row")
        .expect("shipment row");
    let logistics_status: String = row
        .try_get("", "logistics_status")
        .expect("logistics_status");
    let shiprocket_order_id: Option<String> = row.try_get("", "shiprocket_order_id").ok();
    let awb_code: Option<String> = row.try_get("", "awb_code").ok();
    assert_eq!(logistics_status, "booking_pending");
    assert!(shiprocket_order_id.is_none());
    assert!(awb_code.is_none());
    assert_eq!(
        state.lock().expect("lock").create_order_calls,
        0,
        "external booking must not be called before commit"
    );
    txn.commit().await.expect("commit booking intent");

    let channel_order_id = orders::Entity::find_by_id(order_id)
        .one(&db)
        .await
        .expect("load order")
        .expect("order row")
        .public_order_ref;
    let create_calls_before = {
        let guard = state.lock().expect("lock");
        channel_order_scoped_booking_call_count(&guard, &channel_order_id)
    };
    process_booking_intents_batch(&db, 25)
        .await
        .expect("run booking worker");
    let create_calls_after_first_batch = {
        let guard = state.lock().expect("lock");
        channel_order_scoped_booking_call_count(&guard, &channel_order_id)
    };
    assert_eq!(
        create_calls_after_first_batch - create_calls_before,
        1,
        "worker should perform one external booking call for this order"
    );

    process_booking_intents_batch(&db, 25)
        .await
        .expect("run booking worker again");
    let create_calls_after_second_batch = {
        let guard = state.lock().expect("lock");
        channel_order_scoped_booking_call_count(&guard, &channel_order_id)
    };
    assert_eq!(
        create_calls_after_second_batch - create_calls_after_first_batch,
        0,
        "duplicate worker run must not create duplicate shiprocket orders"
    );
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and mocked local APIs"]
async fn integration_booking_worker_failure_before_external_call_creates_no_shiprocket_order() {
    let state = Arc::new(Mutex::new(MockState::default()));
    let _server = spawn_mock_server(18125, state.clone()).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    let tag = unique_tag(18125);
    let (order_id, _user_id, _inventory_marker) = place_and_pay_order(&db, 18125, tag).await;
    make_order_eligible_for_delayed_booking(&db, order_id).await;

    let claim_txn = db.begin().await.expect("claim txn");
    book_order_after_validation(&claim_txn, order_id, Utc::now(), "itest_booking_claim")
        .await
        .expect("persist booking intent");
    claim_txn.commit().await.expect("commit claim");

    let trigger_name = format!("trg_itest_booking_pre_external_fail_{}", unique_tag(18125));
    create_trigger(
        &db,
        &trigger_name,
        &format!(
            r#"CREATE TRIGGER `{trigger_name}`
BEFORE UPDATE ON `Shipments`
FOR EACH ROW
BEGIN
    IF NEW.order_id = {order_id} AND NEW.logistics_status = 'booking_in_progress' THEN
        SIGNAL SQLSTATE '45000'
            SET MESSAGE_TEXT = 'itest forced booking pre-external failure';
    END IF;
END"#
        ),
    )
    .await;

    process_booking_intents_batch(&db, 25)
        .await
        .expect("booking batch run with forced failure");
    assert_eq!(
        state.lock().expect("lock").create_order_calls,
        0,
        "external Shiprocket create order call must not happen when DB step fails before call"
    );

    drop_trigger(&db, &trigger_name).await;
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and mocked local APIs"]
async fn integration_cancel_request_is_durable_before_external_call_and_retry_is_idempotent() {
    let state = Arc::new(Mutex::new(MockState::default()));
    let _server = spawn_mock_server(18126, state.clone()).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    let tag = unique_tag(18126);
    let (order_id, user_id, _inventory_marker) = place_and_pay_order(&db, 18126, tag).await;
    ensure_delayed_shipment_booked(&db, order_id).await;
    let shipment = shipment_meta(&db, order_id).await;
    let cancel_order_id: i64 = shipment
        .try_get::<Option<String>>("", "shiprocket_external_order_id")
        .ok()
        .flatten()
        .or_else(|| {
            shipment
                .try_get::<Option<String>>("", "shiprocket_order_id")
                .ok()
                .flatten()
        })
        .expect("shiprocket cancel reference")
        .parse()
        .expect("numeric shiprocket cancel reference");

    let cancel_req_txn = db.begin().await.expect("cancel request txn");
    let cancel_req = cancel_order_via_logistics(&cancel_req_txn, order_id, Some(user_id)).await;
    assert!(
        cancel_req.is_err(),
        "cancel request should return retryable unavailable while pending logistics cancel"
    );
    assert_eq!(
        cancel_req.expect_err("checked err").code(),
        tonic::Code::Unavailable
    );
    assert_eq!(
        state
            .lock()
            .expect("lock")
            .cancel_order_ids
            .iter()
            .filter(|id| **id == cancel_order_id)
            .count(),
        0,
        "external cancel must not run before durable cancel intent commit"
    );
    cancel_req_txn
        .commit()
        .await
        .expect("commit cancel request");

    assert_eq!(
        order_status_name(&db, order_id).await,
        "cancel_pending_logistics"
    );
    process_cancel_pending_logistics(&db, 25)
        .await
        .expect("process cancel pending logistics");
    assert_eq!(
        state
            .lock()
            .expect("lock")
            .cancel_order_ids
            .iter()
            .filter(|id| **id == cancel_order_id)
            .count(),
        1
    );
    process_cancel_pending_logistics(&db, 25)
        .await
        .expect("process cancel pending logistics second run");
    assert_eq!(
        state
            .lock()
            .expect("lock")
            .cancel_order_ids
            .iter()
            .filter(|id| **id == cancel_order_id)
            .count(),
        1,
        "retry worker should not duplicate external cancel after success"
    );
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and mocked local APIs"]
async fn integration_booking_worker_external_success_with_persistence_retry_is_reconciled() {
    let state = Arc::new(Mutex::new(MockState::default()));
    let _server = spawn_mock_server(18130, state.clone()).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    let tag = unique_tag(18130);
    let (order_id, _user_id, _inventory_marker) = place_and_pay_order(&db, 18130, tag).await;
    make_order_eligible_for_delayed_booking(&db, order_id).await;

    let claim_txn = db.begin().await.expect("claim txn");
    book_order_after_validation(
        &claim_txn,
        order_id,
        Utc::now(),
        "itest_booking_persist_retry",
    )
    .await
    .expect("persist booking intent");
    claim_txn.commit().await.expect("commit booking intent");

    let trigger_name = format!("trg_itest_booking_persist_fail_{}", unique_tag(18130));
    create_trigger(
        &db,
        &trigger_name,
        &format!(
            r#"CREATE TRIGGER `{trigger_name}`
BEFORE UPDATE ON `Orders`
FOR EACH ROW
BEGIN
    IF NEW.OrderID = {order_id} AND NEW.fulfillment_status = 'booked' THEN
        SIGNAL SQLSTATE '45000'
            SET MESSAGE_TEXT = 'itest forced booking persistence failure';
    END IF;
END"#
        ),
    )
    .await;

    process_booking_intents_batch(&db, 25)
        .await
        .expect("run booking worker with forced persist failure");
    assert_eq!(
        state.lock().expect("lock").create_order_calls,
        1,
        "external booking call must happen once before persistence failure fallback"
    );
    assert_eq!(
        shipment_logistics_status(&db, order_id).await.as_deref(),
        Some("booking_persist_pending")
    );
    assert_eq!(order_fulfillment_status(&db, order_id).await, "not_created");

    drop_trigger(&db, &trigger_name).await;

    process_booking_intents_batch(&db, 25)
        .await
        .expect("retry booking worker after persist failure");
    assert_eq!(
        state.lock().expect("lock").create_order_calls,
        1,
        "reconcile retry must not issue a duplicate external booking call"
    );
    assert_eq!(
        shipment_logistics_status(&db, order_id).await.as_deref(),
        Some("booked")
    );
    assert_eq!(order_fulfillment_status(&db, order_id).await, "booked");
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and mocked local APIs"]
async fn integration_cancel_worker_external_success_with_persistence_retry_is_reconciled() {
    let state = Arc::new(Mutex::new(MockState::default()));
    let _server = spawn_mock_server(18131, state.clone()).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    let tag = unique_tag(18131);
    let (order_id, user_id, _inventory_marker) = place_and_pay_order(&db, 18131, tag).await;
    ensure_delayed_shipment_booked(&db, order_id).await;
    let shipment = shipment_meta(&db, order_id).await;
    let cancel_order_id: i64 = shipment
        .try_get::<Option<String>>("", "shiprocket_external_order_id")
        .ok()
        .flatten()
        .or_else(|| {
            shipment
                .try_get::<Option<String>>("", "shiprocket_order_id")
                .ok()
                .flatten()
        })
        .expect("shiprocket cancel reference")
        .parse()
        .expect("numeric shiprocket cancel reference");

    let cancel_req_txn = db.begin().await.expect("cancel request txn");
    let cancel_req = cancel_order_via_logistics(&cancel_req_txn, order_id, Some(user_id)).await;
    assert!(
        cancel_req.is_err(),
        "cancel request should return retryable unavailable"
    );
    assert_eq!(
        cancel_req.expect_err("checked err").code(),
        tonic::Code::Unavailable
    );
    cancel_req_txn
        .commit()
        .await
        .expect("commit cancel request intent");

    let trigger_name = format!("trg_itest_cancel_persist_fail_{}", unique_tag(18131));
    create_trigger(
        &db,
        &trigger_name,
        &format!(
            r#"CREATE TRIGGER `{trigger_name}`
BEFORE UPDATE ON `Shipments`
FOR EACH ROW
BEGIN
    IF NEW.order_id = {order_id} AND NEW.logistics_status = 'cancelled' THEN
        SIGNAL SQLSTATE '45000'
            SET MESSAGE_TEXT = 'itest forced cancel persistence failure';
    END IF;
END"#
        ),
    )
    .await;

    process_cancel_pending_logistics(&db, 25)
        .await
        .expect("run cancel worker with forced persistence failure");
    assert_eq!(
        state
            .lock()
            .expect("lock")
            .cancel_order_ids
            .iter()
            .filter(|id| **id == cancel_order_id)
            .count(),
        1,
        "external cancel should be attempted once"
    );
    assert_eq!(
        order_status_name(&db, order_id).await,
        "cancel_pending_logistics"
    );
    assert_eq!(
        shipment_logistics_status(&db, order_id).await.as_deref(),
        Some("cancel_persist_pending"),
        "failed local persistence after external cancel must remain visibly retryable"
    );

    drop_trigger(&db, &trigger_name).await;

    process_cancel_pending_logistics(&db, 25)
        .await
        .expect("retry cancel worker after persistence failure");
    assert_eq!(
        state
            .lock()
            .expect("lock")
            .cancel_order_ids
            .iter()
            .filter(|id| **id == cancel_order_id)
            .count(),
        1,
        "retry reconcile must not issue duplicate external cancel"
    );
    assert_eq!(
        shipment_logistics_status(&db, order_id).await.as_deref(),
        Some("cancelled")
    );
    assert_eq!(order_status_name(&db, order_id).await, "cancelled");
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and mocked local APIs"]
async fn integration_cancel_worker_failure_leaves_visible_retryable_state() {
    let state = Arc::new(Mutex::new(MockState {
        cancel_should_fail: true,
        ..MockState::default()
    }));
    let _server = spawn_mock_server(18132, state.clone()).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    let tag = unique_tag(18132);
    let (order_id, user_id, _inventory_marker) = place_and_pay_order(&db, 18132, tag).await;
    ensure_delayed_shipment_booked(&db, order_id).await;
    let shipment = shipment_meta(&db, order_id).await;
    let cancel_order_id: i64 = shipment
        .try_get::<Option<String>>("", "shiprocket_external_order_id")
        .ok()
        .flatten()
        .or_else(|| {
            shipment
                .try_get::<Option<String>>("", "shiprocket_order_id")
                .ok()
                .flatten()
        })
        .expect("shiprocket cancel reference")
        .parse()
        .expect("numeric shiprocket cancel reference");

    let cancel_req_txn = db.begin().await.expect("cancel request txn");
    let cancel_req = cancel_order_via_logistics(&cancel_req_txn, order_id, Some(user_id)).await;
    assert!(
        cancel_req.is_err(),
        "cancel request should return retryable unavailable"
    );
    assert_eq!(
        cancel_req.expect_err("checked err").code(),
        tonic::Code::Unavailable
    );
    cancel_req_txn
        .commit()
        .await
        .expect("commit cancel request");

    process_cancel_pending_logistics(&db, 25)
        .await
        .expect("run cancel worker with provider failure");
    assert_eq!(
        state
            .lock()
            .expect("lock")
            .cancel_order_ids
            .iter()
            .filter(|id| **id == cancel_order_id)
            .count(),
        1
    );
    assert_eq!(
        order_status_name(&db, order_id).await,
        "cancel_pending_logistics"
    );
    assert_eq!(
        shipment_logistics_status(&db, order_id).await.as_deref(),
        Some("cancel_pending_logistics")
    );

    let shipment = shipment_meta(&db, order_id).await;
    let can_customer_cancel = shipment
        .try_get::<i8>("", "can_customer_cancel")
        .map(|v| v != 0)
        .unwrap_or(false);
    assert!(
        can_customer_cancel,
        "failed cancel attempt should leave shipment in a visible retryable state"
    );
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_payment_mismatch_transition_failure_marks_webhook_failed_not_processed() {
    let state = Arc::new(Mutex::new(MockState::default()));
    let _server = spawn_mock_server(18133, state).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    let tag = unique_tag(18133);
    let (order_id, _user_id, _inventory_marker) =
        place_order_without_payment_verification(&db, 18133, tag, Some("prepaid")).await;
    let (razorpay_order_id, _payment_id) = latest_payment_intent_ids_for_order(&db, order_id).await;

    let trigger_name = format!(
        "trg_itest_webhook_mismatch_transition_fail_{}",
        unique_tag(18133)
    );
    create_trigger(
        &db,
        &trigger_name,
        &format!(
            r#"CREATE TRIGGER `{trigger_name}`
BEFORE UPDATE ON `Orders`
FOR EACH ROW
BEGIN
    IF NEW.OrderID = {order_id} AND NEW.StatusID <> OLD.StatusID THEN
        SIGNAL SQLSTATE '45000'
            SET MESSAGE_TEXT = 'itest forced order transition failure';
    END IF;
END"#
        ),
    )
    .await;

    let webhook_id = format!("itest_mismatch_fail_{tag}");
    let webhook_txn = db.begin().await.expect("webhook txn");
    let response = ingest_webhook(
        &webhook_txn,
        Request::new(IngestWebhookRequest {
            provider: "razorpay".to_string(),
            event_type: "payment.captured".to_string(),
            webhook_id: webhook_id.clone(),
            payload_json: json!({
                "event": "payment.captured",
                "payload": {
                    "payment": {
                        "entity": {
                            "id": format!("pay_mismatch_fail_{tag}"),
                            "order_id": razorpay_order_id,
                            "amount": 1,
                            "currency": "INR"
                        }
                    }
                }
            })
            .to_string(),
            signature_verified: true,
            provider_event_id: None,
        }),
    )
    .await
    .expect("ingest mismatch webhook");
    assert_eq!(response.get_ref().items[0].status, "failed");
    webhook_txn.commit().await.expect("commit webhook");

    assert_eq!(
        webhook_status_by_webhook_id(&db, webhook_id.as_str())
            .await
            .as_deref(),
        Some("failed")
    );
    drop_trigger(&db, &trigger_name).await;
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and mocked local APIs"]
async fn integration_shiprocket_cancel_settlement_failure_marks_webhook_failed_not_processed() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_shiprocket_cancel_settlement_failure_marks_webhook_failed_not_processed",
    ) {
        return;
    }

    let state = Arc::new(Mutex::new(MockState::default()));
    let _server = spawn_mock_server(18134, state.clone()).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    let tag = unique_tag(18134);
    let (order_id, _user_id, _inventory_marker) = place_and_pay_order(&db, 18134, tag).await;
    ensure_delayed_shipment_booked(&db, order_id).await;
    let shipment = shipment_meta(&db, order_id).await;
    let shiprocket_order_id: String = shipment
        .try_get("", "shiprocket_order_id")
        .expect("shiprocket_order_id");
    let awb_code: String = shipment.try_get("", "awb_code").expect("awb_code");

    let trigger_name = format!("trg_itest_settlement_fail_{}", unique_tag(18134));
    create_trigger(
        &db,
        &trigger_name,
        &format!(
            r#"CREATE TRIGGER `{trigger_name}`
BEFORE INSERT ON `RefundAttempts`
FOR EACH ROW
BEGIN
    IF NEW.order_id = {order_id} THEN
        SIGNAL SQLSTATE '45000'
            SET MESSAGE_TEXT = 'itest forced settlement persistence failure';
    END IF;
END"#
        ),
    )
    .await;

    let webhook_id = format!("itest_shiprocket_cancel_fail_{tag}");
    let webhook_txn = db.begin().await.expect("webhook txn");
    let response = ingest_webhook(
        &webhook_txn,
        Request::new(IngestWebhookRequest {
            provider: "shiprocket".to_string(),
            event_type: "shiprocket.update".to_string(),
            webhook_id: webhook_id.clone(),
            payload_json: json!({
                "awb": awb_code,
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
    .expect("ingest shiprocket cancel webhook");
    assert_eq!(response.get_ref().items[0].status, "failed");
    webhook_txn.commit().await.expect("commit webhook");

    assert_eq!(
        webhook_status_by_webhook_id(&db, webhook_id.as_str())
            .await
            .as_deref(),
        Some("failed")
    );
    drop_trigger(&db, &trigger_name).await;
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and mocked local APIs"]
async fn integration_refund_webhook_persistence_failure_marks_webhook_failed_not_processed() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_refund_webhook_persistence_failure_marks_webhook_failed_not_processed",
    ) {
        return;
    }

    let state = Arc::new(Mutex::new(MockState::default()));
    let _server = spawn_mock_server(18135, state.clone()).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    let tag = unique_tag(18135);
    let (order_id, _user_id, _inventory_marker) = place_and_pay_order(&db, 18135, tag).await;
    let (_rzp_order_id, payment_id_opt) = latest_payment_intent_ids_for_order(&db, order_id).await;
    let payment_id = payment_id_opt.expect("captured payment id");

    let trigger_name = format!("trg_itest_refund_webhook_fail_{}", unique_tag(18135));
    create_trigger(
        &db,
        &trigger_name,
        &format!(
            r#"CREATE TRIGGER `{trigger_name}`
BEFORE INSERT ON `Refunds`
FOR EACH ROW
BEGIN
    IF NEW.order_id = {order_id} THEN
        SIGNAL SQLSTATE '45000'
            SET MESSAGE_TEXT = 'itest forced refund webhook persistence failure';
    END IF;
END"#
        ),
    )
    .await;

    let webhook_id = format!("itest_refund_webhook_fail_{tag}");
    let webhook_txn = db.begin().await.expect("refund webhook txn");
    let response = ingest_webhook(
        &webhook_txn,
        Request::new(IngestWebhookRequest {
            provider: "razorpay".to_string(),
            event_type: "refund.processed".to_string(),
            webhook_id: webhook_id.clone(),
            payload_json: json!({
                "event": "refund.processed",
                "payload": {
                    "refund": {
                        "entity": {
                            "id": format!("rfnd_webhook_fail_{tag}"),
                            "payment_id": payment_id,
                            "amount": 500,
                            "currency": "INR",
                            "status": "processed"
                        }
                    }
                }
            })
            .to_string(),
            signature_verified: true,
            provider_event_id: None,
        }),
    )
    .await
    .expect("ingest refund webhook");
    assert_eq!(response.get_ref().items[0].status, "failed");
    webhook_txn.commit().await.expect("commit refund webhook");

    assert_eq!(
        webhook_status_by_webhook_id(&db, webhook_id.as_str())
            .await
            .as_deref(),
        Some("failed")
    );
    drop_trigger(&db, &trigger_name).await;
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and mocked local APIs"]
async fn integration_payment_success_auto_books_shiprocket_and_is_idempotent() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_payment_success_auto_books_shiprocket_and_is_idempotent",
    ) {
        return;
    }

    let state = Arc::new(Mutex::new(MockState::default()));
    let _server = spawn_mock_server(18101, state.clone()).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    let tag = unique_tag(18101);
    let (order_id, _user_id, _inventory_id) = place_and_pay_order(&db, 18101, tag).await;
    assert_eq!(shipment_count(&db, order_id).await, 0);
    assert_eq!(order_fulfillment_status(&db, order_id).await, "not_created");

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
    assert_eq!(
        shipment_count(&db, order_id).await,
        0,
        "payment verification replay must not auto-create shipment"
    );

    ensure_delayed_shipment_booked(&db, order_id).await;

    let shipment = shipment_meta(&db, order_id).await;
    let selected_courier_id: Option<i64> = shipment
        .try_get("", "selected_courier_id")
        .expect("courier");
    let logistics_status: String = shipment.try_get("", "logistics_status").expect("status");
    let awb_code: String = shipment.try_get("", "awb_code").expect("awb");
    let pickup_scheduled_for: chrono::DateTime<Utc> = shipment
        .try_get("", "pickup_scheduled_for")
        .expect("pickup");
    assert_eq!(selected_courier_id, None);
    assert_eq!(logistics_status, "booked");
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
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_pre_pickup_cancel_restores_stock_and_refunds_once",
    ) {
        return;
    }

    let state = Arc::new(Mutex::new(MockState::default()));
    let _server = spawn_mock_server(18102, state.clone()).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    let tag = unique_tag(18102);
    let (order_id, user_id, inventory_marker) = place_and_pay_order(&db, 18102, tag).await;
    ensure_delayed_shipment_booked(&db, order_id).await;

    let cancel_txn = db.begin().await.expect("cancel txn");
    let err = delete_order(
        &cancel_txn,
        Request::new(DeleteOrderRequest {
            order_id,
            acting_user_id: Some(user_id),
        }),
    )
    .await
    .expect_err("booked order cancel should be blocked");
    assert_eq!(err.code(), tonic::Code::FailedPrecondition);
    cancel_txn.rollback().await.ok();

    assert_eq!(inventory_available(&db, inventory_marker).await, 4);
    assert_eq!(refund_attempt_count(&db, order_id).await, 0);
    let guard = state.lock().expect("lock");
    assert_eq!(guard.cancel_calls, 0);
    assert_eq!(guard.refund_calls, 0);
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and mocked local APIs"]
async fn integration_customer_cancel_and_webhook_cancel_race_refunds_once() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_customer_cancel_and_webhook_cancel_race_refunds_once",
    ) {
        return;
    }

    let state = Arc::new(Mutex::new(MockState::default()));
    let _server = spawn_mock_server(18106, state.clone()).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    neutralize_open_refund_attempts(
        &db,
        "itest cleanup: isolate refund worker before cancel/webhook race flow",
    )
    .await;
    let tag = unique_tag(18106);
    let (order_id, user_id, _inventory_marker) = place_and_pay_order(&db, 18106, tag).await;
    let frozen_order_total = order_grand_total_minor(&db, order_id).await;
    ensure_delayed_shipment_booked(&db, order_id).await;
    let shipment = shipment_meta(&db, order_id).await;
    let shiprocket_order_id: String = shipment
        .try_get("", "shiprocket_order_id")
        .expect("ship id");
    let awb_code: String = shipment.try_get("", "awb_code").expect("awb");

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
                    "awb": awb_code,
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
    match cancel_res {
        Ok(_) => panic!("booked order cancel should be blocked"),
        Err(e) => assert_eq!(e.code(), tonic::Code::FailedPrecondition),
    }
    assert!(
        webhook_res.is_ok(),
        "webhook cancel path should converge: {webhook_res:?}"
    );
    {
        let guard = state.lock().expect("lock");
        assert_eq!(
            order_scoped_refund_call_count(&guard, order_id),
            0,
            "race should queue refund attempt durably before worker, without inline outbound refund call"
        );
    }
    assert!(
        refund_attempt_count(&db, order_id).await == 1,
        "race should create exactly one durable refund attempt row"
    );
    assert_eq!(
        refund_attempt_status(&db, order_id).await.as_deref(),
        Some("pending_external")
    );

    process_refund_attempts(&db, 25)
        .await
        .expect("run refund worker after race");
    let expected_idempotency_key = refund_idempotency_key(
        order_id,
        &format!("pay_logistics_{tag}"),
        frozen_order_total,
    );
    {
        let guard = state.lock().expect("lock");
        assert_eq!(
            order_scoped_refund_call_count(&guard, order_id),
            1,
            "race should converge to exactly one outbound refund call after worker execution"
        );
        assert_eq!(
            order_scoped_refund_idempotency_keys(&guard, order_id),
            vec![expected_idempotency_key],
            "race should preserve exactly one deterministic idempotency key"
        );
    }
    assert_eq!(refund_row_count(&db, order_id).await, 1);
    assert_eq!(
        refund_attempt_status(&db, order_id).await.as_deref(),
        Some("processed")
    );

    process_refund_attempts(&db, 25)
        .await
        .expect("second worker pass should stay idempotent");
    {
        let guard = state.lock().expect("lock");
        assert_eq!(
            order_scoped_refund_call_count(&guard, order_id),
            1,
            "second worker pass must not issue duplicate outbound refund calls"
        );
    }
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and mocked local APIs"]
async fn integration_failed_shiprocket_cancel_moves_to_cancel_pending_without_refund() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_failed_shiprocket_cancel_moves_to_cancel_pending_without_refund",
    ) {
        return;
    }

    let state = Arc::new(Mutex::new(MockState {
        cancel_should_fail: true,
        ..MockState::default()
    }));
    let _server = spawn_mock_server(18103, state.clone()).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    neutralize_open_refund_attempts(
        &db,
        "itest cleanup: isolate refund worker before failed shiprocket cancel flow",
    )
    .await;
    let tag = unique_tag(18103);
    let (order_id, user_id, inventory_marker) = place_and_pay_order(&db, 18103, tag).await;
    let frozen_order_total = order_grand_total_minor(&db, order_id).await;

    let cancel_txn = db.begin().await.expect("cancel txn");
    delete_order(
        &cancel_txn,
        Request::new(DeleteOrderRequest {
            order_id,
            acting_user_id: Some(user_id),
        }),
    )
    .await
    .expect("within-window cancellation should succeed without logistics API cancel");
    cancel_txn.commit().await.expect("commit cancel");

    assert_eq!(order_status_name(&db, order_id).await, "cancelled");
    assert_eq!(
        refund_attempt_count(&db, order_id).await,
        1,
        "cancellation should persist exactly one durable refund attempt before worker"
    );
    let latest_attempt = latest_refund_attempt_snapshot(&db, order_id)
        .await
        .expect("latest refund attempt");
    assert_eq!(latest_attempt.status, "pending_external");
    assert_eq!(
        latest_attempt.amount_requested_paise, frozen_order_total,
        "full cancellation should target frozen order grand total"
    );
    assert_eq!(
        latest_attempt.amount_sent_to_gateway_paise, frozen_order_total,
        "pending attempt should queue full frozen order amount"
    );
    assert_eq!(inventory_available(&db, inventory_marker).await, 6);
    {
        let guard = state.lock().expect("lock");
        assert_eq!(guard.cancel_calls, 0);
        assert_eq!(
            order_scoped_refund_call_count(&guard, order_id),
            0,
            "no refund gateway call should happen before worker"
        );
    }

    process_refund_attempts(&db, 25)
        .await
        .expect("run refund worker");
    let expected_idempotency_key = refund_idempotency_key(
        order_id,
        &format!("pay_logistics_{tag}"),
        frozen_order_total,
    );
    {
        let guard = state.lock().expect("lock");
        assert_eq!(
            order_scoped_refund_call_count(&guard, order_id),
            1,
            "worker should issue exactly one outbound refund call for this order"
        );
        assert_eq!(
            order_scoped_refund_idempotency_keys(&guard, order_id),
            vec![expected_idempotency_key]
        );
    }
    assert_eq!(order_status_name(&db, order_id).await, "refunded");
    assert_eq!(
        refund_attempt_status(&db, order_id).await.as_deref(),
        Some("processed")
    );
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and mocked local APIs"]
async fn integration_pickup_completed_disables_customer_cancellation() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_pickup_completed_disables_customer_cancellation",
    ) {
        return;
    }

    let state = Arc::new(Mutex::new(MockState::default()));
    let _server = spawn_mock_server(18104, state.clone()).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    let tag = unique_tag(18104);
    let (order_id, user_id, _inventory_marker) = place_and_pay_order(&db, 18104, tag).await;
    ensure_delayed_shipment_booked(&db, order_id).await;
    let shipment = shipment_meta(&db, order_id).await;
    let shiprocket_order_id: String = shipment
        .try_get("", "shiprocket_order_id")
        .expect("ship id");
    let awb_code: String = shipment.try_get("", "awb_code").expect("awb");

    let webhook_txn = db.begin().await.expect("webhook txn");
    ingest_webhook(
        &webhook_txn,
        Request::new(IngestWebhookRequest {
            provider: "shiprocket".to_string(),
            event_type: "shiprocket.update".to_string(),
            webhook_id: format!("pickup_{tag}"),
            payload_json: json!({
                "awb": awb_code,
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
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_rto_terminal_webhook_refunds_once",
    ) {
        return;
    }

    let state = Arc::new(Mutex::new(MockState::default()));
    let _server = spawn_mock_server(18105, state.clone()).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    neutralize_open_refund_attempts(
        &db,
        "itest cleanup: isolate refund worker before RTO webhook flow",
    )
    .await;
    let tag = unique_tag(18105);
    let (order_id, _user_id, inventory_marker) = place_and_pay_order(&db, 18105, tag).await;
    let frozen_order_total = order_grand_total_minor(&db, order_id).await;
    ensure_delayed_shipment_booked(&db, order_id).await;
    let shipment = shipment_meta(&db, order_id).await;
    let shiprocket_order_id: String = shipment
        .try_get("", "shiprocket_order_id")
        .expect("ship id");
    let awb_code: String = shipment.try_get("", "awb_code").expect("awb");

    for attempt in 0..2 {
        let webhook_txn = db.begin().await.expect("webhook txn");
        ingest_webhook(
            &webhook_txn,
            Request::new(IngestWebhookRequest {
                provider: "shiprocket".to_string(),
                event_type: "shiprocket.update".to_string(),
                webhook_id: format!("rto_{tag}_{attempt}"),
                payload_json: json!({
                    "awb": awb_code,
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

    assert_eq!(
        refund_attempt_count(&db, order_id).await,
        1,
        "duplicate terminal RTO webhooks should queue exactly one durable refund attempt"
    );
    assert_eq!(
        refund_attempt_status(&db, order_id).await.as_deref(),
        Some("pending_external")
    );
    let shipment = shipment_meta(&db, order_id).await;
    let refund_id_before_worker: Option<String> = shipment
        .try_get("", "razorpay_refund_id")
        .expect("refund id before worker");
    assert!(
        refund_id_before_worker.is_none(),
        "shipment refund id must stay null until refund worker persists gateway success"
    );
    {
        let guard = state.lock().expect("lock");
        assert_eq!(
            order_scoped_refund_call_count(&guard, order_id),
            0,
            "webhook path must not call refund gateway before worker"
        );
    }

    process_refund_attempts(&db, 25)
        .await
        .expect("run refund worker after RTO webhook");
    let expected_idempotency_key = refund_idempotency_key(
        order_id,
        &format!("pay_logistics_{tag}"),
        frozen_order_total,
    );
    {
        let guard = state.lock().expect("lock");
        assert_eq!(
            order_scoped_refund_call_count(&guard, order_id),
            1,
            "RTO flow should issue exactly one outbound refund call after worker"
        );
        assert_eq!(
            order_scoped_refund_idempotency_keys(&guard, order_id),
            vec![expected_idempotency_key]
        );
    }
    let shipment = shipment_meta(&db, order_id).await;
    let refund_id_after_worker: Option<String> = shipment
        .try_get("", "razorpay_refund_id")
        .expect("refund id after worker");
    assert!(
        refund_id_after_worker
            .as_deref()
            .is_some_and(|id| id.starts_with("rfnd_logistics_")),
        "shipment refund id should be persisted after worker reconciliation"
    );
    assert_eq!(
        refund_attempt_status(&db, order_id).await.as_deref(),
        Some("processed")
    );
    assert_eq!(inventory_available(&db, inventory_marker).await, 6);

    process_refund_attempts(&db, 25)
        .await
        .expect("second worker pass should stay idempotent");
    {
        let guard = state.lock().expect("lock");
        assert_eq!(
            order_scoped_refund_call_count(&guard, order_id),
            1,
            "RTO retry worker pass must not produce duplicate gateway refunds"
        );
    }
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and mocked local APIs"]
async fn integration_refund_retry_reuses_same_idempotency_key() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_refund_retry_reuses_same_idempotency_key",
    ) {
        return;
    }

    let state = Arc::new(Mutex::new(MockState {
        refund_should_fail: true,
        ..MockState::default()
    }));
    let _server = spawn_mock_server(18107, state.clone()).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    neutralize_open_refund_attempts(
        &db,
        "itest cleanup: isolate refund worker before refund retry idempotency-key flow",
    )
    .await;
    let tag = unique_tag(18107);
    let (order_id, _user_id, _inventory_marker) = place_and_pay_order(&db, 18107, tag).await;
    let frozen_order_total = order_grand_total_minor(&db, order_id).await;
    ensure_delayed_shipment_booked(&db, order_id).await;
    let shipment = shipment_meta(&db, order_id).await;
    let shiprocket_order_id: String = shipment
        .try_get("", "shiprocket_order_id")
        .expect("ship id");
    let awb_code: String = shipment.try_get("", "awb_code").expect("awb");

    let first_webhook_txn = db.begin().await.expect("first webhook txn");
    ingest_webhook(
        &first_webhook_txn,
        Request::new(IngestWebhookRequest {
            provider: "shiprocket".to_string(),
            event_type: "shiprocket.update".to_string(),
            webhook_id: format!("retry_cancel_{tag}_first"),
            payload_json: json!({
                "awb": awb_code,
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
    .expect("first webhook should queue refund attempt");
    first_webhook_txn
        .commit()
        .await
        .expect("commit first webhook");
    assert_eq!(
        refund_attempt_count(&db, order_id).await,
        1,
        "first terminal webhook should queue exactly one refund attempt"
    );
    assert_eq!(
        refund_attempt_status(&db, order_id).await.as_deref(),
        Some("pending_external")
    );
    {
        let guard = state.lock().expect("lock");
        assert_eq!(
            order_scoped_refund_call_count(&guard, order_id),
            0,
            "webhook processing must not call gateway before refund worker"
        );
    }

    process_refund_attempts(&db, 25)
        .await
        .expect("run worker with forced gateway failure");
    let expected_idempotency_key = refund_idempotency_key(
        order_id,
        &format!("pay_logistics_{tag}"),
        frozen_order_total,
    );
    {
        let guard = state.lock().expect("lock");
        assert_eq!(
            order_scoped_refund_call_count(&guard, order_id),
            1,
            "first worker run should perform exactly one outbound refund call"
        );
        assert_eq!(
            order_scoped_refund_idempotency_keys(&guard, order_id),
            vec![expected_idempotency_key.clone()],
            "first worker run should use deterministic idempotency key"
        );
    }
    assert_eq!(
        refund_attempt_status(&db, order_id).await.as_deref(),
        Some("pending_external"),
        "failed gateway call should reset attempt to pending_external for retry"
    );

    {
        let mut guard = state.lock().expect("lock");
        guard.refund_should_fail = false;
    }

    let webhook_txn = db.begin().await.expect("webhook txn");
    ingest_webhook(
        &webhook_txn,
        Request::new(IngestWebhookRequest {
            provider: "shiprocket".to_string(),
            event_type: "shiprocket.update".to_string(),
            webhook_id: format!("retry_cancel_{tag}"),
            payload_json: json!({
                "awb": awb_code,
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
    assert_eq!(
        refund_attempt_count(&db, order_id).await,
        1,
        "duplicate webhook must not create a second refund attempt"
    );

    process_refund_attempts(&db, 25)
        .await
        .expect("run worker retry after webhook replay");
    {
        let guard = state.lock().expect("lock");
        assert_eq!(
            order_scoped_refund_call_count(&guard, order_id),
            2,
            "retry path should issue exactly one additional outbound call after first failure"
        );
        assert_eq!(
            order_scoped_refund_idempotency_keys(&guard, order_id),
            vec![expected_idempotency_key.clone(), expected_idempotency_key],
            "retry path must reuse the same idempotency key across worker retries"
        );
    }
    assert_eq!(
        refund_attempt_status(&db, order_id).await.as_deref(),
        Some("processed")
    );
    assert_eq!(refund_row_count(&db, order_id).await, 1);
    assert_eq!(
        processed_refund_total_minor(&db, order_id).await,
        frozen_order_total,
        "retry path should reconcile to full frozen-order refund amount once gateway succeeds"
    );
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and mocked local APIs"]
async fn integration_partial_cancel_refunds_items_then_shipping_on_full_cancel() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_partial_cancel_refunds_items_then_shipping_on_full_cancel",
    ) {
        return;
    }

    let state = Arc::new(Mutex::new(MockState::default()));
    let _server = spawn_mock_server(18108, state.clone()).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    neutralize_open_refund_attempts(
        &db,
        "itest cleanup: isolate refund worker before partial/full cancel settlement flow",
    )
    .await;
    let tag = unique_tag(18108);
    let (order_id, user_id, _inventory_marker) = place_and_pay_order(&db, 18108, tag).await;
    let detail_ids = split_order_into_two_lines(&db, order_id).await;
    assert_eq!(detail_ids.len(), 2);
    let frozen_order_total = order_grand_total_minor(&db, order_id).await;

    let first_cancel_txn = db.begin().await.expect("cancel one txn");
    cancel_order_items(
        &first_cancel_txn,
        Request::new(CancelOrderItemsRequest {
            order_id,
            order_detail_ids: vec![detail_ids[0]],
            acting_user_id: Some(user_id),
        }),
    )
    .await
    .expect("cancel first line");
    first_cancel_txn
        .commit()
        .await
        .expect("commit first cancel");
    assert_eq!(
        refund_attempt_count(&db, order_id).await,
        1,
        "first partial cancel should queue exactly one durable refund attempt"
    );
    let first_attempt = latest_refund_attempt_snapshot(&db, order_id)
        .await
        .expect("first refund attempt snapshot");
    assert_eq!(first_attempt.status, "pending_external");
    assert_eq!(
        first_attempt.amount_requested_paise, 2000,
        "first partial cancel should target cancelled line total only"
    );
    assert_eq!(
        first_attempt.amount_sent_to_gateway_paise, 2000,
        "first queued refund amount should match cancelled line total"
    );
    {
        let guard = state.lock().expect("lock");
        assert_eq!(
            order_scoped_refund_call_count(&guard, order_id),
            0,
            "first partial cancel must not call gateway before worker"
        );
    }

    process_refund_attempts(&db, 25)
        .await
        .expect("run worker for first partial refund");
    {
        let guard = state.lock().expect("lock");
        assert_eq!(
            order_scoped_refund_call_count(&guard, order_id),
            1,
            "worker should execute first partial refund exactly once"
        );
    }
    assert_eq!(
        processed_refund_total_minor(&db, order_id).await,
        2000,
        "first worker pass should persist only cancelled line total"
    );

    let second_cancel_txn = db.begin().await.expect("cancel second txn");
    cancel_order_items(
        &second_cancel_txn,
        Request::new(CancelOrderItemsRequest {
            order_id,
            order_detail_ids: vec![detail_ids[1]],
            acting_user_id: Some(user_id),
        }),
    )
    .await
    .expect("cancel second line");
    second_cancel_txn
        .commit()
        .await
        .expect("commit second cancel");
    assert_eq!(
        refund_attempt_count(&db, order_id).await,
        2,
        "full cancellation after first settlement should queue second refund attempt"
    );
    let second_attempt = latest_refund_attempt_snapshot(&db, order_id)
        .await
        .expect("second refund attempt snapshot");
    let expected_remaining = frozen_order_total - 2000;
    assert_eq!(second_attempt.status, "pending_external");
    assert_eq!(
        second_attempt.amount_requested_paise, expected_remaining,
        "second attempt should target remaining amount after processed partial refund"
    );
    assert_eq!(
        second_attempt.amount_sent_to_gateway_paise, expected_remaining,
        "second queued amount should send only the remaining frozen total"
    );

    process_refund_attempts(&db, 25)
        .await
        .expect("run worker for full settlement remainder");
    let expected_idempotency_keys = vec![
        refund_idempotency_key(order_id, &format!("pay_logistics_{tag}"), 2000),
        refund_idempotency_key(
            order_id,
            &format!("pay_logistics_{tag}"),
            frozen_order_total,
        ),
    ];
    {
        let guard = state.lock().expect("lock");
        assert_eq!(
            order_scoped_refund_call_count(&guard, order_id),
            2,
            "partial then full flow should execute exactly two gateway settlements"
        );
        assert_eq!(
            order_scoped_refund_amounts(&guard, order_id),
            vec![2000, expected_remaining],
            "gateway amounts should match partial line total then full-order remainder"
        );
        assert_eq!(
            order_scoped_refund_idempotency_keys(&guard, order_id),
            expected_idempotency_keys,
            "each settlement should use deterministic idempotency key based on target amount"
        );
    }
    assert_eq!(
        processed_refund_total_minor(&db, order_id).await,
        frozen_order_total,
        "partial then full cancel should reconcile to full frozen order refund total"
    );
    assert_eq!(order_status_name(&db, order_id).await, "refunded");

    process_refund_attempts(&db, 25)
        .await
        .expect("idempotency check worker rerun");
    {
        let guard = state.lock().expect("lock");
        assert_eq!(
            order_scoped_refund_call_count(&guard, order_id),
            2,
            "rerunning worker must not create a third settlement call"
        );
    }
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and mocked local APIs"]
async fn integration_partial_duplicate_line_cancel_does_not_double_refund() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_partial_duplicate_line_cancel_does_not_double_refund",
    ) {
        return;
    }

    let state = Arc::new(Mutex::new(MockState::default()));
    let _server = spawn_mock_server(18109, state.clone()).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    neutralize_open_refund_attempts(
        &db,
        "itest cleanup: isolate refund worker before duplicate-line cancel flow",
    )
    .await;
    let tag = unique_tag(18109);
    let (order_id, user_id, _inventory_marker) = place_and_pay_order(&db, 18109, tag).await;
    let detail_ids = split_order_into_two_lines(&db, order_id).await;

    let first_cancel_txn = db.begin().await.expect("cancel first txn");
    cancel_order_items(
        &first_cancel_txn,
        Request::new(CancelOrderItemsRequest {
            order_id,
            order_detail_ids: vec![detail_ids[0]],
            acting_user_id: Some(user_id),
        }),
    )
    .await
    .expect("cancel first line");
    first_cancel_txn.commit().await.expect("commit first");
    assert_eq!(
        refund_attempt_count(&db, order_id).await,
        1,
        "first line cancel should queue exactly one refund attempt"
    );
    assert_eq!(
        refund_attempt_status(&db, order_id).await.as_deref(),
        Some("pending_external")
    );

    let replay_txn = db.begin().await.expect("cancel replay txn");
    let replay = cancel_order_items(
        &replay_txn,
        Request::new(CancelOrderItemsRequest {
            order_id,
            order_detail_ids: vec![detail_ids[0]],
            acting_user_id: Some(user_id),
        }),
    )
    .await;
    replay_txn.rollback().await.ok();
    assert!(replay.is_err(), "duplicate line cancel should be rejected");
    assert_eq!(
        refund_attempt_count(&db, order_id).await,
        1,
        "duplicate cancel must not queue an additional refund attempt"
    );
    {
        let guard = state.lock().expect("lock");
        assert_eq!(
            order_scoped_refund_call_count(&guard, order_id),
            0,
            "duplicate line cancel should not trigger inline gateway refund before worker"
        );
    }

    process_refund_attempts(&db, 25)
        .await
        .expect("run refund worker for first line cancel");
    let expected_idempotency_key =
        refund_idempotency_key(order_id, &format!("pay_logistics_{tag}"), 2000);
    {
        let guard = state.lock().expect("lock");
        assert_eq!(
            order_scoped_refund_call_count(&guard, order_id),
            1,
            "worker should execute queued refund once despite duplicate cancel request"
        );
        assert_eq!(
            order_scoped_refund_idempotency_keys(&guard, order_id),
            vec![expected_idempotency_key]
        );
    }
    assert_eq!(
        processed_refund_total_minor(&db, order_id).await,
        2000,
        "single cancelled line should settle exactly one line total"
    );

    process_refund_attempts(&db, 25)
        .await
        .expect("worker rerun should stay idempotent");
    {
        let guard = state.lock().expect("lock");
        assert_eq!(
            order_scoped_refund_call_count(&guard, order_id),
            1,
            "worker rerun must not produce duplicate outbound refunds"
        );
    }
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and mocked local APIs"]
async fn integration_delayed_shipment_worker_books_only_after_cancel_window() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_delayed_shipment_worker_books_only_after_cancel_window",
    ) {
        return;
    }

    let state = Arc::new(Mutex::new(MockState::default()));
    let _server = spawn_mock_server(18110, state.clone()).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    let tag = unique_tag(18110);
    let (order_id, _user_id, _inventory_marker) = place_and_pay_order(&db, 18110, tag).await;

    assert_eq!(
        shipment_count(&db, order_id).await,
        0,
        "shipment must not be created at order placement/payment verification"
    );
    assert_eq!(order_fulfillment_status(&db, order_id).await, "not_created");

    process_create_shipments_after_cancel_window(&db, 500)
        .await
        .expect("run delayed shipment worker before window");
    assert_eq!(
        shipment_count(&db, order_id).await,
        0,
        "worker must not create shipment before cancel window elapses"
    );

    ensure_delayed_shipment_booked(&db, order_id).await;
    assert_eq!(shipment_count(&db, order_id).await, 1);
    assert_eq!(order_fulfillment_status(&db, order_id).await, "booked");
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and mocked local APIs"]
async fn integration_order_creation_sets_cancel_booking_and_pickup_timestamps() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_order_creation_sets_cancel_booking_and_pickup_timestamps",
    ) {
        return;
    }

    let state = Arc::new(Mutex::new(MockState::default()));
    let _server = spawn_mock_server(18118, state.clone()).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    let tag = unique_tag(18118);
    let (order_id, _user_id, _inventory_marker) =
        place_order_without_payment_verification(&db, 18118, tag, Some("prepaid")).await;

    let (created_at, cancel_window_ends_at, earliest_booking_at, pickup_target_at) =
        order_fulfillment_timestamps(&db, order_id).await;
    assert_eq!(
        cancel_window_ends_at, earliest_booking_at,
        "earliest_booking_at should be initialized from cancel_window_ends_at"
    );

    let expected_cancel_minutes = core_operations::order_policy::cancel_window_hours() * 60;
    let actual_cancel_minutes = (cancel_window_ends_at - created_at).num_minutes();
    assert!(
        (actual_cancel_minutes - expected_cancel_minutes).abs() <= 1,
        "cancel_window_ends_at should be created_at + CANCEL_WINDOW_HOURS (expected ~{expected_cancel_minutes}m, got {actual_cancel_minutes}m)"
    );

    let expected_pickup_minutes = core_operations::order_policy::pickup_delay_hours() * 60;
    let actual_pickup_minutes = (pickup_target_at - created_at).num_minutes();
    assert!(
        (actual_pickup_minutes - expected_pickup_minutes).abs() <= 1,
        "pickup_target_at should be created_at + PICKUP_DELAY_HOURS (expected ~{expected_pickup_minutes}m, got {actual_pickup_minutes}m)"
    );
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and mocked local APIs"]
async fn integration_admin_pickup_target_update_does_not_create_shipment_or_reopen_cancel_window() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_admin_pickup_target_update_does_not_create_shipment_or_reopen_cancel_window",
    ) {
        return;
    }

    let state = Arc::new(Mutex::new(MockState::default()));
    let _server = spawn_mock_server(18119, state.clone()).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    let tag = unique_tag(18119);
    let (order_id, user_id, _inventory_marker) =
        place_order_without_payment_verification(&db, 18119, tag, Some("cod")).await;
    let detail_ids = split_order_into_two_lines(&db, order_id).await;
    assert_eq!(detail_ids.len(), 2);

    let moved_earlier = Utc::now() - chrono::Duration::hours(2);
    let update_txn = db.begin().await.expect("update pickup target txn");
    let update_resp = update_pickup_target(
        &update_txn,
        Request::new(UpdatePickupTargetRequest {
            order_id,
            pickup_target_at: moved_earlier.to_rfc3339(),
            reason: Some("ops_pullin".to_string()),
            actor_id: Some("admin:fulfillment".to_string()),
        }),
    )
    .await
    .expect("update pickup target earlier")
    .into_inner();
    update_txn.commit().await.expect("commit pickup update");

    assert_eq!(update_resp.order_id, order_id);
    assert_eq!(
        update_resp.pickup_target_reason.as_deref(),
        Some("ops_pullin")
    );
    assert_eq!(
        update_resp.pickup_target_set_by.as_deref(),
        Some("admin:fulfillment")
    );
    assert_eq!(
        shipment_count(&db, order_id).await,
        0,
        "updating pickup target should not create shipment rows"
    );

    let cancel_txn = db.begin().await.expect("cancel txn");
    cancel_order_items(
        &cancel_txn,
        Request::new(CancelOrderItemsRequest {
            order_id,
            order_detail_ids: vec![detail_ids[0]],
            acting_user_id: Some(user_id),
        }),
    )
    .await
    .expect("pickup target change must not block valid in-window cancel");
    cancel_txn.commit().await.expect("commit cancel");
    assert_eq!(
        order_status_name(&db, order_id).await,
        "partially_cancelled"
    );

    let second_order_tag = unique_tag(19119);
    let (locked_order_id, locked_user_id, _inventory_marker) =
        place_order_without_payment_verification(&db, 18119, second_order_tag, Some("cod")).await;

    let close_window_txn = db.begin().await.expect("close window txn");
    close_window_txn
        .execute(Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            "UPDATE Orders SET cancel_window_ends_at = UTC_TIMESTAMP() - INTERVAL 1 MINUTE WHERE OrderID = ?",
            [locked_order_id.into()],
        ))
        .await
        .expect("close cancel window");
    close_window_txn
        .commit()
        .await
        .expect("commit close window");

    let moved_later = Utc::now() + chrono::Duration::hours(72);
    let update_later_txn = db.begin().await.expect("update later txn");
    update_pickup_target(
        &update_later_txn,
        Request::new(UpdatePickupTargetRequest {
            order_id: locked_order_id,
            pickup_target_at: moved_later.to_rfc3339(),
            reason: Some("ops_delay".to_string()),
            actor_id: Some("admin:fulfillment".to_string()),
        }),
    )
    .await
    .expect("update pickup target later");
    update_later_txn
        .commit()
        .await
        .expect("commit pickup target later");

    let blocked_cancel_txn = db.begin().await.expect("blocked cancel txn");
    let blocked = delete_order(
        &blocked_cancel_txn,
        Request::new(DeleteOrderRequest {
            order_id: locked_order_id,
            acting_user_id: Some(locked_user_id),
        }),
    )
    .await
    .expect_err("pickup_target update must not reopen cancel window");
    assert_eq!(blocked.code(), tonic::Code::FailedPrecondition);
    blocked_cancel_txn.rollback().await.ok();
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and mocked local APIs"]
async fn integration_manual_shipment_paths_use_shared_booking_validation() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_manual_shipment_paths_use_shared_booking_validation",
    ) {
        return;
    }

    let state = Arc::new(Mutex::new(MockState::default()));
    let _server = spawn_mock_server(18120, state.clone()).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    let tag = unique_tag(18120);
    let (order_id, user_id, _inventory_marker) = place_and_pay_order(&db, 18120, tag).await;

    let prewindow_manual_txn = db.begin().await.expect("manual create prewindow txn");
    let prewindow_manual = create_shipment(
        &prewindow_manual_txn,
        Request::new(CreateShipmentRequest {
            order_id,
            shiprocket_order_id: Some("manual_prewindow".to_string()),
            awb_code: Some("AWB-MANUAL-PRE".to_string()),
            carrier: Some("ManualCarrier".to_string()),
            shiprocket_status_id: None,
            shiprocket_status_label: None,
        }),
    )
    .await
    .expect_err("manual shipment creation must fail before earliest_booking_at");
    assert_eq!(prewindow_manual.code(), tonic::Code::FailedPrecondition);
    prewindow_manual_txn.rollback().await.ok();

    let prewindow_admin_txn = db.begin().await.expect("admin shipped prewindow txn");
    let prewindow_admin = admin_mark_order_shipped(
        &prewindow_admin_txn,
        Request::new(AdminMarkOrderShippedRequest {
            order_id,
            awb_code: None,
            carrier: None,
            shiprocket_book: Some(true),
            shiprocket_order_id: None,
            shiprocket_status_id: None,
            shiprocket_status_label: None,
        }),
    )
    .await
    .expect_err("admin booking must fail before earliest_booking_at");
    assert_eq!(prewindow_admin.code(), tonic::Code::FailedPrecondition);
    prewindow_admin_txn.rollback().await.ok();

    make_order_eligible_for_delayed_booking(&db, order_id).await;

    let manual_txn = db.begin().await.expect("manual create txn");
    create_shipment(
        &manual_txn,
        Request::new(CreateShipmentRequest {
            order_id,
            shiprocket_order_id: Some("manual_after_window".to_string()),
            awb_code: Some("AWB-MANUAL-OK".to_string()),
            carrier: Some("ManualCarrier".to_string()),
            shiprocket_status_id: None,
            shiprocket_status_label: None,
        }),
    )
    .await
    .expect("manual path should pass shared validator after booking window opens");
    manual_txn.commit().await.expect("commit manual create");

    assert_eq!(shipment_count(&db, order_id).await, 1);
    assert_eq!(
        order_fulfillment_status(&db, order_id).await,
        "booked",
        "manual shipment creation must update fulfillment_status via shared path"
    );

    let replay_txn = db.begin().await.expect("manual replay txn");
    let replay = create_shipment(
        &replay_txn,
        Request::new(CreateShipmentRequest {
            order_id,
            shiprocket_order_id: Some("manual_replay".to_string()),
            awb_code: Some("AWB-MANUAL-REPLAY".to_string()),
            carrier: Some("ManualCarrier".to_string()),
            shiprocket_status_id: None,
            shiprocket_status_label: None,
        }),
    )
    .await
    .expect_err("existing shipment must fail shared validator");
    assert_eq!(replay.code(), tonic::Code::FailedPrecondition);
    replay_txn.rollback().await.ok();

    let cancel_txn = db.begin().await.expect("cancel booked txn");
    let cancel_err = delete_order(
        &cancel_txn,
        Request::new(DeleteOrderRequest {
            order_id,
            acting_user_id: Some(user_id),
        }),
    )
    .await
    .expect_err("booked orders must stay non-cancellable");
    assert_eq!(cancel_err.code(), tonic::Code::FailedPrecondition);
    cancel_txn.rollback().await.ok();
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and mocked local APIs"]
async fn integration_delayed_worker_duplicate_run_does_not_create_duplicate_shipment() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_delayed_worker_duplicate_run_does_not_create_duplicate_shipment",
    ) {
        return;
    }

    let state = Arc::new(Mutex::new(MockState::default()));
    let _server = spawn_mock_server(18113, state.clone()).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    let tag = unique_tag(18113);
    let (order_id, _user_id, _inventory_marker) = place_and_pay_order(&db, 18113, tag).await;

    ensure_delayed_shipment_booked(&db, order_id).await;
    assert_eq!(shipment_count(&db, order_id).await, 1);

    process_create_shipments_after_cancel_window(&db, 500)
        .await
        .expect("second delayed worker run");
    assert_eq!(
        shipment_count(&db, order_id).await,
        1,
        "duplicate worker run must not create another shipment row"
    );
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and mocked local APIs"]
async fn integration_delayed_worker_skips_orders_without_active_items() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_delayed_worker_skips_orders_without_active_items",
    ) {
        return;
    }

    let state = Arc::new(Mutex::new(MockState::default()));
    let _server = spawn_mock_server(18114, state.clone()).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    let tag = unique_tag(18114);
    let (order_id, user_id, _inventory_marker) = place_and_pay_order(&db, 18114, tag).await;

    let cancel_txn = db.begin().await.expect("cancel txn");
    delete_order(
        &cancel_txn,
        Request::new(DeleteOrderRequest {
            order_id,
            acting_user_id: Some(user_id),
        }),
    )
    .await
    .expect("full cancel should succeed inside cancellation window");
    cancel_txn.commit().await.expect("commit cancel");

    assert_eq!(
        shipment_count(&db, order_id).await,
        0,
        "fully cancelled order must not have shipment before delayed worker"
    );

    make_order_eligible_for_delayed_booking(&db, order_id).await;
    process_create_shipments_after_cancel_window(&db, 500)
        .await
        .expect("run delayed shipment worker");

    assert_eq!(
        shipment_count(&db, order_id).await,
        0,
        "delayed worker must skip orders with no active line items"
    );
    assert_eq!(
        order_fulfillment_status(&db, order_id).await,
        "not_created",
        "skipped cancelled/refunded orders should remain unbooked"
    );
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and mocked local APIs"]
async fn integration_cancel_after_window_is_blocked() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_cancel_after_window_is_blocked",
    ) {
        return;
    }

    let state = Arc::new(Mutex::new(MockState::default()));
    let _server = spawn_mock_server(18111, state.clone()).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    let tag = unique_tag(18111);
    let (order_id, user_id, _inventory_marker) = place_and_pay_order(&db, 18111, tag).await;

    let rewind_txn = db.begin().await.expect("rewind txn");
    rewind_txn
        .execute(Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            "UPDATE Orders SET cancel_window_ends_at = UTC_TIMESTAMP() - INTERVAL 1 MINUTE WHERE OrderID = ?",
            [order_id.into()],
        ))
        .await
        .expect("rewind cancel window");
    rewind_txn.commit().await.expect("commit rewind");

    let cancel_txn = db.begin().await.expect("cancel txn");
    let err = delete_order(
        &cancel_txn,
        Request::new(DeleteOrderRequest {
            order_id,
            acting_user_id: Some(user_id),
        }),
    )
    .await
    .expect_err("cancel should be blocked after cancellation window");
    assert_eq!(err.code(), tonic::Code::FailedPrecondition);
    cancel_txn.rollback().await.ok();
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and mocked local APIs"]
async fn integration_delayed_worker_books_only_active_items_after_partial_cancel() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_delayed_worker_books_only_active_items_after_partial_cancel",
    ) {
        return;
    }

    let state = Arc::new(Mutex::new(MockState::default()));
    let _server = spawn_mock_server(18112, state.clone()).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    let tag = unique_tag(18112);
    let (order_id, user_id, _inventory_marker) = place_and_pay_order(&db, 18112, tag).await;
    let detail_ids = split_order_into_two_lines(&db, order_id).await;
    assert_eq!(detail_ids.len(), 2);

    let cancel_txn = db.begin().await.expect("cancel one txn");
    cancel_order_items(
        &cancel_txn,
        Request::new(CancelOrderItemsRequest {
            order_id,
            order_detail_ids: vec![detail_ids[0]],
            acting_user_id: Some(user_id),
        }),
    )
    .await
    .expect("partial cancel should succeed");
    cancel_txn.commit().await.expect("commit partial cancel");

    make_order_eligible_for_delayed_booking(&db, order_id).await;

    process_create_shipments_after_cancel_window(&db, 25)
        .await
        .expect("run delayed shipment worker");

    let guard = state.lock().expect("lock");
    let item_count = *guard
        .create_order_item_counts
        .last()
        .expect("create order payload should be captured");
    assert_eq!(
        item_count, 1,
        "only active line items should be sent to Shiprocket after partial cancellation"
    );
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and mocked local APIs"]
async fn integration_delayed_worker_skips_unpaid_prepaid_orders() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_delayed_worker_skips_unpaid_prepaid_orders",
    ) {
        return;
    }

    let state = Arc::new(Mutex::new(MockState::default()));
    let _server = spawn_mock_server(18115, state.clone()).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    let tag = unique_tag(18115);
    let (order_id, _user_id, _inventory_marker) =
        place_order_without_payment_verification(&db, 18115, tag, Some("prepaid")).await;

    assert_eq!(
        order_payment_status(&db, order_id).await,
        "pending",
        "new prepaid order should remain pending before payment verification"
    );
    make_order_eligible_for_delayed_booking(&db, order_id).await;
    process_create_shipments_after_cancel_window(&db, 500)
        .await
        .expect("run delayed shipment worker");

    assert_eq!(
        shipment_count(&db, order_id).await,
        0,
        "unpaid prepaid orders must not be booked by delayed worker"
    );
    assert_eq!(order_fulfillment_status(&db, order_id).await, "not_created");
    let guard = state.lock().expect("lock");
    assert_eq!(guard.create_order_calls, 0);
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and mocked local APIs"]
async fn integration_cod_partial_cancel_has_no_refund_attempt_and_books_remaining_items() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_cod_partial_cancel_has_no_refund_attempt_and_books_remaining_items",
    ) {
        return;
    }

    let state = Arc::new(Mutex::new(MockState::default()));
    let _server = spawn_mock_server(18116, state.clone()).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    let tag = unique_tag(18116);
    let (order_id, user_id, inventory_marker) =
        place_order_without_payment_verification(&db, 18116, tag, Some("cod")).await;
    let detail_ids = split_order_into_two_lines(&db, order_id).await;
    assert_eq!(detail_ids.len(), 2);
    let original_total = order_grand_total_minor(&db, order_id).await;

    let cancel_txn = db.begin().await.expect("cancel txn");
    cancel_order_items(
        &cancel_txn,
        Request::new(CancelOrderItemsRequest {
            order_id,
            order_detail_ids: vec![detail_ids[0]],
            acting_user_id: Some(user_id),
        }),
    )
    .await
    .expect("partial COD cancel should succeed");
    cancel_txn.commit().await.expect("commit cancel");

    assert_eq!(
        order_status_name(&db, order_id).await,
        "partially_cancelled"
    );
    assert_eq!(refund_attempt_count(&db, order_id).await, 0);
    assert_eq!(
        order_refund_settlement_status(&db, order_id).await,
        Some("refund_not_applicable".to_string())
    );
    assert!(
        order_grand_total_minor(&db, order_id).await < original_total,
        "COD partial cancel should reduce final payable"
    );
    assert_eq!(
        inventory_available(&db, inventory_marker).await,
        5,
        "single cancelled line should restore one unit of inventory"
    );
    {
        let guard = state.lock().expect("lock");
        assert_eq!(guard.refund_calls, 0, "COD path must not trigger refunds");
        assert_eq!(
            guard.razorpay_order_calls, 0,
            "COD path must not create Razorpay orders"
        );
    }

    make_order_eligible_for_delayed_booking(&db, order_id).await;
    process_create_shipments_after_cancel_window(&db, 25)
        .await
        .expect("run delayed shipment worker");
    assert_eq!(
        shipment_count(&db, order_id).await,
        1,
        "COD partially_cancelled orders should still be bookable after cancel window"
    );
    let guard = state.lock().expect("lock");
    let item_count = *guard
        .create_order_item_counts
        .last()
        .expect("create order payload should be captured");
    assert_eq!(item_count, 1, "only active COD line items should be booked");
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and mocked local APIs"]
async fn integration_cod_full_cancel_sets_payable_zero_without_refund_attempt() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_cod_full_cancel_sets_payable_zero_without_refund_attempt",
    ) {
        return;
    }

    let state = Arc::new(Mutex::new(MockState::default()));
    let _server = spawn_mock_server(18117, state.clone()).await;
    let db = Database::connect(&test_db_url()).await.expect("connect");
    let tag = unique_tag(18117);
    let (order_id, user_id, inventory_marker) =
        place_order_without_payment_verification(&db, 18117, tag, Some("cod")).await;

    let cancel_txn = db.begin().await.expect("cancel txn");
    delete_order(
        &cancel_txn,
        Request::new(DeleteOrderRequest {
            order_id,
            acting_user_id: Some(user_id),
        }),
    )
    .await
    .expect("COD full cancel should succeed inside cancellation window");
    cancel_txn.commit().await.expect("commit cancel");

    assert_eq!(order_status_name(&db, order_id).await, "cancelled");
    assert_eq!(
        order_grand_total_minor(&db, order_id).await,
        0,
        "COD full cancel should zero payable amount"
    );
    assert_eq!(refund_attempt_count(&db, order_id).await, 0);
    assert_eq!(
        order_refund_settlement_status(&db, order_id).await,
        Some("refund_not_applicable".to_string())
    );
    assert_eq!(
        inventory_available(&db, inventory_marker).await,
        6,
        "full cancel should restore reserved inventory"
    );

    make_order_eligible_for_delayed_booking(&db, order_id).await;
    process_create_shipments_after_cancel_window(&db, 25)
        .await
        .expect("run delayed shipment worker");
    assert_eq!(
        shipment_count(&db, order_id).await,
        0,
        "fully cancelled COD order must never be booked"
    );

    let guard = state.lock().expect("lock");
    assert_eq!(guard.refund_calls, 0, "COD flow must not call refund API");
}
