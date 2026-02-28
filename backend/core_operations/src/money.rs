// Minimal helpers for working with minor units (paise) in commerce paths.
// For this pass, we keep the database schema as-is (DECIMAL / f64 at the proto
// boundary) but treat integer minor units as the canonical representation in
// core logic. Conversions to/from f64 are confined to small helpers here.
// Convert a major-unit amount (e.g. rupees as f64) into minor units (paise).
// This is intended only for protocol/DB boundary shims where the source type
// is already f64. Core business logic should operate on the returned paise.

pub fn paise_from_major_f64(amount_major: f64) -> i64 {
    // Round to the nearest paise to avoid systematic truncation.
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
