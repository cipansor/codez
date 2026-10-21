//! Port of `gitea.dev/modules/uri` — open file:// and http(s):// URIs.
//!
//! | upstream | here |
//! | --- | --- |
//! | `uri.go` | this module |

use std::fmt;
use std::io::Read;

/// `uri.ErrURISchemeNotSupported`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrURISchemeNotSupported {
    pub scheme: String,
}

impl fmt::Display for ErrURISchemeNotSupported {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unsupported scheme: {}", self.scheme)
    }
}

impl std::error::Error for ErrURISchemeNotSupported {}

/// `uri.Open` — opens a local file or a remote resource with the default client.
pub fn open(uri_str: &str) -> Result<Box<dyn Read + Send>, UriError> {
    open_with_client(uri_str, None)
}

/// `uri.OpenWithClient` — opens a local file or a remote resource.
///
/// `client` is used only for `http`/`https`; pass `None` for the shared default
/// client. Callers that must confine remote access should pass a client whose
/// transport validates the peer at dial time, since a redirect can otherwise
/// reach an internal address.
pub fn open_with_client(
    uri_str: &str,
    client: Option<&reqwest::blocking::Client>,
) -> Result<Box<dyn Read + Send>, UriError> {
    let url = url::Url::parse(uri_str).map_err(|e| UriError::Parse(e.to_string()))?;

    match url.scheme().to_ascii_lowercase().as_str() {
        "http" | "https" => {
            let default;
            let client = match client {
                Some(c) => c,
                None => {
                    default = default_client();
                    default
                }
            };
            let resp = client
                .get(uri_str)
                .send()
                .map_err(|e| UriError::Http(e.to_string()))?;
            Ok(Box::new(resp))
        }
        "file" => {
            let path = url.to_file_path().map_err(|_| {
                UriError::UnsupportedScheme(ErrURISchemeNotSupported {
                    scheme: "file".to_string(),
                })
            })?;
            let f = std::fs::File::open(path).map_err(UriError::Io)?;
            Ok(Box::new(f))
        }
        other => Err(UriError::UnsupportedScheme(ErrURISchemeNotSupported {
            scheme: other.to_string(),
        })),
    }
}

/// The shared default HTTP client, standing in for `http.DefaultClient`.
fn default_client() -> &'static reqwest::blocking::Client {
    static CLIENT: std::sync::OnceLock<reqwest::blocking::Client> = std::sync::OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::blocking::Client::builder()
            .build()
            .expect("default HTTP client must build")
    })
}

/// Errors from [`open`].
#[derive(Debug)]
pub enum UriError {
    /// The URL could not be parsed.
    Parse(String),
    /// The HTTP request failed.
    Http(String),
    /// The scheme has no handler.
    UnsupportedScheme(ErrURISchemeNotSupported),
    /// Opening the local file failed.
    Io(std::io::Error),
}

impl fmt::Display for UriError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UriError::Parse(e) => write!(f, "invalid URI: {e}"),
            UriError::Http(e) => write!(f, "http request failed: {e}"),
            UriError::UnsupportedScheme(e) => e.fmt(f),
            UriError::Io(e) => write!(f, "io error: {e}"),
        }
    }
}

impl std::error::Error for UriError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Transcribed from the scheme-switch cases in `TestOpen` in
    /// modules/uri/uri_test.go.
    #[test]
    fn a_file_uri_reads_the_local_file() {
        let path = std::env::temp_dir().join(format!("codez-uri-{}", std::process::id()));
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(b"uri content").unwrap();
        drop(f);

        let uri = format!("file://{}", path.display());
        let mut r = open(&uri).unwrap();
        let mut got = String::new();
        r.read_to_string(&mut got).unwrap();
        assert_eq!(got, "uri content");

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn an_unknown_scheme_is_rejected() {
        let Err(err) = open("ftp://example.com/x") else {
            panic!("an unknown scheme must be rejected");
        };
        match err {
            UriError::UnsupportedScheme(e) => {
                assert_eq!(e.scheme, "ftp");
                assert_eq!(e.to_string(), "Unsupported scheme: ftp");
            }
            other => panic!("expected an unsupported-scheme error, got {other:?}"),
        }
    }

    #[test]
    fn a_missing_local_file_is_an_io_error() {
        let uri = "file:///nonexistent/codez/definitely-missing";
        assert!(matches!(open(uri), Err(UriError::Io(_))));
    }
}
