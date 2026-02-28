//! Phase 8: Strong input validation for GraphQL inputs.
//!
//! Email, phone, address, SKU/slug, quantities and cart size bounds.

use crate::resolvers::error::{Code, GqlError};

/// Max length for SKU/slug identifiers.
pub const MAX_SKU_SLUG_LEN: usize = 128;

/// Max cart item quantity per line.
pub const MAX_QUANTITY_PER_ITEM: i64 = 999;
/// Max total cart size (number of lines).
pub const MAX_CART_ITEMS: usize = 100;
/// Max road/address line length.
pub const MAX_ADDRESS_LINE_LEN: usize = 500;

/// Validates email format (must contain @ and a dot in domain). Returns `Ok(())` or `GqlError::InvalidArgument`.
pub fn validate_email(email: &str) -> Result<(), GqlError> {
    let e = email.trim();
    if e.is_empty() {
        return Err(GqlError::new("Email is required", Code::InvalidArgument));
    }
    if e.len() > 254 {
        return Err(GqlError::new("Email too long", Code::InvalidArgument));
    }
    let at = match e.find('@') {
        Some(i) => i,
        None => return Err(GqlError::new("Invalid email format", Code::InvalidArgument)),
    };
    if at == 0 || at == e.len() - 1 {
        return Err(GqlError::new("Invalid email format", Code::InvalidArgument));
    }
    if !e[at + 1..].contains('.') {
        return Err(GqlError::new("Invalid email format", Code::InvalidArgument));
    }
    Ok(())
}

/// Validates phone format (optional field). Empty is allowed. At least 10 digits.
pub fn validate_phone(phone: Option<&str>) -> Result<(), GqlError> {
    let Some(p) = phone else {
        return Ok(());
    };
    let p = p.trim();
    if p.is_empty() {
        return Ok(());
    }
    let digit_count = p.chars().filter(|c| c.is_ascii_digit()).count();
    if digit_count < 10 {
        return Err(GqlError::new(
            "Phone must contain at least 10 digits",
            Code::InvalidArgument,
        ));
    }
    if !p
        .chars()
        .all(|c| c.is_ascii_digit() || c.is_ascii_whitespace() || "+-()".contains(c))
    {
        return Err(GqlError::new("Invalid phone format", Code::InvalidArgument));
    }
    Ok(())
}

/// Validates quantity: positive and not exceeding MAX_QUANTITY_PER_ITEM.
pub fn validate_quantity(quantity: i64, label: &str) -> Result<(), GqlError> {
    if quantity < 1 {
        return Err(GqlError::new(
            &format!("{} must be at least 1", label),
            Code::InvalidArgument,
        ));
    }
    if quantity > MAX_QUANTITY_PER_ITEM {
        return Err(GqlError::new(
            &format!("{} must not exceed {}", label, MAX_QUANTITY_PER_ITEM),
            Code::InvalidArgument,
        ));
    }
    Ok(())
}

/// Validates SKU/slug: length and allowed charset (alphanumeric, hyphen, underscore).
pub fn validate_sku_slug(s: &str, label: &str) -> Result<(), GqlError> {
    let s = s.trim();
    if s.is_empty() {
        return Err(GqlError::new(
            &format!("{} must not be empty", label),
            Code::InvalidArgument,
        ));
    }
    if s.len() > MAX_SKU_SLUG_LEN {
        return Err(GqlError::new(
            &format!("{} must not exceed {} characters", label, MAX_SKU_SLUG_LEN),
            Code::InvalidArgument,
        ));
    }
    if !s
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(GqlError::new(
            &format!(
                "{} may only contain letters, numbers, hyphen and underscore",
                label
            ),
            Code::InvalidArgument,
        ));
    }
    Ok(())
}

/// Validates address road (required, max length).
pub fn validate_address_road(road: &str) -> Result<(), GqlError> {
    let r = road.trim();
    if r.is_empty() {
        return Err(GqlError::new(
            "Address road is required",
            Code::InvalidArgument,
        ));
    }
    if r.len() > MAX_ADDRESS_LINE_LEN {
        return Err(GqlError::new(
            &format!(
                "Address line must not exceed {} characters",
                MAX_ADDRESS_LINE_LEN
            ),
            Code::InvalidArgument,
        ));
    }
    Ok(())
}

/// Validates cart size (number of items). Use after fetching cart to enforce MAX_CART_ITEMS before add.
pub fn validate_cart_size(current_count: usize) -> Result<(), GqlError> {
    if current_count >= MAX_CART_ITEMS {
        return Err(GqlError::new(
            &format!("Cart cannot exceed {} items", MAX_CART_ITEMS),
            Code::InvalidArgument,
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_valid() {
        assert!(validate_email("a@b.co").is_ok());
        assert!(validate_email("user+tag@example.com").is_ok());
    }

    #[test]
    fn email_invalid() {
        assert!(validate_email("").is_err());
        assert!(validate_email("no-at").is_err());
        assert!(validate_email("@nodomain").is_err());
    }

    #[test]
    fn quantity_bounds() {
        assert!(validate_quantity(1, "quantity").is_ok());
        assert!(validate_quantity(999, "quantity").is_ok());
        assert!(validate_quantity(0, "quantity").is_err());
        assert!(validate_quantity(1000, "quantity").is_err());
    }

    #[test]
    fn sku_slug_valid() {
        assert!(validate_sku_slug("ABC-123", "sku").is_ok());
        assert!(validate_sku_slug("my_slug", "slug").is_ok());
    }

    #[test]
    fn sku_slug_invalid() {
        assert!(validate_sku_slug("", "sku").is_err());
        assert!(validate_sku_slug("has space", "sku").is_err());
    }
}
