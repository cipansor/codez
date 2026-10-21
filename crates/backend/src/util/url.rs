//! Port of `modules/util/url.go`.

/// `util.SanitizeURL` — parses a URL and strips any embedded credentials.
pub fn sanitize_url(s: &str) -> Result<String, url::ParseError> {
    let mut parsed = url::Url::parse(s)?;
    if !parsed.username().is_empty() || parsed.password().is_some() {
        let _ = parsed.set_username("");
        let _ = parsed.set_password(None);
    }
    Ok(parsed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_userinfo() {
        assert_eq!(
            sanitize_url("https://user:pass@example.com/x").unwrap(),
            "https://example.com/x"
        );
    }

    #[test]
    fn leaves_credential_free_urls_unchanged() {
        assert_eq!(
            sanitize_url("https://example.com/x").unwrap(),
            "https://example.com/x"
        );
    }

    #[test]
    fn rejects_invalid_urls() {
        assert!(sanitize_url("://nope").is_err());
    }
}
