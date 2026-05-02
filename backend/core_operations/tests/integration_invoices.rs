//! DB-backed invoice generation and outbox tests.
//!
//! Setup:
//! - TEST_DATABASE_URL must point to a migrated schema.

mod integration_common;

use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use chrono::Utc;
use core_db_entities::entity::sea_orm_active_enums::{
    AuthProvider, FulfillmentStatus, PaymentStatus, Status as OutboxStatus,
};
use core_db_entities::entity::{
    invoices, order_details, order_status, orders, outbox_events, shipping_addresses, users,
};
use core_operations::handlers::invoices::{ensure_invoice_for_order, InvoiceDocumentSnapshot};
use core_operations::handlers::outbox::INVOICE_GENERATED;
use core_operations::handlers::payment_intents::finalize_order_paid;
use core_operations::procedures::outbox_worker::process_pending_outbox_events;
use pdf_extract::extract_text_from_mem;
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Database, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, TransactionTrait,
};

async fn db_conn() -> sea_orm::DatabaseConnection {
    let url = integration_common::test_db_url();
    Database::connect(url).await.expect("connect test db")
}

fn unique_tag(prefix: &str) -> String {
    format!(
        "{prefix}_{}",
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    )
}

async fn ensure_status(txn: &sea_orm::DatabaseTransaction, name: &str) -> i64 {
    if let Some(existing) = order_status::Entity::find()
        .filter(order_status::Column::StatusName.eq(name))
        .one(txn)
        .await
        .expect("query order status")
    {
        return existing.status_id;
    }
    order_status::ActiveModel {
        status_id: ActiveValue::NotSet,
        status_name: ActiveValue::Set(name.to_string()),
    }
    .insert(txn)
    .await
    .expect("insert order status")
    .status_id
}

async fn seed_user(txn: &sea_orm::DatabaseTransaction, tag: &str, email: &str) -> i64 {
    users::ActiveModel {
        user_id: ActiveValue::NotSet,
        username: ActiveValue::Set(format!("invoice_user_{tag}")),
        auth_provider: ActiveValue::Set(AuthProvider::Google),
        password_hash: ActiveValue::Set(None),
        google_sub: ActiveValue::Set(Some(format!("invoice_sub_{tag}"))),
        email: ActiveValue::Set(email.to_string()),
        email_verified: ActiveValue::Set(Some(1)),
        email_verified_at: ActiveValue::Set(Some(Utc::now())),
        full_name: ActiveValue::Set(Some("Invoice Test User".to_string())),
        address: ActiveValue::Set(None),
        phone: ActiveValue::Set(None),
        user_status_id: ActiveValue::Set(None),
        role_id: ActiveValue::Set(None),
        last_login_at: ActiveValue::Set(None),
        marketing_opt_out: ActiveValue::Set(Some(0)),
        create_date: ActiveValue::Set(Utc::now()),
        updated_at: ActiveValue::Set(Some(Utc::now())),
    }
    .insert(txn)
    .await
    .expect("insert user")
    .user_id
}

async fn seed_address(txn: &sea_orm::DatabaseTransaction, user_id: i64, tag: &str) -> i64 {
    shipping_addresses::ActiveModel {
        shipping_address_id: ActiveValue::NotSet,
        user_id: ActiveValue::Set(Some(user_id)),
        is_default: ActiveValue::Set(1),
        country: ActiveValue::Set("India".to_string()),
        state_region: ActiveValue::Set("Karnataka".to_string()),
        city: ActiveValue::Set("Bengaluru".to_string()),
        postal_code: ActiveValue::Set("560001".to_string()),
        road: ActiveValue::Set(Some(format!("Road {tag}"))),
        apartment_no_or_name: ActiveValue::Set(Some("Apt 1".to_string())),
        recipient_name: ActiveValue::Set(Some("Invoice Test".to_string())),
        phone_number: ActiveValue::Set(Some("+919999999999".to_string())),
    }
    .insert(txn)
    .await
    .expect("insert address")
    .shipping_address_id
}

struct SeededOrder {
    order_id: i64,
    item_total_minor: i64,
    discount_minor: i64,
    shipping_minor: i64,
    grand_total_minor: i64,
}

async fn seed_order(
    txn: &sea_orm::DatabaseTransaction,
    tag: &str,
    status_name: &str,
    payment_method: &str,
    payment_status: PaymentStatus,
    email: &str,
) -> SeededOrder {
    let short_tag = tag
        .chars()
        .rev()
        .take(8)
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();
    let normalized_email = if email.trim().is_empty() {
        String::new()
    } else if let Some((local, domain)) = email.trim().split_once('@') {
        format!("{local}+{short_tag}@{domain}")
    } else {
        email.trim().to_string()
    };

    let user_id = seed_user(txn, tag, &normalized_email).await;
    let shipping_address_id = seed_address(txn, user_id, tag).await;
    let status_id = ensure_status(txn, status_name).await;

    let item_total_minor = 12_500;
    let discount_minor = 1_500;
    let shipping_minor = 500;
    let grand_total_minor = item_total_minor - discount_minor + shipping_minor;
    let now = Utc::now();
    let order = orders::ActiveModel {
        order_id: ActiveValue::NotSet,
        order_number: ActiveValue::Set(None),
        public_order_ref: ActiveValue::Set(format!("INVREF_{tag}")),
        user_id: ActiveValue::Set(user_id),
        order_date: ActiveValue::Set(now),
        created_at: ActiveValue::Set(now),
        cancel_window_ends_at: ActiveValue::Set(None),
        earliest_booking_at: ActiveValue::Set(None),
        pickup_target_at: ActiveValue::Set(None),
        pickup_target_reason: ActiveValue::Set(None),
        pickup_target_set_by: ActiveValue::Set(None),
        pickup_target_updated_at: ActiveValue::Set(None),
        shipping_address_id: ActiveValue::Set(shipping_address_id),
        total_amount: ActiveValue::Set(Some(Decimal::new(grand_total_minor, 2))),
        status_id: ActiveValue::Set(status_id),
        payment_status: ActiveValue::Set(Some(payment_status)),
        payment_method: ActiveValue::Set(Some(payment_method.to_string())),
        currency: ActiveValue::Set(Some("INR".to_string())),
        updated_at: ActiveValue::Set(Some(now)),
        subtotal_minor: ActiveValue::Set(item_total_minor),
        items_total_minor_before_discount: ActiveValue::Set(Some(item_total_minor)),
        shipping_minor: ActiveValue::Set(Some(shipping_minor)),
        shipping_charge_minor: ActiveValue::Set(Some(shipping_minor)),
        tax_total_minor: ActiveValue::Set(Some(0)),
        discount_total_minor: ActiveValue::Set(Some(discount_minor)),
        items_total_minor_after_discount: ActiveValue::Set(Some(item_total_minor - discount_minor)),
        grand_total_minor: ActiveValue::Set(grand_total_minor),
        invoice_id: ActiveValue::Set(None),
        invoice_number: ActiveValue::Set(None),
        invoice_generated_at: ActiveValue::Set(None),
        invoice_storage_path: ActiveValue::Set(None),
        applied_coupon_id: ActiveValue::Set(None),
        applied_coupon_code: ActiveValue::Set(None),
        applied_discount_paise: ActiveValue::Set(None),
        refund_settlement_status: ActiveValue::Set(None),
        fulfillment_status: ActiveValue::Set(FulfillmentStatus::NotCreated),
    }
    .insert(txn)
    .await
    .expect("insert order");

    order_details::ActiveModel {
        order_detail_id: ActiveValue::NotSet,
        order_id: ActiveValue::Set(order.order_id),
        variant_id: ActiveValue::Set(1),
        quantity: ActiveValue::Set(1),
        price: ActiveValue::Set(None),
        line_total_minor: ActiveValue::Set(item_total_minor - discount_minor),
        unit_price_minor: ActiveValue::Set(item_total_minor as i32),
        discount_minor: ActiveValue::Set(Some(discount_minor as i32)),
        tax_minor: ActiveValue::Set(Some(0)),
        sku: ActiveValue::Set(Some(format!("SKU-{tag}"))),
        title: ActiveValue::Set(Some("Invoice Test Saree".to_string())),
        line_attrs: ActiveValue::Set(None),
        item_status: ActiveValue::Set("active".to_string()),
        cancelled_at: ActiveValue::Set(None),
    }
    .insert(txn)
    .await
    .expect("insert order detail");

    SeededOrder {
        order_id: order.order_id,
        item_total_minor,
        discount_minor,
        shipping_minor,
        grand_total_minor,
    }
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn invoice_generated_exactly_once_and_outbox_not_duplicated() {
    let db = db_conn().await;
    let txn = db.begin().await.expect("begin");
    let seeded = seed_order(
        &txn,
        &unique_tag("prepaid_once"),
        "confirmed",
        "prepaid",
        PaymentStatus::Captured,
        "invoice_once@example.com",
    )
    .await;

    let first = ensure_invoice_for_order(&txn, seeded.order_id, "itest")
        .await
        .expect("invoice")
        .expect("invoice should be created");
    let second = ensure_invoice_for_order(&txn, seeded.order_id, "itest")
        .await
        .expect("invoice second call")
        .expect("invoice should still exist");
    assert_eq!(first.invoice_id, second.invoice_id);
    txn.commit().await.expect("commit");

    let invoice_rows = invoices::Entity::find()
        .filter(invoices::Column::OrderId.eq(seeded.order_id))
        .all(&db)
        .await
        .expect("query invoices");
    assert_eq!(invoice_rows.len(), 1, "invoice must be generated once");

    let outbox_rows = outbox_events::Entity::find()
        .filter(outbox_events::Column::EventType.eq(INVOICE_GENERATED))
        .filter(outbox_events::Column::AggregateId.eq(seeded.order_id.to_string()))
        .all(&db)
        .await
        .expect("query outbox");
    assert_eq!(
        outbox_rows.len(),
        1,
        "invoice email event must be queued once"
    );
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn unpaid_prepaid_order_does_not_generate_invoice() {
    let db = db_conn().await;
    let txn = db.begin().await.expect("begin");
    let seeded = seed_order(
        &txn,
        &unique_tag("prepaid_unpaid"),
        "active_sale",
        "prepaid",
        PaymentStatus::Pending,
        "invoice_unpaid@example.com",
    )
    .await;

    let row = ensure_invoice_for_order(&txn, seeded.order_id, "itest")
        .await
        .expect("no invoice failure");
    assert!(row.is_none(), "unpaid order should not have invoice");
    txn.rollback().await.expect("rollback");
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn cancelled_before_payment_does_not_generate_invoice() {
    let db = db_conn().await;
    let txn = db.begin().await.expect("begin");
    let seeded = seed_order(
        &txn,
        &unique_tag("cancelled_prepaid"),
        "cancelled",
        "prepaid",
        PaymentStatus::Pending,
        "invoice_cancelled@example.com",
    )
    .await;

    let row = ensure_invoice_for_order(&txn, seeded.order_id, "itest")
        .await
        .expect("no invoice failure");
    assert!(
        row.is_none(),
        "cancelled-before-payment order should not have invoice"
    );
    txn.rollback().await.expect("rollback");
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn cod_confirmed_order_generates_invoice() {
    let db = db_conn().await;
    let txn = db.begin().await.expect("begin");
    let seeded = seed_order(
        &txn,
        &unique_tag("cod_confirmed"),
        "confirmed",
        "cod",
        PaymentStatus::Pending,
        "invoice_cod@example.com",
    )
    .await;

    let row = ensure_invoice_for_order(&txn, seeded.order_id, "itest")
        .await
        .expect("invoice generation");
    assert!(row.is_some(), "confirmed COD order should generate invoice");
    txn.rollback().await.expect("rollback");
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn invoice_snapshot_uses_frozen_order_totals() {
    let db = db_conn().await;
    let txn = db.begin().await.expect("begin");
    let seeded = seed_order(
        &txn,
        &unique_tag("snapshot_totals"),
        "confirmed",
        "prepaid",
        PaymentStatus::Captured,
        "invoice_snapshot@example.com",
    )
    .await;

    let invoice = ensure_invoice_for_order(&txn, seeded.order_id, "itest")
        .await
        .expect("invoice generation")
        .expect("invoice expected");
    let snapshot: InvoiceDocumentSnapshot =
        serde_json::from_value(invoice.snapshot_json.clone()).expect("parse invoice snapshot");
    assert_eq!(snapshot.item_total_minor, seeded.item_total_minor);
    assert_eq!(snapshot.discount_minor, seeded.discount_minor);
    assert_eq!(snapshot.shipping_minor, seeded.shipping_minor);
    assert_eq!(snapshot.grand_total_minor, seeded.grand_total_minor);
    txn.rollback().await.expect("rollback");
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn duplicate_payment_finalization_does_not_regenerate_invoice_or_email_event() {
    let db = db_conn().await;
    let txn = db.begin().await.expect("begin");
    let seeded = seed_order(
        &txn,
        &unique_tag("dup_finalize"),
        "active_sale",
        "prepaid",
        PaymentStatus::Pending,
        "invoice_dup_finalize@example.com",
    )
    .await;

    finalize_order_paid(
        &txn,
        seeded.order_id,
        "payment_captured",
        "system",
        "itest payment captured",
    )
    .await
    .expect("first finalize");
    finalize_order_paid(
        &txn,
        seeded.order_id,
        "payment_captured_duplicate",
        "system",
        "itest duplicate payment captured",
    )
    .await
    .expect("duplicate finalize should be idempotent");
    txn.commit().await.expect("commit");

    let invoice_count = invoices::Entity::find()
        .filter(invoices::Column::OrderId.eq(seeded.order_id))
        .count(&db)
        .await
        .expect("count invoices");
    assert_eq!(invoice_count, 1);

    let email_events = outbox_events::Entity::find()
        .filter(outbox_events::Column::EventType.eq(INVOICE_GENERATED))
        .filter(outbox_events::Column::AggregateId.eq(seeded.order_id.to_string()))
        .all(&db)
        .await
        .expect("query outbox");
    assert_eq!(
        email_events.len(),
        1,
        "duplicate payment must not queue duplicate invoice email"
    );
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn invoice_generation_survives_email_delivery_failure() {
    let db = db_conn().await;
    let txn = db.begin().await.expect("begin");
    let seeded = seed_order(
        &txn,
        &unique_tag("email_fail"),
        "confirmed",
        "prepaid",
        PaymentStatus::Captured,
        "invoice_email_fail@example.com",
    )
    .await;

    let invoice = ensure_invoice_for_order(&txn, seeded.order_id, "itest")
        .await
        .expect("invoice generation")
        .expect("invoice expected");
    txn.commit().await.expect("commit");

    std::env::set_var("OUTBOX_DELIVER_FAIL", "1");
    let processed = process_pending_outbox_events(&db, 20)
        .await
        .expect("outbox worker should continue on delivery failures");
    std::env::remove_var("OUTBOX_DELIVER_FAIL");

    assert_eq!(
        processed, 0,
        "failed email send should not mark invoice event processed"
    );
    let persisted_invoice = invoices::Entity::find_by_id(invoice.invoice_id)
        .one(&db)
        .await
        .expect("query persisted invoice");
    assert!(
        persisted_invoice.is_some(),
        "invoice row must persist even when email fails"
    );

    let invoice_event = outbox_events::Entity::find()
        .filter(outbox_events::Column::EventType.eq(INVOICE_GENERATED))
        .filter(outbox_events::Column::AggregateId.eq(seeded.order_id.to_string()))
        .order_by_desc(outbox_events::Column::EventId)
        .one(&db)
        .await
        .expect("query invoice outbox event")
        .expect("invoice event should exist");
    assert_eq!(
        invoice_event.status,
        OutboxStatus::Pending,
        "failed delivery must leave event retryable"
    );
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn missing_email_is_handled_safely_without_crashing_invoice_generation() {
    let db = db_conn().await;
    let txn = db.begin().await.expect("begin");
    let seeded = seed_order(
        &txn,
        &unique_tag("missing_email"),
        "confirmed",
        "prepaid",
        PaymentStatus::Captured,
        "",
    )
    .await;

    let invoice = ensure_invoice_for_order(&txn, seeded.order_id, "itest")
        .await
        .expect("invoice generation should not crash for missing email")
        .expect("invoice expected");
    let snapshot: InvoiceDocumentSnapshot =
        serde_json::from_value(invoice.snapshot_json.clone()).expect("parse snapshot");
    assert!(snapshot.customer_email.is_empty());
    txn.rollback().await.expect("rollback");
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn generated_invoice_blob_contains_expected_content_and_no_fallback_text() {
    let db = db_conn().await;
    let txn = db.begin().await.expect("begin");
    let seeded = seed_order(
        &txn,
        &unique_tag("content_check"),
        "confirmed",
        "cod",
        PaymentStatus::Pending,
        "invoice_content@example.com",
    )
    .await;

    let invoice = ensure_invoice_for_order(&txn, seeded.order_id, "itest")
        .await
        .expect("invoice generation")
        .expect("invoice expected");
    let pdf_bytes = BASE64_STANDARD
        .decode(invoice.pdf_blob.as_bytes())
        .expect("base64 pdf decode");
    assert!(pdf_bytes.starts_with(b"%PDF-"), "expected PDF header");
    assert!(
        pdf_bytes.len() < 300 * 1024,
        "invoice pdf size {} exceeds 300KB",
        pdf_bytes.len()
    );

    let text = extract_text_from_mem(&pdf_bytes).expect("extract text from generated invoice");
    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(normalized.contains("TAX INVOICE"));
    assert!(normalized.contains("Sudatta's"));
    assert!(normalized.contains("Designer Boutique"));
    assert!(normalized.contains("Invoice Number"));
    assert!(normalized.contains("Sold By"));
    assert!(normalized.contains("Bill To"));
    assert!(normalized.contains("Ship To"));
    assert!(normalized.contains("sudattasdesignerboutique@gmail.com"));
    assert!(normalized.contains("Cash on Delivery"));
    assert!(normalized.contains("To be collected on delivery"));
    assert!(!normalized.contains("Invoice rendering had a temporary formatting issue"));
    assert!(
        normalized.contains("Invoice Test User"),
        "expected Bill To customer name"
    );
    assert!(
        normalized.contains("example.com"),
        "expected Bill To email domain"
    );
    assert!(
        !normalized.contains("@ "),
        "email text should not split directly after @: {normalized}"
    );
    assert!(
        normalized.contains("Grand Total") && normalized.contains("\u{20B9}115.00"),
        "expected grand total amount in extracted text: {normalized}"
    );
    txn.rollback().await.expect("rollback");
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn stale_fallback_invoice_blob_is_regenerated() {
    let db = db_conn().await;
    let txn = db.begin().await.expect("begin");
    let seeded = seed_order(
        &txn,
        &unique_tag("stale_repair"),
        "confirmed",
        "cod",
        PaymentStatus::Pending,
        "invoice_stale@example.com",
    )
    .await;

    let first = ensure_invoice_for_order(&txn, seeded.order_id, "itest")
        .await
        .expect("first invoice")
        .expect("invoice expected");

    let mut stale: invoices::ActiveModel = first.clone().into();
    let fake_pdf = b"%PDF-1.4 Invoice rendering had a temporary formatting issue.";
    stale.pdf_blob = ActiveValue::Set(BASE64_STANDARD.encode(fake_pdf));
    let _ = stale.update(&txn).await.expect("update stale blob");

    let repaired = ensure_invoice_for_order(&txn, seeded.order_id, "itest_repair")
        .await
        .expect("repair invoice")
        .expect("invoice expected after repair");

    let repaired_pdf = BASE64_STANDARD
        .decode(repaired.pdf_blob.as_bytes())
        .expect("decode repaired pdf");
    let repaired_text = extract_text_from_mem(&repaired_pdf).expect("extract repaired pdf text");
    assert!(
        !repaired_text.contains("Invoice rendering had a temporary formatting issue"),
        "stale fallback marker should be removed after regeneration"
    );
    assert!(
        repaired_text.contains("Sudatta's") && repaired_text.contains("Designer Boutique"),
        "regenerated invoice should contain text branding"
    );
    txn.rollback().await.expect("rollback");
}
