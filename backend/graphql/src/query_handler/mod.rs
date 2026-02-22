use crate::security::jwks_loader::JWKSet;
use warp::Reply;

pub mod mutation_root;
pub mod query_root;

/// Describes how the current request was authenticated.
#[derive(Clone, Debug, PartialEq)]
pub enum AuthSource {
    /// Full login — authenticated via a valid JWT.  Value is the JWT `sub` claim (user identifier).
    Jwt(String),
    /// Guest session — validated via Redis `X-Session-Id` header.  Value is the stored `user_id`.
    Session(String),
}

#[derive(Clone, Debug)]
pub struct Context {
    pub jwks: JWKSet,
    /// Redis URL used for session-based auth fallback (`X-Session-Id` header).
    /// `None` when `REDIS_URL` is not configured (sessions disabled).
    pub redis_url: Option<String>,
    /// Authentication source for this specific request.
    /// `None` only during initial context construction; the auth gate always ensures this is `Some`
    /// before a resolver runs.
    pub auth: Option<AuthSource>,
}

impl Context {
    /// JWKS used for JWT validation (read by auth filter).
    pub fn jwks(&self) -> &JWKSet {
        &self.jwks
    }

    /// Returns the authenticated user ID **only when the request carried a valid JWT**.
    /// Returns `None` for guest (session-only) requests.
    /// Use this to guard operations that require a full login (e.g. checkout).
    pub fn jwt_user_id(&self) -> Option<&str> {
        match &self.auth {
            Some(AuthSource::Jwt(id)) => Some(id.as_str()),
            _ => None,
        }
    }

    /// Returns the user ID from either a JWT or a session, or `None` if unauthenticated.
    pub fn user_id(&self) -> Option<&str> {
        match &self.auth {
            Some(AuthSource::Jwt(id)) | Some(AuthSource::Session(id)) => Some(id.as_str()),
            None => None,
        }
    }
}

impl Reply for Context {
    fn into_response(self) -> warp::reply::Response {
        warp::reply::Response::new("foo".into())
    }
}
