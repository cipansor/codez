//! Package registry management endpoints.
//!
//! The registries (npm, cargo, maven, and so on) each speak their own wire
//! protocol; this module covers the management API Gitea documents under
//! `/packages/{owner}`. Packages live in the in-memory store, seeded with a
//! couple of demo rows, so the listings and detail endpoints report real state
//! rather than a fixed empty set.

use axum::http::Method;
use axum::response::{IntoResponse, Response};

use crate::respond::{self, ApiError};
use crate::state::{AppState, StoredPackage};

use super::ApiRequest;

/// Route a `/api/v1/packages/*` request.
pub fn handle(state: &AppState, req: &ApiRequest) -> Response {
    let Some(owner) = req.param("owner") else {
        return unimplemented(req);
    };
    if state.user(&owner).is_none() && state.org(&owner).is_none() {
        return ApiError::not_found("owner does not exist").into_response();
    }

    // `POST /packages/{owner}/{type}/{name}/-/link/{repo_name}` and `/-/unlink`
    // attach or detach a package from a repository.
    if req.method == Method::POST && req.path.contains("/-/") {
        return link(state, req, &owner);
    }

    match req.method {
        Method::GET => read(state, req, &owner),
        Method::DELETE => delete(state, req, &owner),
        _ => unimplemented(req),
    }
}

/// `POST /packages/{owner}/{type}/{name}/-/{link/{repo_name},unlink}`.
fn link(state: &AppState, req: &ApiRequest, owner: &str) -> Response {
    let (Some(kind), Some(name)) = (req.param("type"), req.param("name")) else {
        return ApiError::not_found("package does not exist").into_response();
    };
    let unlink = req.path.ends_with("/-/unlink");
    let repo_name = req.param("repo_name");
    let mut packages = state.packages_mut();
    let mut found = false;
    for package in packages
        .iter_mut()
        .filter(|p| p.owner == owner && p.kind == kind && p.name == name)
    {
        found = true;
        if unlink {
            package.repository = None;
        } else {
            package.repository = repo_name.clone();
        }
    }
    drop(packages);
    if !found {
        return ApiError::not_found("package does not exist").into_response();
    }
    respond::no_content()
}

/// The `GET` surface: listings, a single version, its files and `-/latest`.
fn read(state: &AppState, req: &ApiRequest, owner: &str) -> Response {
    let kind = req.param("type");
    let name = req.param("name");
    let version = req.param("version");
    let all = state.owner_packages(owner);
    let owner_dto = || state.user(owner).map(|u| u.to_dto());

    // `GET /packages/{owner}` — every version of every package.
    let (Some(kind), Some(name)) = (kind, name) else {
        return respond::ok(
            all.iter()
                .map(|p| p.to_dto(owner_dto()))
                .collect::<Vec<_>>(),
        );
    };

    let versions = filter(&all, &kind, &name);
    if versions.is_empty() {
        return ApiError::not_found("package does not exist").into_response();
    }

    // `GET /packages/{owner}/{type}/{name}/-/latest`.
    if req.path.ends_with("/-/latest") {
        // `versions` is newest-first, so the head is the latest.
        return respond::ok(versions[0].to_dto(owner_dto()));
    }

    // `GET /packages/{owner}/{type}/{name}` — every version of one package.
    let Some(version) = version else {
        return respond::ok(
            versions
                .iter()
                .map(|p| p.to_dto(owner_dto()))
                .collect::<Vec<_>>(),
        );
    };

    // `GET /packages/{owner}/{type}/{name}/{version}[/files]`.
    let Some(package) = versions.iter().find(|p| p.version == version) else {
        return ApiError::not_found("package does not exist").into_response();
    };
    if req.path.ends_with("/files") {
        respond::ok(package.files.iter().map(|f| f.to_dto()).collect::<Vec<_>>())
    } else {
        respond::ok(package.to_dto(owner_dto()))
    }
}

/// `DELETE /packages/{owner}/{type}/{name}[/{version}]`.
fn delete(state: &AppState, req: &ApiRequest, owner: &str) -> Response {
    let kind = req.param("type");
    let name = req.param("name");
    let version = req.param("version");

    let mut packages = state.packages_mut();
    let before = packages.len();
    packages.retain(|p| {
        // Keep anything that does not match every supplied coordinate.
        p.owner != owner
            || kind.as_ref().is_some_and(|k| &p.kind != k)
            || name.as_ref().is_some_and(|n| &p.name != n)
            || version.as_ref().is_some_and(|v| &p.version != v)
    });
    let removed = before - packages.len();
    drop(packages);

    if removed == 0 {
        ApiError::not_found("package does not exist").into_response()
    } else {
        respond::no_content()
    }
}

/// Versions of one package, newest first.
fn filter(all: &[StoredPackage], kind: &str, name: &str) -> Vec<StoredPackage> {
    all.iter()
        .filter(|p| p.kind == kind && p.name == name)
        .cloned()
        .collect()
}

/// Documented fallback for operations this module does not implement yet.
fn unimplemented(req: &ApiRequest) -> Response {
    super::dispatch::fallback(req)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::QueryParams;
    use crate::state::{StoredPackage, StoredUser};
    use axum::body::Bytes;
    use axum::http::StatusCode;

    fn req(method: Method, path: &str) -> ApiRequest {
        let pattern = super::super::dispatch::match_pattern(path);
        let op = pattern
            .as_deref()
            .and_then(|p| super::super::ops::find(method.as_str(), p));
        ApiRequest {
            method,
            uri: path.parse().unwrap(),
            path: path.to_string(),
            query: QueryParams::new(),
            body: Bytes::new(),
            op,
        }
    }

    fn state() -> AppState {
        let state = AppState::default();
        state.users_mut().push(StoredUser {
            id: 1,
            login: "alice".into(),
            ..Default::default()
        });
        state.packages_mut().push(StoredPackage {
            id: 1,
            owner: "alice".into(),
            kind: "npm".into(),
            name: "demo".into(),
            version: "1.0.0".into(),
            created: "2026-01-01T00:00:00Z".into(),
            ..Default::default()
        });
        state
    }

    #[test]
    fn lists_packages_for_a_known_owner() {
        assert_eq!(
            handle(&state(), &req(Method::GET, "/api/v1/packages/alice")).status(),
            StatusCode::OK
        );
    }

    #[test]
    fn unknown_owner_is_a_404() {
        assert_eq!(
            handle(&state(), &req(Method::GET, "/api/v1/packages/nobody")).status(),
            StatusCode::NOT_FOUND
        );
    }

    #[test]
    fn a_package_version_reports_its_identity() {
        assert_eq!(
            handle(
                &state(),
                &req(Method::GET, "/api/v1/packages/alice/npm/demo/1.0.0"),
            )
            .status(),
            StatusCode::OK
        );
    }

    #[test]
    fn deleting_a_package_is_a_204() {
        assert_eq!(
            handle(
                &state(),
                &req(Method::DELETE, "/api/v1/packages/alice/npm/demo")
            )
            .status(),
            StatusCode::NO_CONTENT
        );
    }
}
