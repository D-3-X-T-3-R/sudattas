use juniper::graphql_object;

#[derive(Default, Debug, Clone)]
pub struct InvoiceDownload {
    pub download_url: String,
}

#[graphql_object]
#[graphql(description = "Invoice download payload")]
impl InvoiceDownload {
    async fn download_url(&self) -> &String {
        &self.download_url
    }
}
