use juniper::graphql_object;

#[derive(Default, Debug, Clone)]
pub struct InvoiceDownload {
    pub invoice_id: String,
    pub invoice_number: String,
    pub order_id: String,
    pub generated_at: String,
    pub file_name: String,
    pub content_type: String,
    pub pdf_base64: String,
}

#[graphql_object]
#[graphql(description = "Invoice download payload")]
impl InvoiceDownload {
    async fn invoice_id(&self) -> &String {
        &self.invoice_id
    }
    async fn invoice_number(&self) -> &String {
        &self.invoice_number
    }
    async fn order_id(&self) -> &String {
        &self.order_id
    }
    async fn generated_at(&self) -> &String {
        &self.generated_at
    }
    async fn file_name(&self) -> &String {
        &self.file_name
    }
    async fn content_type(&self) -> &String {
        &self.content_type
    }
    async fn pdf_base64(&self) -> &String {
        &self.pdf_base64
    }
}
