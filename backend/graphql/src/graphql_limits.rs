//! Phase 8: Query depth and complexity limits for GraphQL.
//!
//! Runs before execution to reject overly deep or complex queries with clear error responses.

/// Default maximum allowed query depth (configurable via env).
pub const DEFAULT_MAX_QUERY_DEPTH: u32 = 10;

/// Maximum allowed page size for list fields (limit/offset pagination).
pub const MAX_PAGE_SIZE: i64 = 50;

/// Computes the maximum nesting depth of a GraphQL query (selection sets).
/// Ignores depth inside string literals. Returns the maximum depth reached (root = 1).
pub fn compute_query_depth(query: &str) -> u32 {
    let mut depth = 0u32;
    let mut max_depth = 0u32;
    let mut in_string = false;
    let mut escape = false;
    let mut quote_char = '\0';
    let mut chars = query.chars().peekable();

    while let Some(c) = chars.next() {
        if escape {
            escape = false;
            continue;
        }
        if in_string {
            if c == '\\' {
                escape = true;
            } else if c == quote_char {
                in_string = false;
            }
            continue;
        }
        match c {
            '"' | '\'' => {
                in_string = true;
                quote_char = c;
            }
            '#' => {
                for next in chars.by_ref() {
                    if next == '\n' {
                        break;
                    }
                }
            }
            '{' => {
                depth += 1;
                if depth > max_depth {
                    max_depth = depth;
                }
            }
            '}' => {
                depth = depth.saturating_sub(1);
            }
            _ => {}
        }
    }
    max_depth
}

/// Checks that the query depth does not exceed `max_depth`.
/// Returns `Ok(())` if allowed, or an error message string.
pub fn check_query_depth(query: &str, max_depth: u32) -> Result<(), String> {
    let depth = compute_query_depth(query);
    if depth > max_depth {
        return Err(format!(
            "Query depth limit exceeded: depth {} exceeds maximum {}",
            depth, max_depth
        ));
    }
    Ok(())
}

/// Caps a requested limit to MAX_PAGE_SIZE. Use for list/offset pagination.
#[inline]
pub fn cap_page_size(limit: Option<i64>) -> Option<i64> {
    limit.map(|n| n.clamp(1, MAX_PAGE_SIZE))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn depth_simple_query() {
        assert_eq!(compute_query_depth("{ apiVersion }"), 1);
    }

    #[test]
    fn depth_nested() {
        assert_eq!(compute_query_depth("{ a { b { c } } }"), 3);
    }

    #[test]
    fn depth_ignores_strings() {
        assert_eq!(compute_query_depth(r#"{ a(b: " { { { ") { c } }"#), 2);
    }

    #[test]
    fn check_depth_ok() {
        assert!(check_query_depth("{ a { b } }", 3).is_ok());
    }

    #[test]
    fn check_depth_exceeded() {
        let err = check_query_depth("{ a { b { c { d } } } }", 2).unwrap_err();
        assert!(err.contains("exceeds maximum"));
    }

    #[test]
    fn cap_page_size_clamps() {
        assert_eq!(cap_page_size(Some(100)), Some(50));
        assert_eq!(cap_page_size(Some(10)), Some(10));
        assert_eq!(cap_page_size(None), None);
    }
}
