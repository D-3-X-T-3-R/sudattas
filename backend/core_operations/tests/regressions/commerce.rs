//! Opt-in, isolated-storage regressions. See README.md before running.
//! Reuses the launch-concurrency seeds, real handlers, and gateway cache helper.
use super::*;
use core_db_entities::entity::{invoices, outbox_events, refunds, transactions, webhook_events};
use core_operations::handlers::webhooks::replay_webhook_by_id;
use core_operations::procedures::orders::place_order;
use proto::proto::core::PlaceOrderRequest;
use sea_orm::{DatabaseConnection, PaginatorTrait};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use warp::Filter;

#[path = "razorpay_sandbox.rs"]
mod razorpay_sandbox;

const SECRET: &str = "regression-only-checkout-secret";
const WEBHOOK_SECRET: &str = "regression-only-webhook-secret";
static ENV_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

// Local runs read only backend/.env. GitHub Actions may use its ephemeral MySQL
// service through TEST_DATABASE_URL when that file is absent. Never use DATABASE_URL.
fn repo_setting(name: &str) -> Option<String> {
    dotenvy::from_path_iter(concat!(env!("CARGO_MANIFEST_DIR"), "/../.env"))
        .ok()?
        .filter_map(Result::ok)
        .find_map(|(key, value)| (key == name).then_some(value))
}

async fn isolated_db() -> DatabaseConnection {
    assert_eq!(
        std::env::var("COMMERCE_TEST_ISOLATED_STORAGE").as_deref(),
        Ok("1"),
        "explicit confirmation of an isolated migrated MySQL schema is required"
    );
    let env_file = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.env"));
    let github_ci = std::env::var("CI").as_deref() == Ok("true")
        && std::env::var("GITHUB_ACTIONS").as_deref() == Ok("true");
    let ci_fallback = github_ci && !env_file.exists();
    let url = if ci_fallback {
        std::env::var("TEST_DATABASE_URL").expect("CI needs TEST_DATABASE_URL")
    } else {
        repo_setting("TEST_DATABASE_URL").expect("backend/.env needs TEST_DATABASE_URL")
    };
    assert!(
        url.starts_with("mysql://"),
        "transaction regressions require MySQL"
    );
    // Reject aliases to the live schema too (different users/hosts are not isolation).
    let schema = |u: &str| {
        u.rsplit('/')
            .next()
            .unwrap_or("")
            .split('?')
            .next()
            .unwrap_or("")
            .to_owned()
    };
    if ci_fallback {
        let authority = url.split('/').nth(2).unwrap_or("");
        assert!(
            authority.ends_with("@127.0.0.1:3306") && schema(&url) == "SUDATTAS",
            "CI fallback must target the ephemeral local SUDATTAS schema"
        );
    }
    for live in [
        repo_setting("DATABASE_URL"),
        std::env::var("DATABASE_URL").ok(),
    ]
    .into_iter()
    .flatten()
    {
        assert_ne!(
            url, live,
            "TEST_DATABASE_URL must differ from the live database"
        );
        assert_ne!(
            schema(&url),
            schema(&live),
            "test schema must have a distinct name"
        );
    }
    Database::connect(url)
        .await
        .unwrap_or_else(|_| panic!("isolated MySQL connection failed"))
}

struct Env(Vec<(String, Option<String>)>);
impl Env {
    fn new() -> Self {
        Self(Vec::new())
    }
    fn set(&mut self, key: &str, value: impl AsRef<str>) {
        self.0.push((key.into(), std::env::var(key).ok()));
        std::env::set_var(key, value.as_ref());
    }
}
impl Drop for Env {
    fn drop(&mut self) {
        for (key, value) in self.0.iter().rev() {
            if let Some(value) = value {
                std::env::set_var(key, value);
            } else {
                std::env::remove_var(key);
            }
        }
    }
}

#[derive(Default)]
struct ProviderState {
    orders: Vec<String>,
    refunds: Vec<String>,
    accepted_refunds: std::collections::HashMap<String, String>,
    payment_status: String,
    payment_order: String,
    reject_pin: bool,
    quote_calls: usize,
    timeout_order_once: bool,
    timeout_refund_once: bool,
    refund_pending: bool,
}
struct Provider {
    state: Arc<Mutex<ProviderState>>,
    task: tokio::task::JoinHandle<()>,
    _env: Env,
}
impl Drop for Provider {
    fn drop(&mut self) {
        self.task.abort();
    }
}

impl Provider {
    async fn start() -> Self {
        let state = Arc::new(Mutex::new(ProviderState {
            payment_status: "captured".into(),
            ..Default::default()
        }));
        let shared = state.clone();
        let route = warp::path::full()
            .and(warp::method())
            .and(warp::header::optional::<String>("x-refund-idempotency"))
            .and(warp::body::bytes())
            .and_then(
                move |path: warp::path::FullPath,
                      method: warp::http::Method,
                      key: Option<String>,
                      _body| {
                    let shared = shared.clone();
                    async move {
                        let (body, timeout) = {
                            let mut s = shared.lock().unwrap();
                            let p = path.as_str().trim_end_matches('/');
                            if p == "/v1/orders" && method == warp::http::Method::POST {
                                let id = format!("order_{}", uuid::Uuid::new_v4().simple());
                                s.orders.push(id.clone()); // Acceptance happens BEFORE response loss.
                                let timeout = std::mem::take(&mut s.timeout_order_once);
                                (json!({"id":id}), timeout)
                            } else if p.ends_with("/refund") {
                                let key = key.unwrap_or_default();
                                s.refunds.push(key.clone());
                                let id = s
                                    .accepted_refunds
                                    .entry(key)
                                    .or_insert_with(|| {
                                        format!("rfnd_{}", uuid::Uuid::new_v4().simple())
                                    })
                                    .clone();
                                let status = if s.refund_pending {
                                    "pending"
                                } else {
                                    "processed"
                                };
                                let body = json!({"id":id,"status":status});
                                let timeout = std::mem::take(&mut s.timeout_refund_once);
                                (body, timeout)
                            } else if p.starts_with("/v1/payments/") {
                                (
                                    json!({"id":p.rsplit('/').next(),"order_id":s.payment_order,
                                "amount":1000,"currency":"INR","status":s.payment_status,
                                "captured":s.payment_status == "captured"}),
                                    false,
                                )
                            } else if p.ends_with("/auth/login") {
                                (json!({"token":"regression-token"}), false)
                            } else if p.ends_with("/courier/serviceability") {
                                s.quote_calls += 1;
                                let couriers = if s.reject_pin {
                                    json!([])
                                } else {
                                    json!([{"courier_company_id":777,"courier_name":"Fixture",
                                    "freight_charge":1,"etd":"3 days"}])
                                };
                                (
                                    json!({"data":{"available_courier_companies":couriers}}),
                                    false,
                                )
                            } else {
                                panic!("unexpected provider path: {p}");
                            }
                        };
                        if timeout {
                            // Intentionally never respond: exercise the real client's 15s timeout,
                            // not a synthetic HTTP error. Server task is aborted by the fixture.
                            std::future::pending::<()>().await;
                        }
                        Ok::<_, std::convert::Infallible>(warp::reply::json(&body))
                    }
                },
            );
        let (addr, server) = warp::serve(route).bind_ephemeral(([127, 0, 0, 1], 0));
        let mut env = Env::new();
        env.set("RAZORPAY_KEY_ID", "rzp_test_regression");
        env.set("RAZORPAY_KEY_SECRET", SECRET);
        env.set("RAZORPAY_WEBHOOK_SECRET", WEBHOOK_SECRET);
        env.set("RAZORPAY_API_BASE", format!("http://{addr}/v1"));
        env.set("SHIPROCKET_API_BASE", format!("http://{addr}/v1/external"));
        env.set("SHIPROCKET_EMAIL", "fixture@example.invalid");
        env.set("SHIPROCKET_PASSWORD", "fixture-only");
        env.set("SHIPROCKET_PICKUP_POSTCODE", "560001");
        env.set("SHIPROCKET_PICKUP_LOCATION", "Fixture");
        env.set("FREE_SHIPPING_THRESHOLD_MINOR", "999");
        env.set("REFUND_MAX_ATTEMPTS", "5");
        Self {
            state,
            task: tokio::spawn(server),
            _env: env,
        }
    }
}

fn signed(body: &str, secret: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(body.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

async fn checkout_fixture(db: &DatabaseConnection) -> (PlaceOrderRequest, i64) {
    let txn = db.begin().await.unwrap();
    let tag = uuid::Uuid::new_v4().simple().to_string();
    let user_id = seed_user(&txn, &tag).await;
    ensure_order_status_row(&txn, "pending").await;
    ensure_order_status_row(&txn, "confirmed").await;
    let variant = seed_variant(&txn, &tag, 1000, 5).await;
    let address = seed_address(&txn, user_id, &tag).await;
    let row = core_operations::handlers::cart::create_cart_item(
        &txn,
        Request::new(CreateCartItemRequest {
            user_id: Some(user_id),
            session_id: None,
            variant_id: variant,
            quantity: 1,
        }),
    )
    .await
    .unwrap()
    .into_inner()
    .items
    .remove(0);
    txn.commit().await.unwrap();
    (
        PlaceOrderRequest {
            user_id,
            shipping_address_id: address,
            selected_cart_ids: vec![row.cart_id],
            coupon_code: None,
            payment_mode: Some("prepaid".into()),
        },
        variant,
    )
}

async fn checkout(
    db: &DatabaseConnection,
    payload: PlaceOrderRequest,
    key: &str,
) -> Result<i64, tonic::Status> {
    let mut req = Request::new(payload);
    req.metadata_mut()
        .insert("idempotency-key", key.parse().unwrap());
    Ok(place_order(db, req).await?.into_inner().items[0].order_id)
}

async fn count(db: &DatabaseConnection, sql: &str, id: i64) -> i64 {
    db.query_one(Statement::from_sql_and_values(
        sea_orm::DbBackend::MySql,
        sql,
        [id.into()],
    ))
    .await
    .unwrap()
    .unwrap()
    .try_get("", "n")
    .unwrap()
}
async fn order_count(db: &DatabaseConnection, user: i64) -> i64 {
    count(
        db,
        "SELECT COUNT(*) AS n FROM Orders WHERE UserID = ?",
        user,
    )
    .await
}
async fn stock(db: &DatabaseConnection, variant: i64) -> i64 {
    inventory::Entity::find()
        .filter(inventory::Column::VariantId.eq(variant))
        .one(db)
        .await
        .unwrap()
        .unwrap()
        .quantity_available
        .unwrap()
}

async fn cached_checkout(
    db: &DatabaseConnection,
    redis: &str,
    payload: PlaceOrderRequest,
    key: &str,
) -> Result<i64, graphql::resolvers::error::GqlError> {
    // Same gateway helper and operation name as the resolver, not a fake cache.
    graphql::idempotency::with_idempotency(Some(redis), "place_order", Some(key), || async {
        checkout(db, payload, key)
            .await
            .map_err(graphql::resolvers::error::map_err)
    })
    .await
}
async fn isolated_redis() -> String {
    assert_eq!(
        std::env::var("COMMERCE_TEST_ISOLATED_REDIS").as_deref(),
        Ok("1")
    );
    let url = std::env::var("TEST_REDIS_URL").expect("dedicated TEST_REDIS_URL required");
    assert!(
        url.starts_with("redis://127.0.0.1:") || url.starts_with("redis://localhost:"),
        "regression Redis must be explicitly local and isolated"
    );
    assert!(repo_setting("REDIS_URL").as_deref() != Some(url.as_str()));
    let client = redis::Client::open(url.as_str()).unwrap();
    let mut conn = client.get_multiplexed_async_connection().await.unwrap();
    let pong: String = redis::cmd("PING").query_async(&mut conn).await.unwrap();
    assert_eq!(pong, "PONG");
    url
}

#[tokio::test]
#[ignore = "isolated MySQL + Redis; intended to expose F1"]
async fn f1_gateway_cache_never_crosses_customer_boundary() {
    let _lock = ENV_LOCK.lock().await;
    let db = isolated_db().await;
    let _provider = Provider::start().await;
    let redis = isolated_redis().await;
    let (a, _) = checkout_fixture(&db).await;
    let (b, _) = checkout_fixture(&db).await;
    let key = uuid::Uuid::new_v4().to_string();
    let a_id = cached_checkout(&db, &redis, a.clone(), &key).await.unwrap();
    // Simulate an entry left by the old gateway. The fixed gateway must ignore
    // it, even though it contains A's successful response under this shared key.
    let mut conn = redis::Client::open(redis.as_str())
        .unwrap()
        .get_multiplexed_async_connection()
        .await
        .unwrap();
    let _: () = redis::cmd("SETEX")
        .arg(format!("idempotency:place_order:{key}"))
        .arg(60)
        .arg(serde_json::to_string(&a_id).unwrap())
        .query_async(&mut conn)
        .await
        .unwrap();
    let value: String = redis::cmd("GET")
        .arg(format!("idempotency:place_order:{key}"))
        .query_async(&mut conn)
        .await
        .unwrap();
    assert_eq!(serde_json::from_str::<i64>(&value).unwrap(), a_id);
    let result = cached_checkout(&db, &redis, b.clone(), &key).await;
    let _: i64 = redis::cmd("DEL")
        .arg(format!("idempotency:place_order:{key}"))
        .query_async(&mut conn)
        .await
        .unwrap();
    if let Ok(id) = result {
        assert_ne!(id, a_id, "B received A's cached order");
        assert_eq!(
            orders::Entity::find_by_id(id)
                .one(&db)
                .await
                .unwrap()
                .unwrap()
                .user_id,
            b.user_id
        );
    }
    assert_eq!(order_count(&db, a.user_id).await, 1);
    assert!(order_count(&db, b.user_id).await <= 1);
}

#[tokio::test]
#[ignore = "isolated MySQL + Redis; intended to expose F1 payload aliasing"]
async fn f1_changed_payload_conflicts_even_on_cache_hit() {
    let _lock = ENV_LOCK.lock().await;
    let db = isolated_db().await;
    let _provider = Provider::start().await;
    let redis = isolated_redis().await;
    let (a, _) = checkout_fixture(&db).await;
    let key = uuid::Uuid::new_v4().to_string();
    let id = cached_checkout(&db, &redis, a.clone(), &key).await.unwrap();
    let mut legacy = redis::Client::open(redis.as_str())
        .unwrap()
        .get_multiplexed_async_connection()
        .await
        .unwrap();
    let _: () = redis::cmd("SETEX")
        .arg(format!("idempotency:place_order:{key}"))
        .arg(60)
        .arg(serde_json::to_string(&id).unwrap())
        .query_async(&mut legacy)
        .await
        .unwrap();
    let mut changed = a.clone();
    changed.payment_mode = Some("cod".into());
    let result = cached_checkout(&db, &redis, changed, &key).await;
    let mut conn = redis::Client::open(redis.as_str())
        .unwrap()
        .get_multiplexed_async_connection()
        .await
        .unwrap();
    let _: i64 = redis::cmd("DEL")
        .arg(format!("idempotency:place_order:{key}"))
        .query_async(&mut conn)
        .await
        .unwrap();
    assert_eq!(order_count(&db, a.user_id).await, 1);
    let err = result.expect_err("a different payload must not return cached success");
    assert!(matches!(
        err.code,
        graphql::resolvers::error::Code::AlreadyExists
            | graphql::resolvers::error::Code::FailedPrecondition
            | graphql::resolvers::error::Code::Aborted
    ));
}

#[tokio::test]
#[ignore = "isolated MySQL; intended to expose F5"]
async fn f5_lost_response_retry_after_cart_deletion() {
    let _lock = ENV_LOCK.lock().await;
    let db = isolated_db().await;
    let provider = Provider::start().await;
    let (request, variant) = checkout_fixture(&db).await;
    let key = uuid::Uuid::new_v4().to_string();
    let _ = checkout(&db, request.clone(), &key).await.unwrap(); // discard response
    assert_eq!(
        cart::Entity::find()
            .filter(cart::Column::UserId.eq(request.user_id))
            .count(&db)
            .await
            .unwrap(),
        0
    );
    let committed = orders::Entity::find()
        .filter(orders::Column::UserId.eq(request.user_id))
        .one(&db)
        .await
        .unwrap()
        .unwrap()
        .order_id;
    // Calling the core entry point bypasses Redis entirely; durable recovery is required.
    let retry = checkout(&db, request.clone(), &key).await;
    assert_eq!(order_count(&db, request.user_id).await, 1);
    assert_eq!(stock(&db, variant).await, 4);
    assert_eq!(provider.state.lock().unwrap().orders.len(), 1);
    assert_eq!(
        retry.expect("committed request must survive an empty cart"),
        committed
    );
}

struct PaymentFixture {
    order: i64,
    user: i64,
    intent: i64,
    gateway: String,
    payment: String,
}
async fn payment_fixture(db: &DatabaseConnection) -> PaymentFixture {
    let txn = db.begin().await.unwrap();
    let tag = uuid::Uuid::new_v4().simple().to_string();
    let user = seed_user(&txn, &tag).await;
    for name in [
        "pending",
        "confirmed",
        "refunded",
        "cancelled",
        "needs_review",
    ] {
        ensure_order_status_row(&txn, name).await;
    }
    let variant = seed_variant(&txn, &tag, 1000, 5).await;
    let (order, _) = seed_order_with_line(&txn, &tag, user, variant, 1, None).await;
    txn.execute(Statement::from_sql_and_values(
        sea_orm::DbBackend::MySql,
        "UPDATE Orders SET payment_method = 'prepaid' WHERE OrderID = ?",
        [order.into()],
    ))
    .await
    .unwrap();
    let gateway = format!("order_{tag}");
    let response = create_payment_intent(
        &txn,
        Request::new(CreatePaymentIntentRequest {
            razorpay_order_id: Some(gateway.clone()),
            order_id: order,
            user_id: user,
            amount_paise: 1000,
            currency: Some("INR".into()),
        }),
    )
    .await
    .unwrap()
    .into_inner();
    let intent = response.items[0].intent_id;
    txn.commit().await.unwrap();
    PaymentFixture {
        order,
        user,
        intent,
        gateway,
        payment: format!("pay_{tag}"),
    }
}
fn capture_body(f: &PaymentFixture) -> Value {
    json!({"event":"payment.captured","payload":{"payment":{"entity":{
        "id":f.payment,"order_id":f.gateway,"amount":1000,"currency":"INR","status":"captured","captured":true}}}})
}
fn webhook_request(body: Value, event_id: &str) -> IngestWebhookRequest {
    let event = body["event"].as_str().unwrap().to_owned();
    let raw = body.to_string();
    IngestWebhookRequest {
        provider: "razorpay".into(),
        event_type: event,
        webhook_id: event_id.into(),
        provider_event_id: Some(event_id.into()),
        signature_verified: true,
        raw_signature: Some(signed(&raw, WEBHOOK_SECRET)),
        payload_json: raw,
    }
}
async fn ingest(
    db: &DatabaseConnection,
    req: IngestWebhookRequest,
) -> Result<proto::proto::core::WebhookEventsResponse, tonic::Status> {
    // Match the gRPC entry point's commit-on-Ok semantics, including Failed event rows.
    let txn = db.begin().await.unwrap();
    let result = ingest_webhook(&txn, Request::new(req)).await;
    match result {
        Ok(response) => {
            txn.commit().await.unwrap();
            Ok(response.into_inner())
        }
        Err(err) => {
            txn.rollback().await.unwrap();
            Err(err)
        }
    }
}
async fn callback(db: &DatabaseConnection, f: &PaymentFixture) -> Result<(), tonic::Status> {
    let txn = db.begin().await.unwrap();
    let result = core_operations::handlers::payment_intents::verify_razorpay_payment(
        &txn,
        Request::new(VerifyRazorpayPaymentRequest {
            order_id: f.order,
            razorpay_order_id: f.gateway.clone(),
            razorpay_payment_id: f.payment.clone(),
            razorpay_signature: compute_signature(&f.gateway, &f.payment, SECRET),
        }),
    )
    .await;
    match result {
        Ok(_) => {
            txn.commit().await.unwrap();
            Ok(())
        }
        Err(err) => {
            txn.rollback().await.unwrap();
            Err(err)
        }
    }
}
async fn effects(db: &DatabaseConnection, f: &PaymentFixture) -> (u64, u64, u64) {
    let ledger = transactions::Entity::find()
        .filter(transactions::Column::UserId.eq(f.user))
        .filter(transactions::Column::Type.eq("razorpay_payment"))
        .all(db)
        .await
        .unwrap();
    if !ledger.is_empty() {
        assert_eq!(
            ledger
                .iter()
                .map(|r| r.amount)
                .sum::<rust_decimal::Decimal>(),
            rust_decimal::Decimal::new(1000, 2),
            "ledger must contain exactly the paid amount"
        );
    }
    (
        ledger.len() as u64,
        invoices::Entity::find()
            .filter(invoices::Column::OrderId.eq(f.order))
            .count(db)
            .await
            .unwrap(),
        outbox_events::Entity::find()
            .filter(outbox_events::Column::AggregateId.eq(f.order.to_string()))
            .filter(outbox_events::Column::AggregateType.eq("order"))
            .filter(outbox_events::Column::EventType.eq("PaymentCaptured"))
            .count(db)
            .await
            .unwrap(),
    )
}
async fn assert_unpaid(db: &DatabaseConnection, f: &PaymentFixture) {
    let order = orders::Entity::find_by_id(f.order)
        .one(db)
        .await
        .unwrap()
        .unwrap();
    assert_ne!(order.payment_status, Some(PaymentStatus::Captured));
    assert_eq!(order.fulfillment_status, FulfillmentStatus::NotCreated);
    assert_eq!(effects(db, f).await, (0, 0, 0));
    assert_eq!(
        count(
            db,
            "SELECT COUNT(*) AS n FROM Shipments WHERE order_id = ?",
            f.order
        )
        .await,
        0
    );
}
async fn assert_paid_once(db: &DatabaseConnection, f: &PaymentFixture) {
    assert_eq!(
        orders::Entity::find_by_id(f.order)
            .one(db)
            .await
            .unwrap()
            .unwrap()
            .payment_status,
        Some(PaymentStatus::Captured)
    );
    assert_eq!(effects(db, f).await, (1, 1, 1));
}
fn assert_event_processed(response: proto::proto::core::WebhookEventsResponse) {
    assert_eq!(response.items.len(), 1);
    assert_eq!(
        response.items[0].status, "processed",
        "Ok alone does not establish business success"
    );
}

#[tokio::test]
#[ignore = "isolated MySQL; intended to expose F2"]
async fn f2_authorised_signature_is_not_capture() {
    let _lock = ENV_LOCK.lock().await;
    let db = isolated_db().await;
    let provider = Provider::start().await;
    provider.state.lock().unwrap().payment_status = "authorized".into();
    let f = payment_fixture(&db).await;
    provider.state.lock().unwrap().payment_order = f.gateway.clone();
    let _ = callback(&db, &f).await; // Either pending acknowledgement or explicit rejection is OK.
    let txn = db.begin().await.unwrap();
    // Advance only the supplied clock, so cancellation-window timing cannot hide
    // an incorrectly permitted unpaid shipment. This validator performs no I/O to providers.
    let booking = core_operations::handlers::shipments::validate_order_can_be_booked(
        &txn,
        f.order,
        Utc::now() + Duration::days(365),
        false,
    )
    .await;
    txn.rollback().await.unwrap();
    assert!(
        booking.is_err(),
        "authorised-only payment must not pass fulfilment eligibility"
    );
    assert_unpaid(&db, &f).await;
    provider.state.lock().unwrap().payment_status = "captured".into();
    let event = format!("capture_{}", f.order);
    assert_event_processed(
        ingest(&db, webhook_request(capture_body(&f), &event))
            .await
            .unwrap(),
    );
    assert_event_processed(
        ingest(&db, webhook_request(capture_body(&f), &event))
            .await
            .unwrap(),
    );
    assert_paid_once(&db, &f).await;
}

// Extend the old overlap scenario using its existing seeds, now with a valid raw
// signature, a coordinated start, and accounting/outbox assertions.
pub(super) async fn overlap(mode: u8) {
    let _lock = ENV_LOCK.lock().await;
    // ingest_webhook logs a sanitized Status, while map_db_error_to_status logs
    // the actual constraint through log::error!. try_init also installs the log
    // bridge; a scoped tracing subscriber alone loses that diagnostic. This is
    // test-process-only and restricted to errors/warnings for these handlers.
    let _ = tracing_subscriber::fmt()
        .with_env_filter("core_operations::handlers::webhooks=warn,core_operations::handlers::db_errors=error,core_operations::handlers::invoices=warn")
        .with_ansi(false)
        .with_writer(std::io::stderr)
        .try_init();
    let db = isolated_db().await;
    let provider = Provider::start().await;
    let f = payment_fixture(&db).await;
    provider.state.lock().unwrap().payment_order = f.gateway.clone();
    let req = webhook_request(capture_body(&f), &format!("overlap_{}", f.order));
    if mode == 0 {
        callback(&db, &f).await.unwrap();
        assert_event_processed(ingest(&db, req).await.unwrap());
    } else if mode == 1 {
        assert_event_processed(ingest(&db, req).await.unwrap());
        callback(&db, &f).await.unwrap();
    } else {
        let barrier = tokio::sync::Barrier::new(2);
        let (a, b) = tokio::join!(
            async {
                barrier.wait().await;
                callback(&db, &f).await
            },
            async {
                barrier.wait().await;
                ingest(&db, req).await
            }
        );
        a.unwrap();
        assert_event_processed(b.unwrap());
    }
    assert_paid_once(&db, &f).await;
}
#[tokio::test]
#[ignore = "isolated MySQL"]
async fn callback_then_webhook_once() {
    overlap(0).await;
}
#[tokio::test]
#[ignore = "isolated MySQL"]
async fn webhook_then_callback_once() {
    overlap(1).await;
}

#[tokio::test]
#[ignore = "isolated MySQL; intended to expose F3"]
async fn f3_tampered_event_cannot_be_blessed_by_replay() {
    let _lock = ENV_LOCK.lock().await;
    let db = isolated_db().await;
    let _provider = Provider::start().await;
    let f = payment_fixture(&db).await;
    let key = format!("tampered_{}", f.order);
    let mut req = webhook_request(capture_body(&f), &key);
    req.payload_json.push(' '); // JSON semantics unchanged; raw bytes/signature no longer match.
    let result = ingest(&db, req).await;
    if let Ok(response) = result {
        assert!(response.items.iter().all(|e| e.status != "processed"));
    }
    assert_unpaid(&db, &f).await;
    let txn = db.begin().await.unwrap();
    let replay = replay_webhook_by_id(&txn, &key).await;
    if replay.is_ok() {
        txn.commit().await.unwrap();
    } else {
        txn.rollback().await.unwrap();
    }
    assert_unpaid(&db, &f).await;
}

#[tokio::test]
#[ignore = "isolated MySQL with TRIGGER privilege; intended to expose F3"]
async fn f3_halfway_failure_rolls_back_and_replay_converges() {
    let _lock = ENV_LOCK.lock().await;
    let db = isolated_db().await;
    let _provider = Provider::start().await;
    let f = payment_fixture(&db).await;
    let trigger = format!("regression_invoice_{}", f.order);
    // Invoice insertion follows payment-intent and order financial-state updates.
    db.execute_unprepared(&format!("CREATE TRIGGER `{trigger}` BEFORE INSERT ON Invoices FOR EACH ROW BEGIN IF NEW.order_id = {} THEN SET @regression_invoice_failed = 1; SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = 'regression invoice failure'; END IF; END", f.order)).await.unwrap();
    let key = format!("halfway_{}", f.order);
    let txn = db.begin().await.unwrap();
    txn.execute_unprepared("SET @regression_invoice_failed = 0")
        .await
        .unwrap();
    let result = ingest_webhook(&txn, Request::new(webhook_request(capture_body(&f), &key))).await;
    // Session variables survive rollback. Read on the SAME connection to prove this
    // specific fault was reached, rather than accepting any earlier fixture error.
    let fault_seen: i64 = txn
        .query_one(Statement::from_string(
            sea_orm::DbBackend::MySql,
            "SELECT CAST(@regression_invoice_failed AS SIGNED) AS n".to_owned(),
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "n")
        .unwrap();
    if result.is_ok() {
        txn.commit().await.unwrap();
    } else {
        txn.rollback().await.unwrap();
    }
    // Remove injected failure before any behavioural assertion, including expected failures.
    db.execute_unprepared(&format!("DROP TRIGGER `{trigger}`"))
        .await
        .unwrap();
    assert_eq!(
        fault_seen, 1,
        "test must reach the injected financial-finalisation failure"
    );
    if result.is_ok() {
        let event = webhook_events::Entity::find()
            .filter(webhook_events::Column::WebhookId.eq(&key))
            .one(&db)
            .await
            .unwrap()
            .expect("successful acknowledgement needs a durable retry record");
        assert_eq!(event.status, Some(PaymentIntentStatus::Failed));
    }
    assert_unpaid(&db, &f).await;
    let intent = payment_intents::Entity::find_by_id(f.intent)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        intent.status,
        PaymentIntentStatus::Pending,
        "early financial write must roll back"
    );
    // Redelivery uses the exact original signed event, not a manufactured verified flag.
    assert_event_processed(
        ingest(&db, webhook_request(capture_body(&f), &key))
            .await
            .unwrap(),
    );
    assert_event_processed(
        ingest(&db, webhook_request(capture_body(&f), &key))
            .await
            .unwrap(),
    );
    assert_paid_once(&db, &f).await;
}

async fn refund_event(
    db: &DatabaseConnection,
    f: &PaymentFixture,
    id: &str,
    amount: i64,
    status: &str,
) {
    let event = if status == "processed" {
        "refund.processed"
    } else {
        "refund.created"
    };
    let body = json!({"event":event,"payload":{"refund":{"entity":{
        "id":id,"payment_id":f.payment,"amount":amount,"currency":"INR","status":status}}}});
    let key = format!("evt_{}", uuid::Uuid::new_v4().simple());
    let response = ingest(db, webhook_request(body, &key)).await.unwrap();
    assert!(
        response.items.iter().all(|r| r.status == "processed"),
        "refund event must converge, not merely return Ok"
    );
}
async fn refund_totals(db: &DatabaseConnection, f: &PaymentFixture, rows: usize, total: i64) {
    let rows_found = refunds::Entity::find()
        .filter(refunds::Column::OrderId.eq(f.order))
        .all(db)
        .await
        .unwrap();
    assert_eq!(rows_found.len(), rows);
    assert!(rows_found
        .iter()
        .all(|r| r.status == Some(PaymentIntentStatus::Processed)));
    assert_eq!(
        rows_found
            .iter()
            .map(|r| i64::from(r.amount_paise))
            .sum::<i64>(),
        total
    );
}
async fn paid_fixture(db: &DatabaseConnection) -> PaymentFixture {
    let f = payment_fixture(db).await;
    ingest(
        db,
        webhook_request(capture_body(&f), &format!("paid_{}", f.order)),
    )
    .await
    .unwrap();
    assert_paid_once(db, &f).await;
    f
}
#[tokio::test]
#[ignore = "isolated MySQL; intended to expose F4"]
async fn f4_pending_then_processed_promotes_same_refund() {
    let _lock = ENV_LOCK.lock().await;
    let db = isolated_db().await;
    let _p = Provider::start().await;
    let f = paid_fixture(&db).await;
    let id = format!("rfnd_{}", f.order);
    refund_event(&db, &f, &id, 400, "pending").await;
    refund_event(&db, &f, &id, 400, "processed").await;
    refund_totals(&db, &f, 1, 400).await;
}
#[tokio::test]
#[ignore = "isolated MySQL; intended to expose F4"]
async fn f4_stale_pending_cannot_downgrade_processed() {
    let _lock = ENV_LOCK.lock().await;
    let db = isolated_db().await;
    let _p = Provider::start().await;
    let f = paid_fixture(&db).await;
    let id = format!("rfnd_{}", f.order);
    refund_event(&db, &f, &id, 400, "processed").await;
    refund_event(&db, &f, &id, 400, "pending").await;
    refund_totals(&db, &f, 1, 400).await;
}
#[tokio::test]
#[ignore = "isolated MySQL; intended to expose F4"]
async fn f4_duplicate_processed_after_full_refund_is_successful_noop() {
    let _lock = ENV_LOCK.lock().await;
    let db = isolated_db().await;
    let _p = Provider::start().await;
    let f = paid_fixture(&db).await;
    let id = format!("rfnd_{}", f.order);
    refund_event(&db, &f, &id, 1000, "processed").await;
    refund_event(&db, &f, &id, 1000, "processed").await;
    refund_totals(&db, &f, 1, 1000).await;
}
#[tokio::test]
#[ignore = "isolated MySQL; preserves distinct-refund accounting"]
async fn f4_two_partial_refunds_count_each_amount_once() {
    let _lock = ENV_LOCK.lock().await;
    let db = isolated_db().await;
    let _p = Provider::start().await;
    let f = paid_fixture(&db).await;
    for (suffix, amount) in [("a", 300), ("b", 400), ("a", 300), ("b", 400)] {
        refund_event(
            &db,
            &f,
            &format!("rfnd_{}_{suffix}", f.order),
            amount,
            "processed",
        )
        .await;
    }
    refund_totals(&db, &f, 2, 700).await;
}

#[tokio::test]
#[ignore = "isolated MySQL; real client timeout against loopback fake (~15s)"]
async fn accepted_order_timeout_is_recovered_or_explicitly_unresolved() {
    let _lock = ENV_LOCK.lock().await;
    let db = isolated_db().await;
    let p = Provider::start().await;
    p.state.lock().unwrap().timeout_order_once = true;
    let (req, variant) = checkout_fixture(&db).await;
    let key = uuid::Uuid::new_v4().to_string();
    assert!(checkout(&db, req.clone(), &key).await.is_err());
    assert_eq!(
        p.state.lock().unwrap().orders.len(),
        1,
        "provider accepted the lost operation"
    );
    let retry = checkout(&db, req.clone(), &key).await;
    assert_eq!(
        p.state.lock().unwrap().orders.len(),
        1,
        "must not blindly submit a second external order"
    );
    if let Ok(order) = retry {
        let intent = payment_intents::Entity::find()
            .filter(payment_intents::Column::OrderId.eq(order))
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(intent.razorpay_order_id, p.state.lock().unwrap().orders[0]);
        assert_eq!(order_count(&db, req.user_id).await, 1);
        assert_eq!(stock(&db, variant).await, 4);
    } else {
        assert_eq!(order_count(&db, req.user_id).await, 0);
        assert_eq!(stock(&db, variant).await, 5);
        let row = db.query_one(Statement::from_sql_and_values(sea_orm::DbBackend::MySql,
            "SELECT state,fingerprint FROM CheckoutAttemptsV2 WHERE customer_id=? AND operation='place_order' AND request_key=?",
            [req.user_id.into(),key.into()])).await.unwrap().unwrap();
        assert_eq!(
            row.try_get::<String>("", "state").unwrap(),
            "needs_review",
            "generic Failed is not an explicit durable unresolved/reconciliation state"
        );
        assert!(
            !row.try_get::<String>("", "fingerprint").unwrap().is_empty(),
            "reconciliation needs the original request identity"
        );
    }
}

#[tokio::test]
#[ignore = "isolated MySQL; intended to expose F7"]
async fn f7_free_shipping_still_requires_serviceable_pin() {
    let _lock = ENV_LOCK.lock().await;
    let db = isolated_db().await;
    let p = Provider::start().await;
    p.state.lock().unwrap().reject_pin = true;
    let (req, variant) = checkout_fixture(&db).await;
    let result = checkout(&db, req.clone(), &uuid::Uuid::new_v4().to_string()).await;
    {
        let state = p.state.lock().unwrap();
        assert_eq!(
            state.orders.len(),
            0,
            "unsupported destination must be blocked before Razorpay"
        );
        assert!(
            state.quote_calls > 0,
            "free shipping does not imply serviceability"
        );
    }
    assert!(result.is_err());
    assert_eq!(order_count(&db, req.user_id).await, 0);
    assert_eq!(stock(&db, variant).await, 5);
}

async fn queue_refund(db: &DatabaseConnection, f: &PaymentFixture) -> i64 {
    // The public worker scans the whole schema. Refuse pre-existing runnable jobs;
    // never neutralize or delete another fixture's (or another user's) work.
    assert_eq!(count(db, "SELECT COUNT(*) AS n FROM RefundAttempts WHERE status IN ('pending_external','submitted','submitting') AND order_id <> ?", f.order).await,0,
        "refund-worker tests require an exclusive schema without unrelated runnable jobs");
    let txn = db.begin().await.unwrap();
    core_operations::order_state_machine::transition_order_status(
        &txn,
        f.order,
        core_operations::order_state_machine::OrderState::Cancelled,
        "regression_cancel",
        "system",
        None,
        None,
    )
    .await
    .unwrap();
    core_operations::cancellation_saga::run_order_settlement(&txn, f.order)
        .await
        .unwrap();
    txn.commit().await.unwrap();
    let rows = core_db_entities::entity::refund_attempts::Entity::find()
        .filter(core_db_entities::entity::refund_attempts::Column::OrderId.eq(f.order))
        .all(db)
        .await
        .unwrap();
    assert_eq!(rows.len(), 1);
    rows[0].attempt_id
}
async fn worker(db: &DatabaseConnection) {
    core_operations::cancellation_saga::process_pending_refund_attempts(db, 1)
        .await
        .unwrap();
}
async fn attempt(
    db: &DatabaseConnection,
    id: i64,
) -> core_db_entities::entity::refund_attempts::Model {
    core_db_entities::entity::refund_attempts::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap()
        .unwrap()
}

#[tokio::test]
#[ignore = "exclusive isolated MySQL; intended to expose F4 pending provider status"]
async fn f4_worker_pending_id_waits_for_processed_webhook() {
    let _lock = ENV_LOCK.lock().await;
    let db = isolated_db().await;
    let p = Provider::start().await;
    let f = paid_fixture(&db).await;
    let id = queue_refund(&db, &f).await;
    p.state.lock().unwrap().refund_pending = true;
    worker(&db).await;
    let after = attempt(&db, id).await;
    let pending_rows = refunds::Entity::find()
        .filter(refunds::Column::OrderId.eq(f.order))
        .all(&db)
        .await
        .unwrap();
    let gateway = after
        .gateway_refund_id
        .clone()
        .expect("accepted refund id must be retained");
    let submissions = p.state.lock().unwrap().refunds.len();
    // Finish the fixture before asserting the pending snapshot; a red regression
    // must not leave a claimable job that contaminates a later worker test.
    p.state.lock().unwrap().refund_pending = false;
    let body = json!({"event":"refund.processed","payload":{"refund":{"entity":{
        "id":gateway,"payment_id":f.payment,"amount":1000,"currency":"INR","status":"processed"}}}});
    let _ = ingest(
        &db,
        webhook_request(body, &format!("worker_refund_{}", f.order)),
    )
    .await;
    worker(&db).await;
    assert_ne!(
        after.status, "processed",
        "a pending provider response is not a settled refund"
    );
    assert!(matches!(
        after.status.as_str(),
        "submitted" | "pending_external" | "pending"
    ));
    assert!(pending_rows
        .iter()
        .all(|r| r.status != Some(PaymentIntentStatus::Processed)));
    assert_eq!(submissions, 1);
    assert_eq!(
        p.state.lock().unwrap().refunds.len(),
        1,
        "known refund id must prevent resubmission"
    );
    refund_totals(&db, &f, 1, 1000).await;
}

#[tokio::test]
#[ignore = "exclusive isolated MySQL; loopback provider timeout (~15s)"]
async fn accepted_refund_timeout_requires_reconciliation_without_resubmission() {
    let _lock = ENV_LOCK.lock().await;
    let db = isolated_db().await;
    let p = Provider::start().await;
    let f = paid_fixture(&db).await;
    let id = queue_refund(&db, &f).await;
    p.state.lock().unwrap().timeout_refund_once = true;
    worker(&db).await;
    let first = attempt(&db, id).await;
    assert_ne!(first.status, "processed");
    assert_eq!(
        refunds::Entity::find()
            .filter(refunds::Column::OrderId.eq(f.order))
            .count(&db)
            .await
            .unwrap(),
        0
    );
    assert_eq!(p.state.lock().unwrap().refunds.len(), 1);
    worker(&db).await;
    let final_attempt = attempt(&db, id).await;
    {
        let state = p.state.lock().unwrap();
        assert_eq!(
            state.refunds.len(),
            1,
            "unknown provider outcome must not trigger a second POST"
        );
        assert!(!state.refunds[0].is_empty());
        assert_eq!(
            state.accepted_refunds.len(),
            1,
            "provider accepted the first submission"
        );
        assert!(state.accepted_refunds.contains_key(&first.idempotency_key));
    }
    assert_eq!(final_attempt.status, "needs_review");
    assert!(
        final_attempt.gateway_refund_id.is_none(),
        "the lost response did not establish a local provider ID"
    );
    assert!(
        final_attempt.provider_error.is_some(),
        "operator reconciliation needs an explicit reason"
    );
    assert_eq!(
        count(
            &db,
            "SELECT COUNT(*) AS n FROM RefundAttempts WHERE order_id = ?",
            f.order
        )
        .await,
        1
    );
    assert_eq!(
        refunds::Entity::find()
            .filter(refunds::Column::OrderId.eq(f.order))
            .count(&db)
            .await
            .unwrap(),
        0
    );
    worker(&db).await;
    assert_eq!(p.state.lock().unwrap().refunds.len(), 1);
}

#[tokio::test]
#[ignore = "exclusive isolated MySQL; loopback timeout then authenticated recovery (~15s)"]
async fn ambiguous_refund_recovers_by_webhook_without_resubmitting() {
    let _lock = ENV_LOCK.lock().await;
    let db = isolated_db().await;
    let p = Provider::start().await;
    let f = paid_fixture(&db).await;
    let id = queue_refund(&db, &f).await;
    p.state.lock().unwrap().timeout_refund_once = true;
    worker(&db).await;
    let unresolved = attempt(&db, id).await;
    assert_eq!(unresolved.status, "needs_review");
    assert!(unresolved.provider_error.is_some());
    worker(&db).await;
    let gateway = {
        let state = p.state.lock().unwrap();
        assert_eq!(
            state.refunds.len(),
            1,
            "no blind second provider submission"
        );
        assert_eq!(state.accepted_refunds.len(), 1);
        state
            .accepted_refunds
            .get(&unresolved.idempotency_key)
            .unwrap()
            .clone()
    };
    assert_eq!(
        refunds::Entity::find()
            .filter(refunds::Column::OrderId.eq(f.order))
            .count(&db)
            .await
            .unwrap(),
        0
    );
    refund_event(&db, &f, &gateway, 1000, "processed").await;
    refund_event(&db, &f, &gateway, 1000, "processed").await;
    assert_eq!(attempt(&db, id).await.status, "processed");
    assert_eq!(
        attempt(&db, id).await.gateway_refund_id.as_deref(),
        Some(gateway.as_str())
    );
    assert_eq!(
        count(
            &db,
            "SELECT COUNT(*) AS n FROM RefundAttempts WHERE order_id = ?",
            f.order
        )
        .await,
        1
    );
    refund_totals(&db, &f, 1, 1000).await;
    worker(&db).await;
    assert_eq!(p.state.lock().unwrap().refunds.len(), 1);
}
