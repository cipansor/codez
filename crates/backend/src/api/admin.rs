//! Administrative endpoints: user and organization management, notices and
//! the instance statistics shown in the admin dashboard.

use axum::http::Method;
use axum::response::{IntoResponse, Response};

use shared::models as dto;

use crate::pagination::Pagination;
use crate::respond::{self, ApiError};
use crate::state::{now, AppState, StoredUser};

use super::ApiRequest;

/// Route a `/api/v1/admin/*` request.
pub fn handle(state: &AppState, req: &ApiRequest) -> Response {
    let segments: Vec<&str> = req.path.trim_matches('/').split('/').collect();
    // ["api", "v1", "admin", ...]
    let rest = &segments[3..];

    match rest {
        ["stats"] => stats(state, req),
        ["cron"] => respond::ok(cron_list()),
        ["cron", task] => match req.method {
            Method::POST => {
                // Running a scheduled task is a no-op in the in-memory port;
                // Gitea answers `204` and we match that.
                let _ = task;
                respond::no_content()
            }
            _ => respond::ok(cron_list()),
        },
        ["emails"] => emails(state, req),
        ["emails", "search"] => emails(state, req),
        ["orgs"] => orgs(state, req),
        ["unadopted"] => unadopted(state, req),
        ["unadopted", owner, repo] => unadopted_action(state, req, owner, repo),
        ["users"] => users(state, req),
        ["users", login] => user_detail(state, req, login),
        // The same keys `GET /users/{login}/keys` serves; reading the store
        // here keeps the two views of one collection consistent.
        ["users", login, "keys"] => respond::ok(super::users::keys_of(state, login, "ssh")),
        ["users", login, "keys", id] => delete_user_key(state, req, login, id),
        ["users", login, "orgs"] => respond::ok(super::users::orgs_of(state, login)),
        ["users", login, "repos"] => respond::ok(repos_of(state, login)),
        ["users", login, "rename"] => rename_user(state, req, login),
        ["users", login, "badges"] => badges(state, req, login),
        ["orgs", org_name] => org_detail(state, req, org_name),
        ["notices"] => notices(state, req),
        ["notices", id] => notice_detail(state, req, id),
        ["hooks"] => super::users::handle_hooks(state, req, "admin"),
        ["hooks", _id] => super::users::handle_hooks(state, req, "admin"),
        ["packages"] => packages(state, req),
        ["actions", ..] => super::actions::handle(state, req),
        _ => unimplemented(req),
    }
}

/// The instance's scheduled tasks, as Gitea reports them.
fn cron_list() -> Vec<dto::Cron> {
    [
        ("cleanup_repo_archives", "0 0 2 * * *"),
        ("delete_old_actions", "0 0 3 * * *"),
        ("repo_health_check", "0 0 4 * * *"),
        ("update_mirrors", "0 */10 * * * *"),
    ]
    .iter()
    .map(|(name, schedule)| dto::Cron {
        name: Some((*name).into()),
        schedule: Some((*schedule).into()),
        next: Some("2026-01-01T00:00:00Z".into()),
        prev: Some("2025-12-31T00:00:00Z".into()),
        exec_times: Some(0),
    })
    .collect()
}

/// `GET /api/v1/admin/emails` and `/emails/search` — every account's address.
fn emails(state: &AppState, req: &ApiRequest) -> Response {
    let term = req.query_param("q").unwrap_or_default().to_lowercase();
    let items: Vec<dto::Email> = state
        .users()
        .iter()
        .filter(|u| term.is_empty() || u.email.to_lowercase().contains(&term))
        .map(|u| dto::Email {
            email: Some(u.email.clone()),
            primary: Some(true),
            verified: Some(true),
            user_id: Some(u.id),
            username: Some(u.login.clone()),
        })
        .collect();
    let pagination = Pagination {
        page: req.query_i64("page"),
        limit: req.query_i64("limit"),
    };
    let total = items.len();
    respond::page(pagination.apply(&items), &pagination, total)
}

/// `GET /api/v1/admin/unadopted` and the adopt/delete pair below it.
fn unadopted(state: &AppState, req: &ApiRequest) -> Response {
    let _ = req;
    let items: Vec<String> = state
        .unadopted()
        .iter()
        .map(|u| format!("{}/{}", u.owner, u.repo))
        .collect();
    respond::ok(items)
}

/// `POST|DELETE /api/v1/admin/unadopted/{owner}/{repo}`.
fn unadopted_action(state: &AppState, req: &ApiRequest, owner: &str, repo: &str) -> Response {
    match req.method {
        Method::POST => {
            if state.repo(owner, repo).is_some() {
                return ApiError::conflict("repository already exists").into_response();
            }
            let id = state.next_id();
            let created = now();
            state.repos_mut().push(crate::state::StoredRepo {
                id,
                owner: owner.into(),
                name: repo.into(),
                default_branch: "main".into(),
                created: created.clone(),
                updated: created,
                ..Default::default()
            });
            state.unadopted_mut().retain(|u| {
                !(u.owner.eq_ignore_ascii_case(owner) && u.repo.eq_ignore_ascii_case(repo))
            });
            respond::ok(serde_json::json!({
                "message": "repository adopted",
                "repo": format!("{owner}/{repo}"),
            }))
        }
        Method::DELETE => {
            state.unadopted_mut().retain(|u| {
                !(u.owner.eq_ignore_ascii_case(owner) && u.repo.eq_ignore_ascii_case(repo))
            });
            respond::no_content()
        }
        _ => unimplemented(req),
    }
}

/// `GET|POST|DELETE /api/v1/admin/users/{username}/badges`.
///
/// Both writes take a `UserBadgeOption` body (`badge_slugs`); `POST` adds the
/// named badges and `DELETE` removes them.
fn badges(state: &AppState, req: &ApiRequest, login: &str) -> Response {
    if state.user(login).is_none() {
        return ApiError::not_found("user not found").into_response();
    }

    let slugs = req.body_str_vec("badge_slugs").unwrap_or_default();

    match req.method {
        Method::GET => {
            let items: Vec<dto::Badge> = state
                .badges()
                .iter()
                .filter(|b| b.user.eq_ignore_ascii_case(login))
                .map(|b| b.to_dto())
                .collect();
            respond::ok(items)
        }
        Method::POST => {
            let mut created = Vec::new();
            for slug in slugs {
                if state
                    .badges()
                    .iter()
                    .any(|b| b.user.eq_ignore_ascii_case(login) && b.slug == slug)
                {
                    continue;
                }
                let badge = crate::state::StoredBadge {
                    id: state.next_id(),
                    user: login.to_string(),
                    description: slug.clone(),
                    image_url: format!("/badges/{slug}.svg"),
                    slug,
                };
                created.push(badge.to_dto());
                state.badges_mut().push(badge);
            }
            respond::ok(created)
        }
        Method::DELETE => {
            state
                .badges_mut()
                .retain(|b| !(b.user.eq_ignore_ascii_case(login) && slugs.contains(&b.slug)));
            respond::no_content()
        }
        _ => unimplemented(req),
    }
}

/// `DELETE /api/v1/admin/users/{username}/keys/{id}`.
fn delete_user_key(state: &AppState, req: &ApiRequest, login: &str, raw_id: &str) -> Response {
    if req.method != Method::DELETE {
        return unimplemented(req);
    }
    let id: i64 = raw_id.parse().unwrap_or_default();
    let before = state.keys().len();
    state
        .keys_mut()
        .retain(|k| !(k.id == id && k.user.eq_ignore_ascii_case(login) && k.kind == "ssh"));
    if state.keys().len() == before {
        return ApiError::not_found("key not found").into_response();
    }
    respond::no_content()
}

/// `GET /api/v1/admin/packages` — packages across every owner.
fn packages(state: &AppState, req: &ApiRequest) -> Response {
    let items: Vec<dto::Package> = state
        .packages()
        .iter()
        .map(|p| p.to_dto(state.user(&p.owner).map(|u| u.to_dto())))
        .collect();
    let pagination = Pagination {
        page: req.query_i64("page"),
        limit: req.query_i64("limit"),
    };
    let total = items.len();
    respond::page(pagination.apply(&items), &pagination, total)
}

/// `GET /api/v1/admin/stats`.
fn stats(state: &AppState, req: &ApiRequest) -> Response {
    let _ = req;
    let repos = state.repos();
    let users = state.users();
    let orgs = state.orgs();
    let issues = state.issues();
    respond::ok(serde_json::json!({
        "user_count": users.len(),
        "repo_count": repos.len(),
        "organization_count": orgs.len(),
        "issue_count": issues.len(),
        "pull_request_count": issues.iter().filter(|i| i.is_pull).count(),
        "counter": {
            "git_hook_sync": 0,
            "mirror_sync": 0,
            "source_download": 0,
            "lfs": 0,
            "git": 0,
        },
    }))
}

/// `GET /api/v1/admin/orgs`.
fn orgs(state: &AppState, req: &ApiRequest) -> Response {
    let _ = req;
    respond::ok(
        state
            .orgs()
            .iter()
            .map(|o| o.to_dto())
            .collect::<Vec<dto::Organization>>(),
    )
}

/// `GET /api/v1/admin/orgs/{org}`.
fn org_detail(state: &AppState, req: &ApiRequest, name: &str) -> Response {
    let _ = req;
    match state.org(name) {
        Some(org) => respond::ok(org.to_dto()),
        None => ApiError::not_found("organization not found").into_response(),
    }
}

/// `GET|POST /api/v1/admin/users`.
fn users(state: &AppState, req: &ApiRequest) -> Response {
    match req.method {
        Method::GET => {
            let items: Vec<dto::User> = state.users().iter().map(|u| u.to_dto()).collect();
            let pagination = Pagination {
                page: req.query_i64("page"),
                limit: req.query_i64("limit"),
            };
            let total = items.len();
            respond::page(pagination.apply(&items), &pagination, total)
        }
        Method::POST => {
            let login = req
                .body_str("username")
                .or_else(|| req.body_str("login_name"))
                .unwrap_or_default();
            if login.trim().is_empty() {
                return ApiError::unprocessable("username is required").into_response();
            }
            if state.user(&login).is_some() {
                return ApiError::conflict("user already exists").into_response();
            }
            let user = StoredUser {
                id: state.next_id(),
                login: login.clone(),
                full_name: req.body_str("full_name").unwrap_or_default(),
                email: req.body_str("email").unwrap_or_default(),
                password: req.body_str("password").unwrap_or_default(),
                active: true,
                is_admin: req.body_bool("must_change_password").unwrap_or(false),
                created: now(),
                ..Default::default()
            };
            let dto = user.to_dto();
            state.users_mut().push(user);
            respond::created(dto)
        }
        _ => unimplemented(req),
    }
}

/// `GET|PATCH|DELETE /api/v1/admin/users/{username}`.
fn user_detail(state: &AppState, req: &ApiRequest, login: &str) -> Response {
    match req.method {
        Method::GET => match state.user(login) {
            Some(user) => respond::ok(user.to_dto()),
            None => ApiError::not_found("user not found").into_response(),
        },
        Method::PATCH => {
            let body = req.json();
            let mut users = state.users_mut();
            match users
                .iter_mut()
                .find(|u| u.login.eq_ignore_ascii_case(login))
            {
                Some(user) => {
                    if let Some(email) = body.get("email").and_then(|v| v.as_str()) {
                        user.email = email.to_string();
                    }
                    if let Some(full_name) = body.get("full_name").and_then(|v| v.as_str()) {
                        user.full_name = full_name.to_string();
                    }
                    if let Some(active) = body.get("active").and_then(|v| v.as_bool()) {
                        user.active = active;
                    }
                    if let Some(admin) = body.get("admin").and_then(|v| v.as_bool()) {
                        user.is_admin = admin;
                    }
                    if let Some(password) = body.get("password").and_then(|v| v.as_str()) {
                        user.password = password.to_string();
                    }
                    respond::ok(user.to_dto())
                }
                None => ApiError::not_found("user not found").into_response(),
            }
        }
        Method::DELETE => {
            let before = state.users().len();
            state
                .users_mut()
                .retain(|u| !u.login.eq_ignore_ascii_case(login));
            if state.users().len() == before {
                return ApiError::not_found("user not found").into_response();
            }
            respond::no_content()
        }
        _ => unimplemented(req),
    }
}

/// `POST /api/v1/admin/users/{username}/rename`.
fn rename_user(state: &AppState, req: &ApiRequest, login: &str) -> Response {
    let Some(new_name) = req.body_str("new_name") else {
        return ApiError::unprocessable("new_name is required").into_response();
    };
    if state.user(&new_name).is_some() {
        return ApiError::conflict("user already exists").into_response();
    }
    let mut users = state.users_mut();
    match users
        .iter_mut()
        .find(|u| u.login.eq_ignore_ascii_case(login))
    {
        Some(user) => {
            user.login = new_name;
            respond::no_content()
        }
        None => ApiError::not_found("user not found").into_response(),
    }
}

/// `GET|POST /api/v1/admin/notices`.
fn notices(state: &AppState, req: &ApiRequest) -> Response {
    match req.method {
        Method::GET => {
            let list = state.counters().get("notices").copied().unwrap_or(0);
            let items: Vec<serde_json::Value> = (1..=list)
                .map(|id| {
                    serde_json::json!({
                        "id": id,
                        "type": "notice",
                        "message": "Welcome to Codez",
                    })
                })
                .collect();
            respond::ok(items)
        }
        Method::POST => {
            let mut counters = state.counters_mut();
            let id = counters.entry("notices".into()).or_insert(0);
            *id += 1;
            respond::created(serde_json::json!({
                "id": *id,
                "type": req.body_str("type").unwrap_or_else(|| "notice".into()),
                "message": req.body_str("message").unwrap_or_default(),
            }))
        }
        _ => unimplemented(req),
    }
}

/// `DELETE /api/v1/admin/notices/{id}`.
fn notice_detail(state: &AppState, req: &ApiRequest, id: &str) -> Response {
    let _ = req;
    let id: i64 = id.parse().unwrap_or_default();
    let mut counters = state.counters_mut();
    let count = counters.entry("notices".into()).or_insert(0);
    if id < 1 || id > *count {
        return ApiError::not_found("notice not found").into_response();
    }
    *count -= 1;
    respond::no_content()
}

/// Repositories owned by a user.
fn repos_of(state: &AppState, login: &str) -> Vec<dto::Repository> {
    state
        .repos()
        .iter()
        .filter(|r| r.owner.eq_ignore_ascii_case(login))
        .map(|r| r.to_dto())
        .collect()
}

/// Documented fallback for operations this module does not implement yet.
fn unimplemented(req: &ApiRequest) -> Response {
    super::dispatch::fallback(req)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Bytes;
    use axum::http::StatusCode;

    fn req(method: Method, path: &str, body: &str) -> ApiRequest {
        let pattern = super::super::dispatch::match_pattern(path);
        let op = pattern
            .as_deref()
            .and_then(|p| super::super::ops::find(method.as_str(), p));
        ApiRequest {
            method,
            uri: path.parse().unwrap(),
            path: path.to_string(),
            query: crate::api::QueryParams::new(),
            body: Bytes::from(body.to_string()),
            op,
        }
    }

    #[test]
    fn stats_count_every_store() {
        let state = AppState::default();
        state.users_mut().push(StoredUser {
            id: 1,
            login: "alice".into(),
            ..Default::default()
        });
        let response = stats(&state, &req(Method::GET, "/api/v1/admin/stats", ""));
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn creates_and_renames_a_user() {
        let state = AppState::default();
        let created = users(
            &state,
            &req(
                Method::POST,
                "/api/v1/admin/users",
                r#"{"username":"bob","email":"bob@example.com"}"#,
            ),
        );
        assert_eq!(created.status(), StatusCode::CREATED);

        let renamed = rename_user(
            &state,
            &req(
                Method::POST,
                "/api/v1/admin/users/bob/rename",
                r#"{"new_name":"robert"}"#,
            ),
            "bob",
        );
        assert_eq!(renamed.status(), StatusCode::NO_CONTENT);
        assert!(state.user("robert").is_some());
        assert!(state.user("bob").is_none());
    }

    #[test]
    fn refuses_to_create_a_duplicate_user() {
        let state = AppState::default();
        state.users_mut().push(StoredUser {
            id: 1,
            login: "alice".into(),
            ..Default::default()
        });
        let response = users(
            &state,
            &req(
                Method::POST,
                "/api/v1/admin/users",
                r#"{"username":"alice"}"#,
            ),
        );
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn notices_are_counted_and_deletable() {
        let state = AppState::default();
        for _ in 0..2 {
            let response = notices(
                &state,
                &req(
                    Method::POST,
                    "/api/v1/admin/notices",
                    r#"{"message":"hello"}"#,
                ),
            );
            assert_eq!(response.status(), StatusCode::CREATED);
        }
        let removed = notice_detail(
            &state,
            &req(Method::DELETE, "/api/v1/admin/notices/1", ""),
            "1",
        );
        assert_eq!(removed.status(), StatusCode::NO_CONTENT);
        assert_eq!(state.counters().get("notices").copied(), Some(1));

        let missing = notice_detail(
            &state,
            &req(Method::DELETE, "/api/v1/admin/notices/99", ""),
            "99",
        );
        assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    }
}
