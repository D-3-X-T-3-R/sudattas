//! Admin alerts: a plain email to `ADMIN_NOTIFY_EMAILS` and an ntfy push to `NTFY_TOPIC`, each
//! driven by its own outbox event (`AdminOrderEmail` / `AdminOrderPush`) so a failing channel
//! retries on its own without re-sending the customer's emails. The payload's `kind` says what
//! happened — new paid order, customer cancellation, exchange request, or Shiprocket booking.

use super::brand_email::{html_escape, storefront_url};
use super::order_mail::{format_inr_paise, OrderMailSnapshot};
use serde_json::{json, Value};
use tonic::Status;
use tracing::info;

const DEFAULT_NTFY_URL: &str = "https://ntfy.sh";
/// Shiprocket seller-panel order page; `{id}` is the Shiprocket order id.
const DEFAULT_SHIPROCKET_PANEL_ORDER_URL: &str =
    "https://app.shiprocket.in/seller/orders/details/{id}";
const SHIPROCKET_TRACKING_URL: &str = "https://shiprocket.co/tracking/";

/// What the admin is being told about. Serialized into the outbox payload at enqueue time so the
/// alert describes the event itself (e.g. which items were cancelled), not just the order's
/// current state at delivery time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdminAlert {
    Placed,
    Cancelled {
        full: bool,
        items: Vec<String>,
    },
    ExchangeRequested {
        item: String,
        from: String,
        to: String,
        reason: String,
    },
    ShipmentBooked {
        courier: String,
        awb: String,
        shiprocket_order_id: Option<String>,
        /// Pre-formatted for display (IST), or `None` if pickup scheduling failed.
        pickup: Option<String>,
    },
}

impl AdminAlert {
    pub fn to_payload(&self, order_id: i64, user_id: i64) -> Value {
        let mut payload = match self {
            AdminAlert::Placed => json!({ "kind": "placed" }),
            AdminAlert::Cancelled { full, items } => {
                json!({ "kind": "cancelled", "full": full, "items": items })
            }
            AdminAlert::ExchangeRequested {
                item,
                from,
                to,
                reason,
            } => json!({
                "kind": "exchange_requested",
                "item": item,
                "from": from,
                "to": to,
                "reason": reason,
            }),
            AdminAlert::ShipmentBooked {
                courier,
                awb,
                shiprocket_order_id,
                pickup,
            } => json!({
                "kind": "shipment_booked",
                "courier": courier,
                "awb": awb,
                "shiprocket_order_id": shiprocket_order_id,
                "pickup": pickup,
            }),
        };
        payload["order_id"] = json!(order_id);
        payload["user_id"] = json!(user_id);
        payload
    }

    /// Rows enqueued before `kind` existed were always new-order alerts, so missing → `Placed`.
    pub fn from_payload(payload: &Value) -> Self {
        let str_field = |key: &str| {
            payload
                .get(key)
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string()
        };
        match payload.get("kind").and_then(Value::as_str) {
            Some("cancelled") => AdminAlert::Cancelled {
                full: payload.get("full").and_then(Value::as_bool).unwrap_or(true),
                items: payload
                    .get("items")
                    .and_then(Value::as_array)
                    .map(|arr| {
                        arr.iter()
                            .filter_map(Value::as_str)
                            .map(String::from)
                            .collect()
                    })
                    .unwrap_or_default(),
            },
            Some("exchange_requested") => AdminAlert::ExchangeRequested {
                item: str_field("item"),
                from: str_field("from"),
                to: str_field("to"),
                reason: str_field("reason"),
            },
            Some("shipment_booked") => {
                let opt_field = |key: &str| {
                    payload
                        .get(key)
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(String::from)
                };
                AdminAlert::ShipmentBooked {
                    courier: str_field("courier"),
                    awb: str_field("awb"),
                    shiprocket_order_id: opt_field("shiprocket_order_id"),
                    pickup: opt_field("pickup"),
                }
            }
            _ => AdminAlert::Placed,
        }
    }
}

/// Comma-separated `ADMIN_NOTIFY_EMAILS`, trimmed; empty when unset (channel disabled).
pub fn admin_notify_emails() -> Vec<String> {
    std::env::var("ADMIN_NOTIFY_EMAILS")
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

/// `NTFY_TOPIC`, or `None` when unset (channel disabled).
pub fn ntfy_topic() -> Option<String> {
    std::env::var("NTFY_TOPIC")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn payment_label(s: &OrderMailSnapshot) -> &'static str {
    if s.payment_method.eq_ignore_ascii_case("cod") {
        "COD"
    } else {
        "Online"
    }
}

fn admin_url(path: &str) -> String {
    format!(
        "{}/imtheboss/{}",
        storefront_url().trim_end_matches('/'),
        path
    )
}

pub fn admin_order_url(order_id: i64) -> String {
    admin_url(&format!("orders/{order_id}"))
}

/// `SHIPROCKET_PANEL_ORDER_URL` (with `{id}`) or the default seller-panel order page.
fn shiprocket_panel_url(shiprocket_order_id: &str) -> String {
    std::env::var("SHIPROCKET_PANEL_ORDER_URL")
        .ok()
        .filter(|v| v.contains("{id}"))
        .unwrap_or_else(|| DEFAULT_SHIPROCKET_PANEL_ORDER_URL.to_string())
        .replace("{id}", shiprocket_order_id)
}

fn shiprocket_tracking_url(awb: &str) -> String {
    format!("{SHIPROCKET_TRACKING_URL}{awb}")
}

fn item_lines(s: &OrderMailSnapshot) -> Vec<String> {
    s.lines
        .iter()
        .map(|l| format!("{} x{}", l.title, l.quantity))
        .collect()
}

/// Everything both channels need, built once per alert.
struct Rendered {
    subject: String,
    headline: String,
    facts: Vec<(&'static str, String)>,
    list_heading: &'static str,
    list: Vec<String>,
    ship: Option<String>,
    /// (label, url); the first is the push tap target, all become email links / ntfy buttons.
    links: Vec<(&'static str, String)>,
    /// ntfy header values must be ASCII, so push titles avoid ₹ and customer-entered text.
    push_title: String,
    push_body: String,
    push_tags: &'static str,
    push_priority: &'static str,
}

fn render(s: &OrderMailSnapshot, alert: &AdminAlert) -> Rendered {
    let total = format_inr_paise(s.grand_total_paise);
    let payment = payment_label(s);
    let customer = format!("{} <{}>", s.customer_name, s.customer_email);

    let admin_link = ("Admin", admin_order_url(s.order_id));

    match alert {
        AdminAlert::Placed => {
            let booking_note = format!(
                "Shiprocket booking in ~{}h (after the cancellation window), unless the customer cancels",
                crate::order_policy::cancel_window_hours()
            );
            Rendered {
                subject: format!("New order #{} — {} ({})", s.order_id, total, payment),
                headline: format!("New order #{} ({})", s.order_id, s.public_order_ref),
                facts: vec![
                    ("Total", format!("{total} ({payment})")),
                    ("Customer", customer),
                    ("Shipment", booking_note.clone()),
                ],
                list_heading: "Items",
                list: item_lines(s),
                ship: Some(s.shipping_block.clone()),
                links: vec![admin_link],
                push_title: format!(
                    "New order #{} - {} ({})",
                    s.order_id,
                    total.replace('₹', "Rs."),
                    payment
                ),
                push_body: format!(
                    "{}\n{}\n{}",
                    s.customer_name,
                    item_lines(s).join("\n"),
                    booking_note
                ),
                push_tags: "package",
                push_priority: "high",
            }
        }
        AdminAlert::Cancelled { full, items } => {
            let what = if *full {
                "cancelled"
            } else {
                "partially cancelled"
            };
            Rendered {
                subject: format!("Order #{} {} by customer ({})", s.order_id, what, payment),
                headline: format!(
                    "Customer {} order #{} ({})",
                    what, s.order_id, s.public_order_ref
                ),
                facts: vec![
                    ("Order total", format!("{total} ({payment})")),
                    ("Customer", customer),
                ],
                list_heading: "Cancelled items",
                list: items.clone(),
                ship: None,
                links: vec![admin_link],
                push_title: format!("Order #{} {} ({})", s.order_id, what, payment),
                push_body: format!("{}\n{}", s.customer_name, items.join("\n")),
                push_tags: "x",
                push_priority: "high",
            }
        }
        AdminAlert::ExchangeRequested {
            item,
            from,
            to,
            reason,
        } => {
            let swap = format!("{item}: {from} -> {to}");
            Rendered {
                subject: format!("Exchange requested — order #{}", s.order_id),
                headline: format!(
                    "Exchange requested on order #{} ({})",
                    s.order_id, s.public_order_ref
                ),
                facts: vec![("Customer", customer), ("Reason", reason.clone())],
                list_heading: "Exchange",
                list: vec![swap.clone()],
                ship: Some(s.shipping_block.clone()),
                links: vec![("Exchanges", admin_url("exchanges")), admin_link],
                push_title: format!("Exchange requested - order #{}", s.order_id),
                push_body: format!("{}\n{}\nReason: {}", s.customer_name, swap, reason),
                push_tags: "arrows_counterclockwise",
                push_priority: "default",
            }
        }
        AdminAlert::ShipmentBooked {
            courier,
            awb,
            shiprocket_order_id,
            pickup,
        } => {
            let courier = if courier.trim().is_empty() {
                "Courier pending"
            } else {
                courier.as_str()
            };
            let pickup_line = pickup
                .clone()
                .unwrap_or_else(|| "not scheduled — schedule it in Shiprocket".to_string());
            let mut links = Vec::new();
            if let Some(id) = shiprocket_order_id {
                links.push(("Shiprocket", shiprocket_panel_url(id)));
            }
            if !awb.trim().is_empty() {
                links.push(("Tracking", shiprocket_tracking_url(awb.trim())));
            }
            links.push(admin_link);
            Rendered {
                subject: format!(
                    "Shipment booked — order #{} ({}) — pack for pickup",
                    s.order_id, payment
                ),
                headline: format!(
                    "Shipment booked for order #{} ({})",
                    s.order_id, s.public_order_ref
                ),
                facts: vec![
                    ("Courier", courier.to_string()),
                    ("AWB", awb.clone()),
                    ("Pickup", pickup_line.clone()),
                    ("Payment", format!("{total} ({payment})")),
                ],
                list_heading: "Pack these items",
                list: item_lines(s),
                ship: Some(s.shipping_block.clone()),
                links,
                push_title: format!("Shipment booked - order #{} ({})", s.order_id, payment),
                push_body: format!(
                    "{}\n{}\n{} | AWB {}\nPickup: {}",
                    s.customer_name,
                    item_lines(s).join("\n"),
                    courier,
                    awb,
                    pickup_line
                ),
                push_tags: "truck",
                push_priority: "high",
            }
        }
    }
}

/// (subject, text, html) for the admin email.
pub fn build_admin_email(s: &OrderMailSnapshot, alert: &AdminAlert) -> (String, String, String) {
    let r = render(s, alert);

    let mut text = format!("{}\n", r.headline);
    for (label, value) in &r.facts {
        text.push_str(&format!("{label}: {value}\n"));
    }
    text.push_str(&format!("\n{}:\n", r.list_heading));
    for item in &r.list {
        text.push_str(&format!("- {item}\n"));
    }
    if let Some(ship) = &r.ship {
        text.push_str(&format!("\nShip to:\n{ship}\n"));
    }
    text.push('\n');
    for (label, url) in &r.links {
        text.push_str(&format!("{label}: {url}\n"));
    }

    let facts_html = r
        .facts
        .iter()
        .map(|(label, value)| format!("{label}: {}", html_escape(value)))
        .collect::<Vec<_>>()
        .join("<br>");
    let list_html = r
        .list
        .iter()
        .map(|i| format!("<li>{}</li>", html_escape(i)))
        .collect::<String>();
    let ship_html = r
        .ship
        .as_deref()
        .map(|ship| {
            format!(
                "<p>Ship to:<br>{}</p>",
                html_escape(ship).replace('\n', "<br>")
            )
        })
        .unwrap_or_default();
    let links_html = r
        .links
        .iter()
        .map(|(label, url)| format!("<a href=\"{}\">Open in {label}</a>", html_escape(url)))
        .collect::<Vec<_>>()
        .join(" &nbsp;|&nbsp; ");
    let html = format!(
        "<p><strong>{headline}</strong></p>\
         <p>{facts_html}</p>\
         <p>{list_heading}:</p><ul>{list_html}</ul>\
         {ship_html}\
         <p>{links_html}</p>",
        headline = html_escape(&r.headline),
        list_heading = r.list_heading,
    );

    (r.subject, text, html)
}

/// ntfy "view" action buttons (max 3), only worth sending when there's more than the tap target.
fn ntfy_actions(links: &[(&'static str, String)]) -> Option<String> {
    if links.len() < 2 {
        return None;
    }
    Some(
        links
            .iter()
            .take(3)
            .map(|(label, url)| format!("view, {label}, {url}"))
            .collect::<Vec<_>>()
            .join("; "),
    )
}

/// POST the alert to `{NTFY_URL}/{topic}`. Non-2xx is an error so the outbox row retries.
pub async fn send_ntfy(
    topic: &str,
    s: &OrderMailSnapshot,
    alert: &AdminAlert,
) -> Result<(), Status> {
    let r = render(s, alert);
    let base = std::env::var("NTFY_URL")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_NTFY_URL.to_string());
    let endpoint = format!("{}/{}", base.trim().trim_end_matches('/'), topic);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| Status::internal(e.to_string()))?;

    let mut req = client
        .post(&endpoint)
        .header("Title", r.push_title)
        .header("Priority", r.push_priority)
        .header("Tags", r.push_tags)
        .body(r.push_body);
    if let Some((_, url)) = r.links.first() {
        req = req.header("Click", url.as_str());
    }
    if let Some(actions) = ntfy_actions(&r.links) {
        req = req.header("Actions", actions);
    }
    if let Ok(token) = std::env::var("NTFY_TOKEN") {
        if !token.trim().is_empty() {
            req = req.header("Authorization", format!("Bearer {}", token.trim()));
        }
    }

    let res = req
        .send()
        .await
        .map_err(|e| Status::internal(format!("ntfy request failed: {e}")))?;
    if !res.status().is_success() {
        let status = res.status();
        let err_text = res.text().await.unwrap_or_default();
        return Err(Status::internal(format!(
            "ntfy error HTTP {}: {}",
            status, err_text
        )));
    }
    info!(order_id = s.order_id, "admin alert pushed via ntfy");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notifications::order_mail::OrderLineMail;

    fn snapshot(payment_method: &str) -> OrderMailSnapshot {
        OrderMailSnapshot {
            order_id: 42,
            public_order_ref: "SUD-42".to_string(),
            order_number: None,
            order_date_rfc3339: "2026-10-29T10:00:00Z".to_string(),
            customer_email: "buyer@example.com".to_string(),
            customer_name: "Asha <Test>".to_string(),
            currency: "INR".to_string(),
            grand_total_paise: 249_900,
            status_name: "confirmed".to_string(),
            payment_method: payment_method.to_string(),
            lines: vec![OrderLineMail {
                title: "Silk Saree".to_string(),
                quantity: 2,
                line_total_paise: 249_900,
            }],
            shipping_block: "12 Lake Rd\nKolkata".to_string(),
        }
    }

    fn cancelled(full: bool) -> AdminAlert {
        AdminAlert::Cancelled {
            full,
            items: vec!["Silk Saree x1".to_string()],
        }
    }

    fn exchange() -> AdminAlert {
        AdminAlert::ExchangeRequested {
            item: "Silk Saree x1".to_string(),
            from: "M".to_string(),
            to: "L".to_string(),
            reason: "Too tight".to_string(),
        }
    }

    fn shipment(shiprocket_order_id: Option<&str>, pickup: Option<&str>) -> AdminAlert {
        AdminAlert::ShipmentBooked {
            courier: "Delhivery".to_string(),
            awb: "AWB123".to_string(),
            shiprocket_order_id: shiprocket_order_id.map(String::from),
            pickup: pickup.map(String::from),
        }
    }

    #[test]
    fn placed_alert_mentions_cancel_window_booking_delay() {
        let hours = crate::order_policy::cancel_window_hours();
        let expected = format!("Shiprocket booking in ~{hours}h");
        let (_, text, _) = build_admin_email(&snapshot("cod"), &AdminAlert::Placed);
        assert!(text.contains(&expected), "{text}");
        let r = render(&snapshot("cod"), &AdminAlert::Placed);
        assert!(r.push_body.contains(&expected));
        assert!(
            ntfy_actions(&r.links).is_none(),
            "single link needs no buttons"
        );
    }

    #[test]
    fn shipment_booked_links_to_shiprocket_first_then_tracking_and_admin() {
        let r = render(
            &snapshot("cod"),
            &shipment(Some("98765"), Some("03 Nov 2026, 10:00 AM IST")),
        );
        assert_eq!(r.push_title, "Shipment booked - order #42 (COD)");
        assert!(r.push_title.is_ascii());
        assert!(r.push_body.contains("Delhivery | AWB AWB123"));
        assert!(r.push_body.contains("Pickup: 03 Nov 2026, 10:00 AM IST"));
        let labels: Vec<_> = r.links.iter().map(|(l, _)| *l).collect();
        assert_eq!(labels, ["Shiprocket", "Tracking", "Admin"]);
        assert!(r.links[0].1.ends_with("/98765"));
        assert_eq!(r.links[1].1, "https://shiprocket.co/tracking/AWB123");
        let actions = ntfy_actions(&r.links).expect("buttons");
        assert!(actions.starts_with("view, Shiprocket, https://"));
        assert_eq!(actions.matches("view,").count(), 3);
    }

    #[test]
    fn shipment_booked_without_ids_falls_back_to_tracking_and_flags_pickup() {
        let r = render(&snapshot("prepaid"), &shipment(None, None));
        assert_eq!(r.links[0].0, "Tracking");
        let (subject, text, _) = build_admin_email(&snapshot("prepaid"), &shipment(None, None));
        assert!(subject.starts_with("Shipment booked — order #42 (Online)"));
        assert!(text.contains("Pickup: not scheduled"));
        assert!(text.contains("Pack these items:\n- Silk Saree x2"));
    }

    #[test]
    fn placed_subject_shows_total_and_payment_method() {
        let (subject, _, _) = build_admin_email(&snapshot("cod"), &AdminAlert::Placed);
        assert_eq!(subject, "New order #42 — ₹2499.00 (COD)");
        let (subject, _, _) = build_admin_email(&snapshot("prepaid"), &AdminAlert::Placed);
        assert_eq!(subject, "New order #42 — ₹2499.00 (Online)");
    }

    #[test]
    fn placed_bodies_list_items_and_escape_html() {
        let (_, text, html) = build_admin_email(&snapshot("cod"), &AdminAlert::Placed);
        assert!(text.contains("- Silk Saree x2"));
        assert!(text.contains("/imtheboss/orders/42"));
        assert!(html.contains("Asha &lt;Test&gt;"));
        assert!(html.contains("12 Lake Rd<br>Kolkata"));
    }

    #[test]
    fn cancelled_alert_names_cancelled_items_and_scope() {
        let (subject, text, _) = build_admin_email(&snapshot("cod"), &cancelled(false));
        assert_eq!(subject, "Order #42 partially cancelled by customer (COD)");
        assert!(text.contains("Cancelled items:\n- Silk Saree x1"));
        let r = render(&snapshot("prepaid"), &cancelled(true));
        assert_eq!(r.push_title, "Order #42 cancelled (Online)");
        assert_eq!(r.push_tags, "x");
    }

    #[test]
    fn exchange_alert_shows_swap_and_links_to_exchanges() {
        let (subject, text, _) = build_admin_email(&snapshot("prepaid"), &exchange());
        assert_eq!(subject, "Exchange requested — order #42");
        assert!(text.contains("- Silk Saree x1: M -> L"));
        assert!(text.contains("Reason: Too tight"));
        assert!(text.contains("/imtheboss/exchanges"));
    }

    #[test]
    fn push_titles_are_ascii() {
        for alert in [
            AdminAlert::Placed,
            cancelled(true),
            exchange(),
            shipment(None, None),
        ] {
            let title = render(&snapshot("cod"), &alert).push_title;
            assert!(title.is_ascii(), "{title}");
        }
        assert_eq!(
            render(&snapshot("cod"), &AdminAlert::Placed).push_title,
            "New order #42 - Rs.2499.00 (COD)"
        );
    }

    #[test]
    fn payload_round_trips_and_legacy_rows_are_placed() {
        for alert in [
            AdminAlert::Placed,
            cancelled(false),
            exchange(),
            shipment(Some("98765"), Some("03 Nov")),
            shipment(None, None),
        ] {
            let payload = alert.to_payload(42, 7);
            assert_eq!(payload["order_id"], 42);
            assert_eq!(payload["user_id"], 7);
            assert_eq!(AdminAlert::from_payload(&payload), alert);
        }
        let legacy = json!({ "order_id": 42, "user_id": 7 });
        assert_eq!(AdminAlert::from_payload(&legacy), AdminAlert::Placed);
    }
}
