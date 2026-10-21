//! Port of `modules/util/error.go` and `sanitize.go`'s credential scrubbing.
//!
//! Go's `errorForUser` carries an HTTP status implicitly through the error
//! value. In Rust the status is carried explicitly in [`ErrorKind`], and
//! [`UserFacingError`] keeps upstream's contract that the *message* is safe to
//! show to a user while the wrapped source error is not.

use std::fmt;

/// The `errorForUser` sentinels from `modules/util/error.go`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    InvalidArgument,
    PermissionDenied,
    NotExist,
    AlreadyExist,
    ContentTooLarge,
    UnprocessableContent,
}

impl ErrorKind {
    /// The HTTP status code upstream attaches to each sentinel.
    pub fn status_code(self) -> u16 {
        match self {
            ErrorKind::InvalidArgument => 400,
            ErrorKind::PermissionDenied => 403,
            ErrorKind::NotExist => 404,
            ErrorKind::AlreadyExist => 409,
            ErrorKind::ContentTooLarge => 413,
            ErrorKind::UnprocessableContent => 422,
        }
    }

    /// The message upstream attaches to each sentinel.
    pub fn default_message(self) -> &'static str {
        match self {
            ErrorKind::InvalidArgument => "invalid argument",
            ErrorKind::PermissionDenied => "permission denied",
            ErrorKind::NotExist => "resource does not exist",
            ErrorKind::AlreadyExist => "resource already exists",
            ErrorKind::ContentTooLarge => "content exceeds limit",
            ErrorKind::UnprocessableContent => "unprocessable content",
        }
    }
}

/// An error whose message is safe to return to an end user.
///
/// The `source` is deliberately excluded from `Display`, mirroring
/// `errorWrapper` in the Go code where the wrapped error never contributes to
/// the message.
#[derive(Debug)]
pub struct UserFacingError {
    kind: ErrorKind,
    message: String,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl UserFacingError {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            source: None,
        }
    }

    /// `util.ErrorWrap` — attaches a user-safe message to a wrapped error.
    pub fn wrap(
        kind: ErrorKind,
        source: impl std::error::Error + Send + Sync + 'static,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// The message with any URL credentials scrubbed.
    pub fn safe_message(&self) -> String {
        sanitize_credential_urls(&self.message)
    }
}

impl fmt::Display for UserFacingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.safe_message())
    }
}

impl std::error::Error for UserFacingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}

/// `util.SanitizeCredentialURLs` — masks credentials embedded in URLs.
///
/// `"https://user:pass@host"` becomes `"https://(masked)@host"`. A bare
/// `"git@host"` is left alone: upstream only rewrites a userinfo part that
/// contains a colon, or that follows a `://` scheme separator.
pub fn sanitize_credential_urls(s: &str) -> String {
    const USER_INFO_PLACEHOLDER: &str = "(masked)";
    const SCHEME_SEP: &str = "://";

    let bytes = s.as_bytes();
    let Some(colon) = s.find(':') else {
        return s.to_string();
    };
    let Some(rel_at) = s[colon + 1..].find('@') else {
        return s.to_string();
    };
    let mut sep_at = colon + 1 + rel_at;

    let mut out = String::with_capacity(s.len() + USER_INFO_PLACEHOLDER.len());
    let mut rest = bytes;

    loop {
        // Walk left while the bytes are valid RFC 3986 userinfo characters.
        let mut left = sep_at;
        while left > 0 {
            let c = rest[left - 1];
            let valid = c.is_ascii_alphanumeric()
                || matches!(
                    c,
                    b'-' | b'.'
                        | b'_'
                        | b'~'
                        | b'!'
                        | b'$'
                        | b'&'
                        | b'\''
                        | b'('
                        | b')'
                        | b'*'
                        | b'+'
                        | b','
                        | b';'
                        | b'='
                        | b':'
                        | b'%'
                );
            if !valid {
                break;
            }
            left -= 1;
        }

        // Walk right while the bytes are valid host characters.
        let mut right = sep_at + 1;
        while right < rest.len() {
            let c = rest[right];
            let ok = match c {
                b'.' | b'-' => true,
                b'[' => right == sep_at + 1,
                b']' => {
                    right += 1;
                    break;
                }
                _ => {
                    let ipv6 = rest.get(sep_at + 1) == Some(&b'[');
                    if ipv6 {
                        c.is_ascii_hexdigit() || c == b':'
                    } else {
                        c.is_ascii_alphanumeric()
                    }
                }
            };
            if !ok {
                break;
            }
            right += 1;
        }

        let leading = &rest[..left];
        let left_part = &rest[left..sep_at];
        let right_part = &rest[sep_at + 1..right];

        // Rewrite only when this really looks like credentials: either a colon
        // inside the userinfo, or a `://` immediately before it.
        let mut need = (left_part.contains(&b':') || leading.ends_with(SCHEME_SEP.as_bytes()))
            && !left_part.is_empty()
            && !right_part.is_empty();
        if need && right_part.first() == Some(&b'[') {
            need = right_part.last() == Some(&b']');
        }

        if need {
            out.push_str(&String::from_utf8_lossy(leading));
            out.push_str(USER_INFO_PLACEHOLDER);
            out.push('@');
            out.push_str(&String::from_utf8_lossy(right_part));
        } else {
            out.push_str(&String::from_utf8_lossy(&rest[..right]));
        }

        rest = &rest[right..];
        let Some(next_at) = rest.iter().position(|&c| c == b'@') else {
            break;
        };
        sep_at = next_at;
    }

    out.push_str(&String::from_utf8_lossy(rest));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_codes_match_upstream_sentinels() {
        assert_eq!(ErrorKind::InvalidArgument.status_code(), 400);
        assert_eq!(ErrorKind::PermissionDenied.status_code(), 403);
        assert_eq!(ErrorKind::NotExist.status_code(), 404);
        assert_eq!(ErrorKind::AlreadyExist.status_code(), 409);
        assert_eq!(ErrorKind::ContentTooLarge.status_code(), 413);
        assert_eq!(ErrorKind::UnprocessableContent.status_code(), 422);
    }

    #[test]
    fn default_messages_are_stable() {
        assert_eq!(
            ErrorKind::NotExist.default_message(),
            "resource does not exist"
        );
        assert_eq!(
            ErrorKind::AlreadyExist.default_message(),
            "resource already exists"
        );
    }

    #[test]
    fn masks_http_url_credentials() {
        assert_eq!(
            sanitize_credential_urls("https://user:pass@domain.com/x"),
            "https://(masked)@domain.com/x"
        );
    }

    #[test]
    fn masks_bare_userinfo_with_colon() {
        assert_eq!(
            sanitize_credential_urls("user:pass@domain.com"),
            "(masked)@domain.com"
        );
    }

    #[test]
    fn leaves_plain_git_user_at_host_alone() {
        // No colon in the userinfo and no `://` before it.
        assert_eq!(sanitize_credential_urls("git@domain.com"), "git@domain.com");
    }

    #[test]
    fn leaves_strings_without_url_credentials_alone() {
        assert_eq!(sanitize_credential_urls("no colon here"), "no colon here");
        assert_eq!(sanitize_credential_urls("a:b no at sign"), "a:b no at sign");
    }

    #[test]
    fn source_error_is_not_exposed_in_display() {
        let err = UserFacingError::wrap(
            ErrorKind::NotExist,
            std::io::Error::new(std::io::ErrorKind::NotFound, "secret internal path"),
            "resource does not exist",
        );
        assert_eq!(err.to_string(), "resource does not exist");
        assert!(!err.to_string().contains("secret"));
        assert!(std::error::Error::source(&err).is_some());
    }
}
