//! Collision-resistant immutable public order references (`SUD-YYYYMMDD-SUFFIX`).

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};

const PUBLIC_REF_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

/// Build one candidate reference (caller retries on DB unique constraint violation).
pub fn generate_public_order_ref_candidate(order_date: DateTime<Utc>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(uuid::Uuid::new_v4().as_bytes());
    hasher.update(
        order_date
            .timestamp_nanos_opt()
            .unwrap_or_else(|| order_date.timestamp() * 1_000_000_000)
            .to_le_bytes(),
    );
    let h = hasher.finalize();
    let suffix: String = h
        .iter()
        .take(10)
        .map(|b| PUBLIC_REF_CHARSET[(b % 36) as usize] as char)
        .collect();
    format!("SUD-{}-{}", order_date.format("%Y%m%d"), suffix)
}

pub(crate) fn is_duplicate_key_error(err: &sea_orm::DbErr) -> bool {
    match err {
        sea_orm::DbErr::Exec(exec) => {
            let message = exec.to_string();
            message.contains("Duplicate entry") || message.contains("1062")
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn candidate_matches_sud_date_suffix_shape() {
        let d = Utc.with_ymd_and_hms(2026, 4, 16, 12, 0, 0).unwrap();
        let s = generate_public_order_ref_candidate(d);
        assert!(s.starts_with("SUD-20260416-"), "got {s}");
        let suf = s.strip_prefix("SUD-20260416-").expect("suffix");
        assert_eq!(suf.len(), 10);
        assert!(suf
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()));
    }

    #[test]
    fn repeated_generation_is_mostly_unique() {
        let d = Utc::now();
        let mut set = std::collections::HashSet::new();
        for _ in 0..500 {
            set.insert(generate_public_order_ref_candidate(d));
        }
        assert!(
            set.len() > 480,
            "expected few collisions among random suffixes, got {} unique",
            set.len()
        );
    }
}
