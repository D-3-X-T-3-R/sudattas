use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::info;
use tracing::instrument;
use tracing::warn;

/// Timeout applied to both the OIDC discovery request and the JWKS fetch itself.
/// Without this, a slow/unreachable IdP could hang the calling task indefinitely.
const JWKS_HTTP_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Serialize, Deserialize, Debug)]
pub struct OIDCConfig {
    pub issuer: String,
    pub jwks_uri: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JWKey {
    pub e: String,
    pub n: String,
    pub kty: String,
    pub r#use: String,
    pub alg: String,
    pub kid: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JWKSet {
    pub keys: Vec<JWKey>,
}

#[derive(Debug)]
pub enum JWKSLoaderError {
    Fetch(String),
    Parse(String),
    Configuration(String),
}

impl std::fmt::Display for JWKSLoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JWKSLoaderError::Fetch(msg) => write!(f, "JWKS fetch error: {}", msg),
            JWKSLoaderError::Parse(msg) => write!(f, "JWKS parse error: {}", msg),
            JWKSLoaderError::Configuration(msg) => write!(f, "JWKS configuration error: {}", msg),
        }
    }
}

fn http_client() -> Result<reqwest::Client, JWKSLoaderError> {
    reqwest::Client::builder()
        .timeout(JWKS_HTTP_TIMEOUT)
        .build()
        .map_err(|e| JWKSLoaderError::Fetch(format!("Failed to build HTTP client: {e}")))
}

#[instrument]
pub async fn load_jwks() -> Result<JWKSet, JWKSLoaderError> {
    info!("Loading JWKS");

    let issuer = std::env::var("OAUTH_DOMAIN")
        .map_err(|_| JWKSLoaderError::Configuration("OAUTH_DOMAIN env var not set".to_string()))?;

    info!("Using issuer: {issuer}");

    let sep = if issuer.ends_with('/') { "" } else { "/" };
    let client = http_client()?;

    let oidc_config = client
        .get(format!(
            "{issuer}{sep}.well-known/openid-configuration",
            issuer = issuer
        ))
        .send()
        .await
        .map_err(|e| {
            JWKSLoaderError::Fetch(format!(
                "Failed to fetch OIDC Configuration from issuer! {e:#?}"
            ))
        })?
        .text()
        .await
        .map_err(|_e| {
            JWKSLoaderError::Fetch("Failed to read OIDC Configuration from response!".to_string())
        })?;

    let config: OIDCConfig = serde_json::from_str(&oidc_config).map_err(|e| {
        JWKSLoaderError::Parse(format!("Unable to deserialize OIDC configuration: {e:#?}"))
    })?;

    info!("Using jwks_uri: {}", config.jwks_uri);

    let jwks_txt = client
        .get(&config.jwks_uri)
        .send()
        .await
        .map_err(|e| JWKSLoaderError::Fetch(format!("Failed to fetch JWKS from issuer! {e:#?}")))?
        .text()
        .await
        .map_err(|e| {
            JWKSLoaderError::Fetch(format!("Failed to read JWKS from response! {e:#?}"))
        })?;

    let jwks: JWKSet = serde_json::from_str(&jwks_txt)
        .map_err(|e| JWKSLoaderError::Parse(format!("JWKS Parse Error: {e:#?}")))?;

    info!(key_count = jwks.keys.len(), "Finished loading JWKS.");

    Ok(jwks)
}

/// Load JWKS with a small bounded retry, so a transient blip in the IdP (common right
/// after a deploy/restart) doesn't need to fall back to an empty key set. Each attempt
/// still respects `JWKS_HTTP_TIMEOUT`, so this call is bounded overall.
pub async fn load_jwks_with_retries(
    max_attempts: u32,
    retry_delay: Duration,
) -> Result<JWKSet, JWKSLoaderError> {
    let attempts = max_attempts.max(1);
    let mut last_err = None;
    for attempt in 1..=attempts {
        match load_jwks().await {
            Ok(jwks) => return Ok(jwks),
            Err(e) => {
                warn!(attempt, attempts, error = %e, "JWKS load attempt failed");
                last_err = Some(e);
                if attempt < attempts {
                    tokio::time::sleep(retry_delay).await;
                }
            }
        }
    }
    Err(last_err.unwrap_or_else(|| JWKSLoaderError::Fetch("no attempts made".to_string())))
}
