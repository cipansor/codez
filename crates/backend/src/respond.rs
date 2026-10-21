//! Response helpers shared by every handler.

use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

use crate::pagination::Pagination;

/// An API error rendered the way Gitea renders one.
#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
}

impl ApiError {
    /// `404 Not Found`.
    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.into(),
        }
    }

    /// `400 Bad Request`.
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    /// `401 Unauthorized`.
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: message.into(),
        }
    }

    /// `403 Forbidden`.
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            message: message.into(),
        }
    }

    /// `409 Conflict`.
    pub fn conflict(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            message: message.into(),
        }
    }

    /// `422 Unprocessable Entity`.
    pub fn unprocessable(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            message: message.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = serde_json::json!({
            "message": self.message,
            "url": "https://docs.gitea.com/api/1.20/#error",
        });
        (self.status, Json(body)).into_response()
    }
}

/// Convenience alias for handler results.
pub type ApiResult<T> = Result<T, ApiError>;

/// Serialise a value and attach Gitea's pagination headers.
pub fn page<T: Serialize>(items: Vec<T>, pagination: &Pagination, total: usize) -> Response {
    let mut response = Json(items).into_response();
    attach(response.headers_mut(), pagination, total);
    response
}

/// Attach Gitea's pagination headers to an arbitrary JSON body.
///
/// Some list endpoints wrap their results in an envelope (`SearchResults`,
/// `TopicListResponse`) instead of returning a bare array, so the body is
/// built by the caller and only the headers are shared.
pub fn paged<T: Serialize>(body: T, pagination: &Pagination, total: usize) -> Response {
    let mut response = Json(body).into_response();
    attach(response.headers_mut(), pagination, total);
    response
}

/// Write the pagination headers onto a response.
fn attach(headers: &mut axum::http::HeaderMap, pagination: &Pagination, total: usize) {
    for (name, value) in pagination.headers(total) {
        if let Ok(value) = HeaderValue::from_str(&value) {
            headers.insert(name, value);
        }
    }
}

/// `200 OK` with a JSON body.
pub fn ok<T: Serialize>(body: T) -> Response {
    (StatusCode::OK, Json(body)).into_response()
}

/// `201 Created` with a JSON body.
pub fn created<T: Serialize>(body: T) -> Response {
    (StatusCode::CREATED, Json(body)).into_response()
}

/// `202 Accepted` with a JSON body, for work that is queued rather than done.
pub fn accepted<T: Serialize>(body: T) -> Response {
    (StatusCode::ACCEPTED, Json(body)).into_response()
}

/// `204 No Content`.
pub fn no_content() -> Response {
    StatusCode::NO_CONTENT.into_response()
}

/// Serve a JSON document verbatim, used by endpoints that pass through
/// specification fragments.
pub fn raw_json(body: serde_json::Value) -> Response {
    let mut response = Json(body).into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_statuses_are_distinct() {
        assert_eq!(ApiError::not_found("x").status, StatusCode::NOT_FOUND);
        assert_eq!(ApiError::bad_request("x").status, StatusCode::BAD_REQUEST);
        assert_eq!(ApiError::conflict("x").status, StatusCode::CONFLICT);
        assert_eq!(
            ApiError::unprocessable("x").status,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }

    #[test]
    fn error_body_matches_gitea_shape() {
        let response = ApiError::not_found("repo not found").into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            response
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok()),
            Some("application/json")
        );
    }

    #[test]
    fn pagination_headers_are_attached() {
        let pagination = Pagination {
            page: Some(1),
            limit: Some(2),
        };
        let response = page(vec![1, 2], &pagination, 5);
        assert_eq!(response.headers().get("X-Total-Count").unwrap(), "5");
        assert_eq!(response.headers().get("X-HasMore").unwrap(), "true");
    }
}
