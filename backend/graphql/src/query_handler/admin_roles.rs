use crate::resolvers::grpc_client::connect_grpc_client_with_metadata;
use proto::proto::core::{SearchUserRequest, SearchUserRoleRequest};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

pub const ADMIN_ROLE_NAME: &str = "admin";
pub const SUPER_ADMIN_ROLE_NAME: &str = "super_admin";

#[derive(Clone, Copy)]
pub struct AdminResolution {
    pub is_admin: bool,
    pub source: &'static str,
}

#[derive(Clone, Copy)]
struct CacheEntry {
    is_admin: bool,
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

fn cache_get(jwt_sub: &str) -> Option<bool> {
    let cache = ADMIN_ROLE_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let Ok(mut guard) = cache.lock() else {
        return None;
    };
    let now = Instant::now();
    if let Some(entry) = guard.get(jwt_sub).copied() {
        if entry.expires_at > now {
            return Some(entry.is_admin);
        }
        guard.remove(jwt_sub);
    }
    None
}

fn cache_put(jwt_sub: &str, is_admin: bool) {
    let cache = ADMIN_ROLE_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let Ok(mut guard) = cache.lock() else {
        return;
    };
    guard.insert(
        jwt_sub.to_string(),
        CacheEntry {
            is_admin,
            expires_at: Instant::now() + cache_ttl(),
        },
    );
}

pub async fn resolve_admin_from_db(
    jwt_sub: &str,
    request_id: Option<&str>,
) -> Result<AdminResolution, String> {
    if let Some(is_admin) = cache_get(jwt_sub) {
        return Ok(AdminResolution {
            is_admin,
            source: "cache",
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
        cache_put(jwt_sub, false);
        return Ok(AdminResolution {
            is_admin: false,
            source: "db",
        });
    };

    let Some(role_id) = user.role_id else {
        cache_put(jwt_sub, false);
        return Ok(AdminResolution {
            is_admin: false,
            source: "db",
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
    cache_put(jwt_sub, is_admin);
    Ok(AdminResolution {
        is_admin,
        source: "db",
    })
}
