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

    // The order-received email doubles as the invoice email — attach the PDF here instead of
    // InvoiceGenerated sending its own separate email (see deliver_invoice_generated's doc
    // comment). Generation normally finishes, in the same procedure, just before this
    // PaymentCaptured event is even enqueued, so by outbox-delivery time the invoice row
    // should already exist; if it genuinely doesn't (race, or generation failed), send the
    // confirmation without an attachment rather than blocking it on the invoice.
    if matches!(kind, OrderEmailKind::PaymentCaptured) {
        if let Some(attachment) = load_invoice_attachment(db, order_id).await? {
            return send_transactional_email_with_attachments(
                &snap.customer_email,
                &subject,
                &text,
                &html,
                &[attachment],
            )
            .await;
        }
    }

    send_transactional_email(&snap.customer_email, &subject, &text, &html).await
}

/// Load an order's invoice PDF as an email attachment, if one exists and decodes cleanly.
/// Returns `Ok(None)` (not an error) for any of "no invoice yet" / "malformed payload" — those
/// are reasons to send the confirmation without an attachment, not to fail delivery outright.
async fn load_invoice_attachment(
    db: &DatabaseConnection,
    order_id: i64,
) -> Result<Option<EmailAttachment>, Status> {
    let Some(invoice) = invoices::Entity::find()
        .filter(invoices::Column::OrderId.eq(order_id))
        .one(db)
        .await
        .map_err(|e| Status::internal(e.to_string()))?
    else {
        return Ok(None);
    };

    let Ok(pdf_bytes) = BASE64_STANDARD.decode(invoice.pdf_blob.as_bytes()) else {
        warn!(
            order_id,
            "outbox: invoice PDF payload decode failed; sending confirmation without attachment"
        );
        return Ok(None);
    };
    if !pdf_bytes.starts_with(b"%PDF-") {
        warn!(
            order_id,
            "outbox: invoice PDF payload malformed; sending confirmation without attachment"
        );
        return Ok(None);
    }

    Ok(Some(EmailAttachment {
        filename: format!("Invoice_{}.pdf", invoice.invoice_number),
        content_base64: BASE64_STANDARD.encode(&pdf_bytes),
        mime_type: "application/pdf".to_string(),
    }))
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

/// InvoiceGenerated no longer sends its own email — the PaymentCaptured handler above now
/// attaches the invoice PDF to the order-received email directly, so a customer gets one
/// email per order instead of two (a plain "your invoice is ready" email arriving seconds
/// after the branded order-confirmation email, both for the same event). The outbox event is
/// still enqueued by `ensure_invoice_for_order` (unrelated bookkeeping may depend on its
/// existence), so it's consumed here rather than left to retry forever with nothing to do.
async fn deliver_invoice_generated(
    _db: &DatabaseConnection,
    event: &outbox_events::Model,
) -> Result<(), Status> {
    info!(
        event_id = event.event_id,
        "outbox: InvoiceGenerated — no separate email sent; invoice is attached to the PaymentCaptured email instead"
    );
    Ok(())
}
