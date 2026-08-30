//! Shared helpers for resolving between `Users.user_status_id` (a raw FK) and the human-readable
//! `UserStatuses.code` ("active" | "inactive" | "suspended") that callers actually work with.

use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::user_statuses;
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use std::collections::HashMap;
use tonic::Status;

/// The three statuses this app models. Anything else is rejected as invalid input.
pub(crate) fn normalize_status_code(input: &str) -> Option<&'static str> {
    match input.trim().to_lowercase().as_str() {
        "active" => Some("active"),
        "inactive" => Some("inactive"),
        "suspended" => Some("suspended"),
        _ => None,
    }
}

/// `UserStatuses` is a tiny, effectively-static lookup table (3 rows) — fetching it whole and
/// mapping in memory avoids either a per-row join or hardcoding the seeded ids.
pub(crate) async fn fetch_status_code_map(
    txn: &DatabaseTransaction,
) -> Result<HashMap<i64, String>, Status> {
    let rows = user_statuses::Entity::find()
        .all(txn)
        .await
        .map_err(map_db_error_to_status)?;
    Ok(rows.into_iter().map(|r| (r.id, r.code)).collect())
}

pub(crate) fn resolve_status_code(map: &HashMap<i64, String>, user_status_id: Option<i64>) -> Option<String> {
    user_status_id.and_then(|id| map.get(&id).cloned())
}

/// Look up the `UserStatuses` row id for a normalized code. `Err` only if the seed data for
/// this fixed 3-row table is somehow missing — a deployment/migration problem, not user error.
pub(crate) async fn status_id_for_code(
    txn: &DatabaseTransaction,
    code: &str,
) -> Result<i64, Status> {
    user_statuses::Entity::find()
        .filter(user_statuses::Column::Code.eq(code))
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .map(|row| row.id)
        .ok_or_else(|| {
            Status::internal(format!(
                "UserStatuses row for '{code}' not found — was the seed migration applied?"
            ))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_known_codes_case_insensitively() {
        assert_eq!(normalize_status_code("Active"), Some("active"));
        assert_eq!(normalize_status_code(" INACTIVE "), Some("inactive"));
        assert_eq!(normalize_status_code("Suspended"), Some("suspended"));
    }

    #[test]
    fn rejects_unknown_codes() {
        assert_eq!(normalize_status_code("banned"), None);
        assert_eq!(normalize_status_code(""), None);
    }

    #[test]
    fn resolve_status_code_maps_known_id() {
        let mut map = HashMap::new();
        map.insert(2, "inactive".to_string());
        assert_eq!(resolve_status_code(&map, Some(2)), Some("inactive".to_string()));
    }

    #[test]
    fn resolve_status_code_none_for_unset_or_unknown() {
        let map = HashMap::new();
        assert_eq!(resolve_status_code(&map, None), None);
        assert_eq!(resolve_status_code(&map, Some(99)), None);
    }
}
