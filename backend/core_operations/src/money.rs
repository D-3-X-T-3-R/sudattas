/// Minimal helpers for working with minor units (paise) in commerce paths.
///
/// For this pass, we keep the database schema as-is (DECIMAL / f64 at the proto
/// boundary) but treat integer minor units as the canonical representation in
/// core logic. Conversions to/from f64 are confined to small helpers here.

/// Convert a major-unit amount (e.g. rupees as f64) into minor units (paise).
///
/// This is intended only for protocol/DB boundary shims where the source type
/// is already f64. Core business logic should operate on the returned paise.
pub fn paise_from_major_f64(amount_major: f64) -> i64 {
    // Round to the nearest paise to avoid systematic truncation.
    (amount_major * 100.0).round() as i64
}

/// Convert minor units (paise) back into a major-unit f64.
///
/// This is used only when we need to populate proto fields that are currently
/// defined as f64. All internal arithmetic should remain in paise.
pub fn paise_to_major_f64(amount_paise: i64) -> f64 {
    amount_paise as f64 / 100.0
}

/// Safe addition for paise amounts with basic overflow protection.
pub fn paise_checked_add(a: i64, b: i64) -> Result<i64, &'static str> {
    a.checked_add(b).ok_or("paise addition overflow")
}

/// Safe multiplication of a unit price (in paise) by a quantity, with overflow protection.
pub fn paise_checked_mul(price_paise: i64, quantity: i64) -> Result<i64, &'static str> {
    price_paise
        .checked_mul(quantity)
        .ok_or("paise multiplication overflow")
}
