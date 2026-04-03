use crate::security::jwks_loader::JWKSet;
use warp::Reply;

pub mod mutation_root;
pub mod query_root;

/// Describes how the current request was authenticated.
#[derive(Clone, Debug, PartialEq)]
pub enum AuthSource {
    /// Full login authenticated via a valid JWT. Value is the JWT subject/user identifier.
    Jwt(String),
    /// Guest session validated via Redis `X-Session-Id`. Value is the stored `user_id`.
    Session(String),
    /// Internal server-to-server customer auth from trusted frontend proxy.
    /// Value is canonical numeric customer user_id.
    InternalCustomer(String),
    /// Internal server-to-server auth for service operations that do not act as a customer.
    InternalService,
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
    /// Request ID for distributed tracing; propagated to gRPC as `x-request-id` when set.
    pub request_id: Option<String>,
    /// Optional idempotency key from `Idempotency-Key` header; used for place_order and capture_payment.
    pub idempotency_key: Option<String>,
    /// Optional client action name from `X-Client-Action` header.
    pub client_action: Option<String>,
    /// Optional guest session identifier from `X-Guest-Session-Id` header (frontend correlation).
    pub guest_session_id: Option<String>,
}

impl Context {
    fn admin_allowlist_user_ids() -> Vec<String> {
        std::env::var("ADMIN_ALLOWED_USER_IDS")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// JWKS used for JWT validation (read by auth filter).
    pub fn jwks(&self) -> &JWKSet {
        &self.jwks
    }

    /// Returns the authenticated customer user ID for JWT/internal-customer requests.
    /// Returns `None` for guest sessions and internal service calls.
    pub fn jwt_user_id(&self) -> Option<&str> {
        match &self.auth {
            Some(AuthSource::Jwt(id)) | Some(AuthSource::InternalCustomer(id)) => Some(id.as_str()),
            _ => None,
        }
    }

    /// Returns the resolved user ID when available across auth modes.
    pub fn user_id(&self) -> Option<&str> {
        match &self.auth {
            Some(AuthSource::Jwt(id))
            | Some(AuthSource::Session(id))
            | Some(AuthSource::InternalCustomer(id)) => Some(id.as_str()),
            Some(AuthSource::InternalService) | None => None,
        }
    }

    /// Request ID for this request; propagated to gRPC for distributed tracing.
    pub fn request_id(&self) -> Option<&str> {
        self.request_id.as_deref()
    }

    /// Idempotency key from header, when present; used to dedupe place_order and capture_payment.
    pub fn idempotency_key(&self) -> Option<&str> {
        self.idempotency_key.as_deref()
    }

    /// Frontend-provided action route/method for correlation.
    pub fn client_action(&self) -> Option<&str> {
        self.client_action.as_deref()
    }

    /// Guest session identifier forwarded by frontend for correlation.
    pub fn guest_session_id(&self) -> Option<&str> {
        self.guest_session_id.as_deref()
    }

    pub fn auth_mode(&self) -> &'static str {
        match &self.auth {
            Some(AuthSource::Jwt(_)) => "jwt",
            Some(AuthSource::Session(_)) => "session",
            Some(AuthSource::InternalCustomer(_)) => "internal_customer",
            Some(AuthSource::InternalService) => "internal_service",
            None => "none",
        }
    }

    /// Admin is resolved from JWT user id against `ADMIN_ALLOWED_USER_IDS` (comma-separated).
    /// Empty allowlist means no admin privileges are granted.
    pub fn is_admin(&self) -> bool {
        let Some(uid) = self.jwt_user_id() else {
            return false;
        };
        let allowlist = Self::admin_allowlist_user_ids();
        !allowlist.is_empty() && allowlist.iter().any(|a| a == uid)
    }
}

impl Reply for Context {
    fn into_response(self) -> warp::reply::Response {
        warp::reply::Response::new("foo".into())
    }
}
