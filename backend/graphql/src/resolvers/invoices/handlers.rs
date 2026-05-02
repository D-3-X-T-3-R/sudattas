use super::schema::InvoiceDownload;
use crate::query_handler::Context;
use crate::resolvers::{
    error::{Code, GqlError},
    utils::{connect_grpc_client, connect_grpc_client_from_context, parse_i64},
};
use proto::proto::core::{GetOrderInvoiceDownloadRequest, GetOrderInvoiceRequest};
use tracing::instrument;

#[derive(Debug)]
pub struct InvoiceDownloadBinary {
    pub content_type: String,
    pub file_name: String,
    pub pdf_bytes: Vec<u8>,
}

fn invoice_download_base_url() -> String {
    if let Ok(raw) = std::env::var("INVOICE_DOWNLOAD_BASE_URL") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return trimmed.trim_end_matches('/').to_string();
        }
    }
    if let Ok(raw) = std::env::var("GRAPHQL_PUBLIC_BASE_URL") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return trimmed.trim_end_matches('/').to_string();
        }
    }
    if let Ok(raw) = std::env::var("GRAPHQL_URL") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            let normalized = trimmed
                .trim_end_matches('/')
                .trim_end_matches("/v2")
                .trim_end_matches('/');
            if !normalized.is_empty() {
                return normalized.to_string();
            }
        }
    }
    "http://127.0.0.1:8080".to_string()
}

fn parse_order_id_from_invoice_number(invoice_number: &str) -> Result<i64, GqlError> {
    let trimmed = invoice_number.trim();
    if trimmed.is_empty() {
        return Err(GqlError::new(
            "invoice_number cannot be empty",
            Code::InvalidArgument,
        ));
    }
    let parts: Vec<&str> = trimmed.split('-').collect();
    if parts.len() != 3 || parts[0] != "INV" {
        return Err(GqlError::new(
            "invoice_number format is invalid",
            Code::InvalidArgument,
        ));
    }
    let order_part = parts[2];
    if order_part.is_empty() || !order_part.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(GqlError::new(
            "invoice_number format is invalid",
            Code::InvalidArgument,
        ));
    }
    order_part.parse::<i64>().map_err(|_| {
        GqlError::new(
            "invoice_number order segment overflow",
            Code::InvalidArgument,
        )
    })
}

#[instrument]
pub(crate) async fn get_order_invoice_download(
    order_id: String,
) -> Result<InvoiceDownload, GqlError> {
    let parsed_order_id = parse_i64(&order_id, "order_id")?;
    let mut client = connect_grpc_client().await?;
    let response = client
        .get_order_invoice(GetOrderInvoiceRequest {
            order_id: parsed_order_id,
        })
        .await?;
    let row = response.into_inner();
    let download_url = format!(
        "{}/invoices/{}/download",
        invoice_download_base_url(),
        row.invoice_number
    );

    Ok(InvoiceDownload { download_url })
}

#[instrument(skip(context))]
pub async fn download_invoice_pdf(
    context: &Context,
    invoice_number: &str,
) -> Result<InvoiceDownloadBinary, GqlError> {
    let requested_invoice_number = invoice_number.trim();
    let order_id = parse_order_id_from_invoice_number(requested_invoice_number)?;

    let mut client = connect_grpc_client_from_context(context).await?;
    let response = client
        .get_order_invoice_download(GetOrderInvoiceDownloadRequest { order_id })
        .await?;
    let row = response.into_inner();
    let invoice = row
        .invoice
        .ok_or_else(|| GqlError::new("invoice payload missing", Code::Internal))?;

    if invoice.invoice_number != requested_invoice_number {
        return Err(GqlError::new("invoice not found", Code::NotFound));
    }

    if !context.is_admin() {
        let requester_user_id = context
            .jwt_user_id()
            .ok_or_else(|| GqlError::new("authentication required", Code::Unauthenticated))?;
        if requester_user_id != invoice.user_id.to_string() {
            return Err(GqlError::new(
                "invoice does not belong to current user",
                Code::PermissionDenied,
            ));
        }
    }

    if row.pdf_bytes.is_empty() || !row.pdf_bytes.starts_with(b"%PDF-") {
        return Err(GqlError::new(
            "invoice payload is corrupted",
            Code::Internal,
        ));
    }

    let file_name = format!("Invoice_{}.pdf", requested_invoice_number);
    let content_type = if row.content_type.trim().is_empty() {
        "application/pdf".to_string()
    } else {
        row.content_type
    };

    Ok(InvoiceDownloadBinary {
        content_type,
        file_name,
        pdf_bytes: row.pdf_bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_order_id_from_invoice_number() {
        let parsed = parse_order_id_from_invoice_number("INV-20260501-000123").expect("parse");
        assert_eq!(parsed, 123);
    }

    #[test]
    fn rejects_bad_invoice_number() {
        let err = parse_order_id_from_invoice_number("bad").expect_err("should fail");
        assert_eq!(err.code, Code::InvalidArgument);
    }
}
