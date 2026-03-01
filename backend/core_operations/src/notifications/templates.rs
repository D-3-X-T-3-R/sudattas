//! P1 Template versioning and rendering for transactional notifications (emails/SMS).

/// Current template version; bump when breaking template changes are made.
pub const TEMPLATE_VERSION: u32 = 1;

/// Render a simple template: replace {{key}} with value from map. Keys are case-sensitive.
pub fn render(template: &str, vars: &[(impl AsRef<str>, impl AsRef<str>)]) -> String {
    let mut out = template.to_string();
    for (k, v) in vars {
        let needle = format!("{{{{{}}}}}", k.as_ref());
        out = out.replace(&needle, v.as_ref());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn template_version_is_positive() {
        assert!(TEMPLATE_VERSION >= 1);
    }

    #[test]
    fn render_substitutes_single_placeholder() {
        let s = render("Order {{order_id}} placed.", &[("order_id", "42")]);
        assert_eq!(s, "Order 42 placed.");
    }

    #[test]
    fn render_substitutes_multiple_placeholders() {
        let s = render(
            "Hi {{name}}, order {{order_id}} total {{amount}}.",
            &[("name", "Alice"), ("order_id", "100"), ("amount", "₹99.00")],
        );
        assert_eq!(s, "Hi Alice, order 100 total ₹99.00.");
    }

    #[test]
    fn render_replaces_all_occurrences_of_same_key() {
        let s = render("{{x}} and {{x}}", &[("x", "same")]);
        assert_eq!(s, "same and same");
    }

    #[test]
    fn render_missing_key_leaves_placeholder() {
        let s = render("Order {{order_id}}.", &[("other", "1")]);
        assert_eq!(s, "Order {{order_id}}.");
    }

    #[test]
    fn render_empty_vars_returns_unchanged() {
        let t = "Hello {{name}}";
        let empty: &[(&str, &str)] = &[];
        assert_eq!(render(t, empty), t);
    }
}
