//! Typed HTTP helpers for talking to the Codez API.
//!
//! Every page fetches through these helpers instead of hand-rolling
//! `Request::get(..).send().await.unwrap().json()` chains. Read helpers are
//! infallible from the caller's point of view: a transport or decode failure
//! yields `T::default()` (or `None` / a supplied fallback), which keeps Leptos
//! resources simple and keeps the UI rendering an empty state rather than
//! panicking. Write helpers return `bool` — `true` only when the request was
//! sent *and* the server replied `2xx` — so callers can clear a form, navigate
//! or refresh on success and surface an error on failure instead of discarding
//! a rejected write.
//!
//! Paths passed to these helpers are absolute API paths (`/api/v1/repos/owner/name`),
//! not API-relative ones; the helpers do not rewrite the URL.

use gloo_net::http::Request;
use leptos::prelude::LocalResource;
use serde::{de::DeserializeOwned, Serialize};
use std::future::Future;

/// Reactive async resource for client-side rendering.
///
/// `LocalResource` never has to cross a thread boundary, so unlike `Resource`
/// it accepts a non-`Send` future, which is what `gloo_net` request futures
/// are. `source` is tracked reactively; when it yields a new value the
/// `fetcher` runs again.
pub fn local_resource<S, T, Fut>(
    source: impl Fn() -> S + 'static,
    fetcher: impl Fn(S) -> Fut + 'static,
) -> LocalResource<T>
where
    S: PartialEq + 'static,
    T: 'static,
    Fut: Future<Output = T> + 'static,
{
    LocalResource::new(move || fetcher(source()))
}

/// Percent-encode a string for use as a query-parameter value (RFC 3986
/// unreserved set). Keeps spaces, `&`, `#`, and other reserved characters from
/// breaking out of the query string.
pub fn encode_query(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

/// `GET url`, decoding `T`. Falls back to `T::default()` on error.
pub async fn get<T: DeserializeOwned + Default>(url: &str) -> T {
    match Request::get(url).send().await {
        Ok(resp) => resp.json::<T>().await.unwrap_or_default(),
        Err(_) => T::default(),
    }
}

/// `GET url`, decoding `T`. Falls back to `fallback` on error.
pub async fn get_or<T: DeserializeOwned>(url: &str, fallback: T) -> T {
    match Request::get(url).send().await {
        Ok(resp) => resp.json::<T>().await.unwrap_or(fallback),
        Err(_) => fallback,
    }
}

/// `GET url`, invoking `on_ok` with the decoded value when the request succeeds
/// and the body decodes. Used by screens that populate signals instead of
/// returning a value (e.g. form pre-fill).
pub async fn get_then<T, F>(url: &str, on_ok: F)
where
    T: DeserializeOwned,
    F: FnOnce(T),
{
    if let Ok(resp) = Request::get(url).send().await {
        if let Ok(value) = resp.json::<T>().await {
            on_ok(value);
        }
    }
}

/// `GET url`, reporting whether the server replied `2xx`.
///
/// Gitea models several checks as a bare status: a starred repository answers
/// `204`, an unstarred one `404`. The status *is* the payload.
pub async fn probe(url: &str) -> bool {
    matches!(Request::get(url).send().await, Ok(resp) if resp.ok())
}

/// Send a prepared write request and report whether the server accepted it.
///
/// Takes the *result* of building the request so a body that fails to serialize
/// is treated like any other failure. `true` only when the request was actually
/// sent *and* the response status is `2xx`; a transport error or a non-2xx
/// status both yield `false`, so callers never mistake a rejected write for a
/// successful one.
async fn send_ok(req: Result<Request, gloo_net::Error>) -> bool {
    match req {
        Ok(req) => matches!(req.send().await, Ok(resp) if resp.ok()),
        Err(_) => false,
    }
}

/// `PATCH url` with no body. Returns whether the server replied `2xx`.
pub async fn patch(url: &str) -> bool {
    send_ok(Request::patch(url).build()).await
}

/// `PUT url` with no body. Returns whether the server replied `2xx`.
pub async fn put(url: &str) -> bool {
    send_ok(Request::put(url).build()).await
}

/// `DELETE url`. Returns whether the server replied `2xx`.
pub async fn delete(url: &str) -> bool {
    send_ok(Request::delete(url).build()).await
}

/// `POST url` with a JSON body. Returns whether the server replied `2xx`.
///
/// A body that cannot be serialized is a failure (`false`), not a silent no-op.
pub async fn post_json<B: Serialize + ?Sized>(url: &str, body: &B) -> bool {
    send_ok(Request::post(url).json(body)).await
}

/// `PATCH url` with a JSON body. Returns whether the server replied `2xx`.
pub async fn patch_json<B: Serialize + ?Sized>(url: &str, body: &B) -> bool {
    send_ok(Request::patch(url).json(body)).await
}

/// `PUT url` with a JSON body. Returns whether the server replied `2xx`.
pub async fn put_json<B: Serialize + ?Sized>(url: &str, body: &B) -> bool {
    send_ok(Request::put(url).json(body)).await
}

/// `POST url` with a JSON body, decoding the response into `T`. `None` when the
/// request fails *or* the server replies with a non-2xx status — a rejected
/// write must not be mistaken for success, since error responses carry a
/// placeholder body that decodes fine.
pub async fn post_json_resp<B, T>(url: &str, body: &B) -> Option<T>
where
    B: Serialize + ?Sized,
    T: DeserializeOwned,
{
    let req = Request::post(url).json(body).ok()?;
    let resp = req.send().await.ok()?;
    if !resp.ok() {
        return None;
    }
    resp.json::<T>().await.ok()
}

/// Decode standard base64 into text. The contents endpoints return file bodies
/// this way; invalid base64 or non-UTF-8 bytes come back empty rather than
/// rendering replacement characters.
pub fn decode_base64(input: &str) -> String {
    let cleaned: Vec<u8> = input.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    let mut out = Vec::with_capacity(cleaned.len() * 3 / 4);
    let value = |b: u8| -> Option<u8> {
        match b {
            b'A'..=b'Z' => Some(b - b'A'),
            b'a'..=b'z' => Some(b - b'a' + 26),
            b'0'..=b'9' => Some(b - b'0' + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    };
    for chunk in cleaned.chunks(4) {
        let mut buffer = 0u32;
        let mut bits = 0;
        for byte in chunk {
            if *byte == b'=' {
                break;
            }
            let Some(v) = value(*byte) else {
                return String::new();
            };
            buffer = (buffer << 6) | v as u32;
            bits += 6;
        }
        if bits >= 8 {
            let bytes = buffer << (24 - bits);
            out.push((bytes >> 16) as u8);
            if bits >= 16 {
                out.push((bytes >> 8) as u8);
            }
            if bits >= 24 {
                out.push(bytes as u8);
            }
        }
    }
    String::from_utf8(out).unwrap_or_default()
}

/// Encode text as standard base64, as the file-write endpoints expect.
pub fn encode_base64(input: &str) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        out.push(TABLE[(n >> 18) as usize & 63] as char);
        out.push(TABLE[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            TABLE[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

/// The message every failed write shows, so error text stays consistent across
/// pages instead of each call site inventing its own wording.
pub const WRITE_ERROR: &str = "Could not save your changes. Please try again.";

/// Run a non-`Send` future on the current thread.
///
/// `gloo_net` request futures borrow thread-local state, so they cannot be
/// handed to a multithreaded runtime; `spawn_local` is the only executor that
/// can drive them, and re-exporting it here keeps `leptos::task` out of every
/// page's import list.
pub fn spawn<F>(future: F)
where
    F: std::future::Future<Output = ()> + 'static,
{
    leptos::task::spawn_local(future);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_round_trips_text() {
        for original in [
            "",
            "f",
            "fo",
            "foo",
            "hello world",
            "# codez\n\nA platform.\n",
        ] {
            assert_eq!(decode_base64(&encode_base64(original)), original);
        }
    }

    #[test]
    fn base64_matches_the_standard_alphabet() {
        assert_eq!(encode_base64("f"), "Zg==");
        assert_eq!(encode_base64("fo"), "Zm8=");
        assert_eq!(encode_base64("foo"), "Zm9v");
    }

    #[test]
    fn base64_decoding_rejects_garbage() {
        assert_eq!(decode_base64("not base64!!"), "");
    }

    #[test]
    fn query_encoding_escapes_reserved_characters() {
        assert_eq!(encode_query("a b&c#d"), "a%20b%26c%23d");
        assert_eq!(encode_query("plain-_.~"), "plain-_.~");
    }
}
