use serde::{Deserialize, Serialize};
use tracing::info;
use tracing::instrument;

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

#[instrument]
pub async fn load_jwks() -> Result<JWKSet, JWKSLoaderError> {
    info!("Loading JWKS");

    let issuer = std::env::var("OAUTH_DOMAIN")
        .map_err(|_| JWKSLoaderError::Configuration("OAUTH_DOMAIN env var not set".to_string()))?;

    info!("Using issuer: {issuer}");

    let sep = if issuer.ends_with('/') { "" } else { "/" };

    let oidc_config = reqwest::get(format!(
        "{issuer}{sep}.well-known/openid-configuration",
        issuer = issuer
    ))
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

    info!("Retrieved OIDC Config: {oidc_config}");

    let config: OIDCConfig =
        serde_json::from_str(&oidc_config).expect("Unable to deserialize OIDC configuration");

    let jwks_txt = reqwest::get(&config.jwks_uri)
        .await
        .map_err(|e| JWKSLoaderError::Fetch(format!("Failed to fetch JWKS from issuer! {e:#?}")))?
        .text()
        .await
        .map_err(|e| {
            JWKSLoaderError::Fetch(format!("Failed to read JWKS from response! {e:#?}"))
        })?;

    info!(
        "Retrieved JWKS from {uri}: {jwks_txt}",
        uri = config.jwks_uri
    );

    let jwks: JWKSet = serde_json::from_str(&jwks_txt)
        .map_err(|e| JWKSLoaderError::Parse(format!("JWKS Parse Error: {e:#?}")))?;

    info!("Finished loading JWKS.");

    Ok(jwks)
}
