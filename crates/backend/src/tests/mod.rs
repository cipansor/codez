//! Integration tests: drive the real router over the seeded in-memory state.

mod contracts;
mod coverage;
mod domains;
mod ported;
mod repos;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use serde_json::Value;
use tower::ServiceExt;

/// The application router with the demo dataset loaded.
///
/// The router is built once per test thread so that state written by one call
/// (a created repository, a posted comment) is visible to the next call in the
/// same test, while each test still starts from a clean demo state. Rebuilding
/// per request would silently discard every write.
pub fn app() -> Router {
    thread_local! {
        static ROUTER: Router = crate::app();
    }
    ROUTER.with(|router| router.clone())
}

/// Send a request and return the status, headers and parsed JSON body.
pub async fn call(
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, axum::http::HeaderMap, Value) {
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(match body {
            Some(value) => Body::from(serde_json::to_vec(&value).unwrap()),
            None => Body::empty(),
        })
        .unwrap();

    let response = app().oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = axum::body::to_bytes(response.into_body(), 8 * 1024 * 1024)
        .await
        .unwrap();
    let json = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes)
            .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned()))
    };
    (status, headers, json)
}

/// A value unique to this call, for tests that assert on something they create.
///
/// The in-memory state is shared across a test binary, so a fixed string can
/// already exist and make an assertion pass for the wrong reason.
pub fn unique() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}

/// Send a `DELETE` carrying a JSON body.
pub async fn delete_with_body(uri: &str, body: Value) -> StatusCode {
    call("DELETE", uri, Some(body)).await.0
}

/// Send a request against a router with pristine seeded state.
///
/// [`call`] shares one router per test thread so writes accumulate, which is
/// what a multi-step test wants. A sweep that visits every operation must not
/// inherit the previous operation's writes: a `DELETE` early in the table would
/// remove an entity that a later operation needs, and the later request would
/// fail with a plain `404` that says nothing about whether it is implemented.
pub async fn call_fresh(
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, axum::http::HeaderMap, Value) {
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(match body {
            Some(value) => Body::from(serde_json::to_vec(&value).unwrap()),
            None => Body::empty(),
        })
        .unwrap();

    let response = crate::app().oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = axum::body::to_bytes(response.into_body(), 8 * 1024 * 1024)
        .await
        .unwrap();
    let json = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes)
            .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned()))
    };
    (status, headers, json)
}

/// `GET` helper.
pub async fn get(uri: &str) -> (StatusCode, Value) {
    let (status, _, body) = call("GET", uri, None).await;
    (status, body)
}

/// `POST` helper.
pub async fn post(uri: &str, body: Value) -> (StatusCode, Value) {
    let (status, _, value) = call("POST", uri, Some(body)).await;
    (status, value)
}

/// `PATCH` helper.
pub async fn patch(uri: &str, body: Value) -> (StatusCode, Value) {
    let (status, _, value) = call("PATCH", uri, Some(body)).await;
    (status, value)
}

/// `PUT` helper.
pub async fn put(uri: &str, body: Value) -> (StatusCode, Value) {
    let (status, _, value) = call("PUT", uri, Some(body)).await;
    (status, value)
}

/// `DELETE` helper.
pub async fn delete(uri: &str) -> StatusCode {
    let (status, _, _) = call("DELETE", uri, None).await;
    status
}
