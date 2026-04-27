use super::schema::InvoiceDownload;
use crate::resolvers::{
    error::GqlError,
    utils::{connect_grpc_client, parse_i64},
};
use base64::Engine;
use proto::proto::core::GetOrderInvoiceDownloadRequest;
use tracing::instrument;

#[instrument]
pub(crate) async fn get_order_invoice_download(order_id: String) -> Result<InvoiceDownload, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .get_order_invoice_download(GetOrderInvoiceDownloadRequest {
            order_id: parse_i64(&order_id, "order_id")?,
        })
        .await?;
    let row = response.into_inner();
    let invoice = row
        .invoice
        .ok_or_else(|| GqlError::new("invoice payload missing", crate::resolvers::error::Code::Internal))?;
    let pdf_base64 = base64::engine::general_purpose::STANDARD.encode(row.pdf_bytes);

    Ok(InvoiceDownload {
        invoice_id: invoice.invoice_id.to_string(),
        invoice_number: invoice.invoice_number,
        order_id: invoice.order_id.to_string(),
        generated_at: invoice.generated_at,
        file_name: row.file_name,
        content_type: row.content_type,
        pdf_base64,
    })
}
