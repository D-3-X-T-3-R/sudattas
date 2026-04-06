//! Load order + customer + lines for transactional emails.

use core_db_entities::entity::{order_details, order_status, orders, shipping_addresses, users};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde_json::Value;
use tonic::Status;

/// Snapshot used to render order-related emails (avoids holding DB handles in templates).
#[derive(Debug, Clone)]
pub struct OrderMailSnapshot {
    pub order_id: i64,
    pub order_number: Option<String>,
    pub order_date_rfc3339: String,
    pub customer_email: String,
    pub customer_name: String,
    pub currency: String,
    pub grand_total_paise: i64,
    pub status_name: String,
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

pub fn format_inr_paise(paise: i64) -> String {
    let sign = if paise < 0 { "-" } else { "" };
    let abs = paise.unsigned_abs();
    let rupees = abs / 100;
    let sub = abs % 100;
    format!("{}₹{}.{:02}", sign, rupees, sub)
}

fn html_escape(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '"' => "&quot;".to_string(),
            _ => c.to_string(),
        })
        .collect()
}

fn store_display_name() -> String {
    std::env::var("STORE_DISPLAY_NAME").unwrap_or_else(|_| "Sudattas".to_string())
}

fn storefront_url() -> String {
    std::env::var("STOREFRONT_URL").unwrap_or_else(|_| "https://sudattas.com".to_string())
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

    Ok(Some(OrderMailSnapshot {
        order_id: order.order_id,
        order_number: order.order_number.clone(),
        order_date_rfc3339: order.order_date.to_rfc3339(),
        customer_email: email,
        customer_name,
        currency,
        grand_total_paise: order.grand_total_minor,
        status_name,
        lines,
        shipping_block,
    }))
}

pub fn build_payment_captured_email(s: &OrderMailSnapshot) -> (String, String, String) {
    let store = store_display_name();
    let oid = s
        .order_number
        .clone()
        .unwrap_or_else(|| s.order_id.to_string());
    let subject = format!("Order received — order #{oid} ({store})");
    let total = format_inr_paise(s.grand_total_paise);
    let url = storefront_url();

    let mut text = String::new();
    text.push_str(&format!("Hi {},\n\n", s.customer_name));
    text.push_str(&format!(
        "Thank you for your order. We've received order #{} and your payment was successful.\n\n",
        oid
    ));
    text.push_str("— Order summary —\n");
    for line in &s.lines {
        text.push_str(&format!(
            "  • {} × {} — {}\n",
            line.title,
            line.quantity,
            format_inr_paise(line.line_total_paise)
        ));
    }
    text.push_str(&format!("\nAmount paid ({}) — {}\n", s.currency, total));
    text.push_str(&format!("Order status: {}\n\n", s.status_name));
    text.push_str("Ship to:\n");
    text.push_str(&s.shipping_block);
    text.push_str(&format!("\n\n{}", url));

    let mut html = String::new();
    html.push_str(
        "<!DOCTYPE html><html><body style=\"font-family:system-ui,sans-serif;line-height:1.5\">",
    );
    html.push_str(&format!("<p>Hi {},</p>", html_escape(&s.customer_name)));
    html.push_str(&format!(
        "<p>Thank you for your order. We've received <strong>order #{}</strong> and your payment was successful.</p>",
        html_escape(&oid)
    ));
    html.push_str("<h3>Order summary</h3><ul>");
    for line in &s.lines {
        html.push_str(&format!(
            "<li>{} × {} — {}</li>",
            html_escape(&line.title),
            line.quantity,
            html_escape(&format_inr_paise(line.line_total_paise))
        ));
    }
    html.push_str("</ul>");
    html.push_str(&format!(
        "<p><strong>Amount paid ({})</strong> — {}</p>",
        html_escape(&s.currency),
        html_escape(&total)
    ));
    html.push_str(&format!(
        "<p>Order status: {}</p>",
        html_escape(&s.status_name)
    ));
    html.push_str(&format!(
        "<pre style=\"white-space:pre-wrap\">{}</pre>",
        html_escape(&s.shipping_block)
    ));
    html.push_str(&format!(
        "<p><a href=\"{}\">{}</a></p>",
        html_escape(&url),
        html_escape(&url)
    ));
    html.push_str("</body></html>");

    (subject, text, html)
}

pub fn build_shipped_email(s: &OrderMailSnapshot) -> (String, String, String) {
    let oid = s
        .order_number
        .clone()
        .unwrap_or_else(|| s.order_id.to_string());
    let subject = format!("Your order #{oid} has shipped");
    let text = format!(
        "Hi {},\n\nOrder #{} is on its way. Status: {}.\n\n{}\n",
        s.customer_name,
        oid,
        s.status_name,
        storefront_url()
    );
    let html = format!(
        "<p>Hi {},</p><p>Order <strong>#{}</strong> is on its way. Status: {}.</p>",
        html_escape(&s.customer_name),
        html_escape(&oid),
        html_escape(&s.status_name)
    );
    (subject, text, html)
}

pub fn build_delivered_email(s: &OrderMailSnapshot) -> (String, String, String) {
    let oid = s
        .order_number
        .clone()
        .unwrap_or_else(|| s.order_id.to_string());
    let subject = format!("Your order #{oid} was delivered");
    let text = format!(
        "Hi {},\n\nOrder #{} shows as delivered. Thank you for shopping with us!\n\n{}",
        s.customer_name,
        oid,
        storefront_url()
    );
    let html = format!(
        "<p>Hi {},</p><p>Order <strong>#{}</strong> shows as <strong>delivered</strong>. Thank you!</p>",
        html_escape(&s.customer_name),
        html_escape(&oid)
    );
    (subject, text, html)
}

pub fn build_refunded_email(s: &OrderMailSnapshot) -> (String, String, String) {
    let oid = s
        .order_number
        .clone()
        .unwrap_or_else(|| s.order_id.to_string());
    let subject = format!("Refund update — order #{oid}");
    let text = format!(
        "Hi {},\n\nOrder #{} has been updated to refunded status. If you have questions, contact support.\n\n{}",
        s.customer_name,
        oid,
        storefront_url()
    );
    let html = format!(
        "<p>Hi {},</p><p>Order <strong>#{}</strong> has been updated to <strong>refunded</strong>.</p>",
        html_escape(&s.customer_name),
        html_escape(&oid)
    );
    (subject, text, html)
}

pub fn build_abandoned_cart_email(to_name: &str) -> (String, String, String) {
    let store = store_display_name();
    let url = storefront_url();
    let subject = format!("You left something in your cart — {store}");
    let text = format!(
        "Hi {},\n\nYou still have items waiting at {}. Come back when you're ready.\n\n{}",
        to_name, store, url
    );
    let html = format!(
        "<p>Hi {},</p><p>You still have items waiting at <strong>{}</strong>.</p><p><a href=\"{}\">Continue shopping</a></p>",
        html_escape(to_name),
        html_escape(&store),
        html_escape(&url)
    );
    (subject, text, html)
}
