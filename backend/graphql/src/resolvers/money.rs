//! GraphQL Money type: integer minor units (paise) + formatted string.
//! Avoids exposing floats for amounts in the API.

use juniper::graphql_object;

const DEFAULT_CURRENCY: &str = "INR";

/// Money amount with integer minor units (paise for INR) and a formatted display string.
#[derive(Debug, Clone)]
pub struct Money {
    pub amount_paise: i64,
    pub currency: String,
    pub formatted: String,
}

#[graphql_object]
#[graphql(description = "Money amount: integer minor units (paise) and formatted string")]
impl Money {
    /// Amount in minor units (paise for INR; 1 INR = 100 paise). String to avoid GraphQL Int 32-bit limit.
    async fn amount_paise(&self) -> String {
        self.amount_paise.to_string()
    }

    /// Currency code (e.g. INR).
    async fn currency(&self) -> &str {
        &self.currency
    }

    /// Human-readable string (e.g. \"₹123.45\").
    async fn formatted(&self) -> &str {
        &self.formatted
    }
}

/// Build Money from paise. Currency defaults to INR.
pub fn money_from_paise(amount_paise: i64, currency: Option<&str>) -> Money {
    let currency = currency.unwrap_or(DEFAULT_CURRENCY).to_string();
    let formatted = format_paise_inr(amount_paise);
    Money {
        amount_paise,
        currency,
        formatted,
    }
}

/// Build Money from a string that represents major units (e.g. "99.99" rupees).
/// Used when the backend sends total_amount as a decimal string.
pub fn money_from_major_string(s: &str) -> Money {
    let amount_paise = s
        .trim()
        .parse::<f64>()
        .map(|major| (major * 100.0).round() as i64)
        .unwrap_or(0);
    money_from_paise(amount_paise, Some(DEFAULT_CURRENCY))
}

/// Format paise as INR display string (e.g. "₹123.45").
fn format_paise_inr(paise: i64) -> String {
    let sign = if paise < 0 { "-" } else { "" };
    let abs = paise.unsigned_abs();
    let major = abs / 100;
    let minor = abs % 100;
    format!("{}₹{}.{:02}", sign, major, minor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn money_from_paise_formats_correctly() {
        let m = money_from_paise(12345, None);
        assert_eq!(m.amount_paise, 12345);
        assert_eq!(m.currency, "INR");
        assert_eq!(m.formatted, "₹123.45");
    }

    #[test]
    fn money_from_major_string_rounds_to_paise() {
        let m = money_from_major_string("99.99");
        assert_eq!(m.amount_paise, 9999);
        assert_eq!(m.formatted, "₹99.99");
    }

    #[test]
    fn format_negative_paise() {
        let m = money_from_paise(-100, None);
        assert_eq!(m.formatted, "-₹1.00");
    }
}
