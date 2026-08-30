use crate::resolvers::grpc_client::connect_grpc_client_with_metadata;
use proto::proto::core::{SearchUserRequest, SearchUserRoleRequest};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

pub const ADMIN_ROLE_NAME: &str = "admin";
pub const SUPER_ADMIN_ROLE_NAME: &str = "super_admin";

#[derive(Clone)]
pub struct AdminResolution {
    pub is_admin: bool,
    pub source: &'static str,
    /// Resolved `UserStatuses.code` ("active" | "inactive" | "suspended"), or `None` if the
    /// user has never had a status set. `None` also when this resolution short-circuited
    /// before a user row was found/loaded (e.g. no such user) — never treated as deactivated
    /// in that case, only an explicit "inactive"/"suspended" code counts.
    pub account_status: Option<String>,
}

#[derive(Clone)]
struct CacheEntry {
    is_admin: bool,
    account_status: Option<String>,
    expires_at: Instant,
}

static ADMIN_ROLE_CACHE: OnceLock<Mutex<HashMap<String, CacheEntry>>> = OnceLock::new();

fn cache_ttl() -> Duration {
    let secs = std::env::var("ADMIN_ROLE_CACHE_TTL_SEC")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(300);
    Duration::from_secs(secs.max(1))
}

fn is_admin_role_name(role_name: &str) -> bool {
    matches!(
        role_name.trim().to_lowercase().as_str(),
        ADMIN_ROLE_NAME | SUPER_ADMIN_ROLE_NAME
    )
}

fn cache_get(jwt_sub: &str) -> Option<CacheEntry> {
    let cache = ADMIN_ROLE_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let Ok(mut guard) = cache.lock() else {
        return None;
    };
    let now = Instant::now();
    if let Some(entry) = guard.get(jwt_sub).cloned() {
        if entry.expires_at > now {
            return Some(entry);
        }
        guard.remove(jwt_sub);
    }
    None
}

fn cache_put(jwt_sub: &str, is_admin: bool, account_status: Option<String>) {
    let cache = ADMIN_ROLE_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let Ok(mut guard) = cache.lock() else {
        return;
    };
    guard.insert(
        jwt_sub.to_string(),
        CacheEntry {
            is_admin,
            account_status,
            expires_at: Instant::now() + cache_ttl(),
        },
    );
}

pub async fn resolve_admin_from_db(
    jwt_sub: &str,
    request_id: Option<&str>,
) -> Result<AdminResolution, String> {
    if let Some(entry) = cache_get(jwt_sub) {
        return Ok(AdminResolution {
            is_admin: entry.is_admin,
            source: "cache",
            account_status: entry.account_status,
        });
    }

    let mut client = connect_grpc_client_with_metadata(request_id)
        .await
        .map_err(|e| e.message)?;

    let user_resp = client
        .search_user(SearchUserRequest {
            user_id: None,
            username: None,
            email: None,
            auth_provider: Some("google".to_string()),
            google_sub: Some(jwt_sub.to_string()),
            full_name: None,
            address: None,
            phone: None,
            role_id: None,
            user_status_id: None,
            limit: Some(1),
            offset: Some(0),
        })
        .await
        .map_err(|e| e.message().to_string())?
        .into_inner();

    let Some(user) = user_resp.items.into_iter().next() else {
        cache_put(jwt_sub, false, None);
        return Ok(AdminResolution {
            is_admin: false,
            source: "db",
            account_status: None,
        });
    };

    // Carry the resolved status through every path below — a customer with no role_id can
    // still be deactivated, so "not admin" must never imply "not deactivated."
    let account_status = user.user_status.clone();

    let Some(role_id) = user.role_id else {
        cache_put(jwt_sub, false, account_status.clone());
        return Ok(AdminResolution {
            is_admin: false,
            source: "db",
            account_status,
        });
    };

    let role_resp = client
        .search_user_role(SearchUserRoleRequest { role_id })
        .await
        .map_err(|e| e.message().to_string())?
        .into_inner();

    let role_name = role_resp
        .items
        .into_iter()
        .next()
        .map(|r| r.role_name)
        .unwrap_or_default();
    let is_admin = is_admin_role_name(&role_name);
    cache_put(jwt_sub, is_admin, account_status.clone());
    Ok(AdminResolution {
        is_admin,
        source: "db",
        account_status,
    })
}

/// Resolve account_status by internal numeric user_id — used for the trusted internal-customer
/// auth path (`X-Internal-Auth` + `X-Customer-User-Id`, e.g. the invoice-download proxy), which
/// already knows the exact user_id and has no JWT `sub`/google_sub to key the admin-role cache
/// on. Deliberately uncached: this path's request volume is far lower than every JWT request,
/// so correctness (no stale window) matters more than shaving a DB round trip here.
pub async fn resolve_account_status_by_user_id(
    user_id: i64,
    request_id: Option<&str>,
) -> Result<Option<String>, String> {
    let mut client = connect_grpc_client_with_metadata(request_id)
        .await
        .map_err(|e| e.message)?;

    let user_resp = client
        .search_user(SearchUserRequest {
            user_id: Some(user_id),
            username: None,
            email: None,
            auth_provider: None,
            google_sub: None,
            full_name: None,
            address: None,
            phone: None,
            role_id: None,
            user_status_id: None,
            limit: Some(1),
            offset: Some(0),
        })
        .await
        .map_err(|e| e.message().to_string())?
        .into_inner();

    Ok(user_resp.items.into_iter().next().and_then(|u| u.user_status))
}
