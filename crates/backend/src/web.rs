//! Non-`/api/v1` HTTP endpoints, plus the few `/api/v1` paths this port serves
//! beyond Gitea's OpenAPI document.
//!
//! Gitea exposes a handful of routes outside the versioned API: the health
//! probe, the version endpoint clients use for capability detection, and the
//! `/-/` helpers. They are listed here rather than folded into the generated
//! API router, which is reserved for the documented surface.
//!
//! Two `/api/v1` paths are deliberate extensions: `POST /api/v1/signin` issues a
//! token for a login (Gitea's own API has no such endpoint — clients send Basic
//! auth) and `GET /api/v1/admin/stats` backs the dashboard counters. Both are
//! listed in `docs/API-EXTENSIONS.md` so the deviation from the specification is
//! explicit rather than accidental.

use axum::routing::{any, get};
use axum::Json;
use axum::Router;

use crate::api::dispatch::dispatch;
use crate::state::AppState;

/// Routes served outside `/api/v1`, plus the documented extensions.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/healthz", get(healthz))
        .route("/-/version", get(version))
        .route("/-/liveness", get(healthz))
        .route("/-/readiness", get(healthz))
        .route("/api/v1/signin", any(dispatch))
        .route("/api/v1/admin/stats", any(dispatch))
}

/// Liveness probe.
async fn healthz() -> &'static str {
    "OK"
}

/// Version document, mirroring the shape `/api/v1/version` returns.
async fn version() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "version": "1.23.0" }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower::ServiceExt;

    #[tokio::test]
    async fn healthz_answers_ok() {
        let response = router()
            .with_state(AppState::default())
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/healthz")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }

    #[tokio::test]
    async fn version_reports_a_compatibility_string() {
        let response = router()
            .with_state(AppState::default())
            .oneshot(
                axum::http::Request::builder()
                    .uri("/-/version")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        assert!(String::from_utf8_lossy(&body).contains("1.23.0"));
    }
}
