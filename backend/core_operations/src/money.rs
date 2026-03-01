//! Minimal helpers for working with minor units (paise) in commerce paths.
//! All money in proto/API is int64 paise; DB may still have Decimal columns
//! with strict conversion at boundaries (no floats in commerce paths).

use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

/// Convert paise to Decimal for DB columns (e.g. Price, TotalAmount).
pub fn paise_to_decimal(paise: i64) -> Decimal {
    Decimal::from(paise) / Decimal::from(100)
}

/// Convert DB Decimal amount to paise for proto/API.
pub fn decimal_to_paise(d: &Decimal) -> i64 {
    (d * Decimal::from(100)).round().to_i64().unwrap_or(0)
}

/// Discount: basis points (100 = 1%) to DB Decimal percentage (e.g. 10.5).
pub fn basis_points_to_percentage_decimal(basis_points: i32) -> Decimal {
    Decimal::from(basis_points) / Decimal::from(100)
}

/// Discount: DB Decimal percentage to basis points for proto.
pub fn percentage_decimal_to_basis_points(pct: Option<&Decimal>) -> i32 {
    pct.map(|d| (d * Decimal::from(100)).round().to_i32().unwrap_or(0))
        .unwrap_or(0)
}

/// Legacy: major-unit f64 -> paise (only for boundary shims; prefer paise everywhere).
pub fn paise_from_major_f64(amount_major: f64) -> i64 {
    (amount_major * 100.0).round() as i64
}

pub fn paise_to_major_f64(amount_paise: i64) -> f64 {
    amount_paise as f64 / 100.0
}

pub fn paise_checked_add(a: i64, b: i64) -> Result<i64, &'static str> {
    a.checked_add(b).ok_or("paise addition overflow")
}

pub fn paise_checked_mul(price_paise: i64, quantity: i64) -> Result<i64, &'static str> {
    price_paise
        .checked_mul(quantity)
        .ok_or("paise multiplication overflow")
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    /// Rounding: round-trip paise -> major f64 -> paise recovers exact paise for in-range values.
    #[test]
    fn rounding_paise_round_trip() {
        let p = 12345_i64;
        let major = paise_to_major_f64(p);
        let back = paise_from_major_f64(major);
        assert_eq!(back, p, "paise round-trip");
    }

    /// Boundary: 0.005 rounds to 1 paise (half-up).
    #[test]
    fn rounding_half_up_boundary() {
        assert_eq!(paise_from_major_f64(0.005), 1);
        assert_eq!(paise_from_major_f64(-0.005), -1);
    }

    /// paise_checked_add: commutativity and overflow returns Err.
    #[test]
    fn add_commutative() {
        assert_eq!(paise_checked_add(100, 200), paise_checked_add(200, 100));
    }

    #[test]
    fn add_overflow_returns_err() {
        let max = i64::MAX;
        assert!(paise_checked_add(max, 1).is_err());
    }

    /// paise_checked_mul: overflow returns Err; zero and one behave correctly.
    #[test]
    fn mul_overflow_returns_err() {
        let max = i64::MAX;
        assert!(paise_checked_mul(max, 2).is_err());
    }

    #[test]
    fn mul_zero_and_one() {
        assert_eq!(paise_checked_mul(9999, 0), Ok(0));
        assert_eq!(paise_checked_mul(9999, 1), Ok(9999));
    }

    /// paise_to_decimal / decimal_to_paise round-trip for typical amounts.
    #[test]
    fn paise_decimal_round_trip() {
        for paise in [0_i64, 100, 49900, 99999, 1_000_000] {
            let d = paise_to_decimal(paise);
            let back = decimal_to_paise(&d);
            assert_eq!(back, paise, "paise {} -> decimal -> paise", paise);
        }
    }

    /// Basis points: 100 = 1%, 1050 = 10.5%; round-trip through DB Decimal.
    #[test]
    fn basis_points_round_trip() {
        assert_eq!(
            percentage_decimal_to_basis_points(Some(&basis_points_to_percentage_decimal(100))),
            100
        );
        assert_eq!(
            percentage_decimal_to_basis_points(Some(&basis_points_to_percentage_decimal(1050))),
            1050
        );
    }

    /// Multi-line cart: sum of (price_paise * quantity) matches manual total.
    #[test]
    fn multi_line_cart_paise() {
        let lines = [(1000_i64, 2_i64), (49900, 1), (250, 4)]; // ₹10×2, ₹499×1, ₹2.50×4
        let mut total = 0_i64;
        for (price_paise, qty) in lines {
            total = paise_checked_add(total, paise_checked_mul(price_paise, qty).unwrap()).unwrap();
        }
        assert_eq!(total, 2000 + 49900 + 1000, "line totals in paise");
    }

    proptest! {
        /// Any paise in i64 range round-trips through major f64 (may lose precision for huge values).
        #[test]
        fn prop_paise_round_trip(paise in -1_000_000_000_i64..=1_000_000_000_i64) {
            let major = paise_to_major_f64(paise);
            let back = paise_from_major_f64(major);
            assert_eq!(back, paise);
        }

        /// Sum of two paise values: Ok(s) implies s == a+b; overflow implies Err.
        #[test]
        fn prop_add_consistent(a in -1_000_000_000_i64..=1_000_000_000_i64, b in -1_000_000_000_i64..=1_000_000_000_i64) {
            let r = paise_checked_add(a, b);
            match (r, a.checked_add(b)) {
                (Ok(sum), Some(expected)) => assert_eq!(sum, expected),
                (Err(_), None) => { /* overflow correctly detected */ }
                (Ok(_), None) => panic!("add should have overflowed"),
                (Err(_), Some(_)) => panic!("add should not have overflowed"),
            }
        }

        /// Large cart: many lines (price_paise * quantity) summed; reasonable bounds to avoid overflow.
        #[test]
        fn prop_large_cart_line_totals(
            price_paise in 1_i64..=10_000_000_i64,  // up to 1 lakh INR per unit
            quantity in 1_i64..=1000_i64
        ) {
            let line = paise_checked_mul(price_paise, quantity).unwrap();
            assert!(line >= 0);
            assert!(line <= price_paise * quantity);
        }
    }
}
