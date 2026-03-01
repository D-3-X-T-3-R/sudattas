//! P2 CSRF: parse Origin from Referer header for origin check.

/// Parse origin (scheme + host) from a Referer URL for CSRF checks.
pub fn parse_origin_from_referer(referer: &str) -> Option<String> {
    let referer = referer.trim();
    let scheme_end = referer.find("://")?;
    let after = &referer[scheme_end + 3..];
    let host_end = after.find('/').unwrap_or(after.len());
    Some(format!(
        "{}://{}",
        &referer[..scheme_end],
        &after[..host_end]
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_origin_from_referer_returns_scheme_and_host() {
        assert_eq!(
            parse_origin_from_referer("https://app.example.com/path?q=1"),
            Some("https://app.example.com".to_string())
        );
        assert_eq!(
            parse_origin_from_referer("https://app.example.com/"),
            Some("https://app.example.com".to_string())
        );
        assert_eq!(
            parse_origin_from_referer("https://app.example.com"),
            Some("https://app.example.com".to_string())
        );
    }

    #[test]
    fn parse_origin_from_referer_invalid_returns_none() {
        assert_eq!(parse_origin_from_referer("not-a-url"), None);
        assert_eq!(parse_origin_from_referer(""), None);
    }
}
