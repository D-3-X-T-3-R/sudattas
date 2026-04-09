//! Shiprocket REST API: auth, create adhoc order, assign AWB (used when admin marks shipped with `shiprocket_book`).

use core_db_entities::entity::{order_details, orders, shipping_addresses, users};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use thiserror::Error;
use tracing::warn;

use super::shiprocket_status::shiprocket_status_label_for_id;

const DEFAULT_API_BASE: &str = "https://apiv2.shiprocket.in/v1/external";
const TOKEN_TTL: Duration = Duration::from_secs(200 * 3600);

#[derive(Debug, Error)]
pub enum ShiprocketError {
    #[error("Shiprocket is not configured (set SHIPROCKET_EMAIL and SHIPROCKET_PASSWORD)")]
    NotConfigured,
    #[error("order {0} not found")]
    OrderNotFound(i64),
    #[error("shipping address for order {0} not found")]
    AddressNotFound(i64),
    #[error("user {0} not found for order")]
    UserNotFound(i64),
    #[error("order {0} has no line items")]
    NoLineItems(i64),
    #[error("customer phone is required for Shiprocket; no valid phone on shipping address or user {0}")]
    MissingPhone(i64),
    #[error("Shiprocket auth failed: {0}")]
    AuthFailed(String),
    #[error("Shiprocket create order failed: {0}")]
    CreateOrderFailed(String),
    #[error("Shiprocket assign AWB failed: {0}")]
    AssignAwbFailed(String),
    #[error("Shiprocket tracking fetch failed: {0}")]
    TrackFailed(String),
    #[error("Shiprocket quote fetch failed: {0}")]
    QuoteFailed(String),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("database error: {0}")]
    Db(#[from] sea_orm::DbErr),
}

#[derive(Clone)]
struct Config {
    email: String,
    password: String,
    api_base: String,
    pickup_location: String,
    pickup_postcode: Option<String>,
    default_weight_kg: f64,
    estimated_unit_weight_kg: f64,
    length_cm: f64,
    breadth_cm: f64,
    height_cm: f64,
    courier_id: Option<i64>,
}

impl Config {
    fn from_env() -> Option<Self> {
        Self::from_env_prefix("SHIPROCKET", "Primary")
    }

    fn from_env_prefix(prefix: &str, default_pickup_location: &str) -> Option<Self> {
        let email = std::env::var(format!("{prefix}_EMAIL")).ok()?.trim().to_string();
        let password = std::env::var(format!("{prefix}_PASSWORD"))
            .ok()?
            .trim()
            .to_string();
        if email.is_empty() || password.is_empty() {
            return None;
        }
        let api_base = std::env::var(format!("{prefix}_API_BASE"))
            .unwrap_or_else(|_| DEFAULT_API_BASE.to_string())
            .trim()
            .trim_end_matches('/')
            .to_string();
        let pickup_location = std::env::var(format!("{prefix}_PICKUP_LOCATION"))
            .unwrap_or_else(|_| default_pickup_location.to_string())
            .trim()
            .to_string();
        let pickup_postcode = std::env::var(format!("{prefix}_PICKUP_POSTCODE"))
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let default_weight_kg = std::env::var(format!("{prefix}_DEFAULT_WEIGHT_KG"))
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.5);
        let estimated_unit_weight_kg = std::env::var(format!("{prefix}_ESTIMATED_UNIT_WEIGHT_KG"))
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(default_weight_kg);
        let length_cm = std::env::var(format!("{prefix}_PACKAGE_LENGTH_CM"))
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10.0);
        let breadth_cm = std::env::var(format!("{prefix}_PACKAGE_BREADTH_CM"))
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10.0);
        let height_cm = std::env::var(format!("{prefix}_PACKAGE_HEIGHT_CM"))
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10.0);
        let courier_id = std::env::var(format!("{prefix}_COURIER_ID"))
            .ok()
            .and_then(|s| s.parse().ok());
        Some(Self {
            email,
            password,
            api_base,
            pickup_location,
            pickup_postcode,
            default_weight_kg,
            estimated_unit_weight_kg,
            length_cm,
            breadth_cm,
            height_cm,
            courier_id,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ShiprocketBooking {
    pub awb_code: String,
    pub courier_name: String,
    /// Shiprocket shipment id (string) — stored in `Shipments.shiprocket_order_id`.
    pub shiprocket_shipment_id: String,
    pub shiprocket_status_id: Option<i32>,
    pub shiprocket_status_label: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ShiprocketCourierQuote {
    pub courier_id: i64,
    pub courier_name: String,
    /// Shipping charge in paise (minor units).
    pub shipping_amount_minor: i64,
    pub estimated_delivery_days: Option<i32>,
}

#[derive(Deserialize)]
struct LoginResponse {
    token: String,
}

static TOKEN_CACHE: Mutex<Option<(String, Instant)>> = Mutex::new(None);

fn cached_token() -> Option<String> {
    let guard = TOKEN_CACHE.lock().ok()?;
    let (tok, at) = guard.as_ref()?;
    if at.elapsed() < TOKEN_TTL {
        Some(tok.clone())
    } else {
        None
    }
}

fn set_cached_token(token: String) {
    if let Ok(mut g) = TOKEN_CACHE.lock() {
        *g = Some((token, Instant::now()));
    }
}

async fn login(client: &reqwest::Client, cfg: &Config) -> Result<String, ShiprocketError> {
    let url = format!("{}/auth/login", cfg.api_base);
    let res = client
        .post(&url)
        .json(&json!({ "email": cfg.email, "password": cfg.password }))
        .send()
        .await?;
    let status = res.status();
    let body = res.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(ShiprocketError::AuthFailed(format!("HTTP {status}: {body}")));
    }
    let parsed: LoginResponse = serde_json::from_str(&body)
        .map_err(|_| ShiprocketError::AuthFailed(format!("invalid login JSON: {body}")))?;
    Ok(parsed.token)
}

async fn bearer_token(client: &reqwest::Client, cfg: &Config) -> Result<String, ShiprocketError> {
    if let Some(t) = cached_token() {
        return Ok(t);
    }
    let t = login(client, cfg).await?;
    set_cached_token(t.clone());
    Ok(t)
}

fn first_shipment_id(v: &Value) -> Option<i64> {
    v.get("payload")
        .and_then(|p| p.get("shipment_id"))
        .and_then(|x| x.as_i64())
        .or_else(|| v.get("shipment_id").and_then(|x| x.as_i64()))
        .or_else(|| {
            v.get("payload")
                .and_then(|p| p.get("shipment_id"))
                .and_then(|x| x.as_str())
                .and_then(|s| s.parse().ok())
        })
}

fn extract_assign_shipment_status(v: &Value) -> (Option<i32>, Option<String>) {
    let id = v
        .pointer("/response/data/shipment_status_id")
        .and_then(|x| x.as_i64())
        .or_else(|| v.pointer("/payload/shipment_status_id").and_then(|x| x.as_i64()))
        .or_else(|| v.get("shipment_status_id").and_then(|x| x.as_i64()))
        .or_else(|| {
            v.get("data")
                .and_then(|d| d.get("shipment_status_id"))
                .and_then(|x| x.as_i64())
        })
        .map(|x| x as i32);
    let label = v
        .pointer("/response/data/shipment_status")
        .and_then(|x| x.as_str())
        .or_else(|| v.pointer("/payload/shipment_status").and_then(|x| x.as_str()))
        .or_else(|| v.get("shipment_status").and_then(|x| x.as_str()))
        .map(std::string::ToString::to_string);
    (id, label)
}

fn first_awb_and_courier(v: &Value) -> Option<(String, String)> {
    let awb = v
        .get("awb_code")
        .and_then(|x| x.as_str())
        .map(str::to_string)
        .or_else(|| {
            v.get("response")
                .and_then(|r| r.get("data"))
                .and_then(|d| d.get("awb_code"))
                .and_then(|x| x.as_str())
                .map(str::to_string)
        })
        .or_else(|| {
            v.get("payload")
                .and_then(|p| p.get("awb_code"))
                .and_then(|x| x.as_str())
                .map(str::to_string)
        });
    let courier = v
        .get("courier_name")
        .and_then(|x| x.as_str())
        .map(str::to_string)
        .or_else(|| {
            v.get("response")
                .and_then(|r| r.get("data"))
                .and_then(|d| d.get("courier_name"))
                .and_then(|x| x.as_str())
                .map(str::to_string)
        })
        .or_else(|| {
            v.get("payload")
                .and_then(|p| p.get("courier_name"))
                .and_then(|x| x.as_str())
                .map(str::to_string)
        });
    match (awb, courier) {
        (Some(a), Some(c)) => Some((a, c)),
        (Some(a), None) => Some((a, "Courier".to_string())),
        _ => None,
    }
}

fn parse_i64_like(v: Option<&Value>) -> Option<i64> {
    let v = v?;
    v.as_i64()
        .or_else(|| v.as_u64().and_then(|x| i64::try_from(x).ok()))
        .or_else(|| v.as_f64().map(|x| x.round() as i64))
        .or_else(|| v.as_str().and_then(|s| s.trim().parse::<i64>().ok()))
}

fn parse_f64_like(v: Option<&Value>) -> Option<f64> {
    let v = v?;
    v.as_f64()
        .or_else(|| v.as_i64().map(|x| x as f64))
        .or_else(|| v.as_str().and_then(|s| s.trim().parse::<f64>().ok()))
}

fn rupees_to_minor(rupees: f64) -> i64 {
    (rupees * 100.0).round() as i64
}

fn parse_etd_days(v: Option<&Value>) -> Option<i32> {
    let raw = v?.as_str()?.trim();
    if raw.is_empty() {
        return None;
    }
    let mut num = String::new();
    for c in raw.chars() {
        if c.is_ascii_digit() {
            num.push(c);
        } else if !num.is_empty() {
            break;
        }
    }
    num.parse::<i32>().ok()
}

fn extract_courier_rows(v: &Value) -> Vec<Value> {
    let candidates = [
        "/data/available_courier_companies",
        "/available_courier_companies",
        "/response/data/available_courier_companies",
        "/data/courier_data/available_courier_companies",
    ];
    for p in candidates {
        if let Some(arr) = v.pointer(p).and_then(|x| x.as_array()) {
            return arr.clone();
        }
    }
    Vec::new()
}

fn parse_quote_row(row: &Value) -> Option<ShiprocketCourierQuote> {
    let courier_id = parse_i64_like(
        row.get("courier_company_id")
            .or_else(|| row.get("courier_id"))
            .or_else(|| row.get("id")),
    )?;
    let courier_name = row
        .get("courier_name")
        .or_else(|| row.get("courier_company_name"))
        .or_else(|| row.get("name"))
        .and_then(|x| x.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("Courier")
        .to_string();

    let freight_rupees = parse_f64_like(
        row.get("freight_charge")
            .or_else(|| row.get("rate"))
            .or_else(|| row.get("courier_charge"))
            .or_else(|| row.get("total_charge")),
    )?;
    let cod_rupees = parse_f64_like(row.get("cod_charges")).unwrap_or(0.0);
    let shipping_amount_minor = rupees_to_minor(freight_rupees + cod_rupees);
    if shipping_amount_minor < 0 {
        return None;
    }

    let estimated_delivery_days = parse_i64_like(row.get("estimated_delivery_days"))
        .and_then(|d| i32::try_from(d).ok())
        .or_else(|| parse_etd_days(row.get("etd")));

    Some(ShiprocketCourierQuote {
        courier_id,
        courier_name,
        shipping_amount_minor,
        estimated_delivery_days,
    })
}

fn estimate_checkout_weight_kg(cfg: &Config, total_units: i64) -> f64 {
    let units = total_units.max(1) as f64;
    let unit_based = cfg.estimated_unit_weight_kg.max(0.05) * units;
    unit_based.max(cfg.default_weight_kg.max(0.05))
}

pub async fn best_courier_quote_for_checkout(
    delivery_postcode: &str,
    order_value_minor: i64,
    total_units: i64,
) -> Result<Option<ShiprocketCourierQuote>, ShiprocketError> {
    let cfg = Config::from_env().ok_or(ShiprocketError::NotConfigured)?;
    let pickup_postcode = cfg.pickup_postcode.as_deref().ok_or_else(|| {
        ShiprocketError::QuoteFailed("set SHIPROCKET_PICKUP_POSTCODE".to_string())
    })?;
    let pickup: String = pickup_postcode
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect();
    let delivery: String = delivery_postcode
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect();
    if pickup.len() < 6 || delivery.len() < 6 {
        return Err(ShiprocketError::QuoteFailed(
            "pickup/delivery postcode must have at least 6 digits".to_string(),
        ));
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;
    let token = bearer_token(&client, &cfg).await?;
    let weight_kg = estimate_checkout_weight_kg(&cfg, total_units);
    let declared_value_rupees = (order_value_minor.max(0) as f64) / 100.0;
    let url = format!("{}/courier/serviceability/", cfg.api_base);
    let params: Vec<(&str, String)> = vec![
        ("pickup_postcode", pickup),
        ("delivery_postcode", delivery),
        ("weight", format!("{weight_kg:.3}")),
        ("cod", "0".to_string()),
        ("declared_value", format!("{declared_value_rupees:.2}")),
    ];
    let res = client
        .get(&url)
        .header("Authorization", format!("Bearer {token}"))
        .query(&params)
        .send()
        .await?;
    let status = res.status();
    let body = res.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(ShiprocketError::QuoteFailed(format!(
            "HTTP {status}: {body}"
        )));
    }
    let v: Value =
        serde_json::from_str(&body).map_err(|_| ShiprocketError::QuoteFailed(body.clone()))?;
    let mut quotes: Vec<ShiprocketCourierQuote> = extract_courier_rows(&v)
        .iter()
        .filter_map(parse_quote_row)
        .collect();
    if quotes.is_empty() {
        return Ok(None);
    }
    quotes.sort_by_key(|q| {
        (
            q.shipping_amount_minor,
            q.estimated_delivery_days.unwrap_or(999),
            q.courier_id,
        )
    });
    Ok(quotes.into_iter().next())
}

/// Load order + address + user + lines, create Shiprocket order, assign AWB, return tracking fields.
pub async fn book_shipment_for_order(
    db: &DatabaseConnection,
    order_id: i64,
) -> Result<ShiprocketBooking, ShiprocketError> {
    let order = orders::Entity::find_by_id(order_id)
        .one(db)
        .await?
        .ok_or(ShiprocketError::OrderNotFound(order_id))?;

    let address = shipping_addresses::Entity::find_by_id(order.shipping_address_id)
        .one(db)
        .await?
        .ok_or(ShiprocketError::AddressNotFound(order_id))?;
    let cfg = Config::from_env().ok_or(ShiprocketError::NotConfigured)?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(90))
        .build()?;

    let user = users::Entity::find_by_id(order.user_id)
        .one(db)
        .await?
        .ok_or(ShiprocketError::UserNotFound(order.user_id))?;

    let address_phone = address.phone_number.as_deref().unwrap_or("").trim();
    let user_phone = user.phone.as_deref().unwrap_or("").trim();
    let source_phone = if !address_phone.is_empty() {
        address_phone
    } else {
        user_phone
    };
    let phone_digits: String = source_phone.chars().filter(|c| c.is_ascii_digit()).collect();
    if phone_digits.len() < 10 {
        return Err(ShiprocketError::MissingPhone(order.user_id));
    }
    // Shiprocket create-order API expects a 10-digit mobile number for billing_phone.
    let phone = if phone_digits.len() > 10 {
        phone_digits[phone_digits.len() - 10..].to_string()
    } else {
        phone_digits
    };

    let lines = order_details::Entity::find()
        .filter(order_details::Column::OrderId.eq(order_id))
        .all(db)
        .await?;
    if lines.is_empty() {
        return Err(ShiprocketError::NoLineItems(order_id));
    }

    let name_full = address
        .recipient_name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .or_else(|| {
            user.full_name
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
        })
        .unwrap_or_else(|| user.username.as_str());
    let (billing_first, billing_last) = split_name(name_full);
    let billing_last = if billing_last.trim().is_empty() {
        "NA".to_string()
    } else {
        billing_last
    };

    let addr_line1 = [
        address.apartment_no_or_name.as_deref(),
        address.road.as_deref(),
    ]
    .into_iter()
    .flatten()
    .map(str::trim)
    .filter(|s| !s.is_empty())
    .collect::<Vec<_>>()
    .join(", ");
    let addr_line1 = if addr_line1.is_empty() {
        address.city.clone()
    } else {
        addr_line1
    };

    let order_ref = order
        .order_number
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| format!("SUD-{}", order.order_id));

    let order_date = order.order_date.format("%Y-%m-%d %H:%M").to_string();

    let order_items: Vec<Value> = lines
        .iter()
        .map(|l| {
            let name = l.title.as_deref().unwrap_or("Item").to_string();
            let sku = l
                .sku
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .unwrap_or_else(|| format!("OD-{}", l.order_detail_id));
            let units = l.quantity.max(1);
            let rupees = (l.unit_price_minor as f64) / 100.0;
            json!({
                "name": name,
                "sku": sku,
                "units": units,
                "selling_price": format!("{:.2}", rupees),
            })
        })
        .collect();

    let sub_total_rupees = (order.grand_total_minor as f64) / 100.0;
    let total_units: i64 = lines.iter().map(|l| l.quantity.max(1)).sum();
    let payment_method = "Prepaid";

    let body = json!({
        "order_id": order_ref,
        "order_date": order_date,
        "pickup_location": cfg.pickup_location,
        "billing_customer_name": billing_first,
        "billing_last_name": billing_last,
        "billing_address": addr_line1,
        "billing_address_2": "",
        "billing_city": address.city,
        "billing_pincode": address.postal_code.trim(),
        "billing_state": address.state_region,
        "billing_country": address.country,
        "billing_email": user.email,
        "billing_phone": phone,
        "shipping_is_billing": true,
        "order_items": order_items,
        "payment_method": payment_method,
        "sub_total": format!("{:.2}", sub_total_rupees),
        "length": cfg.length_cm,
        "breadth": cfg.breadth_cm,
        "height": cfg.height_cm,
        "weight": estimate_checkout_weight_kg(&cfg, total_units),
    });

    let token = bearer_token(&client, &cfg).await?;
    let create_url = format!("{}/orders/create/adhoc", cfg.api_base);
    let create_res = client
        .post(&create_url)
        .header("Authorization", format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;
    let create_status = create_res.status();
    let create_text = create_res.text().await.unwrap_or_default();
    if !create_status.is_success() {
        return Err(ShiprocketError::CreateOrderFailed(format!(
            "HTTP {create_status}: {create_text}"
        )));
    }
    let create_json: Value = serde_json::from_str(&create_text).map_err(|_| {
        ShiprocketError::CreateOrderFailed(format!("invalid JSON: {create_text}"))
    })?;

    let shipment_id = first_shipment_id(&create_json).ok_or_else(|| {
        ShiprocketError::CreateOrderFailed(format!(
            "missing shipment_id in response: {create_text}"
        ))
    })?;

    let mut assign_map = serde_json::Map::new();
    assign_map.insert("shipment_id".to_string(), json!(shipment_id));
    let selected_courier_id = if let Some(cid) = cfg.courier_id {
        Some(cid)
    } else {
        let delivery_pin = address.postal_code.trim().to_string();
        let order_value_minor = order.grand_total_minor;
        match best_courier_quote_for_checkout(
            &delivery_pin,
            order_value_minor,
            total_units,
        )
        .await
        {
            Ok(Some(q)) => Some(q.courier_id),
            Ok(None) => None,
            Err(e) => {
                warn!("shiprocket quote unavailable during AWB assign fallback: {}", e);
                None
            }
        }
    };
    if let Some(cid) = selected_courier_id {
        assign_map.insert("courier_id".to_string(), json!(cid));
    }
    let assign_body = Value::Object(assign_map);

    let assign_url = format!("{}/courier/assign/awb", cfg.api_base);
    let assign_res = client
        .post(&assign_url)
        .header("Authorization", format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .json(&assign_body)
        .send()
        .await?;
    let assign_status = assign_res.status();
    let assign_text = assign_res.text().await.unwrap_or_default();
    if !assign_status.is_success() {
        return Err(ShiprocketError::AssignAwbFailed(format!(
            "HTTP {assign_status}: {assign_text}"
        )));
    }
    let assign_json: Value = serde_json::from_str(&assign_text).map_err(|_| {
        ShiprocketError::AssignAwbFailed(format!("invalid JSON: {assign_text}"))
    })?;

    let (awb_code, courier_name) = first_awb_and_courier(&assign_json).ok_or_else(|| {
        ShiprocketError::AssignAwbFailed(format!(
            "missing awb in response: {assign_text}"
        ))
    })?;

    let (mut sr_id, mut sr_label) = extract_assign_shipment_status(&assign_json);
    if sr_id.is_none() {
        sr_id = Some(3);
    }
    if sr_label.is_none() {
        if let Some(id) = sr_id {
            sr_label = Some(shiprocket_status_label_for_id(id));
        }
    }

    Ok(ShiprocketBooking {
        awb_code,
        courier_name,
        shiprocket_shipment_id: shipment_id.to_string(),
        shiprocket_status_id: sr_id,
        shiprocket_status_label: sr_label,
    })
}

/// Latest tracking snapshot from Shiprocket `GET /courier/track/awb/{awb}` (used for refresh + webhook fallback).
#[derive(Debug, Clone)]
pub struct ShiprocketTrackingSnapshot {
    pub status_id: Option<i32>,
    pub status_label: Option<String>,
    /// Raw `shipment_track` array when the API returns it (stored as JSON timeline).
    pub scan_events: Option<Value>,
}

fn parse_track_api_response(v: &Value) -> ShiprocketTrackingSnapshot {
    let td = v
        .get("tracking_data")
        .or_else(|| v.pointer("/data/tracking_data"))
        .or_else(|| v.get("data"));
    let mut status_id = td
        .and_then(|t| t.get("shipment_status_id"))
        .and_then(|x| x.as_i64())
        .map(|x| x as i32)
        .or_else(|| {
            v.get("shipment_status_id")
                .and_then(|x| x.as_i64())
                .map(|x| x as i32)
        });
    let mut status_label = td
        .and_then(|t| t.get("shipment_status"))
        .and_then(|x| x.as_str())
        .map(std::string::ToString::to_string)
        .or_else(|| {
            v.get("shipment_status")
                .and_then(|x| x.as_str())
                .map(std::string::ToString::to_string)
        });
    let scan_events = td
        .and_then(|t| t.get("shipment_track"))
        .filter(|x| x.is_array())
        .cloned();
    // Some responses nest status only on the last scan row.
    if status_id.is_none() {
        if let Some(arr) = scan_events.as_ref().and_then(|x| x.as_array()) {
            for row in arr.iter().rev() {
                if let Some(id) = row
                    .get("shipment_status_id")
                    .and_then(|x| x.as_i64())
                    .map(|x| x as i32)
                {
                    status_id = Some(id);
                }
                if status_label.is_none() {
                    status_label = row
                        .get("shipment_status")
                        .or_else(|| row.get("activity"))
                        .and_then(|x| x.as_str())
                        .map(std::string::ToString::to_string);
                }
                if status_id.is_some() {
                    break;
                }
            }
        }
    }
    ShiprocketTrackingSnapshot {
        status_id,
        status_label,
        scan_events,
    }
}

/// Poll Shiprocket for the latest status for an AWB (configure webhooks for push updates in production).
pub async fn track_shipment_by_awb(awb: &str) -> Result<ShiprocketTrackingSnapshot, ShiprocketError> {
    let awb = awb.trim();
    if awb.is_empty() {
        return Err(ShiprocketError::TrackFailed("empty awb".to_string()));
    }
    let cfg = Config::from_env().ok_or(ShiprocketError::NotConfigured)?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()?;
    let token = bearer_token(&client, &cfg).await?;
    let url = format!(
        "{}/courier/track/awb/{}",
        cfg.api_base,
        awb.replace(' ', "")
    );
    let res = client
        .get(&url)
        .header("Authorization", format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .send()
        .await?;
    let status = res.status();
    let body = res.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(ShiprocketError::TrackFailed(format!(
            "HTTP {status}: {body}"
        )));
    }
    let v: Value =
        serde_json::from_str(&body).map_err(|_| ShiprocketError::TrackFailed(body.clone()))?;
    Ok(parse_track_api_response(&v))
}

fn split_name(full: &str) -> (String, String) {
    let parts: Vec<&str> = full.split_whitespace().collect();
    if parts.is_empty() {
        ("Customer".to_string(), String::new())
    } else if parts.len() == 1 {
        (parts[0].to_string(), String::new())
    } else {
        (parts[0].to_string(), parts[1..].join(" "))
    }
}
