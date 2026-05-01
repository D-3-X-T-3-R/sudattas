//! P1 Notification delivery: load outbox payload, render email, send via provider (log or Resend).
//! Returns Ok on skip (missing order / no email) so stale outbox rows do not retry forever.
//! Returns Err on transport failure so the worker leaves the event Pending for retry.

use crate::handlers::outbox::{
    ABANDONED_CART, DELIVERED, INVOICE_GENERATED, ORDER_PLACED, PAYMENT_CAPTURED, REFUNDED, SHIPPED,
};
use crate::notifications::email_provider::send_transactional_email;
use crate::notifications::order_mail::{
    build_abandoned_cart_email, build_delivered_email, build_payment_captured_email,
    build_refunded_email, build_shipped_email, load_order_mail_snapshot,
    parse_abandoned_cart_email, parse_payload_order_id,
};
use core_db_entities::entity::{invoices, outbox_events};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use tonic::Status;
use tracing::{info, warn};

/// Deliver one outbox event. Uses `db` for read-only enrichment (no long-held txn during HTTP).
/// Set env `OUTBOX_DELIVER_FAIL=1` to simulate delivery failure (for retry-path tests).
pub async fn deliver_event(
    db: &DatabaseConnection,
    event: &outbox_events::Model,
) -> Result<(), Status> {
    if std::env::var("OUTBOX_DELIVER_FAIL").as_deref() == Ok("1") {
        return Err(Status::internal("simulated delivery failure for test"));
    }

    match event.event_type.as_str() {
        // Confirmation email is sent on PaymentCaptured (paid), not when the order row is created.
        ORDER_PLACED => {
            info!(
                event_id = event.event_id,
                "outbox: OrderPlaced skipped (no email); use PaymentCaptured after successful payment"
            );
            Ok(())
        }
        PAYMENT_CAPTURED => deliver_order_typed(db, event, OrderEmailKind::PaymentCaptured).await,
        SHIPPED => deliver_order_typed(db, event, OrderEmailKind::Shipped).await,
        DELIVERED => deliver_order_typed(db, event, OrderEmailKind::Delivered).await,
        REFUNDED => deliver_order_typed(db, event, OrderEmailKind::Refunded).await,
        INVOICE_GENERATED => deliver_invoice_generated(db, event).await,
        ABANDONED_CART => deliver_abandoned_cart(event).await,
        other => {
            info!(
                event_type = other,
                event_id = event.event_id,
                "outbox: no template for event type; marking ok without send"
            );
            Ok(())
        }
    }
}

#[derive(Clone, Copy)]
enum OrderEmailKind {
    PaymentCaptured,
    Shipped,
    Delivered,
    Refunded,
}

async fn deliver_order_typed(
    db: &DatabaseConnection,
    event: &outbox_events::Model,
    kind: OrderEmailKind,
) -> Result<(), Status> {
    let Some(order_id) = parse_payload_order_id(&event.payload) else {
        warn!(
            event_id = event.event_id,
            event_type = %event.event_type,
            "outbox: missing order_id in payload; skip"
        );
        return Ok(());
    };

    let Some(snap) = load_order_mail_snapshot(db, order_id).await? else {
        warn!(
            event_id = event.event_id,
            order_id, "outbox: order or user not found; skip email"
        );
        return Ok(());
    };

    let (subject, text, html) = match kind {
        OrderEmailKind::PaymentCaptured => build_payment_captured_email(&snap),
        OrderEmailKind::Shipped => build_shipped_email(&snap),
        OrderEmailKind::Delivered => build_delivered_email(&snap),
        OrderEmailKind::Refunded => build_refunded_email(&snap),
    };

    send_transactional_email(&snap.customer_email, &subject, &text, &html).await
}

async fn deliver_abandoned_cart(event: &outbox_events::Model) -> Result<(), Status> {
    let Some(to) = parse_abandoned_cart_email(&event.payload) else {
        warn!(
            event_id = event.event_id,
            "outbox: AbandonedCart missing email; skip"
        );
        return Ok(());
    };
    let name = "there";
    let (subject, text, html) = build_abandoned_cart_email(name);
    send_transactional_email(&to, &subject, &text, &html).await
}

async fn deliver_invoice_generated(
    db: &DatabaseConnection,
    event: &outbox_events::Model,
) -> Result<(), Status> {
    let Some(order_id) = parse_payload_order_id(&event.payload) else {
        warn!(
            event_id = event.event_id,
            "outbox: InvoiceGenerated missing order_id; skip"
        );
        return Ok(());
    };

    let Some(invoice) = invoices::Entity::find()
        .filter(invoices::Column::OrderId.eq(order_id))
        .one(db)
        .await
        .map_err(|e| Status::internal(e.to_string()))?
    else {
        warn!(
            event_id = event.event_id,
            order_id, "outbox: invoice row not found; skip"
        );
        return Ok(());
    };

    let snapshot: crate::handlers::invoices::InvoiceDocumentSnapshot =
        match serde_json::from_value(invoice.snapshot_json.clone()) {
            Ok(v) => v,
            Err(e) => {
                warn!(
                    event_id = event.event_id,
                    order_id,
                    error = %e,
                    "outbox: invoice snapshot parse failed; skip"
                );
                return Ok(());
            }
        };

    let recipient = snapshot.customer_email.trim().to_string();
    if recipient.is_empty() {
        warn!(
            event_id = event.event_id,
            order_id, "outbox: invoice recipient missing email; skip"
        );
        return Ok(());
    }

    let storefront =
        std::env::var("STOREFRONT_URL").unwrap_or_else(|_| "https://sudattas.com".to_string());
    let download_url = format!(
        "{}/api/account/orders/{}/invoice",
        storefront.trim_end_matches('/'),
        order_id
    );
    let subject = format!("Your Sudatta's invoice for order #{}", order_id);
    let text = format!(
        "Hi {},\n\nYour invoice is ready.\nOrder number: {}\nInvoice number: {}\nTotal amount: {}\nPayment mode: {}\nDownload invoice: {}\n",
        snapshot.customer_name,
        order_id,
        snapshot.invoice_number,
        snapshot.grand_total_formatted,
        snapshot.payment_mode,
        download_url
    );
    let html = format!(
        "<p>Hi {},</p><p>Your invoice is ready.</p><ul><li>Order number: <strong>{}</strong></li><li>Invoice number: <strong>{}</strong></li><li>Total amount: <strong>{}</strong></li><li>Payment mode: <strong>{}</strong></li></ul><p><a href=\"{}\">Download invoice</a></p>",
        snapshot.customer_name,
        order_id,
        snapshot.invoice_number,
        snapshot.grand_total_formatted,
        snapshot.payment_mode,
        download_url
    );

    send_transactional_email(&recipient, &subject, &text, &html).await
}
