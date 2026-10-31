use crate::security::jwks_loader::JWKSet;
use warp::Reply;

pub mod admin_roles;
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
    /// Optional guest session identifier for guest-scoped operations.
    /// Prefer the validated `X-Session-Id` when present; otherwise use `X-Guest-Session-Id`
    /// for frontend correlation/internal forwarding.
    pub guest_session_id: Option<String>,
    /// JWT sub claim when authenticated via JWT.
    pub jwt_subject: Option<String>,
    /// Per-request admin resolution computed at auth gate (DB/cache).
    pub admin_authorized: Option<bool>,
    /// Resolution source for observability (`cache`, `db`, `env_fallback`, `none`).
    pub admin_resolution_source: Option<String>,
    /// Resolved `UserStatuses.code` for the JWT-authenticated user, computed at the same
    /// auth-gate DB/cache lookup as `admin_authorized` (see `admin_roles::resolve_admin_from_db`).
    /// `None` for non-JWT auth (guest session, internal service/customer) or when the user has
    /// never had a status explicitly set (treated as active). Used by `require_jwt` to reject
    /// deactivated/suspended accounts.
    pub account_status: Option<String>,
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
    /// Returns `None` for guest sessions and internal service calls — and, deliberately, for a
    /// deactivated/suspended account, even though `self.auth` itself is still `Some`. This is
    /// the actual enforcement point: several mutations (e.g. `place_order`) check
    /// `jwt_user_id()` directly instead of going through `require_jwt`/`require_customer_actor`,
    /// so gating deactivation here — rather than only in those two helpers — is what makes it
    /// impossible to bypass by hitting one of those call sites instead.
    pub fn jwt_user_id(&self) -> Option<&str> {
        if self.account_deactivated() {
            return None;
        }
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

    pub fn jwt_subject(&self) -> Option<&str> {
        self.jwt_subject.as_deref()
    }

    pub fn admin_resolution_source(&self) -> Option<&str> {
        self.admin_resolution_source.as_deref()
    }

    /// True when the JWT-authenticated user's account is "inactive" or "suspended" — the
    /// admin-set states, not just "never had a status set" (which stays `None`/active).
    pub fn account_deactivated(&self) -> bool {
        matches!(
            self.account_status.as_deref(),
            Some("inactive") | Some("suspended")
        )
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

    /// Admin is resolved at request auth gate via DB role lookup.
    /// Temporary fallback: if request-gate did not compute admin, uses `ADMIN_ALLOWED_USER_IDS`.
    pub fn is_admin(&self) -> bool {
        if let Some(is_admin) = self.admin_authorized {
            return is_admin;
        }
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
