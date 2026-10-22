//! Static file serving for the Leptos bundle.
//!
//! Deep links return `index.html` so client-side routing works, and unknown
//! `/api/*` paths stay `404` rather than being answered with HTML.

use std::path::{Component, Path, PathBuf};

use axum::body::Body;
use axum::http::{header, StatusCode, Uri};
use axum::response::{IntoResponse, Response};

/// Environment variable pointing at the built frontend bundle.
pub const STATIC_DIR_ENV: &str = "CODEZ_STATIC_DIR";

/// Default bundle location, relative to the repository root.
pub const DEFAULT_STATIC_DIR: &str = "crates/frontend/dist";

/// Resolve the configured asset root.
pub fn asset_root() -> PathBuf {
    std::env::var(STATIC_DIR_ENV)
        .unwrap_or_else(|_| DEFAULT_STATIC_DIR.to_string())
        .into()
}

/// Normalise a request path, rejecting anything that escapes the root.
///
/// Normalising `..` is not enough on its own: a symlink inside the bundle can
/// point outside it, so callers must also check the resolved destination.
pub fn safe_relative(uri_path: &str) -> Option<PathBuf> {
    let decoded = percent_decode(uri_path);
    let mut out = PathBuf::new();
    for component in Path::new(decoded.trim_start_matches('/')).components() {
        match component {
            Component::Normal(part) => out.push(part),
            Component::CurDir => {}
            // `..`, `/` and prefixes would escape the root.
            _ => return None,
        }
    }
    Some(out)
}

/// Read a file only when its canonical location stays inside `root`.
pub fn read_within_root(root: &Path, relative: &Path) -> Option<Vec<u8>> {
    let canonical_root = root.canonicalize().ok()?;
    let candidate = canonical_root.join(relative);
    let canonical = candidate.canonicalize().ok()?;
    if !canonical.starts_with(&canonical_root) {
        return None;
    }
    std::fs::read(canonical).ok()
}

/// Fallback handler: serve an asset, else the SPA shell for deep links.
pub async fn serve(uri: Uri) -> Response {
    let path = uri.path();
    if path.starts_with("/api/") {
        return (
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "message": "resource does not exist" })),
        )
            .into_response();
    }

    let root = asset_root();
    if let Some(relative) = safe_relative(path) {
        if !relative.as_os_str().is_empty() {
            if let Some(bytes) = read_within_root(&root, &relative) {
                return asset_response(&relative, bytes);
            }
        }
    }

    match read_within_root(&root, Path::new("index.html")) {
        Some(bytes) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
            bytes,
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
            format!(
                "frontend bundle not found in {}. Build it with `trunk build`.\n",
                root.display()
            ),
        )
            .into_response(),
    }
}

/// Serve a file with a content type derived from its extension.
fn asset_response(path: &Path, bytes: Vec<u8>) -> Response {
    let content_type = match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json") => "application/json",
        Some("wasm") => "application/wasm",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("ico") => "image/x-icon",
        Some("txt") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    };
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, content_type)],
        Body::from(bytes),
    )
        .into_response()
}

/// Decode `%XX` escapes in a request path.
fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or("");
            if let Ok(byte) = u8::from_str_radix(hex, 16) {
                out.push(byte);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_traversal_attempts() {
        assert!(safe_relative("/../../etc/passwd").is_none());
        assert!(safe_relative("/a/../../etc/passwd").is_none());
        assert!(safe_relative("/%2e%2e/secret").is_none());
    }

    #[test]
    fn accepts_ordinary_paths() {
        assert_eq!(
            safe_relative("/assets/app.js"),
            Some(PathBuf::from("assets/app.js"))
        );
        assert_eq!(safe_relative("/"), Some(PathBuf::new()));
    }

    #[test]
    fn reading_outside_the_root_is_refused() {
        let root = std::env::temp_dir().join("codez-static-test");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("index.html"), b"ok").unwrap();

        assert_eq!(
            read_within_root(&root, Path::new("index.html")),
            Some(b"ok".to_vec())
        );
        assert_eq!(read_within_root(&root, Path::new("../etc/passwd")), None);

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn percent_decoding_handles_invalid_escapes() {
        assert_eq!(percent_decode("/a%2Fb"), "/a/b");
        assert_eq!(percent_decode("/a%zzb"), "/a%zzb");
    }
}
