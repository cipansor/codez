//! User, token, key and notification endpoints.
//!
//! `/api/v1/user/*` acts on the authenticated account, `/api/v1/users/*` on
//! anyone. Authentication is deliberately minimal: a `token` query parameter
//! resolves to a stored token, and requests without one act as the demo user.

use axum::http::Method;
use axum::response::{IntoResponse, Response};

use shared::models as dto;

use crate::pagination::Pagination;
use crate::respond::{self, ApiError, ApiResult};
use crate::state::{
    short_sha, AppState, StoredBlock, StoredFollow, StoredHook, StoredKey, StoredOauthApp,
    StoredOrgBlock, StoredProject, StoredProjectColumn, StoredRepo, StoredToken, StoredUser,
};

use super::ApiRequest;

/// The account a request acts as: the token owner, or the demo user.
pub fn current_user(state: &AppState, req: &ApiRequest) -> Option<StoredUser> {
    if let Some(token) = req.token() {
        if let Some(owner) = state
            .tokens()
            .iter()
            .find(|t| t.sha1 == token)
            .map(|t| t.user.clone())
        {
            return state.user(&owner);
        }
    }
    state.users().first().cloned()
}

/// `401` unless a user can be resolved.
pub fn require_user(state: &AppState, req: &ApiRequest) -> ApiResult<StoredUser> {
    current_user(state, req).ok_or_else(|| ApiError::unauthorized("authentication required"))
}

/// Route a `/api/v1/users/*` or `/api/v1/user/*` request.
pub fn handle(state: &AppState, req: &ApiRequest) -> Response {
    let path = req.path.as_str();
    let segments: Vec<&str> = path.trim_matches('/').split('/').collect();
    // ["api", "v1", "users"|"user", ...]
    let rest = &segments[3..];
    let is_self = segments[2] == "user";

    match (is_self, rest) {
        (true, []) => match req.method {
            Method::GET => match require_user(state, req) {
                Ok(user) => respond::ok(with_follow_counts(state, &user)),
                Err(error) => error.into_response(),
            },
            _ => unimplemented(req),
        },
        (true, ["repos"]) => match req.method {
            Method::POST => super::repos::create(state, req),
            _ => respond::page(
                repos_of(state, &current_login(state, req)),
                &pagination(req),
                1,
            ),
        },
        (true, ["starred"]) => match req.method {
            Method::GET => respond::ok(starred_of(state, &current_login(state, req))),
            _ => unimplemented(req),
        },
        (true, ["starred", owner, repo]) => starred(state, req, owner, repo),
        (true, ["subscriptions"]) => {
            respond::ok(subscriptions_of(state, &current_login(state, req)))
        }
        (true, ["followers"]) => respond::ok(followers(state, &current_login(state, req))),
        (true, ["following"]) => respond::ok(following(state, &current_login(state, req))),
        (true, ["following", login]) => follow_edge(state, req, login),
        (true, ["teams"]) => respond::ok(teams_of(state, &current_login(state, req))),
        (true, ["orgs"]) => respond::ok(orgs_of(state, &current_login(state, req))),
        (true, ["tokens"]) => handle_tokens(state, req),
        (true, ["keys"]) => handle_keys(state, req, "ssh"),
        (true, ["keys", _id]) => handle_keys(state, req, "ssh"),
        (true, ["gpg_keys"]) => handle_keys(state, req, "gpg"),
        (true, ["gpg_keys", _id]) => handle_keys(state, req, "gpg"),
        (true, ["emails"]) => handle_emails(state, req),
        (true, ["settings"]) => respond::ok(serde_json::json!({
            "full_name": "",
            "description": "",
            "hide_email": false,
        })),
        (true, ["times"]) => handle_tracked_times(state, req),
        (true, ["stopwatches"]) => handle_stopwatches(state, req),
        (true, ["heatmap"]) => respond::ok(heatmap()),
        (true, ["blocks"]) => respond::ok(blocks_of(state, &current_login(state, req))),
        (true, ["blocks", login]) => block_edge(state, req, login),
        (true, ["hooks"]) => handle_hooks(state, req, "user"),
        (true, ["hooks", _id]) => handle_hooks(state, req, "user"),
        (true, ["projects"]) => handle_projects(state, req, None),
        (true, ["projects", _id]) => handle_projects(state, req, None),
        (true, ["projects", id, "columns"]) => handle_project_columns(state, req, id, None),
        (true, ["projects", id, "columns", "move"]) => handle_columns_move(state, req, id),
        (true, ["projects", id, "columns", column_id]) => {
            handle_project_column(state, req, id, column_id, None)
        }
        (true, ["projects", id, "columns", column_id, "issues"]) => {
            handle_column_issues(state, req, id, column_id)
        }
        (true, ["projects", id, "columns", column_id, "issues", issue_id]) => {
            handle_column_issue(state, req, id, column_id, issue_id)
        }
        (true, ["projects", id, "columns", column_id, "default"]) => {
            handle_column_default(state, req, id, column_id)
        }
        (true, ["projects", id, "issues", issue_id, "move"]) => {
            handle_project_issue_move(state, req, id, issue_id)
        }
        (true, ["actions", ..]) => super::actions::handle(state, req),
        (false, [login, "actions", ..]) => {
            let _ = login;
            super::actions::handle(state, req)
        }
        (true, ["gpg_key_token"]) => respond::ok(short_sha("gpg-token")),
        (true, ["gpg_key_verify"]) => respond::ok(true),
        (true, ["avatar"]) => match req.method {
            Method::POST => respond::no_content(),
            _ => respond::no_content(),
        },
        (true, ["applications", "oauth2"]) => handle_oauth_apps(state, req),
        (true, ["applications", "oauth2", _id]) => handle_oauth_apps(state, req),
        (false, ["search"]) => search_users(state, req),
        (false, [login]) => user_detail(state, req, login),
        (false, [login, "repos"]) => respond::page(repos_of(state, login), &pagination(req), 1),
        (false, [login, "followers"]) => respond::ok(followers(state, login)),
        (false, [login, "following"]) => respond::ok(following(state, login)),
        (false, [login, "following", target]) => {
            if state.follows_user(login, target) {
                respond::no_content()
            } else {
                ApiError::not_found("user is not followed").into_response()
            }
        }
        (false, [login, "orgs"]) => respond::ok(orgs_of(state, login)),
        (false, [login, "orgs", org, "permissions"]) => {
            respond::ok(org_permissions(state, login, org))
        }
        (false, [_login, "heatmap"]) => respond::ok(heatmap()),
        (false, [_login, "activities", "feeds"]) => respond::ok(Vec::<serde_json::Value>::new()),
        (false, [login, "tokens"]) => handle_user_tokens(state, req, login),
        (false, [login, "tokens", token]) => delete_user_token(state, req, login, token),
        (false, [login, "keys"]) => respond::ok(keys_of(state, login, "ssh")),
        (false, [login, "gpg_keys"]) => respond::ok(keys_of(state, login, "gpg")),
        (false, [login, "starred"]) => respond::ok(starred_of(state, login)),
        (false, [login, "subscriptions"]) => respond::ok(subscriptions_of(state, login)),
        (false, [login, "projects"]) => handle_projects(state, req, Some(login)),
        (false, [_login, "block"]) => respond::no_content(),
        _ => unimplemented(req),
    }
}

/// `GET /api/v1/notifications` and its thread sub-resources.
pub fn handle_notification(state: &AppState, req: &ApiRequest) -> Response {
    let path = req.path.as_str();
    let user = current_user(state, req);
    let login = user.map(|u| u.login).unwrap_or_default();

    let mine: Vec<_> = state
        .notifications()
        .iter()
        .filter(|n| n.user.eq_ignore_ascii_case(&login))
        .cloned()
        .collect();

    if path.ends_with("/new") {
        let count = mine.iter().filter(|n| n.unread).count();
        return respond::ok(serde_json::json!({ "new": count }));
    }

    if let Some(thread) = req.param("id").or_else(|| req.param("thread_id")) {
        let id: i64 = thread.parse().unwrap_or_default();
        let found = mine.iter().find(|n| n.id == id);
        return match req.method {
            Method::GET => match found {
                Some(n) => respond::ok(n.to_dto(state.repo_by_id(n.repo_id).as_ref())),
                None => ApiError::not_found("thread not found").into_response(),
            },
            Method::PATCH | Method::PUT => {
                if let Some(n) = state
                    .notifications_mut()
                    .iter_mut()
                    .find(|n| n.id == id && n.user.eq_ignore_ascii_case(&login))
                {
                    n.unread = false;
                }
                respond::no_content()
            }
            _ => unimplemented(req),
        };
    }

    match req.method {
        Method::GET => {
            let total = mine.len();
            let pagination = pagination(req);
            let items: Vec<_> = pagination
                .apply(&mine)
                .into_iter()
                .map(|n| n.to_dto(state.repo_by_id(n.repo_id).as_ref()))
                .collect();
            respond::page(items, &pagination, total)
        }
        Method::PUT => {
            for n in state.notifications_mut().iter_mut() {
                if n.user.eq_ignore_ascii_case(&login) {
                    n.unread = false;
                }
            }
            respond::no_content()
        }
        Method::DELETE => {
            state
                .notifications_mut()
                .retain(|n| !n.user.eq_ignore_ascii_case(&login));
            respond::no_content()
        }
        _ => unimplemented(req),
    }
}

/// `GET /api/v1/users/{username}`.
fn user_detail(state: &AppState, req: &ApiRequest, login: &str) -> Response {
    let _ = req;
    match state.user(login) {
        Some(user) => respond::ok(with_follow_counts(state, &user)),
        None => ApiError::not_found("user not found").into_response(),
    }
}

/// Render a user with its follower / following totals filled in.
///
/// The counts are derived from the follow edges rather than stored on the user,
/// so every write that adds or removes an edge is reflected on the next read.
fn with_follow_counts(state: &AppState, user: &StoredUser) -> dto::User {
    let mut dto = user.to_dto();
    dto.followers_count = Some(followers(state, &user.login).len() as i64);
    dto.following_count = Some(following(state, &user.login).len() as i64);
    dto
}

/// `GET /api/v1/users/search`.
///
/// Gitea wraps user search results in `{ok, data}` rather than a bare array.
fn search_users(state: &AppState, req: &ApiRequest) -> Response {
    let term = req.query_param("q").unwrap_or_default().to_lowercase();
    let users: Vec<dto::User> = state
        .users()
        .iter()
        .filter(|u| {
            term.is_empty()
                || u.login.to_lowercase().contains(&term)
                || u.full_name.to_lowercase().contains(&term)
        })
        .map(|u| u.to_dto())
        .collect();
    let pagination = pagination(req);
    let total = users.len();
    respond::paged(
        serde_json::json!({ "ok": true, "data": pagination.apply(&users) }),
        &pagination,
        total,
    )
}

/// `GET|POST /api/v1/user/tokens`, `DELETE /api/v1/user/tokens/{id}`.
fn handle_tokens(state: &AppState, req: &ApiRequest) -> Response {
    let Ok(user) = require_user(state, req) else {
        return ApiError::unauthorized("authentication required").into_response();
    };

    if let Some(raw_id) = req.param("id").or_else(|| req.param("token_id")) {
        let id: i64 = raw_id.parse().unwrap_or_default();
        let before = state.tokens().len();
        state
            .tokens_mut()
            .retain(|t| !(t.id == id && t.user.eq_ignore_ascii_case(&user.login)));
        if state.tokens().len() == before {
            return ApiError::not_found("token not found").into_response();
        }
        return respond::no_content();
    }

    match req.method {
        Method::GET => {
            let tokens: Vec<dto::AccessToken> = state
                .tokens()
                .iter()
                .filter(|t| t.user.eq_ignore_ascii_case(&user.login))
                .map(|t| t.to_dto())
                .collect();
            respond::ok(tokens)
        }
        Method::POST => {
            let name = req.body_str("name").unwrap_or_else(|| "token".into());
            let id = state.next_id();
            let value = short_sha(&format!("{}:{name}:{id}", user.login));
            let scopes = req.body_str_vec("scopes").unwrap_or_default();
            let token = StoredToken {
                id,
                user: user.login.clone(),
                name,
                sha1: value,
                scopes,
                created: crate::state::now(),
                updated: crate::state::now(),
            };
            let dto = token.to_dto();
            state.tokens_mut().push(token);
            respond::created(dto)
        }
        _ => unimplemented(req),
    }
}

/// `GET|POST /api/v1/user/keys`, `DELETE /api/v1/user/keys/{id}` and GPG twins.
fn handle_keys(state: &AppState, req: &ApiRequest, kind: &str) -> Response {
    let Ok(user) = require_user(state, req) else {
        return ApiError::unauthorized("authentication required").into_response();
    };

    if let Some(raw_id) = req.param("id") {
        let id: i64 = raw_id.parse().unwrap_or_default();
        let before = state.keys().len();
        state.keys_mut().retain(|k| {
            !(k.id == id && k.user.eq_ignore_ascii_case(&user.login) && k.kind == kind)
        });
        if state.keys().len() == before {
            return ApiError::not_found("key not found").into_response();
        }
        return respond::no_content();
    }

    match req.method {
        Method::GET => respond::ok(keys_of(state, &user.login, kind)),
        Method::POST => {
            let title = req
                .body_str("title")
                .or_else(|| req.body_str("name"))
                .unwrap_or_else(|| "key".into());
            let key = req.body_str("key").or_else(|| req.body_str("content"));
            let Some(key) = key else {
                return ApiError::unprocessable("key content is required").into_response();
            };
            let id = state.next_id();
            let stored = StoredKey {
                id,
                user: user.login.clone(),
                title,
                fingerprint: short_sha(&key).chars().take(40).collect(),
                key,
                created: crate::state::now(),
                updated: crate::state::now(),
                kind: kind.to_string(),
            };
            let dto = stored.to_dto();
            state.keys_mut().push(stored);
            respond::created(dto)
        }
        _ => unimplemented(req),
    }
}

/// `GET|POST|DELETE /api/v1/user/emails`.
fn handle_emails(state: &AppState, req: &ApiRequest) -> Response {
    let Ok(user) = require_user(state, req) else {
        return ApiError::unauthorized("authentication required").into_response();
    };
    let primary = dto::Email {
        email: Some(user.email.clone()),
        primary: Some(true),
        verified: Some(true),
        user_id: Some(user.id),
        username: Some(user.login.clone()),
    };
    match req.method {
        Method::GET => {
            let mut emails = vec![primary];
            emails.extend(
                state
                    .emails()
                    .iter()
                    .filter(|e| e.user.eq_ignore_ascii_case(&user.login))
                    .map(|e| dto::Email {
                        email: Some(e.email.clone()),
                        primary: Some(false),
                        verified: Some(e.verified),
                        user_id: Some(user.id),
                        username: Some(user.login.clone()),
                    }),
            );
            respond::ok(emails)
        }
        Method::POST => {
            let requested = req.body_str_vec("emails").unwrap_or_default();
            let mut added = Vec::new();
            for email in requested {
                // Adding an address twice is idempotent.
                let known = state
                    .emails()
                    .iter()
                    .any(|e| e.user.eq_ignore_ascii_case(&user.login) && e.email == email);
                if !known {
                    state.emails_mut().push(crate::state::StoredEmail {
                        id: state.next_id(),
                        user: user.login.clone(),
                        email: email.clone(),
                        primary: false,
                        verified: false,
                    });
                }
                added.push(dto::Email {
                    email: Some(email),
                    primary: Some(false),
                    verified: Some(false),
                    user_id: Some(user.id),
                    username: Some(user.login.clone()),
                });
            }
            respond::created(added)
        }
        Method::DELETE => {
            let requested = req.body_str_vec("emails").unwrap_or_default();
            state.emails_mut().retain(|e| {
                !e.user.eq_ignore_ascii_case(&user.login) || !requested.contains(&e.email)
            });
            respond::no_content()
        }
        _ => unimplemented(req),
    }
}

/// Repositories owned by `login`.
fn repos_of(state: &AppState, login: &str) -> Vec<dto::Repository> {
    state
        .repos()
        .iter()
        .filter(|r| r.owner.eq_ignore_ascii_case(login))
        .map(|r| r.to_dto())
        .collect()
}

/// The login a request acts as, defaulting to the demo user.
pub(super) fn current_login(state: &AppState, req: &ApiRequest) -> String {
    current_user(state, req)
        .map(|u| u.login)
        .unwrap_or_default()
}

/// Repositories `login` has starred.
fn starred_of(state: &AppState, login: &str) -> Vec<dto::Repository> {
    state
        .repos()
        .iter()
        .filter(|r| r.stars.iter().any(|s| s.eq_ignore_ascii_case(login)))
        .map(|r| r.to_dto())
        .collect()
}

/// `GET|PUT|DELETE /api/v1/user/starred/{owner}/{repo}`.
///
/// Gitea answers `204 No Content` for all three: the check is a probe, and the
/// two writes are idempotent.
fn starred(state: &AppState, req: &ApiRequest, owner: &str, name: &str) -> Response {
    let Some(repo) = state.repo(owner, name) else {
        return ApiError::not_found("repository does not exist").into_response();
    };
    let login = current_login(state, req);

    if req.method == Method::GET {
        let starred = repo.stars.iter().any(|s| s.eq_ignore_ascii_case(&login));
        if !starred {
            return ApiError::not_found("user has not starred this repository").into_response();
        }
        return respond::no_content();
    }

    if let Some(stored) = state.repos_mut().iter_mut().find(|r| r.id == repo.id) {
        match req.method {
            Method::PUT => {
                if !stored.stars.iter().any(|s| s.eq_ignore_ascii_case(&login)) {
                    stored.stars.push(login);
                }
            }
            Method::DELETE => stored.stars.retain(|s| !s.eq_ignore_ascii_case(&login)),
            _ => return unimplemented(req),
        }
    }
    respond::no_content()
}

/// Organizations `login` belongs to.
pub(super) fn orgs_of(state: &AppState, login: &str) -> Vec<dto::Organization> {
    state
        .orgs()
        .iter()
        .filter(|o| o.members.iter().any(|m| m.eq_ignore_ascii_case(login)))
        .map(|o| o.to_dto())
        .collect()
}

/// Followers of `login`.
fn followers(state: &AppState, login: &str) -> Vec<dto::User> {
    state
        .follows()
        .iter()
        .filter(|f| f.target.eq_ignore_ascii_case(login))
        .filter_map(|f| state.user(&f.user).map(|u| u.to_dto()))
        .collect()
}

/// Accounts `login` follows.
fn following(state: &AppState, login: &str) -> Vec<dto::User> {
    state
        .follows()
        .iter()
        .filter(|f| f.user.eq_ignore_ascii_case(login))
        .filter_map(|f| state.user(&f.target).map(|u| u.to_dto()))
        .collect()
}

/// Keys of a given kind belonging to `login`.
pub(super) fn keys_of(state: &AppState, login: &str, kind: &str) -> Vec<dto::PublicKey> {
    state
        .keys()
        .iter()
        .filter(|k| k.user.eq_ignore_ascii_case(login) && k.kind == kind)
        .map(|k| k.to_dto())
        .collect()
}

/// Pagination from the request query.
fn pagination(req: &ApiRequest) -> Pagination {
    Pagination {
        page: req.query_i64("page"),
        limit: req.query_i64("limit"),
    }
}

/// A follow edge between the acting user and `login`.
///
/// `PUT` follows (idempotent), `DELETE` unfollows, `GET` is a probe that
/// answers `204` when the edge exists and `404` otherwise, matching Gitea.
fn follow_edge(state: &AppState, req: &ApiRequest, login: &str) -> Response {
    let me = current_login(state, req);
    match req.method {
        Method::GET => {
            if state.follows_user(&me, login) {
                respond::no_content()
            } else {
                ApiError::not_found("user is not followed").into_response()
            }
        }
        Method::PUT => {
            if !state.follows_user(&me, login) {
                state.follows_mut().push(StoredFollow {
                    user: me,
                    target: login.to_string(),
                    created: crate::state::now(),
                });
            }
            respond::no_content()
        }
        Method::DELETE => {
            state.follows_mut().retain(|f| {
                !(f.user.eq_ignore_ascii_case(&me) && f.target.eq_ignore_ascii_case(login))
            });
            respond::no_content()
        }
        _ => unimplemented(req),
    }
}

/// A block edge between the acting user and `login`.
fn block_edge(state: &AppState, req: &ApiRequest, login: &str) -> Response {
    let me = current_login(state, req);
    match req.method {
        Method::GET => {
            if state.blocks_user(&me, login) {
                respond::no_content()
            } else {
                ApiError::not_found("user is not blocked").into_response()
            }
        }
        Method::PUT => {
            if !state.blocks_user(&me, login) {
                state.blocks_mut().push(StoredBlock {
                    user: me,
                    target: login.to_string(),
                    created: crate::state::now(),
                });
            }
            respond::no_content()
        }
        Method::DELETE => {
            state.blocks_mut().retain(|b| {
                !(b.user.eq_ignore_ascii_case(&me) && b.target.eq_ignore_ascii_case(login))
            });
            respond::no_content()
        }
        _ => unimplemented(req),
    }
}

/// `DELETE /api/v1/users/{username}/tokens/{token}`.
fn delete_user_token(state: &AppState, req: &ApiRequest, login: &str, token: &str) -> Response {
    if req.method != Method::DELETE {
        return unimplemented(req);
    }
    let before = state.tokens().len();
    state
        .tokens_mut()
        .retain(|t| !(t.name == token && t.user.eq_ignore_ascii_case(login)));
    if state.tokens().len() == before {
        return ApiError::not_found("token not found").into_response();
    }
    respond::no_content()
}

/// `GET|POST /api/v1/users/{username}/tokens` — the acting user's access
/// tokens. Gitea only ever exposes a user their own tokens, so a request for
/// another login is refused rather than silently answered with an empty list.
fn handle_user_tokens(state: &AppState, req: &ApiRequest, login: &str) -> Response {
    let Ok(user) = require_user(state, req) else {
        return ApiError::unauthorized("authentication required").into_response();
    };
    if !user.login.eq_ignore_ascii_case(login) {
        return ApiError::forbidden("you can only read your own access tokens").into_response();
    }

    match req.method {
        Method::GET => {
            let tokens: Vec<dto::AccessToken> = state
                .tokens()
                .iter()
                .filter(|t| t.user.eq_ignore_ascii_case(login))
                .map(|t| t.to_dto())
                .collect();
            respond::ok(tokens)
        }
        Method::POST => {
            let name = req.body_str("name").unwrap_or_else(|| "token".into());
            let id = state.next_id();
            let value = short_sha(&format!("{login}:{name}:{id}"));
            let token = StoredToken {
                id,
                user: login.to_string(),
                name,
                sha1: value,
                scopes: req.body_str_vec("scopes").unwrap_or_default(),
                created: crate::state::now(),
                updated: crate::state::now(),
            };
            let dto = token.to_dto();
            state.tokens_mut().push(token);
            respond::created(dto)
        }
        _ => unimplemented(req),
    }
}

/// Accounts the acting user has blocked.
fn blocks_of(state: &AppState, login: &str) -> Vec<dto::User> {
    state
        .blocks()
        .iter()
        .filter(|b| b.user.eq_ignore_ascii_case(login))
        .filter_map(|b| state.user(&b.target).map(|u| u.to_dto()))
        .collect()
}

/// `GET /api/v1/orgs/{org}/blocks` — users the organization has blocked.
pub(super) fn handle_org_blocks(state: &AppState, req: &ApiRequest, org: &str) -> Response {
    if req.method != Method::GET {
        return unimplemented(req);
    }
    let users: Vec<dto::User> = state
        .org_blocks()
        .iter()
        .filter(|b| b.org.eq_ignore_ascii_case(org))
        .filter_map(|b| state.user(&b.user).map(|u| u.to_dto()))
        .collect();
    respond::ok(users)
}

/// `GET|PUT|DELETE /api/v1/orgs/{org}/blocks/{username}`.
pub(super) fn handle_org_block(
    state: &AppState,
    req: &ApiRequest,
    org: &str,
    username: &str,
) -> Response {
    let blocked = state
        .org_blocks()
        .iter()
        .any(|b| b.org.eq_ignore_ascii_case(org) && b.user.eq_ignore_ascii_case(username));

    match req.method {
        Method::GET => {
            if blocked {
                respond::no_content()
            } else {
                ApiError::not_found("user is not blocked").into_response()
            }
        }
        Method::PUT => {
            if !blocked {
                state.org_blocks_mut().push(StoredOrgBlock {
                    org: org.to_string(),
                    user: username.to_string(),
                    created: crate::state::now(),
                });
            }
            respond::no_content()
        }
        Method::DELETE => {
            state.org_blocks_mut().retain(|b| {
                !(b.org.eq_ignore_ascii_case(org) && b.user.eq_ignore_ascii_case(username))
            });
            respond::no_content()
        }
        _ => unimplemented(req),
    }
}

/// `GET|PUT|DELETE /api/v1/orgs/{org}/public_members/{username}`.
///
/// Public membership is modelled with the organization's `members` list: the
/// `PUT` is idempotent, and `DELETE` removes the member entirely because the
/// store keeps only one membership list.
pub(super) fn handle_public_member(
    state: &AppState,
    req: &ApiRequest,
    org: &str,
    username: &str,
) -> Response {
    let member = state
        .org(org)
        .map(|o| o.members.iter().any(|m| m.eq_ignore_ascii_case(username)))
        .unwrap_or(false);

    match req.method {
        Method::GET => {
            if member {
                respond::no_content()
            } else {
                ApiError::not_found("user is not a public member").into_response()
            }
        }
        Method::PUT => {
            if let Some(o) = state
                .orgs_mut()
                .iter_mut()
                .find(|o| o.name.eq_ignore_ascii_case(org))
            {
                if !o.members.iter().any(|m| m.eq_ignore_ascii_case(username)) {
                    o.members.push(username.to_string());
                }
            }
            respond::no_content()
        }
        Method::DELETE => {
            if let Some(o) = state
                .orgs_mut()
                .iter_mut()
                .find(|o| o.name.eq_ignore_ascii_case(org))
            {
                o.members.retain(|m| !m.eq_ignore_ascii_case(username));
            }
            respond::no_content()
        }
        _ => unimplemented(req),
    }
}

/// Repositories `login` watches (is subscribed to).
fn subscriptions_of(state: &AppState, login: &str) -> Vec<dto::Repository> {
    state
        .repos()
        .iter()
        .filter(|r| r.watchers.iter().any(|w| w.eq_ignore_ascii_case(login)))
        .map(|r| r.to_dto())
        .collect()
}

/// Teams across every organization that `login` belongs to.
fn teams_of(state: &AppState, login: &str) -> Vec<dto::Team> {
    state
        .teams()
        .iter()
        .filter(|t| t.members.iter().any(|m| m.eq_ignore_ascii_case(login)))
        .filter_map(|t| state.org(&t.org).map(|org| t.to_dto(&org)))
        .collect()
}

/// The permissions `login` holds in `org`, per the specification's shape.
fn org_permissions(state: &AppState, login: &str, org: &str) -> serde_json::Value {
    let member = state
        .org(org)
        .map(|o| o.members.iter().any(|m| m.eq_ignore_ascii_case(login)))
        .unwrap_or(false);
    let admin = state
        .teams()
        .iter()
        .filter(|t| t.org.eq_ignore_ascii_case(org))
        .any(|t| {
            t.permission == "owner" && t.members.iter().any(|m| m.eq_ignore_ascii_case(login))
        });
    serde_json::json!({
        "can_create_org_repo": member,
        "can_read": member,
        "can_write": member,
        "is_admin": admin,
        "is_owner": admin,
    })
}

/// A contribution heatmap, synthesised deterministically for the demo user.
fn heatmap() -> Vec<dto::UserHeatmapData> {
    let mut out = Vec::new();
    for day in 0..30 {
        out.push(dto::UserHeatmapData {
            contributions: Some(((day * 7) % 11) as i64),
            timestamp: Some(dto::TimeStamp::default()),
        });
    }
    out
}

/// `GET /api/v1/user/stopwatches` — the running stopwatches of the caller.
///
/// A stopwatch lives on the issue (`stopwatch_seconds`), so the listing is
/// derived from the issue store rather than a store of its own.
fn handle_stopwatches(state: &AppState, req: &ApiRequest) -> Response {
    let login = current_login(state, req);
    let items: Vec<dto::StopWatch> = state
        .issues()
        .iter()
        .filter(|i| i.stopwatch_seconds.is_some())
        .filter(|i| i.assignees.iter().any(|a| a.eq_ignore_ascii_case(&login)))
        .filter_map(|issue| {
            let repo = state.repo_by_id(issue.repo_id)?;
            let seconds = issue.stopwatch_seconds.unwrap_or(0);
            Some(dto::StopWatch {
                created: Some(issue.created.clone()),
                duration: Some(format!("{}s", seconds.max(0))),
                issue_index: Some(issue.index),
                issue_title: Some(issue.title.clone()),
                repo_name: Some(repo.name.clone()),
                repo_owner_name: Some(repo.owner.clone()),
                seconds: Some(seconds),
            })
        })
        .collect();
    respond::ok(items)
}

/// `GET /api/v1/user/times` — tracked time for the caller.
///
/// Tracked times are derived from the time-tracking comments a user left on
/// issues, which is where Gitea records them.
fn handle_tracked_times(state: &AppState, req: &ApiRequest) -> Response {
    let login = current_login(state, req);
    let mut times: Vec<dto::TrackedTime> = Vec::new();
    for issue in state.issues().iter() {
        let Some(repo) = state.repo_by_id(issue.repo_id) else {
            continue;
        };
        for comment in issue.comments.iter() {
            if !comment.poster.eq_ignore_ascii_case(&login) {
                continue;
            }
            let seconds = super::issues::tracked_seconds(&comment.body).unwrap_or(0);
            times.push(dto::TrackedTime {
                id: Some(comment.id),
                time: Some(seconds),
                created: Some(comment.created.clone()),
                issue_id: Some(issue.id),
                user_id: state.user(&login).map(|u| u.id),
                user_name: Some(login.clone()),
                issue: Some(issue.to_dto(&repo, &[], None)),
            });
        }
    }
    respond::ok(times)
}

/// `GET|POST|PATCH|DELETE /api/v1/user/hooks`, its admin twin and the
/// `org:<name>` scope used by `/api/v1/orgs/{org}/hooks`.
pub fn handle_hooks(state: &AppState, req: &ApiRequest, scope: &str) -> Response {
    // `scope` is `user`, `admin`, or `org:<name>`; the owner is the scope key.
    let owner = match scope {
        "admin" => String::new(),
        _ => match scope.strip_prefix("org:") {
            Some(name) => name.to_string(),
            None => current_login(state, req),
        },
    };
    handle_hooks_for(state, req, scope, &owner)
}

/// `GET|POST|PATCH|DELETE /api/v1/repos/{owner}/{repo}/hooks` and `/{id}`.
pub(super) fn handle_repo_hooks(state: &AppState, req: &ApiRequest, repo: &StoredRepo) -> Response {
    let owner = format!("{}:{}", repo.owner, repo.name);
    handle_hooks_for(state, req, "repo", &owner)
}

/// Webhook storage keyed by an explicit scope and owner.
fn handle_hooks_for(state: &AppState, req: &ApiRequest, scope: &str, owner: &str) -> Response {
    if let Some(raw_id) = req.param("id") {
        let id: i64 = raw_id.parse().unwrap_or_default();
        let found = state
            .hooks()
            .iter()
            .find(|h| h.id == id && h.scope == scope && h.owner.eq_ignore_ascii_case(owner))
            .cloned();
        let Some(hook) = found else {
            return ApiError::not_found("hook not found").into_response();
        };
        return match req.method {
            Method::GET => respond::ok(hook.to_dto()),
            Method::PATCH => {
                if let Some(h) = state
                    .hooks_mut()
                    .iter_mut()
                    .find(|h| h.id == id && h.scope == scope)
                {
                    apply_hook_edits(h, req);
                    h.updated = crate::state::now();
                }
                let updated = state
                    .hooks()
                    .iter()
                    .find(|h| h.id == id)
                    .map(|h| h.to_dto());
                match updated {
                    Some(dto) => respond::ok(dto),
                    None => ApiError::not_found("hook not found").into_response(),
                }
            }
            Method::DELETE => {
                state.hooks_mut().retain(|h| h.id != id);
                respond::no_content()
            }
            _ => unimplemented(req),
        };
    }

    match req.method {
        Method::GET => {
            let hooks: Vec<dto::Hook> = state
                .hooks()
                .iter()
                .filter(|h| h.scope == scope && h.owner.eq_ignore_ascii_case(owner))
                .map(|h| h.to_dto())
                .collect();
            respond::ok(hooks)
        }
        Method::POST => {
            let id = state.next_id();
            let mut hook = StoredHook {
                id,
                owner: owner.to_string(),
                scope: scope.to_string(),
                kind: req.body_str("type").unwrap_or_else(|| "gitea".into()),
                name: req.body_str("name").unwrap_or_default(),
                active: req.body_bool("active").unwrap_or(true),
                branch_filter: req.body_str("branch_filter").unwrap_or_default(),
                authorization_header: req.body_str("authorization_header").unwrap_or_default(),
                created: crate::state::now(),
                updated: crate::state::now(),
                ..Default::default()
            };
            apply_hook_edits(&mut hook, req);
            let dto = hook.to_dto();
            state.hooks_mut().push(hook);
            respond::created(dto)
        }
        _ => unimplemented(req),
    }
}

/// Copy the editable webhook fields present in the request body onto `hook`.
fn apply_hook_edits(hook: &mut StoredHook, req: &ApiRequest) {
    if let Some(active) = req.body_bool("active") {
        hook.active = active;
    }
    if let Some(events) = req.body_str_vec("events") {
        hook.events = events;
    }
    if let Some(filter) = req.body_str("branch_filter") {
        hook.branch_filter = filter;
    }
    if let Some(header) = req.body_str("authorization_header") {
        hook.authorization_header = header;
    }
    if let Some(config) = req.json().get("config").and_then(|v| v.as_object()) {
        hook.config = config
            .iter()
            .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
            .collect();
    }
}

/// The owner a project belongs to, resolved from the request scope.
///
/// A project is filed under a user, a repository or an organization; all three
/// share the `projects` store and differ only in `owner_id`/`kind`/`repo_id`.
struct ProjectScope {
    owner_id: i64,
    kind: &'static str,
    repo_id: Option<i64>,
}

/// Resolve the scope from `login` (individual), `owner`/`repo` or `org`.
fn project_scope(state: &AppState, req: &ApiRequest, login: Option<&str>) -> Option<ProjectScope> {
    if let (Some(owner), Some(repo)) = (req.param("owner"), req.param("repo")) {
        let r = state.repo(&owner, &repo)?;
        return Some(ProjectScope {
            owner_id: r.id,
            kind: "repository",
            repo_id: Some(r.id),
        });
    }
    if let Some(org_name) = req.param("org") {
        let o = state.org(&org_name)?;
        return Some(ProjectScope {
            owner_id: o.id,
            kind: "organization",
            repo_id: None,
        });
    }
    let me = current_user(state, req);
    let login = login
        .map(str::to_string)
        .or_else(|| me.as_ref().map(|u| u.login.clone()))?;
    let user = state.user(&login)?;
    Some(ProjectScope {
        owner_id: user.id,
        kind: "individual",
        repo_id: None,
    })
}

/// `GET|POST|PATCH|DELETE` for projects under any scope.
///
/// `login` scopes the listing to another account; writes always act as the
/// authenticated user, as Gitea does.
pub(super) fn handle_projects(state: &AppState, req: &ApiRequest, login: Option<&str>) -> Response {
    let me = current_user(state, req);
    let Some(scope) = project_scope(state, req, login) else {
        return ApiError::not_found("project owner not found").into_response();
    };

    if let Some(raw_id) = req.param("id") {
        let id: i64 = raw_id.parse().unwrap_or_default();
        let found = state.projects().iter().find(|p| p.id == id).cloned();
        let Some(project) = found else {
            return ApiError::not_found("project not found").into_response();
        };
        return match req.method {
            Method::GET => respond::ok(project.to_dto()),
            Method::PATCH => {
                if let Some(p) = state.projects_mut().iter_mut().find(|p| p.id == id) {
                    if let Some(title) = req.body_str("title") {
                        p.title = title;
                    }
                    if let Some(description) = req.body_str("description") {
                        p.description = description;
                    }
                    if let Some(card_type) = req.body_str("card_type") {
                        p.card_type = card_type;
                    }
                    if let Some(state_wire) = req.body_str("state") {
                        p.closed = state_wire.eq_ignore_ascii_case("closed");
                    }
                    p.updated = crate::state::now();
                }
                let updated = state
                    .projects()
                    .iter()
                    .find(|p| p.id == id)
                    .map(|p| p.to_dto());
                match updated {
                    Some(dto) => respond::ok(dto),
                    None => ApiError::not_found("project not found").into_response(),
                }
            }
            Method::DELETE => {
                state.projects_mut().retain(|p| p.id != id);
                state.project_columns_mut().retain(|c| c.project_id != id);
                respond::no_content()
            }
            _ => unimplemented(req),
        };
    }

    match req.method {
        Method::GET => {
            let projects: Vec<dto::Project> = state
                .projects()
                .iter()
                .filter(|p| p.owner_id == scope.owner_id && p.kind == scope.kind)
                .map(|p| p.to_dto())
                .collect();
            respond::ok(projects)
        }
        Method::POST => {
            let Some(user) = me else {
                return ApiError::unauthorized("authentication required").into_response();
            };
            let title = req.body_str("title").unwrap_or_else(|| "Untitled".into());
            let id = state.next_id();
            let project = StoredProject {
                id,
                title,
                description: req.body_str("description").unwrap_or_default(),
                owner_id: scope.owner_id,
                kind: scope.kind.into(),
                repo_id: scope.repo_id,
                card_type: req
                    .body_str("card_type")
                    .unwrap_or_else(|| "text_only".into()),
                template_type: req
                    .body_str("template_type")
                    .unwrap_or_else(|| "none".into()),
                creator: user.login.clone(),
                created: crate::state::now(),
                updated: crate::state::now(),
                ..Default::default()
            };
            let dto = project.to_dto();
            state.projects_mut().push(project);
            respond::created(dto)
        }
        _ => unimplemented(req),
    }
}

/// `GET|POST /api/v1/user/projects/{id}/columns`.
pub(super) fn handle_project_columns(
    state: &AppState,
    req: &ApiRequest,
    project_id: &str,
    login: Option<&str>,
) -> Response {
    let _ = login;
    let pid: i64 = project_id.parse().unwrap_or_default();
    if !state.projects().iter().any(|p| p.id == pid) {
        return ApiError::not_found("project not found").into_response();
    }

    match req.method {
        Method::GET => {
            let mut columns: Vec<dto::ProjectColumn> = state
                .project_columns()
                .iter()
                .filter(|c| c.project_id == pid)
                .map(|c| c.to_dto())
                .collect();
            columns.sort_by_key(|c| c.sorting.unwrap_or_default());
            respond::ok(columns)
        }
        Method::POST => {
            let title = req.body_str("title").unwrap_or_else(|| "Column".into());
            let id = state.next_id();
            let sorting = state
                .project_columns()
                .iter()
                .filter(|c| c.project_id == pid)
                .count() as i64;
            let is_default = sorting == 0;
            let column = StoredProjectColumn {
                id,
                project_id: pid,
                title,
                sorting,
                is_default,
                created: crate::state::now(),
                updated: crate::state::now(),
                ..Default::default()
            };
            let dto = column.to_dto();
            state.project_columns_mut().push(column);
            respond::created(dto)
        }
        _ => unimplemented(req),
    }
}

/// `GET|PATCH|DELETE /api/v1/user/projects/{id}/columns/{column_id}`.
pub(super) fn handle_project_column(
    state: &AppState,
    req: &ApiRequest,
    project_id: &str,
    column_id: &str,
    _login: Option<&str>,
) -> Response {
    let pid: i64 = project_id.parse().unwrap_or_default();
    let cid: i64 = column_id.parse().unwrap_or_default();
    let found = state
        .project_columns()
        .iter()
        .find(|c| c.id == cid && c.project_id == pid)
        .cloned();
    let Some(column) = found else {
        return ApiError::not_found("column not found").into_response();
    };

    match req.method {
        Method::GET => respond::ok(column.to_dto()),
        Method::PATCH => {
            if let Some(c) = state.project_columns_mut().iter_mut().find(|c| c.id == cid) {
                if let Some(title) = req.body_str("title") {
                    c.title = title;
                }
                c.updated = crate::state::now();
            }
            let updated = state
                .project_columns()
                .iter()
                .find(|c| c.id == cid)
                .map(|c| c.to_dto());
            match updated {
                Some(dto) => respond::ok(dto),
                None => ApiError::not_found("column not found").into_response(),
            }
        }
        Method::DELETE => {
            state.project_columns_mut().retain(|c| c.id != cid);
            respond::no_content()
        }
        _ => unimplemented(req),
    }
}

/// `GET|POST /api/v1/user/projects/{id}/columns/{column_id}/issues`.
pub(super) fn handle_column_issues(
    state: &AppState,
    req: &ApiRequest,
    project_id: &str,
    column_id: &str,
) -> Response {
    let pid: i64 = project_id.parse().unwrap_or_default();
    let cid: i64 = column_id.parse().unwrap_or_default();
    let found = state
        .project_columns()
        .iter()
        .find(|c| c.id == cid && c.project_id == pid)
        .cloned();
    let Some(column) = found else {
        return ApiError::not_found("column not found").into_response();
    };

    match req.method {
        Method::GET => {
            let issues: Vec<dto::Issue> = column
                .issues
                .iter()
                .filter_map(|index| {
                    state
                        .issues()
                        .iter()
                        .find(|i| i.index == *index)
                        .and_then(|i| state.repo_by_id(i.repo_id).map(|r| i.to_dto(&r, &[], None)))
                })
                .collect();
            respond::ok(issues)
        }
        Method::POST => {
            let index = req.body_i64("issue_id").unwrap_or_default();
            if let Some(c) = state.project_columns_mut().iter_mut().find(|c| c.id == cid) {
                if !c.issues.contains(&index) {
                    c.issues.push(index);
                }
            }
            respond::no_content()
        }
        _ => unimplemented(req),
    }
}

/// `DELETE /api/v1/user/projects/{id}/columns/{column_id}/issues/{issue_id}`.
pub(super) fn handle_column_issue(
    state: &AppState,
    req: &ApiRequest,
    project_id: &str,
    column_id: &str,
    issue_id: &str,
) -> Response {
    let pid: i64 = project_id.parse().unwrap_or_default();
    let cid: i64 = column_id.parse().unwrap_or_default();
    let index: i64 = issue_id.parse().unwrap_or_default();
    if !state
        .project_columns()
        .iter()
        .any(|c| c.id == cid && c.project_id == pid)
    {
        return ApiError::not_found("column not found").into_response();
    }
    if req.method != Method::DELETE {
        return unimplemented(req);
    }
    if let Some(c) = state.project_columns_mut().iter_mut().find(|c| c.id == cid) {
        c.issues.retain(|i| *i != index);
    }
    respond::no_content()
}

/// `POST /api/v1/user/projects/{id}/columns/{column_id}/default`.
pub(super) fn handle_column_default(
    state: &AppState,
    req: &ApiRequest,
    project_id: &str,
    column_id: &str,
) -> Response {
    if req.method != Method::POST {
        return unimplemented(req);
    }
    let pid: i64 = project_id.parse().unwrap_or_default();
    let cid: i64 = column_id.parse().unwrap_or_default();
    if !state
        .project_columns()
        .iter()
        .any(|c| c.id == cid && c.project_id == pid)
    {
        return ApiError::not_found("column not found").into_response();
    }
    for column in state.project_columns_mut().iter_mut() {
        if column.project_id == pid {
            column.is_default = column.id == cid;
        }
    }
    respond::no_content()
}

/// `POST /api/v1/user/projects/{id}/columns/move`.
pub(super) fn handle_columns_move(
    state: &AppState,
    req: &ApiRequest,
    project_id: &str,
) -> Response {
    let pid: i64 = project_id.parse().unwrap_or_default();
    if !state.projects().iter().any(|p| p.id == pid) {
        return ApiError::not_found("project not found").into_response();
    }
    let column_id = req.body_i64("column_id").unwrap_or_default();
    let position = req.body_i64("position").unwrap_or_default();
    let mut columns: Vec<StoredProjectColumn> = state
        .project_columns()
        .iter()
        .filter(|c| c.project_id == pid)
        .cloned()
        .collect();
    columns.sort_by_key(|c| c.sorting);
    if let Some(from) = columns.iter().position(|c| c.id == column_id) {
        let moved = columns.remove(from);
        let to = (position.max(0) as usize).min(columns.len());
        columns.insert(to, moved);
    }
    for (i, column) in columns.iter_mut().enumerate() {
        column.sorting = i as i64;
    }
    for column in columns {
        if let Some(stored) = state
            .project_columns_mut()
            .iter_mut()
            .find(|c| c.id == column.id)
        {
            stored.sorting = column.sorting;
        }
    }
    respond::no_content()
}

/// `POST /api/v1/user/projects/{id}/issues/{issue_id}/move`.
pub(super) fn handle_project_issue_move(
    state: &AppState,
    req: &ApiRequest,
    project_id: &str,
    issue_id: &str,
) -> Response {
    let pid: i64 = project_id.parse().unwrap_or_default();
    let index: i64 = issue_id.parse().unwrap_or_default();
    if !state.projects().iter().any(|p| p.id == pid) {
        return ApiError::not_found("project not found").into_response();
    }
    let target = req.body_i64("column_id").unwrap_or_default();
    if !state
        .project_columns()
        .iter()
        .any(|c| c.id == target && c.project_id == pid)
    {
        return ApiError::bad_request("target column does not exist").into_response();
    }
    for column in state.project_columns_mut().iter_mut() {
        if column.project_id != pid {
            continue;
        }
        column.issues.retain(|i| *i != index);
        if column.id == target {
            column.issues.push(index);
        }
    }
    respond::no_content()
}

/// `GET|POST|DELETE /api/v1/user/applications/oauth2` and `/{id}`.
fn handle_oauth_apps(state: &AppState, req: &ApiRequest) -> Response {
    let Some(user) = current_user(state, req) else {
        return ApiError::unauthorized("authentication required").into_response();
    };

    if let Some(raw_id) = req.param("id") {
        let id: i64 = raw_id.parse().unwrap_or_default();
        let found = state
            .oauth_apps()
            .iter()
            .find(|a| a.id == id && a.user.eq_ignore_ascii_case(&user.login))
            .cloned();
        let Some(app) = found else {
            return ApiError::not_found("application not found").into_response();
        };
        return match req.method {
            Method::GET => respond::ok(app.to_dto()),
            Method::DELETE => {
                state.oauth_apps_mut().retain(|a| a.id != id);
                respond::no_content()
            }
            _ => unimplemented(req),
        };
    }

    match req.method {
        Method::GET => {
            let apps: Vec<dto::OAuth2Application> = state
                .oauth_apps()
                .iter()
                .filter(|a| a.user.eq_ignore_ascii_case(&user.login))
                .map(|a| a.to_dto())
                .collect();
            respond::ok(apps)
        }
        Method::POST => {
            let name = req.body_str("name").unwrap_or_else(|| "app".into());
            let id = state.next_id();
            let app = StoredOauthApp {
                id,
                user: user.login.clone(),
                name,
                client_id: short_sha(&format!("client:{id}")),
                client_secret: short_sha(&format!("secret:{id}")),
                redirect_uris: req.body_str_vec("redirect_uris").unwrap_or_default(),
                confidential: req.body_bool("confidential_client").unwrap_or(true),
                skip_secondary_authorization: req
                    .body_bool("skip_secondary_authorization")
                    .unwrap_or(false),
                created: crate::state::now(),
            };
            let dto = app.to_dto();
            state.oauth_apps_mut().push(app);
            respond::created(dto)
        }
        _ => unimplemented(req),
    }
}

/// Documented fallback for operations this module does not implement yet.
fn unimplemented(req: &ApiRequest) -> Response {
    super::dispatch::fallback(req)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::StoredRepo;
    use axum::body::Bytes;
    use axum::http::StatusCode;

    fn state() -> AppState {
        let state = AppState::default();
        state.users_mut().push(StoredUser {
            id: 1,
            login: "alice".into(),
            full_name: "Alice".into(),
            email: "alice@example.com".into(),
            active: true,
            ..Default::default()
        });
        state
    }

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
    fn resolves_the_demo_user_without_a_token() {
        let state = state();
        let user = current_user(&state, &req(Method::GET, "/api/v1/user", "")).unwrap();
        assert_eq!(user.login, "alice");
    }

    #[test]
    fn token_resolves_to_its_owner() {
        let state = state();
        state.tokens_mut().push(StoredToken {
            id: 9,
            user: "alice".into(),
            sha1: "abc123".into(),
            ..Default::default()
        });
        let mut request = req(Method::GET, "/api/v1/user", "");
        request.query.insert("token".into(), "abc123".into());
        assert_eq!(current_user(&state, &request).unwrap().login, "alice");
    }

    #[test]
    fn user_dto_hides_the_password() {
        let state = state();
        state.users_mut()[0].password = "hunter2".into();
        let json = serde_json::to_string(&state.users()[0].to_dto()).unwrap();
        assert!(!json.contains("hunter2"));
    }

    #[test]
    fn starring_is_idempotent_and_scoped_to_the_user() {
        let state = state();
        state.repos_mut().push(StoredRepo {
            id: 10,
            owner: "alice".into(),
            name: "demo".into(),
            default_branch: "main".into(),
            created: crate::state::now(),
            updated: crate::state::now(),
            ..Default::default()
        });
        for _ in 0..2 {
            let response = starred(
                &state,
                &req(Method::PUT, "/api/v1/user/starred/alice/demo", ""),
                "alice",
                "demo",
            );
            assert_eq!(response.status(), StatusCode::NO_CONTENT);
        }
        assert_eq!(starred_of(&state, "alice").len(), 1);

        let response = starred(
            &state,
            &req(Method::GET, "/api/v1/user/starred/alice/demo", ""),
            "alice",
            "demo",
        );
        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        let response = starred(
            &state,
            &req(Method::DELETE, "/api/v1/user/starred/alice/demo", ""),
            "alice",
            "demo",
        );
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        assert!(starred_of(&state, "alice").is_empty());
    }
}
