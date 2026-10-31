//! Load order + customer + lines for transactional emails, and build their content. The
//! actual header/body/footer HTML shell is shared with the newsletter — see `brand_email.rs`
//! — so an order confirmation looks like it came from the same store as everything else,
//! not a bare unstyled `<body>`.

use super::brand_email::{
    cta_button_html, html_escape, paragraph_html, render_branded_email, store_display_name,
    storefront_url,
};
use core_db_entities::entity::{order_details, order_status, orders, shipping_addresses, users};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde_json::Value;
use tonic::Status;

/// Snapshot used to render order-related emails (avoids holding DB handles in templates).
#[derive(Debug, Clone)]
pub struct OrderMailSnapshot {
    pub order_id: i64,
    pub public_order_ref: String,
    pub order_number: Option<String>,
    pub order_date_rfc3339: String,
    pub customer_email: String,
    pub customer_name: String,
    pub currency: String,
    pub grand_total_paise: i64,
    pub status_name: String,
    /// "cod" or "prepaid" (normalized lowercase), or empty if unset. Determines whether the
    /// confirmation email can honestly say "your payment was successful" — for COD it hasn't
    /// been collected yet, only committed to.
    pub payment_method: String,
    pub lines: Vec<OrderLineMail>,
    pub shipping_block: String,
}

#[derive(Debug, Clone)]
pub struct OrderLineMail {
    pub title: String,
    pub quantity: i64,
    pub line_total_paise: i64,
}

pub fn parse_payload_order_id(payload: &Value) -> Option<i64> {
    payload.get("order_id").and_then(|x| {
        if let Some(i) = x.as_i64() {
            return Some(i);
        }
        if let Some(u) = x.as_u64() {
            return Some(u as i64);
        }
        x.as_str()?.parse().ok()
    })
}

pub fn parse_abandoned_cart_email(payload: &Value) -> Option<String> {
    payload
        .get("email")
        .and_then(|x| x.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
}

/// The order number shown to the customer in email. Deliberately the plain numeric
/// `order_id`, not `public_order_ref` — the rest of the product (the profile page's order
/// history, the PDF invoice's "Order ID" field) shows the customer the bare numeric id, so an
/// email using a different-looking reference like "SUD-20260829-..." looks like a mismatch
/// even though it's the same order, rather than helping the customer look it up.
fn customer_visible_order_number(s: &OrderMailSnapshot) -> String {
    s.order_id.to_string()
}

pub fn format_inr_paise(paise: i64) -> String {
    let sign = if paise < 0 { "-" } else { "" };
    let abs = paise.unsigned_abs();
    let rupees = abs / 100;
    let sub = abs % 100;
    format!("{}₹{}.{:02}", sign, rupees, sub)
}

/// Load order snapshot for email. Returns `Ok(None)` if order row is missing (stale outbox / tests).
pub async fn load_order_mail_snapshot(
    db: &DatabaseConnection,
    order_id: i64,
) -> Result<Option<OrderMailSnapshot>, Status> {
    let order = orders::Entity::find_by_id(order_id)
        .one(db)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;
    let Some(order) = order else {
        return Ok(None);
    };

    let user = users::Entity::find_by_id(order.user_id)
        .one(db)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;
    let Some(user) = user else {
        return Ok(None);
    };

    let email = user.email.trim().to_string();
    if email.is_empty() {
        return Ok(None);
    }

    let customer_name = user
        .full_name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(user.username.as_str())
        .to_string();

    let ship = shipping_addresses::Entity::find_by_id(order.shipping_address_id)
        .one(db)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    let shipping_block = ship
        .map(|a| {
            let mut parts: Vec<String> = Vec::new();
            if let Some(r) = a.road.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
                parts.push(r.to_string());
            }
            if let Some(ap) = a
                .apartment_no_or_name
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                parts.push(ap.to_string());
            }
            parts.push(format!("{}, {} {}", a.city, a.state_region, a.postal_code));
            parts.push(a.country.clone());
            parts.join("\n")
        })
        .unwrap_or_else(|| "(Shipping address not found)".to_string());

    let status_row = order_status::Entity::find_by_id(order.status_id)
        .one(db)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;
    let status_name = status_row
        .map(|s| s.status_name)
        .unwrap_or_else(|| format!("status_id {}", order.status_id));

    let detail_rows = order_details::Entity::find()
        .filter(order_details::Column::OrderId.eq(order_id))
        .all(db)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    let lines: Vec<OrderLineMail> = detail_rows
        .into_iter()
        .map(|d| {
            let title = d
                .title
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from)
                .unwrap_or_else(|| format!("Item (variant {})", d.variant_id));
            let line_total_paise = (d.unit_price_minor as i64).saturating_mul(d.quantity);
            OrderLineMail {
                title,
                quantity: d.quantity,
                line_total_paise,
            }
        })
        .collect();

    let currency = order
        .currency
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("INR")
        .to_uppercase();

    let payment_method = order
        .payment_method
        .as_deref()
        .map(str::trim)
        .map(str::to_lowercase)
        .unwrap_or_default();

    Ok(Some(OrderMailSnapshot {
        order_id: order.order_id,
        public_order_ref: order.public_order_ref.clone(),
        order_number: order.order_number.clone(),
        order_date_rfc3339: order.order_date.to_rfc3339(),
        customer_email: email,
        customer_name,
        currency,
        grand_total_paise: order.grand_total_minor,
        status_name,
        payment_method,
        lines,
        shipping_block,
    }))
}

/// The line-items table shared by every order email that shows an order summary.
fn order_summary_table_html(lines: &[OrderLineMail]) -> String {
    let rows: String = lines
        .iter()
        .map(|line| {
            format!(
                "<tr><td style=\"padding:10px 0;border-bottom:1px solid #EFE8DC;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Helvetica,Arial,sans-serif;font-size:14px;color:#2D2A26;\">{}</td>\
                <td style=\"padding:10px 0;border-bottom:1px solid #EFE8DC;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Helvetica,Arial,sans-serif;font-size:14px;color:#6B6257;text-align:center;white-space:nowrap;\">&times;{}</td>\
                <td style=\"padding:10px 0;border-bottom:1px solid #EFE8DC;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Helvetica,Arial,sans-serif;font-size:14px;color:#2D2A26;text-align:right;white-space:nowrap;\">{}</td></tr>",
                html_escape(&line.title),
                line.quantity,
                html_escape(&format_inr_paise(line.line_total_paise))
            )
        })
        .collect();
    format!(
        "<table role=\"presentation\" width=\"100%\" cellpadding=\"0\" cellspacing=\"0\" style=\"margin:4px 0 20px;border-collapse:collapse;\">\
        <tr><td colspan=\"3\" style=\"padding-bottom:8px;border-bottom:1px solid #DED7C8;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Helvetica,Arial,sans-serif;font-size:11px;letter-spacing:1px;text-transform:uppercase;color:#6B6257;\">Order summary</td></tr>\
        {rows}\
        </table>"
    )
}

fn shipping_block_html(shipping_block: &str) -> String {
    format!(
        "<p style=\"margin:0 0 18px;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Helvetica,Arial,sans-serif;font-size:13px;line-height:1.6;color:#6B6257;\"><strong style=\"color:#2D2A26;\">Ship to</strong><br>{}</p>",
        html_escape(shipping_block).replace('\n', "<br>")
    )
}

pub fn build_payment_captured_email(s: &OrderMailSnapshot) -> (String, String, String) {
    let store = store_display_name();
    let oid = customer_visible_order_number(s);
    let subject = format!("Order received — order #{oid} ({store})");
    let total = format_inr_paise(s.grand_total_paise);
    let url = storefront_url();
    let orders_url = format!("{}/profile", url.trim_end_matches('/'));
    let is_cod = s.payment_method == "cod";

    // COD hasn't actually been paid yet — it's collected on delivery. Saying "payment was
    // successful" / "amount paid" for a COD order is simply false; the invoice already
    // correctly distinguishes this (Payment Mode / Payment Status), the email must match.
    let confirmation_line = if is_cod {
        format!(
            "Thank you for your order. We've received order #{} — pay on delivery.",
            oid
        )
    } else {
        format!(
            "Thank you for your order. We've received order #{} and your payment was successful.",
            oid
        )
    };
    let confirmation_line_html = if is_cod {
        format!(
            "Thank you for your order. We've received <strong>order #{}</strong> — pay on delivery.",
            html_escape(&oid)
        )
    } else {
        format!(
            "Thank you for your order. We've received <strong>order #{}</strong> and your payment was successful.",
            html_escape(&oid)
        )
    };
    let amount_label = if is_cod {
        format!("Amount due on delivery ({})", s.currency)
    } else {
        format!("Amount paid ({})", s.currency)
    };

    let mut text = String::new();
    text.push_str(&format!("Hi {},\n\n", s.customer_name));
    text.push_str(&confirmation_line);
    text.push_str("\n\n");
    text.push_str("— Order summary —\n");
    for line in &s.lines {
        text.push_str(&format!(
            "  • {} × {} — {}\n",
            line.title,
            line.quantity,
            format_inr_paise(line.line_total_paise)
        ));
    }
    text.push_str(&format!("\n{} — {}\n", amount_label, total));
    text.push_str(&format!("Order status: {}\n\n", s.status_name));
    text.push_str("Ship to:\n");
    text.push_str(&s.shipping_block);
    text.push_str("\n\nYour invoice is attached as a PDF.");
    text.push_str(&format!("\n\n{}", orders_url));

    let body_html = format!(
        "<h1 style=\"margin:0 0 20px;font-family:Georgia,'Times New Roman',serif;font-size:27px;line-height:1.3;color:#2D2A26;font-weight:600;\">Order received</h1>\
        {greeting}{confirmed}{summary}{paid}{status}{ship_to}{invoice_note}{cta}",
        greeting = paragraph_html(&format!("Hi {},", html_escape(&s.customer_name))),
        confirmed = paragraph_html(&confirmation_line_html),
        summary = order_summary_table_html(&s.lines),
        invoice_note = paragraph_html("Your invoice is attached as a PDF."),
        paid = paragraph_html(&format!(
            "<strong>{}</strong> — {}",
            html_escape(&amount_label),
            html_escape(&total)
        )),
        status = paragraph_html(&format!("Order status: {}", html_escape(&s.status_name))),
        ship_to = shipping_block_html(&s.shipping_block),
        cta = cta_button_html("View your orders", &orders_url),
    );
    let html = render_branded_email(&subject, &body_html, None);

    (subject, text, html)
}

pub fn build_shipped_email(s: &OrderMailSnapshot) -> (String, String, String) {
    let oid = customer_visible_order_number(s);
    let subject = format!("Your order #{oid} has shipped");
    let url = storefront_url();
    let orders_url = format!("{}/profile", url.trim_end_matches('/'));
    let text = format!(
        "Hi {},\n\nOrder #{} is on its way. Status: {}.\n\n{}\n",
        s.customer_name, oid, s.status_name, orders_url
    );
    let body_html = format!(
        "<h1 style=\"margin:0 0 20px;font-family:Georgia,'Times New Roman',serif;font-size:27px;line-height:1.3;color:#2D2A26;font-weight:600;\">Your order is on its way</h1>\
        {greeting}{status}{cta}",
        greeting = paragraph_html(&format!(
            "Hi {}, order <strong>#{}</strong> is on its way.",
            html_escape(&s.customer_name),
            html_escape(&oid)
        )),
        status = paragraph_html(&format!("Status: {}", html_escape(&s.status_name))),
        cta = cta_button_html("View your orders", &orders_url),
    );
    let html = render_branded_email(&subject, &body_html, None);
    (subject, text, html)
}

pub fn build_delivered_email(s: &OrderMailSnapshot) -> (String, String, String) {
    let oid = customer_visible_order_number(s);
    let subject = format!("Your order #{oid} was delivered");
    let url = storefront_url();
    let shop_url = url.trim_end_matches('/').to_string();
    let text = format!(
        "Hi {},\n\nOrder #{} shows as delivered. Thank you for shopping with us!\n\n{}",
        s.customer_name, oid, shop_url
    );
    let body_html = format!(
        "<h1 style=\"margin:0 0 20px;font-family:Georgia,'Times New Roman',serif;font-size:27px;line-height:1.3;color:#2D2A26;font-weight:600;\">Delivered!</h1>\
        {greeting}{cta}",
        greeting = paragraph_html(&format!(
            "Hi {}, order <strong>#{}</strong> shows as <strong>delivered</strong>. Thank you for shopping with us!",
            html_escape(&s.customer_name),
            html_escape(&oid)
        )),
        cta = cta_button_html("Shop again", &shop_url),
    );
    let html = render_branded_email(&subject, &body_html, None);
    (subject, text, html)
}

pub fn build_refunded_email(s: &OrderMailSnapshot) -> (String, String, String) {
    let oid = customer_visible_order_number(s);
    let subject = format!("Refund update — order #{oid}");
    let url = storefront_url();
    let orders_url = format!("{}/profile", url.trim_end_matches('/'));
    let text = format!(
        "Hi {},\n\nOrder #{} has been updated to refunded status. If you have questions, contact support.\n\n{}",
        s.customer_name, oid, orders_url
    );
    let body_html = format!(
        "<h1 style=\"margin:0 0 20px;font-family:Georgia,'Times New Roman',serif;font-size:27px;line-height:1.3;color:#2D2A26;font-weight:600;\">Refund update</h1>\
        {greeting}{cta}",
        greeting = paragraph_html(&format!(
            "Hi {}, order <strong>#{}</strong> has been updated to <strong>refunded</strong>. If you have questions, just reply to this email.",
            html_escape(&s.customer_name),
            html_escape(&oid)
        )),
        cta = cta_button_html("View your orders", &orders_url),
    );
    let html = render_branded_email(&subject, &body_html, None);
    (subject, text, html)
}

pub fn build_abandoned_cart_email(to_name: &str) -> (String, String, String) {
    let store = store_display_name();
    let url = storefront_url();
    let bag_url = format!("{}/bag", url.trim_end_matches('/'));
    let subject = format!("You left something in your cart — {store}");
    let text = format!(
        "Hi {},\n\nYou still have items waiting at {}. Come back when you're ready.\n\n{}",
        to_name, store, bag_url
    );
    let body_html = format!(
        "<h1 style=\"margin:0 0 20px;font-family:Georgia,'Times New Roman',serif;font-size:27px;line-height:1.3;color:#2D2A26;font-weight:600;\">You left something behind</h1>\
        {greeting}{cta}",
        greeting = paragraph_html(&format!(
            "Hi {}, you still have items waiting in your bag at <strong>{}</strong>.",
            html_escape(to_name),
            html_escape(&store)
        )),
        cta = cta_button_html("Continue shopping", &bag_url),
    );
    let html = render_branded_email(&subject, &body_html, None);
    (subject, text, html)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_snapshot() -> OrderMailSnapshot {
        OrderMailSnapshot {
            order_id: 42,
            public_order_ref: "SUD-20260830-ABC123".to_string(),
            order_number: None,
            order_date_rfc3339: "2026-08-30T00:00:00Z".to_string(),
            customer_email: "shopper@example.com".to_string(),
            customer_name: "Priya".to_string(),
            currency: "INR".to_string(),
            grand_total_paise: 179999,
            status_name: "confirmed".to_string(),
            payment_method: "prepaid".to_string(),
            lines: vec![OrderLineMail {
                title: "Kanjivaram Saree".to_string(),
                quantity: 1,
                line_total_paise: 179999,
            }],
            shipping_block: "12 MG Road\nBengaluru, Karnataka 560001\nIndia".to_string(),
        }
    }

    fn sample_cod_snapshot() -> OrderMailSnapshot {
        OrderMailSnapshot {
            payment_method: "cod".to_string(),
            ..sample_snapshot()
        }
    }

    #[test]
    fn payment_captured_email_uses_branded_shell_with_order_details() {
        let (subject, text, html) = build_payment_captured_email(&sample_snapshot());
        assert!(subject.contains("Order received"));
        // Branded shell markers.
        assert!(html.contains("Order received"));
        // Store name is uppercased via CSS text-transform, not in the raw HTML text node.
        assert!(html.contains("Sudatta"));
        assert!(html.contains("Order summary"));
        assert!(html.contains("Kanjivaram Saree"));
        assert!(html.contains("View your orders"));
        assert!(html.contains("border-radius:999px;background-color:#C9A646"));
        // Plain-text fallback still has the same substance.
        assert!(text.contains("Kanjivaram Saree"));
        assert!(text.contains("Priya"));
    }

    #[test]
    fn customer_visible_order_number_matches_what_the_rest_of_the_product_shows() {
        // The profile page's order history and the PDF invoice's "Order ID" field both show
        // the bare numeric order_id — the email must use the same one, not public_order_ref,
        // or a customer can't tell it's the same order.
        let (subject, _, html) = build_payment_captured_email(&sample_snapshot());
        assert!(subject.contains("#42"));
        assert!(html.contains("#42"));
        assert!(!subject.contains("SUD-20260830-ABC123"));
        assert!(!html.contains("SUD-20260830-ABC123"));
    }

    #[test]
    fn prepaid_email_says_payment_was_successful() {
        let (_, text, html) = build_payment_captured_email(&sample_snapshot());
        assert!(text.contains("payment was successful"));
        assert!(text.contains("Amount paid"));
        assert!(html.contains("payment was successful"));
        assert!(html.contains("Amount paid"));
    }

    #[test]
    fn cod_email_never_claims_payment_was_successful() {
        // This was a real bug: a COD order (nothing collected yet, collected on delivery)
        // said "your payment was successful" and "Amount paid" — directly contradicting the
        // attached invoice's own "Payment Status: To be collected on delivery".
        let (_, text, html) = build_payment_captured_email(&sample_cod_snapshot());
        assert!(!text.to_lowercase().contains("payment was successful"));
        assert!(!text.contains("Amount paid"));
        assert!(!html.to_lowercase().contains("payment was successful"));
        assert!(!html.contains("Amount paid"));
        assert!(text.to_lowercase().contains("pay on delivery"));
        assert!(html.to_lowercase().contains("pay on delivery"));
        assert!(text.contains("Amount due on delivery"));
        assert!(html.contains("Amount due on delivery"));
    }

    #[test]
    fn order_emails_are_marked_no_reply() {
        let (_, _, html) = build_payment_captured_email(&sample_snapshot());
        assert!(html.to_lowercase().contains("don't reply"));
    }

    #[test]
    fn shipped_email_uses_branded_shell() {
        let (_, _, html) = build_shipped_email(&sample_snapshot());
        assert!(html.contains("Your order is on its way"));
        assert!(html.contains("View your orders"));
    }

    #[test]
    fn delivered_email_uses_branded_shell() {
        let (_, _, html) = build_delivered_email(&sample_snapshot());
        assert!(html.contains("Delivered!"));
        assert!(html.contains("Shop again"));
    }

    #[test]
    fn refunded_email_uses_branded_shell() {
        let (_, _, html) = build_refunded_email(&sample_snapshot());
        assert!(html.contains("Refund update"));
        assert!(html.contains("refunded"));
    }

    #[test]
    fn abandoned_cart_email_uses_branded_shell() {
        let (_, _, html) = build_abandoned_cart_email("Priya");
        assert!(html.contains("You left something behind"));
        assert!(html.contains("Continue shopping"));
        assert!(html.contains("/bag"));
    }

    #[test]
    fn order_emails_never_include_an_unsubscribe_link() {
        // These are required service emails, not marketing — they must not get the
        // newsletter's unsubscribe footer.
        let (_, _, html) = build_payment_captured_email(&sample_snapshot());
        assert!(!html.to_lowercase().contains("unsubscribe"));
    }
}
