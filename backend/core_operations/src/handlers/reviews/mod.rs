pub mod admin_update_review_status;
pub use admin_update_review_status::*;

pub mod create_review;
pub use create_review::*;

pub mod search_review;
pub use search_review::*;

pub mod update_review;
pub use update_review::*;

pub mod delete_review;
pub use delete_review::*;

pub mod get_product_rating_summary;
pub use get_product_rating_summary::*;

use core_db_entities::entity::sea_orm_active_enums::ReviewStatus;

/// Shared by every handler that builds a `ReviewResponse` — DB default is `pending` when unset.
pub(crate) fn review_status_to_string(status: Option<ReviewStatus>) -> String {
    match status {
        Some(ReviewStatus::Approved) => "approved",
        Some(ReviewStatus::Rejected) => "rejected",
        Some(ReviewStatus::Pending) | None => "pending",
    }
    .to_string()
}

pub(crate) fn is_verified_purchase_to_bool(value: Option<i8>) -> bool {
    value.unwrap_or(0) != 0
}

pub(crate) fn created_at_to_string(value: Option<chrono::DateTime<chrono::Utc>>) -> String {
    value.map(|v| v.to_rfc3339()).unwrap_or_default()
}
