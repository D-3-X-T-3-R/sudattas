//! P1 Notification delivery: load outbox payload, render email, send via provider (log or Resend).
//! Returns Ok on skip (missing order / no email) so stale outbox rows do not retry forever.
//! Returns Err on transport failure so the worker leaves the event Pending for retry.

use crate::handlers::outbox::{
    ABANDONED_CART, DELIVERED, INVOICE_GENERATED, ORDER_PLACED, PAYMENT_CAPTURED, REFUNDED, SHIPPED,
};
use crate::notifications::email_provider::{
    send_transactional_email, send_transactional_email_with_attachments, EmailAttachment,
};
use crate::notifications::order_mail::{
    build_abandoned_cart_email, build_delivered_email, build_payment_captured_email,
    build_refunded_email, build_shipped_email, load_order_mail_snapshot,
    parse_abandoned_cart_email, parse_payload_order_id,
};
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use core_db_entities::entity::{invoices, outbox_events};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use tonic::Status;
use tracing::{info, warn};

fn payment_mode_label(raw: &str) -> &'static str {
    if raw.eq_ignore_ascii_case("cod") {
        "Cash on Delivery"
    } else {
        "Prepaid"
    }
}

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

    let pdf_bytes = match BASE64_STANDARD.decode(invoice.pdf_blob.as_bytes()) {
        Ok(bytes) => bytes,
        Err(e) => {
            warn!(
                event_id = event.event_id,
                order_id,
                error = %e,
                "outbox: invoice PDF payload decode failed; skip"
            );
            return Ok(());
        }
    };
    if !pdf_bytes.starts_with(b"%PDF-") {
        warn!(
            event_id = event.event_id,
            order_id, "outbox: invoice PDF payload malformed; skip"
        );
        return Ok(());
    }

    let attachment = EmailAttachment {
        filename: format!("Invoice_{}.pdf", snapshot.invoice_number),
        content_base64: BASE64_STANDARD.encode(&pdf_bytes),
        mime_type: "application/pdf".to_string(),
    };
    let attachments = vec![attachment];

    let subject = format!("Your Sudatta's invoice for order #{}", order_id);
    let text = format!(
        "Hi {},\n\nYour invoice is ready.\nOrder number: {}\nInvoice number: {}\nTotal amount: {}\nPayment mode: {}\n\nPlease find the invoice attached as a PDF.\n",
        snapshot.customer_name,
        order_id,
        snapshot.invoice_number,
        snapshot.grand_total_formatted,
        payment_mode_label(&snapshot.payment_mode),
    );
    let html = format!(
        "<p>Hi {},</p><p>Your invoice is ready.</p><ul><li>Order number: <strong>{}</strong></li><li>Invoice number: <strong>{}</strong></li><li>Total amount: <strong>{}</strong></li><li>Payment mode: <strong>{}</strong></li></ul><p>Please find your invoice attached as a PDF.</p>",
        snapshot.customer_name,
        order_id,
        snapshot.invoice_number,
        snapshot.grand_total_formatted,
        payment_mode_label(&snapshot.payment_mode)
    );

    send_transactional_email_with_attachments(&recipient, &subject, &text, &html, &attachments)
        .await
}
