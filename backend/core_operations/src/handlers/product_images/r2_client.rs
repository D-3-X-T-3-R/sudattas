use aws_config::Region;
use aws_credential_types::{provider::SharedCredentialsProvider, Credentials};
use aws_sdk_s3::Client;

/// Build an S3 client pointed at Cloudflare R2.
pub fn build_r2_client() -> Option<(Client, String, String)> {
    let account_id = std::env::var("R2_ACCOUNT_ID").ok()?;
    let access_key = std::env::var("R2_ACCESS_KEY_ID").ok()?;
    let secret_key = std::env::var("R2_SECRET_ACCESS_KEY").ok()?;
    let bucket = std::env::var("R2_BUCKET_NAME").ok()?;
    let public_url = std::env::var("R2_PUBLIC_URL").ok()?;

    let endpoint = std::env::var("R2_ENDPOINT")
        .unwrap_or_else(|_| format!("https://{}.r2.cloudflarestorage.com", account_id));

    let creds = SharedCredentialsProvider::new(Credentials::new(
        access_key,
        secret_key,
        None,
        None,
        "r2",
    ));

    let config = aws_config::SdkConfig::builder()
        .region(Region::new("auto"))
        .credentials_provider(creds)
        .endpoint_url(endpoint)
        .build();

    let client = Client::new(&config);
    Some((client, bucket, public_url))
}
