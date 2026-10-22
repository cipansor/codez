//! Organization and team endpoints.

use axum::http::Method;
use axum::response::{IntoResponse, Response};

use shared::models as dto;

use crate::pagination::Pagination;
use crate::respond::{self, ApiError};
use crate::state::{now, AppState, StoredOrg, StoredTeam};

use super::ApiRequest;

/// Route a `/api/v1/orgs/*` request.
pub fn handle(state: &AppState, req: &ApiRequest) -> Response {
    let segments: Vec<&str> = req.path.trim_matches('/').split('/').collect();
    // ["api", "v1", "orgs", ...]
    let rest = &segments[3..];

    match rest {
        [] => match req.method {
            Method::GET => list(state, req),
            Method::POST => create(state, req),
            _ => unimplemented(req),
        },
        _ => {
            let Some(org_name) = req.param("org") else {
                return unimplemented(req);
            };
            scoped(state, req, &org_name)
        }
    }
}

/// `POST /api/v1/org/{org}/repos` — the deprecated singular alias of
/// `POST /api/v1/orgs/{org}/repos`.
pub fn handle_deprecated(state: &AppState, req: &ApiRequest) -> Response {
    if req.method == Method::POST && req.path.ends_with("/repos") {
        return super::repos::create(state, req);
    }
    unimplemented(req)
}

/// Endpoints below `/api/v1/orgs/{org}`.
fn scoped(state: &AppState, req: &ApiRequest, org_name: &str) -> Response {
    let Some(org) = state.org(org_name) else {
        return ApiError::not_found("organization not found").into_response();
    };
    let segments: Vec<&str> = req.path.trim_matches('/').split('/').collect();
    // ["api", "v1", "orgs", org, ...]
    let tail = &segments[4..];

    match tail {
        [] => match req.method {
            Method::GET => respond::ok(org.to_dto()),
            Method::PATCH => update(state, req, &org),
            Method::DELETE => {
                state
                    .orgs_mut()
                    .retain(|o| !o.name.eq_ignore_ascii_case(&org.name));
                respond::no_content()
            }
            _ => unimplemented(req),
        },
        ["repos"] => match req.method {
            Method::GET => respond::ok(repos_of(state, &org.name)),
            Method::POST => super::repos::create(state, req),
            Method::DELETE => {
                let before = state.repos().len();
                state
                    .repos_mut()
                    .retain(|r| !r.owner.eq_ignore_ascii_case(&org.name));
                if state.repos().len() == before {
                    return ApiError::not_found("organization has no repositories").into_response();
                }
                respond::no_content()
            }
            _ => unimplemented(req),
        },
        ["actions", ..] => super::actions::handle(state, req),
        ["members"] => match req.method {
            Method::GET => respond::ok(members(state, &org)),
            _ => unimplemented(req),
        },
        ["members", login] => member_detail(state, req, &org, login),
        ["public_members"] => respond::ok(members(state, &org)),
        ["teams"] => match req.method {
            Method::GET => respond::ok(teams_of(state, &org.name)),
            Method::POST => create_team(state, req, &org),
            _ => unimplemented(req),
        },
        ["teams", name] => team_detail(state, req, &org, name),
        ["activities", "feeds"] | ["pulse"] => respond::ok(serde_json::json!({
            "opened_issues": 0,
            "closed_issues": 0,
            "opened_pull_requests": 0,
            "closed_pull_requests": 0,
        })),
        ["avatar"] => respond::no_content(),
        ["hooks"] => super::users::handle_hooks(state, req, &format!("org:{}", org.name)),
        ["hooks", _id] => super::users::handle_hooks(state, req, &format!("org:{}", org.name)),
        ["labels", ..] => super::issues::handle_org_labels(state, req, &org),
        ["blocks"] => super::users::handle_org_blocks(state, req, &org.name),
        ["blocks", username] => super::users::handle_org_block(state, req, &org.name, username),
        ["projects"] => super::users::handle_projects(state, req, None),
        ["projects", ..] => org_project_subresource(state, req),
        ["public_members", username] => {
            super::users::handle_public_member(state, req, &org.name, username)
        }
        ["rename"] => rename(state, req, &org),
        _ => unimplemented(req),
    }
}

/// `/api/v1/orgs/{org}/projects/{id}/…` — same shape as the repository scope.
fn org_project_subresource(state: &AppState, req: &ApiRequest) -> Response {
    let segments: Vec<&str> = req.path.trim_matches('/').split('/').collect();
    // ["api", "v1", "orgs", org, "projects", id, ...]
    let tail = &segments[5..];
    match tail {
        [] => super::users::handle_projects(state, req, None),
        [id] => super::users::handle_projects(state, req, Some(id)),
        [id, "columns"] => super::users::handle_project_columns(state, req, id, None),
        [id, "columns", "move"] => super::users::handle_columns_move(state, req, id),
        [id, "columns", column_id] => {
            super::users::handle_project_column(state, req, id, column_id, None)
        }
        [id, "columns", column_id, "issues"] => {
            super::users::handle_column_issues(state, req, id, column_id)
        }
        [id, "columns", column_id, "issues", issue_id] => {
            super::users::handle_column_issue(state, req, id, column_id, issue_id)
        }
        [id, "columns", column_id, "default"] => {
            super::users::handle_column_default(state, req, id, column_id)
        }
        [id, "issues", issue_id, "move"] => {
            super::users::handle_project_issue_move(state, req, id, issue_id)
        }
        _ => unimplemented(req),
    }
}

/// `GET /api/v1/orgs` — every organization.
fn list(state: &AppState, req: &ApiRequest) -> Response {
    let items: Vec<dto::Organization> = state.orgs().iter().map(|o| o.to_dto()).collect();
    let pagination = Pagination {
        page: req.query_i64("page"),
        limit: req.query_i64("limit"),
    };
    let total = items.len();
    respond::page(pagination.apply(&items), &pagination, total)
}

/// `POST /api/v1/orgs`.
fn create(state: &AppState, req: &ApiRequest) -> Response {
    let username = req
        .body_str("username")
        .or_else(|| req.body_str("name"))
        .unwrap_or_default();
    if username.trim().is_empty() {
        return ApiError::unprocessable("username is required").into_response();
    }
    if state.org(&username).is_some() {
        return ApiError::conflict("organization already exists").into_response();
    }
    let creator = super::users::current_user(state, req).map(|u| u.login);
    let org = StoredOrg {
        id: state.next_id(),
        name: username.clone(),
        full_name: req
            .body_str("full_name")
            .unwrap_or_else(|| username.clone()),
        description: req.body_str("description").unwrap_or_default(),
        email: req.body_str("email").unwrap_or_default(),
        location: req.body_str("location").unwrap_or_default(),
        website: req.body_str("website").unwrap_or_default(),
        visibility: req
            .body_str("visibility")
            .unwrap_or_else(|| "public".into()),
        created: now(),
        members: creator.into_iter().collect(),
    };
    let dto = org.to_dto();
    state.orgs_mut().push(org);
    respond::created(dto)
}

/// `PATCH /api/v1/orgs/{org}`.
fn update(state: &AppState, req: &ApiRequest, org: &StoredOrg) -> Response {
    let body = req.json();
    let mut orgs = state.orgs_mut();
    let Some(stored) = orgs
        .iter_mut()
        .find(|o| o.name.eq_ignore_ascii_case(&org.name))
    else {
        return ApiError::not_found("organization not found").into_response();
    };
    if let Some(full_name) = body.get("full_name").and_then(|v| v.as_str()) {
        stored.full_name = full_name.to_string();
    }
    if let Some(description) = body.get("description").and_then(|v| v.as_str()) {
        stored.description = description.to_string();
    }
    if let Some(location) = body.get("location").and_then(|v| v.as_str()) {
        stored.location = location.to_string();
    }
    if let Some(website) = body.get("website").and_then(|v| v.as_str()) {
        stored.website = website.to_string();
    }
    if let Some(visibility) = body.get("visibility").and_then(|v| v.as_str()) {
        stored.visibility = visibility.to_string();
    }
    respond::ok(stored.to_dto())
}

/// `POST /api/v1/orgs/{org}/rename` — change the organization's login name.
fn rename(state: &AppState, req: &ApiRequest, org: &StoredOrg) -> Response {
    if req.method != Method::POST {
        return unimplemented(req);
    }
    let Some(new_name) = req.body_str("new_name").filter(|n| !n.trim().is_empty()) else {
        return ApiError::unprocessable("new_name is required").into_response();
    };
    if state.org(&new_name).is_some() {
        return ApiError::conflict("organization already exists").into_response();
    }
    let old = org.name.clone();
    {
        let mut orgs = state.orgs_mut();
        let Some(stored) = orgs.iter_mut().find(|o| o.name.eq_ignore_ascii_case(&old)) else {
            return ApiError::not_found("organization not found").into_response();
        };
        stored.name = new_name.clone();
    }
    // Repositories owned by the organization follow the rename.
    for repo in state.repos_mut().iter_mut() {
        if repo.owner.eq_ignore_ascii_case(&old) {
            repo.owner = new_name.clone();
        }
    }
    for team in state.teams_mut().iter_mut() {
        if team.org.eq_ignore_ascii_case(&old) {
            team.org = new_name.clone();
        }
    }
    respond::no_content()
}

/// `GET|PUT|DELETE /api/v1/orgs/{org}/members/{username}`.
fn member_detail(state: &AppState, req: &ApiRequest, org: &StoredOrg, login: &str) -> Response {
    let is_member = org.members.iter().any(|m| m.eq_ignore_ascii_case(login));
    match req.method {
        Method::GET => {
            if !is_member {
                return ApiError::not_found("user is not a member").into_response();
            }
            match state.user(login) {
                // Gitea answers 204 for a member lookup, not a user object.
                Some(_) => respond::no_content(),
                None => ApiError::not_found("user not found").into_response(),
            }
        }
        Method::PUT => {
            if state.user(login).is_none() {
                return ApiError::not_found("user not found").into_response();
            }
            if let Some(stored) = state
                .orgs_mut()
                .iter_mut()
                .find(|o| o.name.eq_ignore_ascii_case(&org.name))
            {
                if !stored.members.iter().any(|m| m.eq_ignore_ascii_case(login)) {
                    stored.members.push(login.to_string());
                }
            }
            respond::no_content()
        }
        Method::DELETE => {
            if let Some(stored) = state
                .orgs_mut()
                .iter_mut()
                .find(|o| o.name.eq_ignore_ascii_case(&org.name))
            {
                stored.members.retain(|m| !m.eq_ignore_ascii_case(login));
            }
            respond::no_content()
        }
        _ => unimplemented(req),
    }
}

/// `GET|PATCH|DELETE /api/v1/orgs/{org}/teams/{team}`.
fn team_detail(state: &AppState, req: &ApiRequest, org: &StoredOrg, name: &str) -> Response {
    let id: i64 = name.parse().unwrap_or_default();
    fn matches(team: &StoredTeam, org: &str, id: i64, name: &str) -> bool {
        team.org.eq_ignore_ascii_case(org) && (team.id == id || team.name == name)
    }

    match req.method {
        Method::GET => match state
            .teams()
            .iter()
            .find(|t| matches(t, &org.name, id, name))
        {
            Some(team) => respond::ok(team.to_dto(org)),
            None => ApiError::not_found("team not found").into_response(),
        },
        Method::PATCH => {
            let body = req.json();
            let mut teams = state.teams_mut();
            match teams.iter_mut().find(|t| matches(t, &org.name, id, name)) {
                Some(team) => {
                    if let Some(new_name) = body.get("name").and_then(|v| v.as_str()) {
                        team.name = new_name.to_string();
                    }
                    if let Some(description) = body.get("description").and_then(|v| v.as_str()) {
                        team.description = description.to_string();
                    }
                    if let Some(permission) = body.get("permission").and_then(|v| v.as_str()) {
                        team.permission = permission.to_string();
                    }
                    respond::ok(team.to_dto(org))
                }
                None => ApiError::not_found("team not found").into_response(),
            }
        }
        Method::DELETE => {
            let before = state.teams().len();
            state
                .teams_mut()
                .retain(|t| !matches(t, &org.name, id, name));
            if state.teams().len() == before {
                return ApiError::not_found("team not found").into_response();
            }
            respond::no_content()
        }
        _ => unimplemented(req),
    }
}

/// `POST /api/v1/orgs/{org}/teams`.
fn create_team(state: &AppState, req: &ApiRequest, org: &StoredOrg) -> Response {
    let name = req.body_str("name").unwrap_or_default();
    if name.trim().is_empty() {
        return ApiError::unprocessable("name is required").into_response();
    }
    if state
        .teams()
        .iter()
        .any(|t| t.org.eq_ignore_ascii_case(&org.name) && t.name == name)
    {
        return ApiError::conflict("team already exists").into_response();
    }
    let team = StoredTeam {
        id: state.next_id(),
        org: org.name.clone(),
        name,
        description: req.body_str("description").unwrap_or_default(),
        permission: req.body_str("permission").unwrap_or_else(|| "read".into()),
        members: Vec::new(),
        units: req.body_str_vec("units").unwrap_or_default(),
    };
    let dto = team.to_dto(org);
    state.teams_mut().push(team);
    respond::created(dto)
}

/// `GET|POST /api/v1/teams/{id}`.
pub fn handle_team(state: &AppState, req: &ApiRequest) -> Response {
    let id: i64 = req
        .param("id")
        .and_then(|v| v.parse().ok())
        .unwrap_or_default();
    let Some(team) = state.teams().iter().find(|t| t.id == id).cloned() else {
        return ApiError::not_found("team not found").into_response();
    };
    let org = state.org(&team.org).unwrap_or_default();
    let segments: Vec<&str> = req.path.trim_matches('/').split('/').collect();
    // ["api", "v1", "teams", id, ...]
    let tail = &segments[4..];

    match tail {
        [] => match req.method {
            Method::GET => respond::ok(team.to_dto(&org)),
            Method::DELETE => {
                state.teams_mut().retain(|t| t.id != id);
                respond::no_content()
            }
            _ => unimplemented(req),
        },
        ["members"] => respond::ok(members_of_team(state, &team)),
        ["members", login] => match req.method {
            Method::PUT => {
                if let Some(stored) = state.teams_mut().iter_mut().find(|t| t.id == id) {
                    if !stored.members.iter().any(|m| m.eq_ignore_ascii_case(login)) {
                        stored.members.push(login.to_string());
                    }
                }
                respond::no_content()
            }
            Method::DELETE => {
                if let Some(stored) = state.teams_mut().iter_mut().find(|t| t.id == id) {
                    stored.members.retain(|m| !m.eq_ignore_ascii_case(login));
                }
                respond::no_content()
            }
            _ => unimplemented(req),
        },
        ["repos"] => match req.method {
            Method::GET => {
                // A team sees the repositories of its organization. The demo
                // store grants access at organization scope, so every org repo
                // is in the list.
                let mut repos = repos_of(state, &org.name);
                repos.sort_by(|a, b| a.name.cmp(&b.name));
                respond::ok(repos)
            }
            Method::DELETE => {
                let before = state.repos().len();
                state
                    .repos_mut()
                    .retain(|r| !r.owner.eq_ignore_ascii_case(&org.name));
                if state.repos().len() == before {
                    return ApiError::not_found("organization has no repositories").into_response();
                }
                respond::no_content()
            }
            _ => unimplemented(req),
        },
        _ => unimplemented(req),
    }
}

/// Repositories owned by an organization.
fn repos_of(state: &AppState, org: &str) -> Vec<dto::Repository> {
    state
        .repos()
        .iter()
        .filter(|r| r.owner.eq_ignore_ascii_case(org))
        .map(|r| r.to_dto())
        .collect()
}

/// Organization members as users.
fn members(state: &AppState, org: &StoredOrg) -> Vec<dto::User> {
    org.members
        .iter()
        .filter_map(|login| state.user(login).map(|u| u.to_dto()))
        .collect()
}

/// Teams belonging to an organization.
fn teams_of(state: &AppState, org: &str) -> Vec<dto::Team> {
    let Some(organization) = state.org(org) else {
        return Vec::new();
    };
    state
        .teams()
        .iter()
        .filter(|t| t.org.eq_ignore_ascii_case(org))
        .map(|t| t.to_dto(&organization))
        .collect()
}

/// Members of a team as users.
fn members_of_team(state: &AppState, team: &StoredTeam) -> Vec<dto::User> {
    team.members
        .iter()
        .filter_map(|login| state.user(login).map(|u| u.to_dto()))
        .collect()
}

/// Documented fallback for operations this module does not implement yet.
fn unimplemented(req: &ApiRequest) -> Response {
    super::dispatch::fallback(req)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::StoredUser;
    use axum::body::Bytes;
    use axum::http::StatusCode;

    fn state() -> AppState {
        let state = AppState::default();
        state.users_mut().push(StoredUser {
            id: 1,
            login: "alice".into(),
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
    fn creates_an_organization_with_the_creator_as_member() {
        let state = state();
        let response = create(
            &state,
            &req(Method::POST, "/api/v1/orgs", r#"{"username":"codez-org"}"#),
        );
        assert_eq!(response.status(), StatusCode::CREATED);
        let org = state.org("codez-org").expect("org stored");
        assert_eq!(org.members, vec!["alice"]);
    }

    #[test]
    fn rejects_a_duplicate_organization() {
        let state = state();
        let body = r#"{"username":"codez-org"}"#;
        create(&state, &req(Method::POST, "/api/v1/orgs", body));
        let response = create(&state, &req(Method::POST, "/api/v1/orgs", body));
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn membership_is_idempotent() {
        let state = state();
        create(
            &state,
            &req(Method::POST, "/api/v1/orgs", r#"{"username":"codez-org"}"#),
        );
        let org = state.org("codez-org").unwrap();
        for _ in 0..2 {
            member_detail(
                &state,
                &req(Method::PUT, "/api/v1/orgs/codez-org/members/alice", ""),
                &org,
                "alice",
            );
        }
        assert_eq!(state.org("codez-org").unwrap().members.len(), 1);
    }

    #[test]
    fn teams_are_scoped_to_their_organization() {
        let state = state();
        create(
            &state,
            &req(Method::POST, "/api/v1/orgs", r#"{"username":"codez-org"}"#),
        );
        let org = state.org("codez-org").unwrap();
        let response = create_team(
            &state,
            &req(
                Method::POST,
                "/api/v1/orgs/codez-org/teams",
                r#"{"name":"owners","permission":"owner"}"#,
            ),
            &org,
        );
        assert_eq!(response.status(), StatusCode::CREATED);
        assert_eq!(teams_of(&state, "codez-org").len(), 1);
        assert!(teams_of(&state, "other-org").is_empty());
    }

    #[test]
    fn removing_a_member_takes_effect() {
        let state = state();
        create(
            &state,
            &req(Method::POST, "/api/v1/orgs", r#"{"username":"codez-org"}"#),
        );
        let org = state.org("codez-org").unwrap();
        member_detail(
            &state,
            &req(Method::DELETE, "/api/v1/orgs/codez-org/members/alice", ""),
            &org,
            "alice",
        );
        assert!(state.org("codez-org").unwrap().members.is_empty());
    }
}
