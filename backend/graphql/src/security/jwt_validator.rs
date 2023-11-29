use maplit::hashset;
use serde::{Deserialize, Serialize};
use strum::Display;
use tracing::{debug, instrument, warn};

use super::jwks_loader::JWKSet;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Audience {
    String(String),
    Vec(Vec<String>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub iss: String,
    pub aud: Audience,
    pub iat: u64,
    pub exp: u64,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub user_id: Option<String>,
    pub nonce: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Display)]
pub enum TokenParseError {
    TokenValidation(String),
    TokenParse(String),
}

#[instrument]
pub fn validate_token(b64_token: &str, jwks: &JWKSet) -> Result<Claims, TokenParseError> {
    tracing::info!("Validating token {}", b64_token);

    let b64_token = b64_token.split("Bearer ").collect::<Vec<&str>>()[1];

    let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
    validation.leeway = 5;

    validation.iss =
        Some(hashset! {std::env::var("OAUTH_DOMAIN").expect("OAUTH_DOMAIN env var not set")});

    validation.aud =
        Some(hashset! {std::env::var("OAUTH_AUDIANCE").expect("OAUTH_AUDIANCE env var not set")});

    for jwk in &jwks.keys {
        let key = jsonwebtoken::DecodingKey::from_rsa_components(&jwk.n, &jwk.e).map_err(|_e| {
            TokenParseError::TokenValidation("Creating key from modulus/exponent".to_string())
        })?;
        match jsonwebtoken::decode::<Claims>(b64_token, &key, &validation) {
            Ok(token_data) => {
                debug!("Successfully decoded token with kid {}", jwk.kid);
                return Ok(token_data.claims);
            }
            Err(e) => {
                debug!(
                    "(probably normal) Error decoding token with kid {}: {:#?}",
                    jwk.kid, e
                );
            }
        }
    }

    Err(TokenParseError::TokenValidation(
        "No key found to validate the token".to_string(),
    ))
}
