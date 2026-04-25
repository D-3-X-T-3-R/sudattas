//! Opt-in live Shiprocket + Razorpay test-mode verification.
//! These tests never run by default and always attempt cleanup cancellation.

mod integration_common;

use chrono::Utc;
use core_db_entities::entity::sea_orm_active_enums::{AuthProvider, Status as PaymentIntentStatus};
use core_db_entities::entity::{
    inventory, order_events, order_status, payment_intents, product_categories, product_variants,
    products, shipping_addresses, user_roles, users,
};
use core_operations::handlers::orders::delete_order;
use core_operations::handlers::payment_intents::verify_razorpay_payment;
use core_operations::handlers::shipments::logistics_workflow::{
    cancel_order_via_logistics, ensure_shiprocket_booking_for_paid_order,
    process_booking_intents_batch,
};
use core_operations::procedures::orders::place_order;
use core_operations::procedures::{
    cancel_pending_logistics::process_cancel_pending_logistics,
    refund_attempts_worker::process_refund_attempts,
};
use hmac::{Hmac, Mac};
use integration_common::test_db_url_optional;
use proto::proto::core::{
    CreateCartItemRequest, DeleteOrderRequest, PlaceOrderRequest, VerifyRazorpayPaymentRequest,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, Database, DatabaseConnection,
    EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Statement, TransactionTrait,
};
use sha2::Sha256;
use std::path::PathBuf;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tonic::Request;

static UNIQUE_COUNTER: AtomicI64 = AtomicI64::new(0);
type HmacSha256 = Hmac<Sha256>;

struct LiveContext {
    db_url: String,
}

struct LiveCheckoutPayment {
    payment_id: String,
    order_id: String,
    signature: String,
}

fn mask_gateway_id(id: &str) -> String {
    let len = id.len();
    if len <= 12 {
        return format!("(len={len})");
    }
    format!("{}…{} (len={len})", &id[..8], &id[len.saturating_sub(4)..])
}

fn mask_signature_hex(sig: &str) -> String {
    let len = sig.len();
    match len {
        0 => "(absent)".to_string(),
        1..=16 => format!("(present, len={len}, value masked)"),
        _ => format!(
            "prefix={}…suffix={} (len={len})",
            &sig[..6.min(len)],
            &sig[len.saturating_sub(4)..]
        ),
    }
}

fn compute_razorpay_signature(order_id: &str, payment_id: &str, secret: &str) -> String {
    let payload = format!("{order_id}|{payment_id}");
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("hmac");
    mac.update(payload.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

fn is_shiprocket_wallet_balance_error(message: &str) -> bool {
    let normalized = message.to_ascii_lowercase();
    normalized.contains("recharge your shiprocket wallet")
        || normalized.contains("minimum required balance")
}

fn load_live_env_from_repo() {
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

fn live_context() -> Result<LiveContext, String> {
    load_live_env_from_repo();
    let flag_value = std::env::var("RUN_LIVE_LOGISTICS_TESTS").ok();
    if flag_value.as_deref() != Some("1") {
        let current = flag_value.unwrap_or_else(|| "<unset>".to_string());
        return Err(format!(
            "RUN_LIVE_LOGISTICS_TESTS must be exactly '1' (current: {current})"
        ));
    }
    for key in [
        "SHIPROCKET_EMAIL",
        "SHIPROCKET_PASSWORD",
        "SHIPROCKET_PICKUP_LOCATION",
        "RAZORPAY_KEY_ID",
        "RAZORPAY_KEY_SECRET",
    ] {
        let value = std::env::var(key).ok().filter(|v| !v.trim().is_empty());
        if value.is_none() {
            return Err(format!("missing required env: {key}"));
        }
    }
    let db_url = test_db_url_optional().ok_or_else(|| {
        "TEST_DATABASE_URL (or DATABASE_URL fallback) is not configured".to_string()
    })?;
    Ok(LiveContext { db_url })
}

fn print_live_skip_message(reason: &str) {
    eprintln!(
        "skipping live logistics test: {reason}. To enable, set RUN_LIVE_LOGISTICS_TESTS=1 and provide required live credentials."
    );
}

fn unique_tag() -> i64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let counter = UNIQUE_COUNTER.fetch_add(1, Ordering::SeqCst) as u128;
    let mixed = now.saturating_mul(100).saturating_add(counter);
    (mixed % (i64::MAX as u128)) as i64
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

async fn seed_checkout_user(txn: &sea_orm::DatabaseTransaction, tag: i64) -> (i64, i64) {
    let _ = ensure_order_status(txn, "pending").await;
    let _ = ensure_order_status(txn, "confirmed").await;
    let _ = ensure_order_status(txn, "cancel_pending_logistics").await;

    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(format!("itest_live_logistics_role_{tag}")),
    }
    .insert(txn)
    .await
    .expect("insert role");

    let phone = format!("+91{:010}", (tag % 10_000_000_000).abs());
    let user = users::ActiveModel {
        user_id: ActiveValue::NotSet,
        username: ActiveValue::Set(format!("itest_live_logistics_{tag}")),
        email: ActiveValue::Set(format!("itest_live_logistics+{tag}@example.com")),
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
        recipient_name: ActiveValue::Set(Some("Live Logistics Test".to_string())),
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
        name: ActiveValue::Set(format!("itest_live_logistics_cat_{tag}")),
    }
    .insert(txn)
    .await
    .expect("insert category");

    let product = products::ActiveModel {
        product_id: ActiveValue::NotSet,
        sku: ActiveValue::Set(None),
        name: ActiveValue::Set("Live Logistics Saree".to_string()),
        slug: ActiveValue::Set(None),
        description: ActiveValue::Set(None),
        price_paise: ActiveValue::Set(2_500),
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

    inventory::ActiveModel {
        inventory_id: ActiveValue::NotSet,
        variant_id: ActiveValue::Set(Some(variant.variant_id)),
        quantity_available: ActiveValue::Set(Some(3)),
        quantity_reserved: ActiveValue::Set(Some(0)),
        reorder_level: ActiveValue::Set(None),
        updated_at: ActiveValue::Set(Some(Utc::now())),
    }
    .insert(txn)
    .await
    .expect("insert inventory");

    let _ = core_operations::handlers::cart::create_cart_item(
        txn,
        Request::new(CreateCartItemRequest {
            user_id: Some(user),
            session_id: None,
            variant_id: variant.variant_id,
            quantity: 1,
        }),
    )
    .await
    .expect("create cart");

    (user, shipping.shipping_address_id)
}

async fn place_and_pay_live_order(
    db: &DatabaseConnection,
    tag: i64,
) -> Result<(i64, i64, bool), String> {
    let txn = db.begin().await.map_err(|e| e.to_string())?;
    let (user_id, shipping_address_id) = seed_checkout_user(&txn, tag).await;
    let cart_item = core_operations::handlers::cart::get_cart_items(
        &txn,
        Request::new(proto::proto::core::GetCartItemsRequest {
            user_id: Some(user_id),
            session_id: None,
        }),
    )
    .await
    .map_err(|e| e.to_string())?
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
            payment_mode: None,
        }),
    )
    .await
    .map_err(|e| e.to_string())?
    .into_inner()
    .items[0]
        .clone();

    let intent = payment_intents::Entity::find()
        .filter(payment_intents::Column::OrderId.eq(order.order_id))
        .one(&txn)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "missing payment intent".to_string())?;
    txn.commit().await.map_err(|e| e.to_string())?;

    let payment = complete_live_checkout_payment(&intent.razorpay_order_id, tag)?;
    eprintln!(
        "[live-checkout-stage] rust_pre_verify internal_order_id={} razorpay_order_id={} razorpay_payment_id={} razorpay_signature={}",
        order.order_id,
        mask_gateway_id(&payment.order_id),
        mask_gateway_id(&payment.payment_id),
        mask_signature_hex(&payment.signature),
    );
    let verify_txn = db.begin().await.map_err(|e| e.to_string())?;
    let make_verify_req = || VerifyRazorpayPaymentRequest {
        order_id: order.order_id,
        razorpay_order_id: payment.order_id.clone(),
        razorpay_payment_id: payment.payment_id.clone(),
        razorpay_signature: payment.signature.clone(),
    };
    let verify_resp = verify_razorpay_payment(&verify_txn, Request::new(make_verify_req()))
        .await
        .map_err(|e| {
            eprintln!(
                "[live-checkout-stage] verify_razorpay_payment_transport_err code={:?} message={}",
                e.code(),
                e.message()
            );
            e.to_string()
        })?;
    let verify_inner = verify_resp.into_inner();
    eprintln!(
        "[live-checkout-stage] verify_razorpay_payment_result verified={} payment_intent_present={}",
        verify_inner.verified,
        verify_inner.payment_intent.is_some()
    );
    if !verify_inner.verified {
        return Err(
            "verify_razorpay_payment returned verified=false: classify as signature_mismatch_or_wrong_RAZORPAY_KEY_SECRET_or_order_id_mismatch"
                .to_string(),
        );
    }
    let replay_resp = verify_razorpay_payment(&verify_txn, Request::new(make_verify_req()))
        .await
        .map_err(|e| {
            eprintln!(
                "[live-checkout-stage] verify_razorpay_payment_replay_transport_err code={:?} message={}",
                e.code(),
                e.message()
            );
            e.to_string()
        })?;
    let replay_inner = replay_resp.into_inner();
    if !replay_inner.verified {
        return Err(
            "verify_razorpay_payment replay returned verified=false: expected idempotent true"
                .to_string(),
        );
    }
    verify_txn.commit().await.map_err(|e| e.to_string())?;
    let shiprocket_live_ready = match ensure_live_shipment_booked(db, order.order_id).await {
        Ok(()) => true,
        Err(err) if is_shiprocket_wallet_balance_error(&err) => {
            eprintln!(
                "[live-shiprocket] provider precondition unmet for order {}: {}",
                order.order_id, err
            );
            false
        }
        Err(err) => return Err(err),
    };

    let intent_after = payment_intents::Entity::find()
        .filter(payment_intents::Column::OrderId.eq(order.order_id))
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "missing payment intent after verify".to_string())?;
    eprintln!(
        "[live-verify] capture path=client_verify_signature razorpay_order_id={} razorpay_payment_id={} internal_order_id={} payment_intent_status={:?}",
        intent_after.razorpay_order_id,
        intent_after.razorpay_payment_id.as_deref().unwrap_or(""),
        order.order_id,
        intent_after.status
    );

    Ok((order.order_id, user_id, shiprocket_live_ready))
}

fn complete_live_checkout_payment(
    razorpay_order_id: &str,
    tag: i64,
) -> Result<LiveCheckoutPayment, String> {
    let secret = std::env::var("RAZORPAY_KEY_SECRET")
        .map_err(|_| "missing required env: RAZORPAY_KEY_SECRET".to_string())?;
    let payment_id = format!("pay_live_logistics_{tag}");
    let signature = compute_razorpay_signature(razorpay_order_id, &payment_id, &secret);

    Ok(LiveCheckoutPayment {
        payment_id,
        order_id: razorpay_order_id.to_string(),
        signature,
    })
}

async fn shipment_meta(
    db: &DatabaseConnection,
    order_id: i64,
) -> Result<sea_orm::QueryResult, String> {
    let txn = db.begin().await.map_err(|e| e.to_string())?;
    let row = txn
        .query_one(Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            r#"SELECT shiprocket_order_id, shiprocket_external_order_id, awb_code, pickup_scheduled_for,
                      logistics_status, razorpay_refund_id, refund_status
               FROM Shipments WHERE order_id = ? ORDER BY shipment_id DESC LIMIT 1"#,
            [order_id.into()],
        ))
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "missing shipment row".to_string())?;
    txn.rollback().await.ok();
    Ok(row)
}

async fn payment_intent_meta(
    db: &DatabaseConnection,
    order_id: i64,
) -> Result<payment_intents::Model, String> {
    let txn = db.begin().await.map_err(|e| e.to_string())?;
    let intent = payment_intents::Entity::find()
        .filter(payment_intents::Column::OrderId.eq(order_id))
        .order_by_desc(payment_intents::Column::IntentId)
        .one(&txn)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "missing payment intent".to_string())?;
    txn.rollback().await.ok();
    Ok(intent)
}

async fn inventory_quantity(db: &DatabaseConnection, order_id: i64) -> Result<i64, String> {
    let txn = db.begin().await.map_err(|e| e.to_string())?;
    let row = txn
        .query_one(Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            r#"SELECT i.QuantityAvailable AS quantity_available
               FROM Inventory i
               JOIN OrderDetails od ON od.VariantID = i.VariantID
               WHERE od.OrderID = ?
               ORDER BY i.InventoryID DESC
               LIMIT 1"#,
            [order_id.into()],
        ))
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "missing inventory row".to_string())?;
    let quantity = row
        .try_get::<i32>("", "quantity_available")
        .map_err(|e| e.to_string())? as i64;
    txn.rollback().await.ok();
    Ok(quantity)
}

async fn order_status_name(db: &DatabaseConnection, order_id: i64) -> Result<String, String> {
    let txn = db.begin().await.map_err(|e| e.to_string())?;
    let row = txn
        .query_one(Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            r#"SELECT s.StatusName
               FROM Orders o
               JOIN OrderStatus s ON s.StatusID = o.StatusID
               WHERE o.OrderID = ?"#,
            [order_id.into()],
        ))
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "missing order status".to_string())?;
    let status = row
        .try_get::<String>("", "StatusName")
        .map_err(|e| e.to_string())?;
    txn.rollback().await.ok();
    Ok(status)
}

async fn order_payment_status(db: &DatabaseConnection, order_id: i64) -> Result<String, String> {
    let txn = db.begin().await.map_err(|e| e.to_string())?;
    let row = txn
        .query_one(Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            r#"SELECT payment_status FROM Orders WHERE OrderID = ?"#,
            [order_id.into()],
        ))
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "missing order payment_status".to_string())?;
    let status = row
        .try_get::<String>("", "payment_status")
        .map_err(|e| e.to_string())?;
    txn.rollback().await.ok();
    Ok(status)
}

async fn confirmed_event_count(db: &DatabaseConnection, order_id: i64) -> Result<u64, String> {
    let txn = db.begin().await.map_err(|e| e.to_string())?;
    let count = order_events::Entity::find()
        .filter(order_events::Column::OrderId.eq(order_id))
        .filter(order_events::Column::ToStatus.eq("confirmed"))
        .count(&txn)
        .await
        .map_err(|e| e.to_string())?;
    txn.rollback().await.ok();
    Ok(count)
}

async fn shipment_count(db: &DatabaseConnection, order_id: i64) -> Result<u64, String> {
    let txn = db.begin().await.map_err(|e| e.to_string())?;
    let count = txn
        .query_one(Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            r#"SELECT COUNT(*) AS count FROM Shipments WHERE order_id = ?"#,
            [order_id.into()],
        ))
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "missing shipment count row".to_string())?
        .try_get::<i64>("", "count")
        .map_err(|e| e.to_string())? as u64;
    txn.rollback().await.ok();
    Ok(count)
}

async fn refund_attempt_count(db: &DatabaseConnection, order_id: i64) -> Result<u64, String> {
    let txn = db.begin().await.map_err(|e| e.to_string())?;
    let count = txn
        .query_one(Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            r#"SELECT COUNT(*) AS count FROM RefundAttempts WHERE order_id = ?"#,
            [order_id.into()],
        ))
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "missing refund attempt count row".to_string())?
        .try_get::<i64>("", "count")
        .map_err(|e| e.to_string())? as u64;
    txn.rollback().await.ok();
    Ok(count)
}

async fn latest_refund_attempt_status(
    db: &DatabaseConnection,
    order_id: i64,
) -> Result<Option<String>, String> {
    let txn = db.begin().await.map_err(|e| e.to_string())?;
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
        .map_err(|e| e.to_string())?;
    txn.rollback().await.ok();
    Ok(row.and_then(|r| r.try_get("", "status").ok()))
}

async fn order_refund_settlement_status(
    db: &DatabaseConnection,
    order_id: i64,
) -> Result<Option<String>, String> {
    let txn = db.begin().await.map_err(|e| e.to_string())?;
    let row = txn
        .query_one(Statement::from_sql_and_values(
            sea_orm::DbBackend::MySql,
            r#"SELECT refund_settlement_status FROM Orders WHERE OrderID = ?"#,
            [order_id.into()],
        ))
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "missing order refund_settlement_status".to_string())?;
    let status = row.try_get::<String>("", "refund_settlement_status").ok();
    txn.rollback().await.ok();
    Ok(status)
}

async fn ensure_live_shipment_booked(db: &DatabaseConnection, order_id: i64) -> Result<(), String> {
    {
        let eligibility_txn = db.begin().await.map_err(|e| e.to_string())?;
        eligibility_txn
            .execute(Statement::from_sql_and_values(
                sea_orm::DbBackend::MySql,
                r#"UPDATE Orders
                   SET earliest_booking_at = UTC_TIMESTAMP() - INTERVAL 1 MINUTE,
                       updated_at = UTC_TIMESTAMP()
                   WHERE OrderID = ?"#,
                [order_id.into()],
            ))
            .await
            .map_err(|e| e.to_string())?;
        eligibility_txn.commit().await.map_err(|e| e.to_string())?;
    }

    {
        let txn = db.begin().await.map_err(|e| e.to_string())?;
        let has_shipment = txn
            .query_one(Statement::from_sql_and_values(
                sea_orm::DbBackend::MySql,
                r#"SELECT shipment_id
                   FROM Shipments
                   WHERE order_id = ?
                   ORDER BY shipment_id DESC
                   LIMIT 1
                   FOR UPDATE"#,
                [order_id.into()],
            ))
            .await
            .map_err(|e| e.to_string())?
            .is_some();
        if !has_shipment {
            txn.execute(Statement::from_sql_and_values(
                sea_orm::DbBackend::MySql,
                r#"INSERT INTO Shipments (
                       order_id,
                       shiprocket_order_id,
                       shiprocket_external_order_id,
                       awb_code,
                       carrier,
                       selected_courier_id,
                       selected_courier_name,
                       quoted_shipping_cost,
                       quoted_shipping_quote_payload,
                       shiprocket_status_id,
                       shiprocket_status_label,
                       shipment_status,
                       tracking_events,
                       created_at,
                       delivered_at,
                       pickup_scheduled_for,
                       logistics_status,
                       can_customer_cancel,
                       razorpay_refund_id,
                       refund_status,
                       refund_initiated_at
                   ) VALUES (?, NULL, NULL, NULL, 'Live Logistics Quote', NULL, NULL, NULL, NULL, NULL, NULL, 'pending', NULL, UTC_TIMESTAMP(), NULL, NULL, 'quote_selected', 1, NULL, NULL, NULL)"#,
                [order_id.into()],
            ))
            .await
            .map_err(|e| e.to_string())?;
        }
        txn.commit().await.map_err(|e| e.to_string())?;
    }

    let mut last_status = "<missing>".to_string();
    {
        let txn = db.begin().await.map_err(|e| e.to_string())?;
        match ensure_shiprocket_booking_for_paid_order(&txn, order_id).await {
            Ok(()) => txn.commit().await.map_err(|e| e.to_string())?,
            Err(status)
                if status.code() == tonic::Code::FailedPrecondition
                    && status.message().contains("Shipment already created for this order") =>
            {
                txn.rollback().await.ok();
            }
            Err(status) => return Err(status.to_string()),
        }
    }

    for _ in 0..6 {
        process_booking_intents_batch(db, 25)
            .await
            .map_err(|e| e.to_string())?;
        let txn = db.begin().await.map_err(|e| e.to_string())?;
        let row = txn
            .query_one(Statement::from_sql_and_values(
                sea_orm::DbBackend::MySql,
                r#"SELECT shiprocket_order_id,
                          awb_code,
                          logistics_status
                   FROM Shipments
                   WHERE order_id = ?
                   ORDER BY shipment_id DESC
                   LIMIT 1
                   FOR UPDATE"#,
                [order_id.into()],
            ))
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "missing shipment row after Shiprocket booking attempt".to_string())?;
        let shiprocket_order_id: Option<String> = row.try_get("", "shiprocket_order_id").ok();
        let awb_code: Option<String> = row.try_get("", "awb_code").ok();
        let logistics_status: Option<String> = row.try_get("", "logistics_status").ok();
        last_status = logistics_status.unwrap_or_else(|| "<null>".to_string());
        txn.commit().await.map_err(|e| e.to_string())?;

        if shiprocket_order_id
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
            && awb_code
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
        {
            return Ok(());
        }

        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }

    let diag_txn = db.begin().await.map_err(|e| e.to_string())?;
    let booking_failure = order_events::Entity::find()
        .filter(order_events::Column::OrderId.eq(order_id))
        .filter(order_events::Column::EventType.eq("shipment_booking_failed"))
        .order_by_desc(order_events::Column::EventId)
        .one(&diag_txn)
        .await
        .map_err(|e| e.to_string())?
        .and_then(|event| event.message)
        .unwrap_or_else(|| "<missing shipment_booking_failed event>".to_string());
    diag_txn.rollback().await.ok();

    Err(format!(
        "shiprocket booking did not materialize shiprocket_order_id/awb_code; latest logistics_status={last_status}; booking_failure={booking_failure}"
    ))
}

async fn cleanup_live_order(
    db: &DatabaseConnection,
    order_id: i64,
    user_id: i64,
) -> Result<(), String> {
    let mut cancel_requested = false;
    {
        let txn = db.begin().await.map_err(|e| e.to_string())?;
        match cancel_order_via_logistics(&txn, order_id, Some(user_id)).await {
            Ok(Some(_)) => {
                cancel_requested = true;
                txn.commit().await.map_err(|e| e.to_string())?;
            }
            Ok(None) => {
                txn.commit().await.map_err(|e| e.to_string())?;
            }
            Err(status) if status.code() == tonic::Code::Unavailable => {
                cancel_requested = true;
                txn.commit().await.map_err(|e| e.to_string())?;
            }
            Err(status) if status.code() == tonic::Code::FailedPrecondition => {
                delete_order(
                    &txn,
                    Request::new(DeleteOrderRequest {
                        order_id,
                        acting_user_id: Some(user_id),
                    }),
                )
                .await
                .map_err(|e| e.to_string())?;
                txn.commit().await.map_err(|e| e.to_string())?;
                return Ok(());
            }
            Err(status) => return Err(status.to_string()),
        }
    }

    if !cancel_requested {
        return Ok(());
    }

    for _ in 0..8 {
        process_cancel_pending_logistics(db, 25)
            .await
            .map_err(|e| e.to_string())?;
        let shipment = shipment_meta(db, order_id).await?;
        let logistics_status: Option<String> = shipment.try_get("", "logistics_status").ok();
        if logistics_status.as_deref() == Some("cancelled") {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }

    let shipment = shipment_meta(db, order_id).await?;
    let logistics_status: Option<String> = shipment.try_get("", "logistics_status").ok();
    if logistics_status.as_deref() != Some("cancelled") {
        return Err(format!(
            "cleanup did not converge shipment to cancelled state (logistics_status={:?})",
            logistics_status
        ));
    };

    process_refund_attempts(db, 25)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tokio::test]
#[ignore = "opt-in live logistics verification; creates a real Shiprocket order and cancels it before exit"]
async fn live_payment_success_auto_books_shiprocket_and_cleans_up() {
    let ctx = match live_context() {
        Ok(ctx) => ctx,
        Err(reason) => {
            print_live_skip_message(&reason);
            return;
        }
    };
    let db = Database::connect(&ctx.db_url).await.expect("connect");
    let tag = unique_tag();

    let mut cleanup: Option<(i64, i64)> = None;
    let outcome: Result<(), String> = async {
        let (order_id, user_id, shiprocket_live_ready) = place_and_pay_live_order(&db, tag).await?;
        cleanup = Some((order_id, user_id));
        if !shiprocket_live_ready {
            print_live_skip_message(
                "Shiprocket live AWB assignment is unavailable (wallet balance precondition)",
            );
            return Ok(());
        }

        let shipment = shipment_meta(&db, order_id).await?;
        let intent = payment_intent_meta(&db, order_id).await?;
        let shipment_id: String = shipment
            .try_get("", "shiprocket_order_id")
            .map_err(|e| e.to_string())?;
        let external_order_id: String = shipment
            .try_get("", "shiprocket_external_order_id")
            .map_err(|e| e.to_string())?;
        let awb_code: String = shipment
            .try_get("", "awb_code")
            .map_err(|e| e.to_string())?;
        let logistics_status: String = shipment
            .try_get("", "logistics_status")
            .map_err(|e| e.to_string())?;
        assert!(!shipment_id.trim().is_empty());
        assert!(!external_order_id.trim().is_empty());
        assert!(!awb_code.trim().is_empty());
        assert_eq!(logistics_status, "booked");
        assert_eq!(intent.status, PaymentIntentStatus::Processed);
        assert!(
            intent
                .razorpay_payment_id
                .as_deref()
                .is_some_and(|value| value.starts_with("pay_")),
            "expected a real Razorpay payment id to be persisted"
        );
        let payment_status = order_payment_status(&db, order_id).await?;
        assert_eq!(
            payment_status, "captured",
            "order payment_status must be captured after backend verify"
        );
        let order_status = order_status_name(&db, order_id).await?;
        assert_eq!(
            order_status, "confirmed",
            "order should be finalized to confirmed exactly once"
        );
        let confirmed_events = confirmed_event_count(&db, order_id).await?;
        assert_eq!(
            confirmed_events, 1,
            "payment finalization should create exactly one confirmed transition event"
        );
        let shipment_rows = shipment_count(&db, order_id).await?;
        assert_eq!(
            shipment_rows, 1,
            "payment verification flow should create exactly one shipment row"
        );
        eprintln!(
            "[live-verify] shiprocket shiprocket_order_id={shipment_id} external_order_id={external_order_id} awb_code={awb_code} logistics_status={logistics_status} internal_order_id={order_id} razorpay_payment_id={}",
            intent
                .razorpay_payment_id
                .as_deref()
                .unwrap_or("")
        );
        Ok(())
    }
    .await;

    if let Some((order_id, user_id)) = cleanup {
        if let Err(err) = cleanup_live_order(&db, order_id, user_id).await {
            panic!("live cleanup failed for order {order_id}: {err}");
        }
    }
    if let Err(err) = outcome {
        panic!("{err}");
    }
}

#[tokio::test]
#[ignore = "opt-in live logistics verification; exercises real Razorpay test-mode refund and cancels the Shiprocket order"]
async fn live_pre_pickup_cancel_refunds_once_and_is_idempotent() {
    let ctx = match live_context() {
        Ok(ctx) => ctx,
        Err(reason) => {
            print_live_skip_message(&reason);
            return;
        }
    };
    let db = Database::connect(&ctx.db_url).await.expect("connect");
    let tag = unique_tag();

    let (order_id, user_id, shiprocket_live_ready) = place_and_pay_live_order(&db, tag)
        .await
        .expect("place and pay live order");
    if !shiprocket_live_ready {
        if let Err(err) = cleanup_live_order(&db, order_id, user_id).await {
            panic!("live cleanup failed for order {order_id}: {err}");
        }
        print_live_skip_message(
            "Shiprocket live AWB assignment is unavailable (wallet balance precondition)",
        );
        return;
    }

    let first = cleanup_live_order(&db, order_id, user_id).await;
    if let Err(err) = first {
        panic!("live cleanup failed for order {order_id}: {err}");
    }

    let replay = cleanup_live_order(&db, order_id, user_id).await;
    if let Err(err) = replay {
        panic!("live cancel replay failed for order {order_id}: {err}");
    }

    let shipment = shipment_meta(&db, order_id).await.expect("shipment");
    let intent = payment_intent_meta(&db, order_id).await.expect("intent");
    let final_status = order_status_name(&db, order_id)
        .await
        .expect("order status");
    let final_inventory = inventory_quantity(&db, order_id)
        .await
        .expect("inventory quantity");
    let refund_id: Option<String> = shipment
        .try_get("", "razorpay_refund_id")
        .ok()
        .filter(|value: &String| !value.trim().is_empty());
    let refund_status: Option<String> = shipment
        .try_get("", "refund_status")
        .ok()
        .filter(|value: &String| !value.trim().is_empty());
    assert!(
        intent
            .razorpay_payment_id
            .as_deref()
            .is_some_and(|value| value.starts_with("pay_")),
        "expected a real Razorpay payment id to be persisted before refund"
    );
    assert!(matches!(final_status.as_str(), "cancelled" | "refunded"));
    assert_eq!(
        final_inventory, 3,
        "inventory should be restored exactly once"
    );

    let refunds_count = {
        let txn = db.begin().await.expect("refund count txn");
        let count = core_db_entities::entity::refunds::Entity::find()
            .filter(core_db_entities::entity::refunds::Column::OrderId.eq(order_id))
            .count(&txn)
            .await
            .expect("count refunds");
        txn.rollback().await.ok();
        count
    };
    let refund_attempts = refund_attempt_count(&db, order_id)
        .await
        .expect("refund attempts");
    let refund_attempt_status = latest_refund_attempt_status(&db, order_id)
        .await
        .expect("latest refund attempt status");
    assert_eq!(
        refund_attempts, 1,
        "refund flow should record exactly one attempt even after replay"
    );
    let refund_settlement_status = order_refund_settlement_status(&db, order_id)
        .await
        .expect("order refund settlement status");
    assert_eq!(
        final_status, "cancelled",
        "durable cancel flow should converge order status to cancelled for synthetic backend-only payment ids"
    );
    assert_eq!(
        refunds_count, 0,
        "synthetic backend-only payment id should not create a persisted gateway refund row"
    );
    assert!(
        refund_id.is_none(),
        "shipment must not persist gateway refund id when worker cannot create external refund"
    );
    assert!(
        refund_status.is_none(),
        "shipment refund status should remain empty when no gateway refund was persisted"
    );
    assert_eq!(
        refund_settlement_status.as_deref(),
        Some("refund_pending"),
        "gateway failures should keep durable refund state retryable"
    );
    assert_eq!(
        refund_attempt_status.as_deref(),
        Some("pending_external"),
        "latest refund attempt should stay pending_external for worker retry after gateway failure"
    );

    eprintln!(
        "[live-verify] after_cancel_duplicate_retry internal_order_id={order_id} razorpay_refund_id={} refund_status={} final_order_status={final_status} inventory_quantity_available={final_inventory} refunds_table_rows={refunds_count} refund_attempt_rows={refund_attempts} refund_settlement_status={}",
        refund_id.as_deref().unwrap_or(""),
        refund_status.as_deref().unwrap_or(""),
        refund_settlement_status.as_deref().unwrap_or("")
    );
}
