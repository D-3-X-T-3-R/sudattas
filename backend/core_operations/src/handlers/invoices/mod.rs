pub mod pdf;

use crate::handlers::db_errors::map_db_error_to_status;
use crate::handlers::outbox::{enqueue_outbox_event, INVOICE_GENERATED};
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use chrono::Utc;
use core_db_entities::entity::sea_orm_active_enums::PaymentStatus;
use core_db_entities::entity::{invoices, order_details, order_status, orders, shipping_addresses, users};
use proto::proto::core::{
    GetOrderInvoiceDownloadRequest, GetOrderInvoiceDownloadResponse, GetOrderInvoiceRequest,
    InvoiceResponse,
};
use sea_orm::sea_query::LockType;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseTransaction, DbErr, EntityTrait,
    QueryFilter, QuerySelect,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tonic::{Request, Response, Status};
use tracing::info;

const INVOICE_CONTENT_TYPE: &str = "application/pdf";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceDocumentLineSnapshot {
    pub title: String,
    pub quantity: i64,
    pub unit_price_minor: i64,
    pub unit_price_formatted: String,
    pub line_total_minor: i64,
    pub line_total_formatted: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceDocumentSnapshot {
    pub invoice_number: String,
    pub order_id: i64,
    pub customer_name: String,
    pub customer_email: String,
    pub shipping_address_snapshot: String,
    pub generated_at_rfc3339: String,
    pub lines: Vec<InvoiceDocumentLineSnapshot>,
    pub item_total_minor: i64,
    pub discount_minor: i64,
    pub shipping_minor: i64,
    pub grand_total_minor: i64,
    pub item_total_formatted: String,
    pub discount_formatted: String,
    pub shipping_formatted: String,
    pub grand_total_formatted: String,
    pub payment_mode: String,
    pub payment_status: String,
}

fn format_minor_amount(minor: i64) -> String {
    let sign = if minor < 0 { "-" } else { "" };
    let abs = minor.unsigned_abs();
    let major = abs / 100;
    let fraction = abs % 100;
    format!("{sign}INR {major}.{fraction:02}")
}

fn normalize_payment_mode(raw: Option<&str>) -> String {
    raw.unwrap_or("prepaid").trim().to_ascii_lowercase()
}

fn normalize_payment_status(raw: Option<PaymentStatus>) -> String {
    raw.map(|status| format!("{status:?}").to_ascii_lowercase())
        .unwrap_or_else(|| "pending".to_string())
}

fn status_allows_cod_invoice(status_name: &str) -> bool {
    matches!(
        status_name,
        "confirmed"
            | "processing"
            | "shipped"
            | "delivered"
            | "partially_cancelled"
            | "cancel_pending_logistics"
            | "cancelled"
            | "refunded"
            | "needs_review"
    )
}

fn invoice_is_eligible(order: &orders::Model, status_name: &str) -> bool {
    let mode = normalize_payment_mode(order.payment_method.as_deref());
    if mode == "cod" {
        return status_allows_cod_invoice(status_name);
    }

    matches!(order.payment_status, Some(PaymentStatus::Captured))
}

fn build_invoice_number(order_id: i64, generated_at: chrono::DateTime<Utc>) -> String {
    format!("INV-{}-{order_id:06}", generated_at.format("%Y%m%d"))
}

fn build_storage_path(invoice_number: &str) -> String {
    format!("db://invoices/{invoice_number}.pdf")
}

fn is_duplicate_key(err: &DbErr) -> bool {
    match err {
        DbErr::Exec(exec) => {
            let message = exec.to_string();
            message.contains("Duplicate entry") || message.contains("1062")
        }
        _ => false,
    }
}

fn shipping_snapshot(address: &shipping_addresses::Model) -> String {
    let mut parts = Vec::new();
    if let Some(name) = address
        .recipient_name
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(name.to_string());
    }
    if let Some(phone) = address
        .phone_number
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(phone.to_string());
    }
    if let Some(apartment) = address
        .apartment_no_or_name
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(apartment.to_string());
    }
    if let Some(road) = address.road.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
        parts.push(road.to_string());
    }
    parts.push(format!(
        "{}, {} {}",
        address.city, address.state_region, address.postal_code
    ));
    parts.push(address.country.clone());
    parts.join("\n")
}

fn to_invoice_response(row: &invoices::Model) -> InvoiceResponse {
    InvoiceResponse {
        invoice_id: row.invoice_id,
        invoice_number: row.invoice_number.clone(),
        order_id: row.order_id,
        user_id: row.user_id,
        generated_at: row.generated_at.to_rfc3339(),
        storage_path: row.storage_path.clone(),
    }
}

async fn load_locked_order(
    txn: &DatabaseTransaction,
    order_id: i64,
) -> Result<orders::Model, Status> {
    orders::Entity::find_by_id(order_id)
        .lock(LockType::Update)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::not_found(format!("Order {} not found", order_id)))
}

async fn find_invoice_by_order_id(
    txn: &DatabaseTransaction,
    order_id: i64,
) -> Result<Option<invoices::Model>, Status> {
    invoices::Entity::find()
        .filter(invoices::Column::OrderId.eq(order_id))
        .one(txn)
        .await
        .map_err(map_db_error_to_status)
}

pub async fn ensure_invoice_for_order(
    txn: &DatabaseTransaction,
    order_id: i64,
    trigger: &str,
) -> Result<Option<invoices::Model>, Status> {
    let mut order = load_locked_order(txn, order_id).await?;

    if let Some(existing_invoice_id) = order.invoice_id {
        let existing = invoices::Entity::find_by_id(existing_invoice_id)
            .one(txn)
            .await
            .map_err(map_db_error_to_status)?;
        return Ok(existing);
    }

    if let Some(existing) = find_invoice_by_order_id(txn, order_id).await? {
        let mut active: orders::ActiveModel = order.clone().into();
        active.invoice_id = ActiveValue::Set(Some(existing.invoice_id));
        active.invoice_number = ActiveValue::Set(Some(existing.invoice_number.clone()));
        active.invoice_generated_at = ActiveValue::Set(Some(existing.generated_at));
        active.invoice_storage_path = ActiveValue::Set(Some(existing.storage_path.clone()));
        let _ = active.update(txn).await.map_err(map_db_error_to_status)?;
        return Ok(Some(existing));
    }

    let status = order_status::Entity::find_by_id(order.status_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::internal("Order status lookup failed"))?;
    let status_name = status.status_name.trim().to_ascii_lowercase();
    if !invoice_is_eligible(&order, &status_name) {
        return Ok(None);
    }

    let user = users::Entity::find_by_id(order.user_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::internal("Order user not found"))?;
    let address = shipping_addresses::Entity::find_by_id(order.shipping_address_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::internal("Order shipping address not found"))?;
    let detail_rows = order_details::Entity::find()
        .filter(order_details::Column::OrderId.eq(order_id))
        .all(txn)
        .await
        .map_err(map_db_error_to_status)?;

    let generated_at = Utc::now();
    let invoice_number = build_invoice_number(order_id, generated_at);
    let item_total_minor = order
        .items_total_minor_before_discount
        .unwrap_or_else(|| detail_rows.iter().map(|line| line.line_total_minor).sum());
    let discount_minor = order.discount_total_minor.unwrap_or(0);
    let shipping_minor = order
        .shipping_charge_minor
        .or(order.shipping_minor)
        .unwrap_or(0);
    let grand_total_minor = order.grand_total_minor;

    let lines = detail_rows
        .into_iter()
        .map(|line| {
            let unit_minor = i64::from(line.unit_price_minor);
            let line_total_minor = line.line_total_minor.max(0);
            let title = line
                .title
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_string)
                .unwrap_or_else(|| format!("Variant {}", line.variant_id));
            InvoiceDocumentLineSnapshot {
                title,
                quantity: line.quantity,
                unit_price_minor: unit_minor,
                unit_price_formatted: format_minor_amount(unit_minor),
                line_total_minor,
                line_total_formatted: format_minor_amount(line_total_minor),
            }
        })
        .collect::<Vec<_>>();

    let snapshot = InvoiceDocumentSnapshot {
        invoice_number: invoice_number.clone(),
        order_id,
        customer_name: user
            .full_name
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| user.username.clone()),
        customer_email: user.email.trim().to_string(),
        shipping_address_snapshot: shipping_snapshot(&address),
        generated_at_rfc3339: generated_at.to_rfc3339(),
        lines,
        item_total_minor,
        discount_minor,
        shipping_minor,
        grand_total_minor,
        item_total_formatted: format_minor_amount(item_total_minor),
        discount_formatted: format_minor_amount(-discount_minor),
        shipping_formatted: format_minor_amount(shipping_minor),
        grand_total_formatted: format_minor_amount(grand_total_minor),
        payment_mode: normalize_payment_mode(order.payment_method.as_deref()),
        payment_status: normalize_payment_status(order.payment_status.clone()),
    };
    let snapshot_json =
        serde_json::to_value(&snapshot).map_err(|e| Status::internal(e.to_string()))?;
    let pdf_bytes = pdf::render_invoice_pdf(&snapshot);
    let encoded_pdf = BASE64_STANDARD.encode(&pdf_bytes);
    let storage_path = build_storage_path(&invoice_number);

    let active = invoices::ActiveModel {
        invoice_id: ActiveValue::NotSet,
        invoice_number: ActiveValue::Set(invoice_number.clone()),
        order_id: ActiveValue::Set(order_id),
        user_id: ActiveValue::Set(order.user_id),
        generated_at: ActiveValue::Set(generated_at),
        storage_path: ActiveValue::Set(storage_path.clone()),
        pdf_blob: ActiveValue::Set(encoded_pdf),
        snapshot_json: ActiveValue::Set(snapshot_json),
        created_at: ActiveValue::Set(generated_at),
    };
    let inserted = match active.insert(txn).await {
        Ok(model) => model,
        Err(err) if is_duplicate_key(&err) => {
            if let Some(existing) = find_invoice_by_order_id(txn, order_id).await? {
                existing
            } else {
                return Err(map_db_error_to_status(err));
            }
        }
        Err(err) => return Err(map_db_error_to_status(err)),
    };

    order.invoice_id = Some(inserted.invoice_id);
    order.invoice_number = Some(inserted.invoice_number.clone());
    order.invoice_generated_at = Some(inserted.generated_at);
    order.invoice_storage_path = Some(inserted.storage_path.clone());
    let order_active: orders::ActiveModel = order.into();
    order_active
        .update(txn)
        .await
        .map_err(map_db_error_to_status)?;

    let payload = json!({
        "order_id": order_id,
        "user_id": inserted.user_id,
        "invoice_id": inserted.invoice_id,
        "invoice_number": inserted.invoice_number,
        "trigger": trigger,
    });
    let _ = enqueue_outbox_event(
        txn,
        INVOICE_GENERATED,
        "order",
        &order_id.to_string(),
        payload,
    )
    .await;

    info!(
        order_id,
        invoice_id = inserted.invoice_id,
        invoice_number = %inserted.invoice_number,
        trigger,
        "invoice generated"
    );
    Ok(Some(inserted))
}

pub async fn get_order_invoice(
    txn: &DatabaseTransaction,
    request: Request<GetOrderInvoiceRequest>,
) -> Result<Response<InvoiceResponse>, Status> {
    let req = request.into_inner();
    let invoice = find_invoice_by_order_id(txn, req.order_id)
        .await?
        .ok_or_else(|| Status::not_found(format!("Invoice for order {} not found", req.order_id)))?;
    Ok(Response::new(to_invoice_response(&invoice)))
}

pub async fn get_order_invoice_download(
    txn: &DatabaseTransaction,
    request: Request<GetOrderInvoiceDownloadRequest>,
) -> Result<Response<GetOrderInvoiceDownloadResponse>, Status> {
    let req = request.into_inner();
    let invoice = find_invoice_by_order_id(txn, req.order_id)
        .await?
        .ok_or_else(|| Status::not_found(format!("Invoice for order {} not found", req.order_id)))?;
    let pdf_bytes = BASE64_STANDARD
        .decode(invoice.pdf_blob.as_bytes())
        .map_err(|_| Status::internal("Invoice document is corrupted"))?;

    Ok(Response::new(GetOrderInvoiceDownloadResponse {
        invoice: Some(to_invoice_response(&invoice)),
        pdf_bytes,
        file_name: format!("{}.pdf", invoice.invoice_number),
        content_type: INVOICE_CONTENT_TYPE.to_string(),
    }))
}
