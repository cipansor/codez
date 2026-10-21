//! Repository endpoints: CRUD, branches, commits, contents, releases, tags
//! and the repository-scoped social actions.

use axum::http::Method;
use axum::response::{IntoResponse, Response};

use shared::models as dto;

use crate::pagination::Pagination;
use crate::respond::{self, ApiError};
use crate::state::{
    now, short_sha, AppState, StoredAttachment, StoredBranch, StoredBranchProtection,
    StoredCollaborator, StoredCommit, StoredCommitStatus, StoredFile, StoredKey,
    StoredNotification, StoredPushMirror, StoredRelease, StoredRepo, StoredTagProtection,
};

use super::ApiRequest;

/// Route a `/api/v1/repos/*` request.
pub fn handle(state: &AppState, req: &ApiRequest) -> Response {
    let segments: Vec<&str> = req.path.trim_matches('/').split('/').collect();
    // ["api", "v1", "repos", ...]
    let rest = &segments[3..];

    match rest {
        ["search"] => search(state, req),
        ["issues", "search"] => super::issues::search_all(state, req),
        ["migrate"] => create(state, req),
        [] => match req.method {
            Method::POST => create(state, req),
            _ => unimplemented(req),
        },
        // The generate endpoint names its parameters `template_owner` and
        // `template_repo`, so it cannot go through `repo_spec()`.
        [template_owner, template_repo, "generate"] => {
            match state.repo(template_owner, template_repo) {
                Some(template) => generate(state, req, &template),
                None => ApiError::not_found("repository does not exist").into_response(),
            }
        }
        _ => match req.repo_spec() {
            Some((owner, name)) => scoped(state, req, &owner, &name),
            None => unimplemented(req),
        },
    }
}

/// Endpoints below `/api/v1/repos/{owner}/{repo}`.
fn scoped(state: &AppState, req: &ApiRequest, owner: &str, name: &str) -> Response {
    let Some(repo) = state.repo(owner, name) else {
        return ApiError::not_found("repository does not exist").into_response();
    };
    let segments: Vec<&str> = req.path.trim_matches('/').split('/').collect();
    // ["api", "v1", "repos", owner, repo, ...]
    let tail = &segments[5..];

    match tail {
        [] => match req.method {
            Method::GET => respond::ok(repo.to_dto()),
            Method::PATCH => update(state, req, &repo),
            Method::DELETE => {
                state.repos_mut().retain(|r| r.id != repo.id);
                respond::no_content()
            }
            _ => unimplemented(req),
        },
        ["branches"] => branches(state, req, &repo),
        ["branches", branch] => branch_detail(state, req, &repo, branch),
        ["branch_protections"] => branch_protections(state, req, &repo),
        ["branch_protections", "priority"] => branch_protection_priority(state, req, &repo),
        ["branch_protections", name] => branch_protection(state, req, &repo, name),
        ["editorconfig", rest @ ..] => editorconfig(state, &repo, &rest.join("/")),
        ["contents-ext", rest @ ..] => contents_ext(state, req, &repo, &rest.join("/")),
        ["file-contents"] => file_contents(state, req, &repo),
        ["tag_protections"] => tag_protections(state, req, &repo),
        ["tag_protections", id] => tag_protection(state, req, &repo, id),
        ["push_mirrors"] => push_mirrors(state, req, &repo),
        ["push_mirrors-sync"] => respond::no_content(),
        ["push_mirrors", name] => push_mirror(state, req, &repo, name),
        ["activities", "feeds"] => respond::ok(Vec::<serde_json::Value>::new()),
        ["compare", basehead] => compare(state, &repo, basehead),
        ["diffpatch"] => diffpatch(state, req, &repo),
        ["reviewers"] => respond::ok(reviewers(state, req, &repo)),
        ["statuses", sha] => statuses(state, req, &repo, sha),
        ["times", user] => repo_times(state, req, &repo, user),
        ["commits"] => commits(state, req, &repo),
        ["commits", sha] => commit_detail(state, req, &repo, sha),
        ["commits", sha, "status"] => combined_status(state, req, &repo, sha),
        ["commits", sha, "statuses"] => commit_statuses(state, req, &repo, sha),
        ["commits", sha, "pull"] => commits_pull(state, req, &repo, sha),
        ["git", "refs"] => git_refs(state, &repo),
        ["git", "refs", rest @ ..] => git_ref(state, &repo, &rest.join("/")),
        ["git", "blobs", sha] => git_blob(state, &repo, sha),
        ["git", "trees", sha] => git_tree(state, &repo, sha),
        ["git", "commits", sha] => commit_detail(state, req, &repo, sha),
        ["git", "tags", sha] => git_tag(state, &repo, sha),
        ["git", "notes", sha] => git_notes(state, &repo, sha),
        ["licenses"] => respond::ok(licenses()),
        ["signing-key.gpg"] => respond::ok(String::new()),
        ["signing-key.pub"] => respond::ok(signing_key_pub()),
        ["avatar"] => respond::no_content(),
        ["new_pin_allowed"] => respond::ok(serde_json::json!({ "allowed": true })),
        ["merge-upstream"] => respond::ok(serde_json::json!({ "merged": true })),
        ["mirror-sync"] => respond::no_content(),
        ["notifications"] => repo_notifications(state, req, &repo),
        ["transfer", action @ ("accept" | "reject")] => {
            transfer_decision(state, req, &repo, action)
        }
        ["contents", ..] => contents(state, req, &repo),
        ["releases"] => releases(state, req, &repo),
        ["releases", "tags", rest @ ..] => release_tag(state, req, &repo, &rest.join("/")),
        ["releases", id, "assets"] => release_assets(state, req, &repo, id),
        ["releases", id, "assets", attachment] => release_asset(state, req, &repo, id, attachment),
        ["releases", tag] => release_detail(state, req, &repo, tag),
        ["tags"] => tags(state, req, &repo),
        ["tags", rest @ ..] => tag_detail(state, req, &repo, &rest.join("/")),
        ["actions", ..] => super::actions::handle(state, req),
        ["wiki", ..] => super::wiki::handle(state, req, &repo),
        ["issue_config"] => issue_config(state, req, &repo),
        ["issue_config", "validate"] => issue_config_validate(state, &repo),
        ["issue_templates"] => respond::ok(Vec::<dto::IssueTemplate>::new()),
        ["issues", ..] => super::issues::handle_issues(state, req, &repo),
        ["pulls", ..] => super::issues::handle_pulls(state, req, &repo),
        ["labels", ..] => super::issues::handle_labels(state, req, &repo),
        ["milestones", ..] => super::issues::handle_milestones(state, req, &repo),
        ["projects"] => super::users::handle_projects(state, req, None),
        ["projects", ..] => project_subresource(state, req),
        ["hooks"] => super::users::handle_repo_hooks(state, req, &repo),
        ["hooks", _id] => super::users::handle_repo_hooks(state, req, &repo),
        ["hooks", "git", id] => git_hook(state, req, &repo, id),
        ["hooks", _id, "tests"] => respond::no_content(),
        ["teams"] => respond::ok(teams_of_repo(state, &repo)),
        ["teams", team] => repo_team_detail(state, req, &repo, team),
        ["assignees"] => respond::ok(assignees(state, &repo)),
        ["assignees", login] => match state.user(login) {
            Some(user) => respond::ok(user.to_dto()),
            None => ApiError::not_found("user not found").into_response(),
        },
        ["collaborators"] => match req.method {
            Method::GET => {
                let mut users: Vec<dto::User> = state
                    .collaborators()
                    .iter()
                    .filter(|c| c.repo_id == repo.id)
                    .filter_map(|c| state.user(&c.login).map(|u| u.to_dto()))
                    .collect();
                users.sort_by(|a, b| a.login.cmp(&b.login));
                respond::ok(users)
            }
            _ => unimplemented(req),
        },
        ["collaborators", login] => match req.method {
            Method::PUT => {
                let permission = req.body_str("permission").unwrap_or_else(|| "write".into());
                let mut collaborators = state.collaborators_mut();
                match collaborators
                    .iter_mut()
                    .find(|c| c.repo_id == repo.id && c.login.eq_ignore_ascii_case(login))
                {
                    Some(existing) => existing.permission = permission,
                    None => collaborators.push(StoredCollaborator {
                        repo_id: repo.id,
                        login: login.to_string(),
                        permission,
                    }),
                }
                respond::no_content()
            }
            Method::DELETE => {
                state
                    .collaborators_mut()
                    .retain(|c| !(c.repo_id == repo.id && c.login.eq_ignore_ascii_case(login)));
                respond::no_content()
            }
            _ => match state.user(login) {
                Some(user) => respond::ok(user.to_dto()),
                None => ApiError::not_found("user not found").into_response(),
            },
        },
        ["collaborators", login, "permission"] => match state.user(login) {
            Some(user) => {
                let permission = state
                    .collaborators()
                    .iter()
                    .find(|c| c.repo_id == repo.id && c.login.eq_ignore_ascii_case(login))
                    .map(|c| c.permission.clone())
                    .unwrap_or_else(|| "write".into());
                respond::ok(serde_json::json!({
                    "user": user.to_dto(),
                    "permission": permission,
                    "role_name": permission,
                }))
            }
            None => ApiError::not_found("user not found").into_response(),
        },
        ["keys"] => deploy_keys(state, req, &repo),
        ["keys", "tokens"] => respond::created(serde_json::json!({ "id": state.next_id() })),
        ["keys", id] => deploy_key(state, req, &repo, id),
        ["languages"] => respond::ok(languages(&repo)),
        ["topics"] => topics(state, req, &repo),
        ["topics", topic] => topic_detail(state, req, &repo, topic),
        ["stargazers"] => respond::ok(stargazers(state, &repo)),
        ["subscribers"] => respond::ok(subscribers(state, &repo)),
        ["contributors"] => respond::ok(contributors(state, &repo)),
        ["forks"] => match req.method {
            Method::POST => fork(state, req, &repo),
            _ => respond::ok(forks(state, &repo)),
        },
        ["times"] => respond::ok(repo_tracked_times(state, &repo)),
        ["stopwatches"] => respond::ok(Vec::<serde_json::Value>::new()),
        ["transfer"] => transfer(state, req, &repo),
        ["subscription"] => watch(state, req, &repo),
        ["media", rest @ ..] => media(state, req, &repo, &rest.join("/")),
        ["raw", rest @ ..] => media(state, req, &repo, &rest.join("/")),
        ["archive", rest @ ..] => archive(state, req, &repo, &rest.join("/")),
        ["pulse"] => respond::ok(pulse(&repo)),
        ["activity", "feeds"] => respond::ok(Vec::<serde_json::Value>::new()),
        _ => unimplemented(req),
    }
}

/// `/api/v1/repos/{owner}/{repo}/projects/{id}/…` — the project id is the
/// first segment after `projects`, and the rest maps onto the shared column
/// and issue helpers the user-scoped routes use.
fn project_subresource(state: &AppState, req: &ApiRequest) -> Response {
    let segments: Vec<&str> = req.path.trim_matches('/').split('/').collect();
    // ["api", "v1", "repos", owner, repo, "projects", id, ...]
    let tail = &segments[6..];
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

/// Teams that grant access to `repo`. The store has no explicit
/// repository-team link, so a team qualifies when it includes all repositories
/// or lists the repository's owner among its members.
fn teams_of_repo(state: &AppState, repo: &StoredRepo) -> Vec<dto::Team> {
    let Some(org) = state.org(&repo.owner) else {
        return Vec::new();
    };
    state
        .teams()
        .iter()
        .filter(|t| {
            t.org.eq_ignore_ascii_case(&repo.owner)
                && (t.permission == "owner"
                    || t.permission == "admin"
                    || t.members
                        .iter()
                        .any(|m| m.eq_ignore_ascii_case(&repo.owner)))
        })
        .map(|t| t.to_dto(&org))
        .collect()
}

/// `GET|PUT|DELETE /api/v1/repos/{owner}/{repo}/teams/{team}`.
fn repo_team_detail(state: &AppState, req: &ApiRequest, repo: &StoredRepo, team: &str) -> Response {
    let Some(org) = state.org(&repo.owner) else {
        return ApiError::not_found("organization not found").into_response();
    };
    let team = team.parse::<i64>().unwrap_or_default();
    let found = state
        .teams()
        .iter()
        .find(|t| t.id == team && t.org.eq_ignore_ascii_case(&repo.owner))
        .cloned();
    match (req.method.clone(), found) {
        (Method::GET, Some(t)) => respond::ok(t.to_dto(&org)),
        (Method::GET, None) => ApiError::not_found("team not found").into_response(),
        // Membership is implicit in the team's permission, so a grant or revoke
        // is accepted without a separate repository-team edge.
        (Method::PUT, Some(t)) => respond::ok(t.to_dto(&org)),
        (Method::PUT, None) => ApiError::not_found("team not found").into_response(),
        (Method::DELETE, Some(_)) => respond::no_content(),
        (Method::DELETE, None) => ApiError::not_found("team not found").into_response(),
        _ => unimplemented(req),
    }
}

/// Users who can be assigned to an issue: everyone with access to the
/// repository, which the demo store models as the owner plus the watchers.
fn assignees(state: &AppState, repo: &StoredRepo) -> Vec<dto::User> {
    let mut logins = vec![repo.owner.clone()];
    logins.extend(repo.watchers.iter().cloned());
    let mut seen = std::collections::HashSet::new();
    logins
        .into_iter()
        .filter(|l| seen.insert(l.to_lowercase()))
        .filter_map(|l| state.user(&l).map(|u| u.to_dto()))
        .collect()
}

/// `GET|POST /api/v1/repos/search`.
pub fn search(state: &AppState, req: &ApiRequest) -> Response {
    let term = req.query_param("q").unwrap_or_default().to_lowercase();
    let owner = req.query_param("owner").unwrap_or_default().to_lowercase();

    let mut matches: Vec<StoredRepo> = state
        .repos()
        .iter()
        .filter(|r| {
            let haystack = format!("{} {}", r.name, r.description).to_lowercase();
            (term.is_empty() || haystack.contains(&term))
                && (owner.is_empty() || r.owner.to_lowercase() == owner)
        })
        .cloned()
        .collect();

    match req.query_param("sort").as_deref() {
        Some("updated") => matches.sort_by_key(|r| std::cmp::Reverse(r.updated.clone())),
        Some("stars") => matches.sort_by_key(|r| std::cmp::Reverse(r.stars.len())),
        _ => matches.sort_by_key(|r| r.full_name()),
    }

    let pagination = pagination(req);
    let total = matches.len();
    let items: Vec<dto::Repository> = pagination
        .apply(&matches)
        .into_iter()
        .map(|r| r.to_dto())
        .collect();
    // Gitea wraps repository search results in `SearchResults`, not a bare array.
    respond::paged(
        dto::SearchResults {
            ok: Some(true),
            data: Some(items),
        },
        &pagination,
        total,
    )
}

/// `GET /api/v1/repositories/{id}` — look a repository up by numeric id.
pub fn handle_by_id(state: &AppState, req: &ApiRequest) -> Response {
    let Some(raw_id) = req.param("id") else {
        return ApiError::not_found("repository does not exist").into_response();
    };
    let id: i64 = raw_id.parse().unwrap_or_default();
    match state.repo_by_id(id) {
        Some(repo) if req.method == Method::GET => respond::ok(repo.to_dto()),
        Some(_) => ApiError::not_found("repository does not exist").into_response(),
        None => ApiError::not_found("repository does not exist").into_response(),
    }
}

/// `POST /api/v1/repos/{template_owner}/{template_repo}/generate`.
///
/// Clones the template's metadata into a new repository owned by the caller.
/// Content copying is out of scope for the in-memory store, so the new
/// repository starts empty like a fresh non-`auto_init` repository.
fn generate(state: &AppState, req: &ApiRequest, template: &StoredRepo) -> Response {
    if req.method != Method::POST {
        return unimplemented(req);
    }
    let name = req
        .body_str("name")
        .unwrap_or_else(|| format!("{}-copy", template.name));
    let owner = req
        .body_str("owner")
        .filter(|o| !o.is_empty())
        .or_else(|| super::users::current_user(state, req).map(|u| u.login))
        .unwrap_or_else(|| template.owner.clone());

    if state.repo(&owner, &name).is_some() {
        return ApiError::conflict("repository already exists").into_response();
    }

    let id = state.next_id();
    let repo = StoredRepo {
        id,
        owner,
        name: name.clone(),
        description: req
            .body_str("description")
            .unwrap_or_else(|| template.description.clone()),
        private: req.body_bool("private").unwrap_or(template.private),
        empty: true,
        default_branch: template.default_branch.clone(),
        topics: template.topics.clone(),
        created: now(),
        updated: now(),
        ..Default::default()
    };
    let dto = repo.to_dto();
    state.repos_mut().push(repo);
    respond::created(dto)
}

/// `POST /api/v1/user/repos` and `POST /api/v1/repos/migrate`.
///
/// Gitea answers `201 Created` with the new repository.
pub fn create(state: &AppState, req: &ApiRequest) -> Response {
    let option: dto::CreateRepoOption = match req.parse() {
        Ok(option) => option,
        Err(error) => return error.into_response(),
    };
    if option.name.trim().is_empty() {
        return ApiError::unprocessable("name is required").into_response();
    }

    // An organization route names the owner in the path (`/orgs/{org}/repos`,
    // and its deprecated `/org/{org}/repos` alias), which outranks the body.
    let owner = req
        .param("org")
        .filter(|o| !o.is_empty())
        .or_else(|| req.body_str("owner").filter(|o| !o.is_empty()))
        .or_else(|| super::users::current_user(state, req).map(|u| u.login))
        .unwrap_or_else(|| "alice".into());

    if state.repo(&owner, &option.name).is_some() {
        return ApiError::conflict("repository already exists").into_response();
    }

    let default_branch = option
        .default_branch
        .clone()
        .unwrap_or_else(|| "main".into());
    let id = state.next_id();
    let auto_init = option.auto_init.unwrap_or(false);
    let repo = StoredRepo {
        id,
        owner,
        name: option.name.clone(),
        description: option.description.clone().unwrap_or_default(),
        private: option.private.unwrap_or(false),
        empty: !auto_init,
        default_branch: default_branch.clone(),
        created: now(),
        updated: now(),
        website: req.body_str("clone_addr").unwrap_or_default(),
        ..Default::default()
    };

    if auto_init {
        let sha = short_sha(&format!("{}:{id}", repo.full_name()));
        state.files_mut().push(StoredFile {
            repo_id: id,
            branch: default_branch.clone(),
            path: "README.md".into(),
            content: format!("# {}\n", repo.name),
            sha: short_sha(&format!("readme:{id}")),
            is_dir: false,
            updated: now(),
        });
        state.branches_mut().push(StoredBranch {
            repo_id: id,
            name: default_branch,
            protected: false,
            sha: sha.clone(),
            from: None,
        });
        state.commits_mut().push(StoredCommit {
            repo_id: id,
            sha,
            message: "Initial commit".into(),
            author_name: repo.owner.clone(),
            author_email: format!("{}@localhost", repo.owner),
            created: now(),
            files: vec![("README.md".into(), 1, 0)],
        });
    }

    let dto = repo.to_dto();
    state.repos_mut().push(repo);
    respond::created(dto)
}

/// `PATCH /api/v1/repos/{owner}/{repo}`.
fn update(state: &AppState, req: &ApiRequest, repo: &StoredRepo) -> Response {
    let body = req.json();
    let mut repos = state.repos_mut();
    let Some(stored) = repos.iter_mut().find(|r| r.id == repo.id) else {
        return ApiError::not_found("repository does not exist").into_response();
    };
    if let Some(name) = body.get("name").and_then(|v| v.as_str()) {
        if !name.is_empty() {
            stored.name = name.to_string();
        }
    }
    if let Some(description) = body.get("description").and_then(|v| v.as_str()) {
        stored.description = description.to_string();
    }
    if let Some(private) = body.get("private").and_then(|v| v.as_bool()) {
        stored.private = private;
    }
    if let Some(website) = body.get("website").and_then(|v| v.as_str()) {
        stored.website = website.to_string();
    }
    if let Some(default_branch) = body.get("default_branch").and_then(|v| v.as_str()) {
        stored.default_branch = default_branch.to_string();
    }
    if let Some(archived) = body.get("archived").and_then(|v| v.as_bool()) {
        stored.archived = archived;
    }
    if let Some(topics) = body.get("topics").and_then(|v| v.as_array()) {
        stored.topics = topics
            .iter()
            .filter_map(|t| t.as_str().map(str::to_string))
            .collect();
    }
    stored.updated = now();
    respond::ok(stored.to_dto())
}

/// `GET|POST /api/v1/repos/{owner}/{repo}/branches`.
fn branches(state: &AppState, req: &ApiRequest, repo: &StoredRepo) -> Response {
    match req.method {
        Method::GET => {
            let items: Vec<dto::Branch> = state
                .branches()
                .iter()
                .filter(|b| b.repo_id == repo.id)
                .map(|b| b.to_dto(commit_for(state, repo, &b.sha)))
                .collect();
            respond::ok(items)
        }
        Method::POST => {
            let Some(name) = req
                .body_str("new_branch_name")
                .or_else(|| req.body_str("branch_name"))
            else {
                return ApiError::unprocessable("new_branch_name is required").into_response();
            };
            if state
                .branches()
                .iter()
                .any(|b| b.repo_id == repo.id && b.name == name)
            {
                return ApiError::conflict("branch already exists").into_response();
            }
            let from = req
                .body_str("old_branch_name")
                .unwrap_or_else(|| repo.default_branch.clone());
            let sha = short_sha(&format!("{}:{}:{}", repo.id, name, from));
            let branch = StoredBranch {
                repo_id: repo.id,
                name,
                protected: false,
                sha: sha.clone(),
                from: Some(from),
            };
            let dto = branch.to_dto(commit_for(state, repo, &sha));
            state.branches_mut().push(branch);
            respond::created(dto)
        }
        _ => unimplemented(req),
    }
}

/// `GET|PATCH|DELETE /api/v1/repos/{owner}/{repo}/branches/{branch}`.
fn branch_detail(state: &AppState, req: &ApiRequest, repo: &StoredRepo, name: &str) -> Response {
    match req.method {
        Method::GET => match state
            .branches()
            .iter()
            .find(|b| b.repo_id == repo.id && b.name == name)
        {
            Some(branch) => respond::ok(branch.to_dto(commit_for(state, repo, &branch.sha))),
            None => ApiError::not_found("branch not found").into_response(),
        },
        Method::DELETE => {
            let before = state.branches().len();
            state
                .branches_mut()
                .retain(|b| !(b.repo_id == repo.id && b.name == name));
            if state.branches().len() == before {
                return ApiError::not_found("branch not found").into_response();
            }
            respond::no_content()
        }
        Method::PATCH => {
            let protected = req.body_bool("protected");
            let mut branches = state.branches_mut();
            match branches
                .iter_mut()
                .find(|b| b.repo_id == repo.id && b.name == name)
            {
                Some(branch) => {
                    if let Some(protected) = protected {
                        branch.protected = protected;
                    }
                    let sha = branch.sha.clone();
                    let dto = branch.to_dto(commit_for(state, repo, &sha));
                    respond::ok(dto)
                }
                None => ApiError::not_found("branch not found").into_response(),
            }
        }
        Method::PUT => {
            // Rename a branch: the body carries the new `name`. The demo store
            // has no git objects, so only the branch record moves.
            let Some(new_name) = req.body_str("name") else {
                return ApiError::unprocessable("name is required").into_response();
            };
            let mut branches = state.branches_mut();
            if branches
                .iter()
                .any(|b| b.repo_id == repo.id && b.name == new_name)
            {
                return ApiError::conflict("branch already exists").into_response();
            }
            match branches
                .iter_mut()
                .find(|b| b.repo_id == repo.id && b.name == name)
            {
                Some(branch) => {
                    branch.name = new_name;
                    let sha = branch.sha.clone();
                    let dto = branch.to_dto(commit_for(state, repo, &sha));
                    respond::created(dto)
                }
                None => ApiError::not_found("branch not found").into_response(),
            }
        }
        _ => unimplemented(req),
    }
}

/// `GET /api/v1/repos/{owner}/{repo}/commits`.
fn commits(state: &AppState, req: &ApiRequest, repo: &StoredRepo) -> Response {
    let mut items: Vec<StoredCommit> = state
        .commits()
        .iter()
        .filter(|c| c.repo_id == repo.id)
        .cloned()
        .collect();
    items.sort_by(|a, b| b.created.cmp(&a.created));

    let pagination = pagination(req);
    let total = items.len();
    let page: Vec<dto::Commit> = pagination
        .apply(&items)
        .into_iter()
        .map(|c| c.to_dto(repo))
        .collect();
    respond::page(page, &pagination, total)
}

/// `GET /api/v1/repos/{owner}/{repo}/commits/{sha}/status`.
fn combined_status(state: &AppState, req: &ApiRequest, repo: &StoredRepo, sha: &str) -> Response {
    let _ = req;
    let Some(commit) = resolve_commit(state, repo, sha) else {
        return ApiError::not_found("commit not found").into_response();
    };
    let statuses = statuses_for(state, repo, &commit.sha);
    respond::ok(serde_json::json!({
        "state": combined_state(&statuses),
        "sha": commit.sha,
        "total_count": statuses.len(),
        "statuses": statuses,
        "repository": repo.to_dto(),
    }))
}

/// The commit statuses recorded against `sha`, newest first.
fn statuses_for(state: &AppState, repo: &StoredRepo, sha: &str) -> Vec<dto::CommitStatus> {
    state
        .commit_statuses()
        .iter()
        .filter(|s| s.repo_id == repo.id && s.sha == sha)
        .map(|s| s.to_dto(repo))
        .collect()
}

/// Roll a set of statuses up the way Gitea does: a single failure or error
/// fails the commit, a pending one holds it, and an empty set is pending.
fn combined_state(statuses: &[dto::CommitStatus]) -> &'static str {
    let has =
        |want: &dto::CommitStatusState| statuses.iter().any(|s| s.status.as_ref() == Some(want));
    if has(&dto::CommitStatusState::Failure) || has(&dto::CommitStatusState::Error) {
        "failure"
    } else if statuses.is_empty() || has(&dto::CommitStatusState::Pending) {
        "pending"
    } else {
        "success"
    }
}

/// `GET /api/v1/repos/{owner}/{repo}/commits/{sha}/statuses`.
fn commit_statuses(state: &AppState, req: &ApiRequest, repo: &StoredRepo, sha: &str) -> Response {
    let _ = req;
    respond::ok(statuses_for(state, repo, sha))
}

/// `GET /api/v1/repos/{owner}/{repo}/commits/{sha}/pull` — the pull request a
/// commit belongs to, if any.
fn commits_pull(state: &AppState, req: &ApiRequest, repo: &StoredRepo, sha: &str) -> Response {
    let _ = req;
    let Some(commit) = resolve_commit(state, repo, sha) else {
        return ApiError::not_found("commit not found").into_response();
    };
    match state.issues().iter().find(|i| {
        i.repo_id == repo.id
            && i.is_pull
            && i.head.as_deref().map(|h| h == commit.sha).unwrap_or(false)
    }) {
        Some(pr) => {
            respond::ok(pr.to_pull_dto(repo, &super::issues::labels_of(state, repo.id, pr)))
        }
        None => ApiError::not_found("no pull request for this commit").into_response(),
    }
}

/// Resolve a commit by full or abbreviated sha.
fn resolve_commit(state: &AppState, repo: &StoredRepo, sha: &str) -> Option<StoredCommit> {
    state
        .commits()
        .iter()
        .find(|c| c.repo_id == repo.id && (c.sha == sha || c.sha.starts_with(sha)))
        .cloned()
}

/// `GET /api/v1/repos/{owner}/{repo}/git/refs` — every branch as a git
/// reference, the raw form of what `/branches` renders.
fn git_refs(state: &AppState, repo: &StoredRepo) -> Response {
    let mut refs: Vec<serde_json::Value> = state
        .branches()
        .iter()
        .filter(|b| b.repo_id == repo.id)
        .map(|b| git_ref_json(repo, b))
        .collect();
    refs.sort_by_key(|r| r["ref"].as_str().unwrap_or_default().to_string());
    respond::ok(refs)
}

/// A `GitRef` object for a branch, which the shared crate does not model.
fn git_ref_json(repo: &StoredRepo, branch: &StoredBranch) -> serde_json::Value {
    serde_json::json!({
        "ref": format!("refs/heads/{}", branch.name),
        "url": format!("/api/v1/repos/{}/git/refs/heads/{}", repo.full_name(), branch.name),
        "object": {
            "sha": branch.sha,
            "type": "commit",
            "url": format!("/api/v1/repos/{}/git/commits/{}", repo.full_name(), branch.sha),
        },
    })
}

/// `GET /api/v1/repos/{owner}/{repo}/git/refs/{ref}` — `heads/main` or
/// `heads%2Fmain` style references.
fn git_ref(state: &AppState, repo: &StoredRepo, reference: &str) -> Response {
    let name = reference
        .trim_start_matches('/')
        .trim_start_matches("heads/")
        .trim_start_matches("tags/");
    match state
        .branches()
        .iter()
        .find(|b| b.repo_id == repo.id && b.name == name)
    {
        Some(branch) => respond::ok(git_ref_json(repo, branch)),
        None => ApiError::not_found("reference not found").into_response(),
    }
}

/// `GET /api/v1/repos/{owner}/{repo}/git/blobs/{sha}` — a file's content.
fn git_blob(state: &AppState, repo: &StoredRepo, sha: &str) -> Response {
    match state
        .files()
        .iter()
        .find(|f| f.repo_id == repo.id && !f.is_dir && (f.sha == sha || f.sha.starts_with(sha)))
    {
        Some(file) => {
            let encoded = encode_base64(file.content.as_bytes());
            respond::ok(dto::GitBlobResponse {
                sha: Some(file.sha.clone()),
                size: Some(file.content.len() as i64),
                content: Some(encoded),
                encoding: Some("base64".into()),
                url: Some(format!(
                    "/api/v1/repos/{}/git/blobs/{}",
                    repo.full_name(),
                    file.sha
                )),
                ..Default::default()
            })
        }
        None => ApiError::not_found("blob not found").into_response(),
    }
}

/// `GET /api/v1/repos/{owner}/{repo}/git/trees/{sha}` — the repository tree.
fn git_tree(state: &AppState, repo: &StoredRepo, sha: &str) -> Response {
    let entries: Vec<dto::GitEntry> = state
        .files()
        .iter()
        .filter(|f| f.repo_id == repo.id)
        .map(|f| dto::GitEntry {
            path: Some(f.path.clone()),
            mode: Some(if f.is_dir { "040000" } else { "100644" }.into()),
            type_: Some(if f.is_dir { "tree" } else { "blob" }.into()),
            sha: Some(f.sha.clone()),
            size: Some(f.content.len() as i64),
            ..Default::default()
        })
        .collect();
    respond::ok(dto::GitTreeResponse {
        sha: Some(sha.to_string()),
        truncated: Some(false),
        url: Some(format!(
            "/api/v1/repos/{}/git/trees/{sha}",
            repo.full_name()
        )),
        total_count: Some(entries.len() as i64),
        tree: Some(entries),
        ..Default::default()
    })
}

/// `GET /api/v1/repos/{owner}/{repo}/git/tags/{sha}` — the tag object.
fn git_tag(state: &AppState, repo: &StoredRepo, sha: &str) -> Response {
    let _ = (state, repo);
    respond::ok(dto::AnnotatedTag {
        tag: Some(sha.to_string()),
        sha: Some(sha.to_string()),
        ..Default::default()
    })
}

/// `GET /api/v1/repos/{owner}/{repo}/git/notes/{sha}` — git notes attached to a
/// commit. The demo store keeps none, which is the documented `404`.
fn git_notes(state: &AppState, repo: &StoredRepo, sha: &str) -> Response {
    let _ = (state, repo, sha);
    ApiError::not_found("note not found").into_response()
}

/// The license names Gitea reports when it cannot bundle the full texts.
fn licenses() -> Vec<&'static str> {
    vec!["MIT", "Apache-2.0", "BSD-3-Clause", "GPL-3.0"]
}

/// A deterministic placeholder public key for the demo instance.
fn signing_key_pub() -> String {
    "-----BEGIN PUBLIC KEY-----\nCodez demo signing key\n-----END PUBLIC KEY-----\n".into()
}

/// `GET|PUT /api/v1/repos/{owner}/{repo}/notifications`.
fn repo_notifications(state: &AppState, req: &ApiRequest, repo: &StoredRepo) -> Response {
    let login = super::users::current_login(state, req);
    let mine: Vec<StoredNotification> = state
        .notifications()
        .iter()
        .filter(|n| n.repo_id == repo.id && n.user.eq_ignore_ascii_case(&login))
        .cloned()
        .collect();

    match req.method {
        Method::GET => {
            let items: Vec<dto::NotificationThread> =
                mine.iter().map(|n| n.to_dto(Some(repo))).collect();
            respond::ok(items)
        }
        // Marking a repository's notifications read touches only this repo's threads.
        Method::PUT => {
            for n in state
                .notifications_mut()
                .iter_mut()
                .filter(|n| n.repo_id == repo.id && n.user.eq_ignore_ascii_case(&login))
            {
                n.unread = false;
            }
            respond::no_content()
        }
        _ => unimplemented(req),
    }
}

/// `POST /api/v1/repos/{owner}/{repo}/transfer/{accept,reject}`.
fn transfer_decision(
    state: &AppState,
    req: &ApiRequest,
    repo: &StoredRepo,
    action: &str,
) -> Response {
    if req.method != Method::POST {
        return unimplemented(req);
    }
    if action == "reject" {
        return respond::no_content();
    }
    let Some(new_owner) = req.body_str("new_owner").filter(|s| !s.trim().is_empty()) else {
        return ApiError::unprocessable("new_owner is required").into_response();
    };
    let owner_exists = state.user(&new_owner).is_some() || state.org(&new_owner).is_some();
    if !owner_exists {
        return ApiError::not_found("new owner does not exist").into_response();
    }
    if let Some(stored) = state.repos_mut().iter_mut().find(|r| r.id == repo.id) {
        stored.owner = new_owner;
    }
    respond::no_content()
}

/// `GET /api/v1/repos/{owner}/{repo}/commits/{sha}`.
fn commit_detail(state: &AppState, req: &ApiRequest, repo: &StoredRepo, sha: &str) -> Response {
    let _ = req;
    match state
        .commits()
        .iter()
        .find(|c| c.repo_id == repo.id && (c.sha == sha || c.sha.starts_with(sha)))
    {
        Some(commit) => respond::ok(commit.to_dto(repo)),
        None => ApiError::not_found("commit not found").into_response(),
    }
}

/// `GET|POST|PUT|DELETE /api/v1/repos/{owner}/{repo}/contents/{filepath}`.
fn contents(state: &AppState, req: &ApiRequest, repo: &StoredRepo) -> Response {
    let segments: Vec<&str> = req.path.trim_matches('/').split('/').collect();
    let path = segments
        .iter()
        .skip(6)
        .copied()
        .collect::<Vec<_>>()
        .join("/");
    let path = req
        .query_param("filepath")
        .filter(|f| !f.is_empty())
        .unwrap_or(path);
    let reference = req
        .query_param("ref")
        .unwrap_or_else(|| repo.default_branch.clone());

    match req.method {
        Method::GET => {
            if path.is_empty() {
                let entries: Vec<dto::ContentsResponse> = state
                    .files()
                    .iter()
                    .filter(|f| f.repo_id == repo.id && f.branch == reference)
                    .map(|f| f.to_contents_dto(repo))
                    .collect();
                return respond::ok(entries);
            }
            match state
                .files()
                .iter()
                .find(|f| f.repo_id == repo.id && f.path == path)
            {
                Some(file) => respond::ok(file.to_contents_dto(repo)),
                None => ApiError::not_found("file does not exist").into_response(),
            }
        }
        Method::POST | Method::PUT => {
            let option: dto::CreateFileOptions = match req.parse() {
                Ok(option) => option,
                Err(error) => return error.into_response(),
            };
            let content = decode_base64(&option.content);
            let branch = option.branch.clone().unwrap_or(reference);
            let sha = short_sha(&format!("{}:{}:{}", repo.id, path, content));
            let mut files = state.files_mut();
            // Gitea answers `201 Created` for a new file and `200 OK` when the
            // write updates one that already exists.
            let created = match files
                .iter_mut()
                .find(|f| f.repo_id == repo.id && f.path == path)
            {
                Some(existing) => {
                    existing.content = content.clone();
                    existing.sha = sha.clone();
                    existing.updated = now();
                    false
                }
                None => {
                    files.push(StoredFile {
                        repo_id: repo.id,
                        branch: branch.clone(),
                        path: path.clone(),
                        content: content.clone(),
                        sha: sha.clone(),
                        is_dir: false,
                        updated: now(),
                    });
                    true
                }
            };
            drop(files);
            record_commit(
                state,
                repo,
                option.message.as_deref().unwrap_or(""),
                &path,
                &sha,
            );
            let file = StoredFile {
                repo_id: repo.id,
                branch,
                path,
                content,
                sha,
                is_dir: false,
                updated: now(),
            };
            let body = dto::FileResponse {
                content: Some(file.to_contents_dto(repo)),
                ..Default::default()
            };
            if created {
                respond::created(body)
            } else {
                respond::ok(body)
            }
        }
        Method::DELETE => {
            // Gitea's `DeleteFileOptions.sha` is required, but the port accepts
            // a body without it so a delete does not depend on the caller
            // having read the blob first.
            let option: dto::DeleteFileOptions = match req.parse_lenient() {
                Ok(option) => option,
                Err(error) => return error.into_response(),
            };
            let before = state.files().len();
            state
                .files_mut()
                .retain(|f| !(f.repo_id == repo.id && f.path == path));
            if state.files().len() == before {
                return ApiError::not_found("file does not exist").into_response();
            }
            record_commit(
                state,
                repo,
                option.message.as_deref().unwrap_or(""),
                &path,
                "",
            );
            respond::ok(dto::FileDeleteResponse::default())
        }
        _ => unimplemented(req),
    }
}

/// `GET|POST /api/v1/repos/{owner}/{repo}/releases`.
fn releases(state: &AppState, req: &ApiRequest, repo: &StoredRepo) -> Response {
    match req.method {
        Method::GET => {
            let items: Vec<dto::Release> = state
                .releases()
                .iter()
                .filter(|r| r.repo_id == repo.id)
                .map(|r| r.to_dto(repo))
                .collect();
            respond::ok(items)
        }
        Method::POST => {
            let option: dto::CreateReleaseOption = match req.parse() {
                Ok(option) => option,
                Err(error) => return error.into_response(),
            };
            if option.tag_name.trim().is_empty() {
                return ApiError::unprocessable("tag_name is required").into_response();
            }
            if state
                .releases()
                .iter()
                .any(|r| r.repo_id == repo.id && r.tag_name == option.tag_name)
            {
                return ApiError::conflict("release already exists").into_response();
            }
            let release = StoredRelease {
                id: state.next_id(),
                repo_id: repo.id,
                tag_name: option.tag_name.clone(),
                name: option
                    .name
                    .clone()
                    .unwrap_or_else(|| option.tag_name.clone()),
                body: option.body.clone().unwrap_or_default(),
                draft: option.draft.unwrap_or(false),
                prerelease: option.prerelease.unwrap_or(false),
                target: option
                    .target_commitish
                    .clone()
                    .unwrap_or_else(|| repo.default_branch.clone()),
                publisher: super::users::current_user(state, req)
                    .map(|u| u.login)
                    .unwrap_or_else(|| repo.owner.clone()),
                created: now(),
            };
            let dto = release.to_dto(repo);
            state.releases_mut().push(release);
            respond::created(dto)
        }
        _ => unimplemented(req),
    }
}

/// `GET|PATCH|DELETE /api/v1/repos/{owner}/{repo}/releases/{tag}`.
fn release_detail(state: &AppState, req: &ApiRequest, repo: &StoredRepo, tag: &str) -> Response {
    fn matches(release: &StoredRelease, repo_id: i64, tag: &str) -> bool {
        release.repo_id == repo_id && (release.tag_name == tag || release.id.to_string() == tag)
    }

    match req.method {
        Method::GET => match state.releases().iter().find(|r| matches(r, repo.id, tag)) {
            Some(release) => respond::ok(release.to_dto(repo)),
            None => ApiError::not_found("release not found").into_response(),
        },
        Method::PATCH => {
            let body = req.json();
            let mut releases = state.releases_mut();
            match releases.iter_mut().find(|r| matches(r, repo.id, tag)) {
                Some(release) => {
                    if let Some(name) = body.get("name").and_then(|v| v.as_str()) {
                        release.name = name.to_string();
                    }
                    if let Some(text) = body.get("body").and_then(|v| v.as_str()) {
                        release.body = text.to_string();
                    }
                    if let Some(draft) = body.get("draft").and_then(|v| v.as_bool()) {
                        release.draft = draft;
                    }
                    if let Some(prerelease) = body.get("prerelease").and_then(|v| v.as_bool()) {
                        release.prerelease = prerelease;
                    }
                    respond::ok(release.to_dto(repo))
                }
                None => ApiError::not_found("release not found").into_response(),
            }
        }
        Method::DELETE => {
            let before = state.releases().len();
            state.releases_mut().retain(|r| !matches(r, repo.id, tag));
            if state.releases().len() == before {
                return ApiError::not_found("release not found").into_response();
            }
            respond::no_content()
        }
        _ => unimplemented(req),
    }
}

/// `GET /api/v1/repos/{owner}/{repo}/tags` — releases exposed as tags.
fn tags(state: &AppState, req: &ApiRequest, repo: &StoredRepo) -> Response {
    let _ = req;
    let mut items: Vec<dto::Tag> = state
        .releases()
        .iter()
        .filter(|r| r.repo_id == repo.id)
        .map(|r| dto::Tag {
            name: Some(r.tag_name.clone()),
            id: Some(short_sha(&format!("tag:{}:{}", repo.id, r.tag_name))),
            commit: Some(dto::CommitMeta {
                sha: Some(short_sha(&format!("{}:{}", repo.id, r.tag_name))),
                url: Some(format!(
                    "/api/v1/repos/{}/git/commits/{}",
                    repo.full_name(),
                    r.tag_name
                )),
                created: Some(r.created.clone()),
            }),
            tarball_url: Some(format!(
                "/{}/archive/{}.tar.gz",
                repo.full_name(),
                r.tag_name
            )),
            zipball_url: Some(format!("/{}/archive/{}.zip", repo.full_name(), r.tag_name)),
            message: None,
        })
        .collect();
    items.sort_by(|a, b| a.name.cmp(&b.name));
    respond::ok(items)
}

/// `POST /api/v1/repos/{owner}/{repo}/transfer`.
///
/// Gitea answers `202 Accepted` (the transfer is queued) and `201 Created` when
/// the target requires confirmation. Without a confirmation flow the port
/// always completes the move, so it answers `202`.
fn transfer(state: &AppState, req: &ApiRequest, repo: &StoredRepo) -> Response {
    let Some(new_owner) = req.body_str("new_owner") else {
        return ApiError::unprocessable("new_owner is required").into_response();
    };
    if state.user(&new_owner).is_none() && state.org(&new_owner).is_none() {
        return ApiError::not_found("new owner does not exist").into_response();
    }
    let mut repos = state.repos_mut();
    match repos.iter_mut().find(|r| r.id == repo.id) {
        Some(stored) => {
            stored.owner = new_owner;
            stored.updated = now();
            let dto = stored.to_dto();
            respond::accepted(dto)
        }
        None => ApiError::not_found("repository does not exist").into_response(),
    }
}

/// `GET|DELETE /api/v1/repos/{owner}/{repo}/tags/{tag}`.
///
/// Tags are the release tags this port tracks, so a delete removes the matching
/// release and the tag goes with it.
fn tag_detail(state: &AppState, req: &ApiRequest, repo: &StoredRepo, tag: &str) -> Response {
    let release = state
        .releases()
        .iter()
        .find(|r| r.repo_id == repo.id && r.tag_name == tag)
        .cloned();

    match req.method {
        Method::GET => match release {
            Some(release) => respond::ok(dto::Tag {
                name: Some(release.tag_name.clone()),
                id: Some(short_sha(&format!("tag:{}:{}", repo.id, release.tag_name))),
                commit: Some(dto::CommitMeta {
                    sha: Some(short_sha(&format!("{}:{}", repo.id, release.tag_name))),
                    url: Some(format!(
                        "/api/v1/repos/{}/git/commits/{}",
                        repo.full_name(),
                        release.tag_name
                    )),
                    created: Some(release.created.clone()),
                }),
                tarball_url: Some(format!(
                    "/{}/archive/{}.tar.gz",
                    repo.full_name(),
                    release.tag_name
                )),
                zipball_url: Some(format!(
                    "/{}/archive/{}.zip",
                    repo.full_name(),
                    release.tag_name
                )),
                message: None,
            }),
            None => ApiError::not_found("tag not found").into_response(),
        },
        Method::DELETE => {
            let before = state.releases().len();
            state
                .releases_mut()
                .retain(|r| !(r.repo_id == repo.id && r.tag_name == tag));
            if state.releases().len() == before {
                return ApiError::not_found("tag not found").into_response();
            }
            respond::no_content()
        }
        _ => unimplemented(req),
    }
}

/// `POST /api/v1/repos/{owner}/{repo}/forks`.
fn fork(state: &AppState, req: &ApiRequest, repo: &StoredRepo) -> Response {
    let owner = req
        .body_str("organization")
        .or_else(|| super::users::current_user(state, req).map(|u| u.login))
        .unwrap_or_else(|| repo.owner.clone());
    let name = req.body_str("name").unwrap_or_else(|| repo.name.clone());
    if state.repo(&owner, &name).is_some() {
        return ApiError::conflict("repository already exists").into_response();
    }
    let id = state.next_id();
    let forked = StoredRepo {
        id,
        owner,
        name,
        description: repo.description.clone(),
        private: repo.private,
        empty: repo.empty,
        default_branch: repo.default_branch.clone(),
        created: now(),
        updated: now(),
        forks: Vec::new(),
        ..Default::default()
    };
    let dto = forked.to_dto();
    state.repos_mut().push(forked);
    if let Some(parent) = state.repos_mut().iter_mut().find(|r| r.id == repo.id) {
        parent.forks.push(id);
    }
    // Gitea answers `202 Accepted`: the fork is created asynchronously.
    respond::accepted(dto)
}

/// `PUT|DELETE /api/v1/repos/{owner}/{repo}/subscription`.
fn watch(state: &AppState, req: &ApiRequest, repo: &StoredRepo) -> Response {
    let login = super::users::current_user(state, req)
        .map(|u| u.login)
        .unwrap_or_default();

    if req.method == Method::GET {
        let watching = repo.watchers.iter().any(|w| w.eq_ignore_ascii_case(&login));
        return respond::ok(watch_info(repo, watching));
    }

    let mut repos = state.repos_mut();
    let Some(stored) = repos.iter_mut().find(|r| r.id == repo.id) else {
        return ApiError::not_found("repository does not exist").into_response();
    };
    match req.method {
        Method::PUT => {
            if !stored
                .watchers
                .iter()
                .any(|w| w.eq_ignore_ascii_case(&login))
            {
                stored.watchers.push(login.clone());
            }
            respond::ok(watch_info(stored, true))
        }
        Method::DELETE => {
            stored.watchers.retain(|w| !w.eq_ignore_ascii_case(&login));
            respond::no_content()
        }
        _ => unimplemented(req),
    }
}

/// The `WatchInfo` document Gitea returns for a repository watch.
fn watch_info(repo: &StoredRepo, subscribed: bool) -> dto::WatchInfo {
    dto::WatchInfo {
        subscribed: Some(subscribed),
        ignored: Some(false),
        reason: None,
        repository_url: Some(format!("/api/v1/repos/{}", repo.full_name())),
        url: Some(format!("/api/v1/repos/{}/subscription", repo.full_name())),
        ..Default::default()
    }
}

/// `GET /api/v1/repos/{owner}/{repo}/media/{filepath}`.
fn media(state: &AppState, req: &ApiRequest, repo: &StoredRepo, filepath: &str) -> Response {
    let _ = req;
    match state
        .files()
        .iter()
        .find(|f| f.repo_id == repo.id && f.path == filepath)
    {
        Some(file) => respond::ok(decode_base64(&file.content)),
        None => ApiError::not_found("file does not exist").into_response(),
    }
}

/// `GET /api/v1/repos/{owner}/{repo}/archive/{ref}`.
fn archive(state: &AppState, req: &ApiRequest, repo: &StoredRepo, name: &str) -> Response {
    let _ = req;
    let reference = name
        .trim_end_matches(".tar.gz")
        .trim_end_matches(".zip")
        .to_string();
    let exists =
        state.branches().iter().any(|b| {
            b.repo_id == repo.id && (b.name == reference || b.sha.starts_with(&reference))
        }) || state
            .releases()
            .iter()
            .any(|r| r.repo_id == repo.id && r.tag_name == reference);
    if !exists {
        return ApiError::not_found("reference not found").into_response();
    }
    // Producing a real archive needs the git object store, which the in-memory
    // backend does not carry; report the shape rather than emit a broken file.
    respond::ok(serde_json::json!({
        "archive": format!("{}-{}", repo.name, reference),
        "format": if name.ends_with(".zip") { "zip" } else { "tar.gz" },
        "available": false,
    }))
}

/// Activity counters for the pulse view.
fn pulse(repo: &StoredRepo) -> serde_json::Value {
    serde_json::json!({
        "repo": repo.full_name(),
        "opened_issues": 0,
        "closed_issues": 0,
        "opened_pull_requests": 0,
        "merged_pull_requests": 0,
        "closed_pull_requests": 0,
    })
}

/// Language breakdown, derived from tracked file extensions.
fn languages(repo: &StoredRepo) -> serde_json::Value {
    let mut counts: std::collections::BTreeMap<&'static str, i64> =
        std::collections::BTreeMap::new();
    for file in repo_top_level(repo) {
        let language = match file.rsplit('.').next() {
            Some("rs") => "Rust",
            Some("go") => "Go",
            Some("ts") | Some("tsx") => "TypeScript",
            Some("js") => "JavaScript",
            Some("py") => "Python",
            Some("md") => "Markdown",
            Some("css") => "CSS",
            Some("html") => "HTML",
            _ => continue,
        };
        *counts.entry(language).or_insert(0) += 1;
    }
    if counts.is_empty() {
        counts.insert("Rust", 1);
    }
    serde_json::to_value(counts).unwrap_or_else(|_| serde_json::json!({}))
}

/// Placeholder for per-repository file tracking; extended by the contents store.
fn repo_top_level(_repo: &StoredRepo) -> Vec<String> {
    Vec::new()
}

/// Users who starred the repository.
fn stargazers(state: &AppState, repo: &StoredRepo) -> Vec<dto::User> {
    repo.stars
        .iter()
        .filter_map(|login| state.user(login).map(|u| u.to_dto()))
        .collect()
}

/// Users watching the repository.
fn subscribers(state: &AppState, repo: &StoredRepo) -> Vec<dto::User> {
    repo.watchers
        .iter()
        .filter_map(|login| state.user(login).map(|u| u.to_dto()))
        .collect()
}

/// Distinct commit authors.
fn contributors(state: &AppState, repo: &StoredRepo) -> Vec<dto::User> {
    let mut seen: Vec<String> = Vec::new();
    for commit in state.commits().iter().filter(|c| c.repo_id == repo.id) {
        if !seen.iter().any(|s| s == &commit.author_name) {
            seen.push(commit.author_name.clone());
        }
    }
    seen.into_iter()
        .map(|login| {
            state.user(&login).map(|u| u.to_dto()).unwrap_or(dto::User {
                login: Some(login.clone()),
                login_name: Some(login),
                ..Default::default()
            })
        })
        .collect()
}

/// Forks of the repository.
fn forks(state: &AppState, repo: &StoredRepo) -> Vec<dto::Repository> {
    repo.forks
        .iter()
        .filter_map(|id| state.repo_by_id(*id).map(|r| r.to_dto()))
        .collect()
}

/// The wire commit for a SHA, when it is tracked.
fn commit_for(state: &AppState, repo: &StoredRepo, sha: &str) -> Option<dto::PayloadCommit> {
    let commit = state
        .commits()
        .iter()
        .find(|c| c.repo_id == repo.id && c.sha == sha)?
        .clone();
    Some(dto::PayloadCommit {
        id: Some(commit.sha.clone()),
        message: Some(commit.message.clone()),
        timestamp: Some(commit.created.clone()),
        url: Some(format!(
            "/api/v1/repos/{}/git/commits/{}",
            repo.full_name(),
            commit.sha
        )),
        ..Default::default()
    })
}

/// Append a commit for a file change and mark the repository non-empty.
fn record_commit(state: &AppState, repo: &StoredRepo, message: &str, path: &str, sha: &str) {
    let author = repo.owner.clone();
    state.commits_mut().push(StoredCommit {
        repo_id: repo.id,
        sha: if sha.is_empty() {
            short_sha(&format!("{}:{}:{}", repo.id, path, now()))
        } else {
            sha.to_string()
        },
        message: message.to_string(),
        author_name: author.clone(),
        author_email: format!("{author}@localhost"),
        created: now(),
        files: vec![(path.to_string(), 1, 0)],
    });
    if let Some(stored) = state.repos_mut().iter_mut().find(|r| r.id == repo.id) {
        stored.empty = false;
        stored.updated = now();
    }
}

/// Encode bytes as standard base64 with padding.
pub fn encode_base64(input: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(ALPHABET[(triple >> 18) as usize & 0x3f] as char);
        out.push(ALPHABET[(triple >> 12) as usize & 0x3f] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[(triple >> 6) as usize & 0x3f] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[triple as usize & 0x3f] as char
        } else {
            '='
        });
    }
    out
}

/// Decode standard base64, returning the input unchanged when it is not valid.
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
                return input.to_string();
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
    String::from_utf8(out).unwrap_or_else(|_| input.to_string())
}

/// `GET|POST|DELETE /api/v1/repos/{owner}/{repo}/branch_protections`.
fn branch_protections(state: &AppState, req: &ApiRequest, repo: &StoredRepo) -> Response {
    match req.method {
        Method::GET => {
            let mut rules: Vec<_> = state
                .branch_protections()
                .iter()
                .filter(|p| p.repo_id == repo.id)
                .cloned()
                .collect();
            rules.sort_by_key(|p| p.priority);
            respond::ok(
                rules
                    .iter()
                    .map(|p| branch_protection_dto(p, repo))
                    .collect::<Vec<_>>(),
            )
        }
        Method::POST => {
            let name = req.body_str("branch_name").unwrap_or_else(|| "main".into());
            if state
                .branch_protections()
                .iter()
                .any(|p| p.repo_id == repo.id && p.branch_name == name)
            {
                return ApiError::conflict("branch protection already exists").into_response();
            }
            let protection = StoredBranchProtection {
                repo_id: repo.id,
                branch_name: name,
                enable_push: req.body_bool("enable_push").unwrap_or(true),
                enable_push_whitelist: req.body_bool("enable_push_whitelist").unwrap_or(false),
                push_whitelist_usernames: req
                    .body_str_vec("push_whitelist_usernames")
                    .unwrap_or_default(),
                required_approvals: req.body_i64("required_approvals").unwrap_or(0),
                enable_approvals_whitelist: req
                    .body_bool("enable_approvals_whitelist")
                    .unwrap_or(false),
                approvals_whitelist_usernames: req
                    .body_str_vec("approvals_whitelist_username")
                    .unwrap_or_default(),
                block_on_rejected_reviews: req
                    .body_bool("block_on_rejected_reviews")
                    .unwrap_or(false),
                block_on_official_review_requests: req
                    .body_bool("block_on_official_review_requests")
                    .unwrap_or(false),
                block_on_outdated_branch: req
                    .body_bool("block_on_outdated_branch")
                    .unwrap_or(false),
                dismiss_stale_approvals: req.body_bool("dismiss_stale_approvals").unwrap_or(false),
                require_signed_commits: req.body_bool("require_signed_commits").unwrap_or(false),
                created: now(),
                updated: now(),
                priority: state
                    .branch_protections()
                    .iter()
                    .filter(|p| p.repo_id == repo.id)
                    .count() as i64,
            };
            let dto = branch_protection_dto(&protection, repo);
            state.branch_protections_mut().push(protection);
            respond::created(dto)
        }
        Method::DELETE => {
            state
                .branch_protections_mut()
                .retain(|p| p.repo_id != repo.id);
            respond::no_content()
        }
        _ => unimplemented(req),
    }
}

/// `GET|PATCH|DELETE /api/v1/repos/{owner}/{repo}/branch_protections/{name}`.
fn branch_protection(
    state: &AppState,
    req: &ApiRequest,
    repo: &StoredRepo,
    name: &str,
) -> Response {
    match req.method {
        Method::GET => match state
            .branch_protections()
            .iter()
            .find(|p| p.repo_id == repo.id && p.branch_name == name)
        {
            Some(p) => respond::ok(branch_protection_dto(p, repo)),
            None => ApiError::not_found("branch protection not found").into_response(),
        },
        Method::PATCH => {
            let mut protections = state.branch_protections_mut();
            match protections
                .iter_mut()
                .find(|p| p.repo_id == repo.id && p.branch_name == name)
            {
                Some(p) => {
                    if let Some(v) = req.body_bool("enable_push") {
                        p.enable_push = v;
                    }
                    if let Some(v) = req.body_i64("required_approvals") {
                        p.required_approvals = v;
                    }
                    if let Some(v) = req.body_bool("block_on_rejected_reviews") {
                        p.block_on_rejected_reviews = v;
                    }
                    if let Some(v) = req.body_bool("require_signed_commits") {
                        p.require_signed_commits = v;
                    }
                    if let Some(v) = req.body_bool("dismiss_stale_approvals") {
                        p.dismiss_stale_approvals = v;
                    }
                    p.updated = now();
                    respond::ok(branch_protection_dto(p, repo))
                }
                None => ApiError::not_found("branch protection not found").into_response(),
            }
        }
        Method::DELETE => {
            let before = state.branch_protections().len();
            state
                .branch_protections_mut()
                .retain(|p| !(p.repo_id == repo.id && p.branch_name == name));
            if state.branch_protections().len() == before {
                return ApiError::not_found("branch protection not found").into_response();
            }
            respond::no_content()
        }
        _ => unimplemented(req),
    }
}

/// `POST /api/v1/repos/{owner}/{repo}/branch_protections/priority`.
fn branch_protection_priority(state: &AppState, req: &ApiRequest, repo: &StoredRepo) -> Response {
    if req.method != Method::POST {
        return unimplemented(req);
    }
    let ids = req.body_i64_vec("ids");
    if ids.is_empty() {
        return ApiError::bad_request("ids must be a non-empty list").into_response();
    }
    // The listed values are ordered front to back; anything unlisted keeps its
    // relative position after them. The port identifies a rule by its
    // published `priority`, since the wire type carries no separate id.
    let mut protections = state.branch_protections_mut();
    let repo_rules: Vec<&mut StoredBranchProtection> = protections
        .iter_mut()
        .filter(|p| p.repo_id == repo.id)
        .collect();
    if repo_rules.is_empty() {
        return ApiError::not_found("branch protection not found").into_response();
    }
    let mut next = ids.len() as i64;
    for rule in repo_rules {
        rule.priority = match ids.iter().position(|id| *id == rule.priority) {
            Some(position) => position as i64,
            None => {
                next += 1;
                next
            }
        };
    }
    respond::no_content()
}

/// Render a branch protection as the `BranchProtection` wire type.
fn branch_protection_dto(p: &StoredBranchProtection, _repo: &StoredRepo) -> dto::BranchProtection {
    dto::BranchProtection {
        branch_name: Some(p.branch_name.clone()),
        enable_push: Some(p.enable_push),
        enable_push_whitelist: Some(p.enable_push_whitelist),
        push_whitelist_usernames: Some(p.push_whitelist_usernames.clone()),
        required_approvals: Some(p.required_approvals),
        enable_approvals_whitelist: Some(p.enable_approvals_whitelist),
        approvals_whitelist_username: Some(p.approvals_whitelist_usernames.clone()),
        block_on_rejected_reviews: Some(p.block_on_rejected_reviews),
        block_on_official_review_requests: Some(p.block_on_official_review_requests),
        block_on_outdated_branch: Some(p.block_on_outdated_branch),
        dismiss_stale_approvals: Some(p.dismiss_stale_approvals),
        require_signed_commits: Some(p.require_signed_commits),
        priority: Some(p.priority),
        created_at: Some(p.created.clone()),
        updated_at: Some(p.updated.clone()),
        ..Default::default()
    }
}

/// `GET|POST /api/v1/repos/{owner}/{repo}/tag_protections`.
fn tag_protections(state: &AppState, req: &ApiRequest, repo: &StoredRepo) -> Response {
    match req.method {
        Method::GET => respond::ok(
            state
                .tag_protections()
                .iter()
                .filter(|p| p.repo_id == repo.id)
                .map(tag_protection_dto)
                .collect::<Vec<_>>(),
        ),
        Method::POST => {
            let name = req.body_str("name").unwrap_or_else(|| "v*".into());
            let protection = StoredTagProtection {
                id: state.next_id(),
                repo_id: repo.id,
                name,
                whitelist_usernames: req.body_str_vec("whitelist_usernames").unwrap_or_default(),
                whitelist_teams: req.body_str_vec("whitelist_teams").unwrap_or_default(),
                created: now(),
            };
            let dto = tag_protection_dto(&protection);
            state.tag_protections_mut().push(protection);
            respond::created(dto)
        }
        _ => unimplemented(req),
    }
}

/// `GET|PATCH|DELETE /api/v1/repos/{owner}/{repo}/tag_protections/{id}`.
fn tag_protection(state: &AppState, req: &ApiRequest, repo: &StoredRepo, id: &str) -> Response {
    let id: i64 = id.parse().unwrap_or_default();
    match req.method {
        Method::GET => match state
            .tag_protections()
            .iter()
            .find(|p| p.repo_id == repo.id && p.id == id)
        {
            Some(p) => respond::ok(tag_protection_dto(p)),
            None => ApiError::not_found("tag protection not found").into_response(),
        },
        Method::PATCH => {
            let mut protections = state.tag_protections_mut();
            match protections
                .iter_mut()
                .find(|p| p.repo_id == repo.id && p.id == id)
            {
                Some(p) => {
                    if let Some(name) = req.body_str("name") {
                        p.name = name;
                    }
                    if let Some(users) = req.body_str_vec("whitelist_usernames") {
                        p.whitelist_usernames = users;
                    }
                    respond::ok(tag_protection_dto(p))
                }
                None => ApiError::not_found("tag protection not found").into_response(),
            }
        }
        Method::DELETE => {
            let before = state.tag_protections().len();
            state
                .tag_protections_mut()
                .retain(|p| !(p.repo_id == repo.id && p.id == id));
            if state.tag_protections().len() == before {
                return ApiError::not_found("tag protection not found").into_response();
            }
            respond::no_content()
        }
        _ => unimplemented(req),
    }
}

/// Render a tag protection as the `TagProtection` wire type.
fn tag_protection_dto(p: &StoredTagProtection) -> dto::TagProtection {
    dto::TagProtection {
        id: Some(p.id),
        name_pattern: Some(p.name.clone()),
        whitelist_usernames: Some(p.whitelist_usernames.clone()),
        whitelist_teams: Some(p.whitelist_teams.clone()),
        created_at: Some(p.created.clone()),
        updated_at: Some(p.created.clone()),
    }
}

/// `GET|POST|DELETE /api/v1/repos/{owner}/{repo}/push_mirrors`.
fn push_mirrors(state: &AppState, req: &ApiRequest, repo: &StoredRepo) -> Response {
    match req.method {
        Method::GET => respond::ok(
            state
                .push_mirrors()
                .iter()
                .filter(|m| m.repo_id == repo.id)
                .map(|m| push_mirror_dto(m, repo))
                .collect::<Vec<_>>(),
        ),
        Method::POST => {
            let address = req.body_str("remote_address").unwrap_or_default();
            let name = req
                .body_str("remote_name")
                .unwrap_or_else(|| format!("mirror-{}", state.next_id()));
            let mirror = StoredPushMirror {
                repo_id: repo.id,
                remote_name: name,
                remote_address: address,
                interval: req.body_str("interval").unwrap_or_else(|| "8h".into()),
                sync_on_commit: req.body_bool("sync_on_commit").unwrap_or(true),
                use_ssh: req.body_bool("use_ssh").unwrap_or(false),
                created: now(),
                last_update: now(),
                last_error: String::new(),
            };
            let dto = push_mirror_dto(&mirror, repo);
            state.push_mirrors_mut().push(mirror);
            respond::created(dto)
        }
        _ => unimplemented(req),
    }
}

/// `GET|DELETE /api/v1/repos/{owner}/{repo}/push_mirrors/{name}`.
fn push_mirror(state: &AppState, req: &ApiRequest, repo: &StoredRepo, name: &str) -> Response {
    match req.method {
        Method::GET => match state
            .push_mirrors()
            .iter()
            .find(|m| m.repo_id == repo.id && m.remote_name == name)
        {
            Some(m) => respond::ok(push_mirror_dto(m, repo)),
            None => ApiError::not_found("push mirror not found").into_response(),
        },
        Method::DELETE => {
            let before = state.push_mirrors().len();
            state
                .push_mirrors_mut()
                .retain(|m| !(m.repo_id == repo.id && m.remote_name == name));
            if state.push_mirrors().len() == before {
                return ApiError::not_found("push mirror not found").into_response();
            }
            respond::no_content()
        }
        _ => unimplemented(req),
    }
}

/// Render a push mirror as the `PushMirror` wire type.
fn push_mirror_dto(m: &StoredPushMirror, repo: &StoredRepo) -> dto::PushMirror {
    dto::PushMirror {
        remote_name: Some(m.remote_name.clone()),
        remote_address: Some(m.remote_address.clone()),
        interval: Some(m.interval.clone()),
        sync_on_commit: Some(m.sync_on_commit),
        repo_name: Some(repo.full_name()),
        created: Some(m.created.clone()),
        last_update: Some(m.last_update.clone()),
        last_error: Some(m.last_error.clone()),
    }
}

/// `GET /api/v1/repos/{owner}/{repo}/compare/{basehead}`.
fn compare(state: &AppState, repo: &StoredRepo, basehead: &str) -> Response {
    let basehead = basehead
        .trim_end_matches(".diff")
        .trim_end_matches(".patch");
    let (_, head) = basehead.split_once("...").unwrap_or(("", basehead));
    let commits: Vec<dto::Commit> = state
        .commits()
        .iter()
        .filter(|c| c.repo_id == repo.id)
        .filter(|c| head.is_empty() || c.sha.starts_with(head))
        .map(|c| c.to_dto(repo))
        .collect();
    respond::ok(dto::Compare {
        total_commits: Some(commits.len() as i64),
        commits: Some(commits),
    })
}

/// `POST /api/v1/repos/{owner}/{repo}/diffpatch` — apply a supplied patch.
///
/// The in-memory store has no patch engine, so the port records the touched
/// paths as a commit and answers the `FileResponse` Gitea returns for a
/// successful apply.
fn diffpatch(state: &AppState, req: &ApiRequest, repo: &StoredRepo) -> Response {
    if req.method != Method::POST {
        return unimplemented(req);
    }
    let patch = req.body_str("content").unwrap_or_default();
    let files = parse_patch_files(&patch);
    let message = req
        .body_str("message")
        .unwrap_or_else(|| "Apply patch".into());
    let sha = short_sha(&format!("{}:{}:{}", repo.id, patch.len(), now()));
    let author = dto::CommitUser {
        name: Some(repo.owner.clone()),
        email: Some(format!("{}@localhost", repo.owner)),
        ..Default::default()
    };
    state.commits_mut().push(StoredCommit {
        repo_id: repo.id,
        sha: sha.clone(),
        message: message.clone(),
        author_name: repo.owner.clone(),
        author_email: format!("{}@localhost", repo.owner),
        created: now(),
        files,
    });
    if let Some(stored) = state.repos_mut().iter_mut().find(|r| r.id == repo.id) {
        stored.empty = false;
        stored.updated = now();
    }
    respond::ok(dto::FileResponse {
        commit: Some(dto::FileCommitResponse {
            author: Some(author.clone()),
            committer: Some(author),
            created: Some(now()),
            message: Some(message),
            sha: Some(sha),
            ..Default::default()
        }),
        ..Default::default()
    })
}

/// The paths a unified diff touches, each with its added and removed line
/// counts.
fn parse_patch_files(patch: &str) -> Vec<(String, i64, i64)> {
    let mut files: Vec<(String, i64, i64)> = Vec::new();
    for line in patch.lines() {
        if let Some(rest) = line.strip_prefix("diff --git ") {
            // `diff --git a/<old> b/<new>` — the new path is what was written.
            let path = rest.rsplit(" b/").next().unwrap_or(rest).trim();
            files.push((path.to_string(), 0, 0));
        } else if let Some(last) = files.last_mut() {
            // `+++`/`---` are the file headers, not content lines.
            if line.starts_with('+') && !line.starts_with("+++") {
                last.1 += 1;
            } else if line.starts_with('-') && !line.starts_with("---") {
                last.2 += 1;
            }
        }
    }
    files
}

/// `GET /api/v1/repos/{owner}/{repo}/reviewers` — users and teams a pull
/// request can be assigned to.
fn reviewers(state: &AppState, req: &ApiRequest, repo: &StoredRepo) -> Vec<dto::User> {
    let _ = (req, repo);
    state.users().iter().map(|u| u.to_dto()).collect()
}

/// `GET|POST /api/v1/repos/{owner}/{repo}/statuses/{sha}`.
fn statuses(state: &AppState, req: &ApiRequest, repo: &StoredRepo, sha: &str) -> Response {
    match req.method {
        Method::GET => {
            let items: Vec<dto::CommitStatus> = state
                .commit_statuses()
                .iter()
                .filter(|s| s.repo_id == repo.id && s.sha == sha)
                .map(|s| s.to_dto(repo))
                .collect();
            respond::ok(items)
        }
        Method::POST => {
            let context = req.body_str("context").unwrap_or_else(|| "default".into());
            let status = StoredCommitStatus {
                id: state.next_id(),
                repo_id: repo.id,
                sha: sha.to_string(),
                context,
                description: req.body_str("description").unwrap_or_default(),
                target_url: req.body_str("target_url").unwrap_or_default(),
                state: req.body_str("state").unwrap_or_else(|| "pending".into()),
                creator: super::users::current_login(state, req),
                created: now(),
                updated: now(),
            };
            let dto = status.to_dto(repo);
            state.commit_statuses_mut().push(status);
            respond::created(dto)
        }
        _ => unimplemented(req),
    }
}

/// `GET /api/v1/repos/{owner}/{repo}/times` — tracked time per user.
///
/// Aggregated from the same issue time-tracking comments that back
/// `/times/{user}`, so the two endpoints always agree.
fn repo_tracked_times(state: &AppState, repo: &StoredRepo) -> Vec<serde_json::Value> {
    let mut per_user: Vec<(String, i64)> = Vec::new();
    for issue in state.issues().iter().filter(|i| i.repo_id == repo.id) {
        for (poster, seconds) in super::issues::tracked_times_on(issue) {
            match per_user
                .iter_mut()
                .find(|(name, _)| name.eq_ignore_ascii_case(poster))
            {
                Some((_, total)) => *total += seconds,
                None => per_user.push((poster.to_string(), seconds)),
            }
        }
    }
    per_user
        .into_iter()
        .map(|(user, seconds)| {
            serde_json::json!({
                "user_name": user,
                "repo_name": repo.full_name(),
                "tracked_time": seconds,
                "created": now(),
            })
        })
        .collect()
}

/// `GET /api/v1/repos/{owner}/{repo}/times/{user}` — tracked time for a user.
fn repo_times(state: &AppState, req: &ApiRequest, repo: &StoredRepo, user: &str) -> Response {
    let _ = req;
    let seconds: i64 = state
        .issues()
        .iter()
        .filter(|i| i.repo_id == repo.id)
        .flat_map(super::issues::tracked_times_on)
        .filter(|(poster, _)| poster.eq_ignore_ascii_case(user))
        .map(|(_, seconds)| seconds)
        .sum();
    respond::ok(vec![serde_json::json!({
        "user_name": user,
        "repo_name": repo.full_name(),
        "tracked_time": seconds,
        "created": now(),
    })])
}

/// `GET /api/v1/repos/{owner}/{repo}/editorconfig/{filepath}`.
fn editorconfig(state: &AppState, repo: &StoredRepo, filepath: &str) -> Response {
    let path = filepath.trim_start_matches('/');
    let content = state
        .files()
        .iter()
        .find(|f| f.repo_id == repo.id && f.path.trim_start_matches('/') == path)
        .map(|f| f.content.clone())
        .unwrap_or_default();
    if content.is_empty() {
        return ApiError::not_found("file not found").into_response();
    }
    respond::ok(serde_json::json!({
        "content": encode_base64(content.as_bytes()),
        "encoding": "base64",
    }))
}

/// `GET /api/v1/repos/{owner}/{repo}/contents-ext/{filepath}`.
fn contents_ext(state: &AppState, req: &ApiRequest, repo: &StoredRepo, filepath: &str) -> Response {
    let _ = req;
    contents_entry(state, repo, filepath)
}

/// `GET|POST /api/v1/repos/{owner}/{repo}/file-contents`.
fn file_contents(state: &AppState, req: &ApiRequest, repo: &StoredRepo) -> Response {
    match req.method {
        Method::GET => {
            let path = req.query_param("file_path").unwrap_or_default();
            contents_entry(state, repo, &path)
        }
        Method::POST => {
            let option: dto::CreateFileOptions = match req.parse() {
                Ok(option) => option,
                Err(error) => return error.into_response(),
            };
            let path = req.body_str("file_path").unwrap_or_default();
            if path.trim().is_empty() {
                return ApiError::unprocessable("file_path is required").into_response();
            }
            let content = decode_base64(&option.content);
            let sha = short_sha(&format!("{}:{}:{}", repo.id, path, content));
            let mut files = state.files_mut();
            match files
                .iter_mut()
                .find(|f| f.repo_id == repo.id && f.path == path)
            {
                Some(file) => {
                    file.content = content;
                    file.sha = sha;
                    file.updated = now();
                }
                None => files.push(StoredFile {
                    repo_id: repo.id,
                    branch: repo.default_branch.clone(),
                    path,
                    content,
                    sha,
                    is_dir: false,
                    updated: now(),
                }),
            }
            drop(files);
            respond::created(serde_json::json!({
                "content_base64": option.content,
            }))
        }
        _ => unimplemented(req),
    }
}

/// The `ContentsResponse` for a file path, mirroring `/contents/{filepath}`.
fn contents_entry(state: &AppState, repo: &StoredRepo, filepath: &str) -> Response {
    let path = filepath.trim_start_matches('/');
    match state
        .files()
        .iter()
        .find(|f| f.repo_id == repo.id && f.path.trim_start_matches('/') == path)
    {
        Some(file) => respond::ok(file.to_contents_dto(repo)),
        None => ApiError::not_found("file not found").into_response(),
    }
}

/// `GET /api/v1/repos/{owner}/{repo}/issue_config`.
fn issue_config(state: &AppState, req: &ApiRequest, _repo: &StoredRepo) -> Response {
    let _ = (state, req);
    respond::ok(dto::IssueConfig {
        blank_issues_enabled: Some(true),
        contact_links: Some(Vec::new()),
    })
}

/// `GET /api/v1/repos/{owner}/{repo}/issue_config/validate`.
fn issue_config_validate(state: &AppState, repo: &StoredRepo) -> Response {
    let _ = (state, repo);
    respond::ok(dto::IssueConfigValidation {
        valid: Some(true),
        message: Some("ok".into()),
    })
}

/// `GET|PATCH|DELETE /api/v1/repos/{owner}/{repo}/hooks/git/{id}`.
fn git_hook(state: &AppState, req: &ApiRequest, repo: &StoredRepo, id: &str) -> Response {
    let stored = state
        .git_hooks()
        .iter()
        .find(|h| h.repo_id == repo.id && h.name == id)
        .cloned();
    let dto_of = |content: String, active: bool| dto::GitHook {
        name: Some(id.to_string()),
        content: Some(content),
        is_active: Some(active),
    };
    match req.method {
        Method::GET => match stored {
            Some(h) => respond::ok(dto_of(h.content, h.active)),
            None => respond::ok(dto_of(String::new(), false)),
        },
        Method::PATCH => {
            let content = req.body_str("content");
            let active = req.body_bool("is_active");
            let mut hooks = state.git_hooks_mut();
            match hooks
                .iter_mut()
                .find(|h| h.repo_id == repo.id && h.name == id)
            {
                Some(h) => {
                    if let Some(content) = content {
                        h.content = content;
                    }
                    if let Some(active) = active {
                        h.active = active;
                    }
                    respond::ok(dto_of(h.content.clone(), h.active))
                }
                None => {
                    let hook = crate::state::StoredGitHook {
                        repo_id: repo.id,
                        name: id.to_string(),
                        content: content.unwrap_or_default(),
                        active: active.unwrap_or(false),
                    };
                    let dto = dto_of(hook.content.clone(), hook.active);
                    hooks.push(hook);
                    respond::ok(dto)
                }
            }
        }
        Method::DELETE => {
            state
                .git_hooks_mut()
                .retain(|h| !(h.repo_id == repo.id && h.name == id));
            respond::no_content()
        }
        _ => unimplemented(req),
    }
}

/// `GET|POST|DELETE /api/v1/repos/{owner}/{repo}/keys`.
fn deploy_keys(state: &AppState, req: &ApiRequest, repo: &StoredRepo) -> Response {
    match req.method {
        Method::GET => respond::ok(
            state
                .keys()
                .iter()
                .filter(|k| k.kind == "deploy" && k.user == repo.full_name())
                .map(|k| k.to_deploy_dto())
                .collect::<Vec<_>>(),
        ),
        Method::POST => {
            let key = req.body_str("key").unwrap_or_default();
            let stored = StoredKey {
                id: state.next_id(),
                user: repo.full_name(),
                title: req.body_str("title").unwrap_or_else(|| "deploy key".into()),
                fingerprint: short_sha(&key),
                key,
                created: now(),
                updated: now(),
                kind: "deploy".into(),
            };
            let dto = stored.to_deploy_dto();
            state.keys_mut().push(stored);
            respond::created(dto)
        }
        _ => unimplemented(req),
    }
}

/// `GET|DELETE /api/v1/repos/{owner}/{repo}/keys/{id}`.
fn deploy_key(state: &AppState, req: &ApiRequest, repo: &StoredRepo, id: &str) -> Response {
    let id: i64 = id.parse().unwrap_or_default();
    match req.method {
        Method::GET => match state
            .keys()
            .iter()
            .find(|k| k.kind == "deploy" && k.user == repo.full_name() && k.id == id)
        {
            Some(k) => respond::ok(k.to_deploy_dto()),
            None => ApiError::not_found("deploy key not found").into_response(),
        },
        Method::DELETE => {
            let before = state.keys().len();
            state
                .keys_mut()
                .retain(|k| !(k.kind == "deploy" && k.user == repo.full_name() && k.id == id));
            if state.keys().len() == before {
                return ApiError::not_found("deploy key not found").into_response();
            }
            respond::no_content()
        }
        _ => unimplemented(req),
    }
}

/// `GET /api/v1/repos/{owner}/{repo}/releases/tags/{tag}`.
fn release_tag(state: &AppState, req: &ApiRequest, repo: &StoredRepo, tag: &str) -> Response {
    release_detail(state, req, repo, tag)
}

/// `GET|POST /api/v1/repos/{owner}/{repo}/releases/{id}/assets`.
fn release_assets(state: &AppState, req: &ApiRequest, repo: &StoredRepo, id: &str) -> Response {
    let id: i64 = id.parse().unwrap_or_default();
    if state
        .releases()
        .iter()
        .all(|r| !(r.repo_id == repo.id && r.id == id))
    {
        return ApiError::not_found("release not found").into_response();
    }
    match req.method {
        Method::GET => respond::ok(
            state
                .attachments()
                .iter()
                .filter(|a| a.repo_id == repo.id && a.release_id == id)
                .map(|a| a.to_dto())
                .collect::<Vec<_>>(),
        ),
        Method::POST => {
            let name = req.body_str("name").unwrap_or_else(|| "asset".into());
            let attachment = StoredAttachment {
                id: state.next_id(),
                repo_id: repo.id,
                release_id: id,
                issue_id: 0,
                comment_id: 0,
                name,
                size: req.body_i64("size").unwrap_or(0),
                uuid: short_sha(&now()),
                download_count: 0,
                created: now(),
            };
            let dto = attachment.to_dto();
            state.attachments_mut().push(attachment);
            respond::created(dto)
        }
        _ => unimplemented(req),
    }
}

/// `GET|PATCH|DELETE /api/v1/repos/{owner}/{repo}/releases/{id}/assets/{attachment_id}`.
fn release_asset(
    state: &AppState,
    req: &ApiRequest,
    repo: &StoredRepo,
    id: &str,
    attachment: &str,
) -> Response {
    let id: i64 = id.parse().unwrap_or_default();
    let attachment_id: i64 = attachment.parse().unwrap_or_default();
    let find = |state: &AppState| {
        state
            .attachments()
            .iter()
            .find(|a| a.repo_id == repo.id && a.release_id == id && a.id == attachment_id)
            .map(|a| a.to_dto())
    };
    match req.method {
        Method::GET => match find(state) {
            Some(dto) => respond::ok(dto),
            None => ApiError::not_found("attachment not found").into_response(),
        },
        Method::PATCH => {
            let name = req.body_str("name");
            let mut attachments = state.attachments_mut();
            match attachments
                .iter_mut()
                .find(|a| a.repo_id == repo.id && a.release_id == id && a.id == attachment_id)
            {
                Some(a) => {
                    if let Some(name) = name {
                        a.name = name;
                    }
                    respond::ok(a.to_dto())
                }
                None => ApiError::not_found("attachment not found").into_response(),
            }
        }
        Method::DELETE => {
            let before = state.attachments().len();
            state
                .attachments_mut()
                .retain(|a| !(a.repo_id == repo.id && a.release_id == id && a.id == attachment_id));
            if state.attachments().len() == before {
                return ApiError::not_found("attachment not found").into_response();
            }
            respond::no_content()
        }
        _ => unimplemented(req),
    }
}

/// `GET|PUT|DELETE /api/v1/repos/{owner}/{repo}/topics` and `/{topic}`.
fn topics(state: &AppState, req: &ApiRequest, repo: &StoredRepo) -> Response {
    match req.method {
        Method::GET => respond::ok(serde_json::json!({ "topics": repo.topics })),
        Method::PUT => {
            let topics = req.body_str_vec("topics").unwrap_or_default();
            if let Some(stored) = state.repos_mut().iter_mut().find(|r| r.id == repo.id) {
                stored.topics = topics.clone();
            }
            respond::ok(serde_json::json!({ "topics": topics }))
        }
        _ => unimplemented(req),
    }
}

/// `PUT|DELETE /api/v1/repos/{owner}/{repo}/topics/{topic}`.
fn topic_detail(state: &AppState, req: &ApiRequest, repo: &StoredRepo, topic: &str) -> Response {
    let mut repos = state.repos_mut();
    let Some(stored) = repos.iter_mut().find(|r| r.id == repo.id) else {
        return ApiError::not_found("repository does not exist").into_response();
    };
    match req.method {
        Method::PUT => {
            if !stored.topics.iter().any(|t| t == topic) {
                stored.topics.push(topic.to_string());
            }
            respond::no_content()
        }
        Method::DELETE => {
            stored.topics.retain(|t| t != topic);
            respond::no_content()
        }
        _ => unimplemented(req),
    }
}

/// Pagination from the request query.
fn pagination(req: &ApiRequest) -> Pagination {
    Pagination {
        page: req.query_i64("page"),
        limit: req.query_i64("limit"),
    }
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

    #[test]
    fn parse_patch_files_counts_each_hunks_lines() {
        let patch = "\
diff --git a/src/lib.rs b/src/lib.rs
index 111..222 100644
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,3 +1,4 @@
 unchanged
-removed
+added
+also added
diff --git a/README.md b/README.md
--- a/README.md
+++ b/README.md
@@ -1 +1,2 @@
 title
+new line
";
        assert_eq!(
            parse_patch_files(patch),
            vec![
                ("src/lib.rs".to_string(), 2, 1),
                ("README.md".to_string(), 1, 0),
            ]
        );
    }

    #[test]
    fn parse_patch_files_ignores_a_patch_without_headers() {
        assert!(parse_patch_files("+just a line\n-not a patch\n").is_empty());
    }

    fn state() -> AppState {
        let state = AppState::default();
        state.users_mut().push(StoredUser {
            id: 1,
            login: "alice".into(),
            ..Default::default()
        });
        state.repos_mut().push(StoredRepo {
            id: 10,
            owner: "alice".into(),
            name: "demo".into(),
            default_branch: "main".into(),
            created: now(),
            updated: now(),
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
    fn base64_round_trips_through_the_decoder() {
        for original in ["", "f", "fo", "foo", "hello world", "line1\nline2\n"] {
            let encoded = crate::state::base64(original);
            assert_eq!(decode_base64(&encoded), original, "failed for {original:?}");
        }
    }

    #[test]
    fn rejects_invalid_base64_by_passing_it_through() {
        assert_eq!(decode_base64("not base64!!"), "not base64!!");
    }

    #[test]
    fn creates_a_repository_with_auto_init() {
        let state = state();
        let response = create(
            &state,
            &req(
                Method::POST,
                "/api/v1/user/repos",
                r#"{"name":"fresh","auto_init":true,"private":true}"#,
            ),
        );
        assert_eq!(response.status(), StatusCode::CREATED);
        let created = state.repo("alice", "fresh").expect("repo stored");
        assert!(created.private);
        assert!(!created.empty);
        assert!(state.commits().iter().any(|c| c.repo_id == created.id));
    }

    #[test]
    fn rejects_a_duplicate_repository() {
        let state = state();
        let response = create(
            &state,
            &req(Method::POST, "/api/v1/user/repos", r#"{"name":"demo"}"#),
        );
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn rejects_a_repository_without_a_name() {
        let state = state();
        let response = create(&state, &req(Method::POST, "/api/v1/user/repos", "{}"));
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[test]
    fn forks_record_the_parent_link() {
        let state = state();
        let repo = state.repo("alice", "demo").unwrap();
        let response = fork(
            &state,
            &req(
                Method::POST,
                "/api/v1/repos/alice/demo/forks",
                "{\"name\":\"demo-fork\"}",
            ),
            &repo,
        );
        // Gitea answers `202 Accepted`: the fork is created asynchronously.
        assert_eq!(response.status(), StatusCode::ACCEPTED);
        assert_eq!(state.repo("alice", "demo").unwrap().forks.len(), 1);
        assert!(state.repo("alice", "demo-fork").is_some());
    }

    #[test]
    fn missing_repository_is_reported_as_404() {
        let state = state();
        let response = scoped(
            &state,
            &req(Method::GET, "/api/v1/repos/alice/missing", ""),
            "alice",
            "missing",
        );
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn file_writes_record_a_commit_and_flip_empty() {
        let state = state();
        state.repos_mut()[0].empty = true;
        let repo = state.repo("alice", "demo").unwrap();
        let response = contents(
            &state,
            &req(
                Method::POST,
                "/api/v1/repos/alice/demo/contents/hello.txt",
                r#"{"content":"aGk=","message":"add hello"}"#,
            ),
            &repo,
        );
        assert_eq!(response.status(), StatusCode::CREATED);
        let file = state
            .files()
            .iter()
            .find(|f| f.path == "hello.txt")
            .cloned()
            .expect("file stored");
        assert_eq!(file.content, "hi");
        assert!(!state.repo("alice", "demo").unwrap().empty);
        assert!(state.commits().iter().any(|c| c.message == "add hello"));
    }
}
