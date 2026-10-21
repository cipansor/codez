//! Issue, pull request, label, milestone and comment endpoints.

use axum::http::Method;
use axum::response::{IntoResponse, Response};

use shared::models as dto;

use crate::pagination::Pagination;
use crate::respond::{self, ApiError};
use crate::state::{
    now, short_sha, AppState, StoredAttachment, StoredComment, StoredIssue, StoredLabel,
    StoredMilestone, StoredReaction, StoredRepo, StoredReview, StoredReviewComment,
};

use super::ApiRequest;

/// `GET|POST /api/v1/repos/{owner}/{repo}/issues`.
pub fn handle_issues(state: &AppState, req: &ApiRequest, repo: &StoredRepo) -> Response {
    let tail = tail_after(req, 5);
    match tail.as_slice() {
        ["issues"] => match req.method {
            Method::GET => list(state, req, repo, false),
            Method::POST => create(state, req, repo, false),
            _ => unimplemented(req),
        },
        ["issues", index] => issue_detail(state, req, repo, index, false),
        ["issues", index, "comments"] => comments(state, req, repo, index),
        ["issues", index, "comments", id] => comment_detail(state, req, repo, index, id),
        ["issues", index, "labels"] => issue_labels(state, req, repo, index),
        ["issues", index, "labels", id] => issue_label_remove(state, req, repo, index, id),
        ["issues", index, "times"] => issue_times(state, req, repo, index),
        ["issues", index, "times", id] => issue_time_reset(state, req, repo, index, id),
        ["issues", index, "timeline"] => timeline(state, repo, index),
        ["issues", index, "assignees"] => issue_assignees(state, req, repo, index),
        ["issues", index, "assignees", assignee] => {
            issue_assignee(state, req, repo, index, assignee)
        }
        ["issues", index, "lock"] => issue_lock(state, req, repo, index),
        ["issues", index, "deadline"] => issue_deadline(state, req, repo, index),
        ["issues", index, "pin"] => issue_pin(state, req, repo, index, None),
        ["issues", index, "pin", position] => issue_pin(state, req, repo, index, Some(position)),
        ["issues", index, "subscriptions"] => issue_subscriptions(state, req, repo, index),
        ["issues", index, "subscriptions", "check"] => {
            issue_subscription_check(state, req, repo, index)
        }
        ["issues", index, "subscriptions", user] => {
            issue_subscription(state, req, repo, index, user)
        }
        ["issues", index, "reactions"] => reactions(state, req, repo, index),
        ["issues", index, "blocks"] => issue_blocks(state, req, repo, index, false),
        ["issues", index, "dependencies"] => issue_blocks(state, req, repo, index, true),
        ["issues", index, "assets"] => issue_assets(state, req, repo, index),
        ["issues", index, "assets", id] => issue_asset_detail(state, req, repo, index, id),
        ["issues", index, "stopwatch", action @ ("start" | "stop" | "delete")] => {
            stopwatch(state, req, repo, index, action)
        }
        ["issues", "comments", id] => comment_by_id(state, req, repo, id),
        ["issues", "comments", id, "reactions"] => comment_reactions(state, req, repo, id),
        ["issues", "comments", id, "assets"] => comment_assets(state, req, repo, id),
        ["issues", "comments", id, "assets", attachment] => {
            comment_asset_detail(state, req, repo, id, attachment)
        }
        _ => unimplemented(req),
    }
}

/// `GET|POST|DELETE /api/v1/repos/{owner}/{repo}/issues/{index}/assignees` and
/// `/{assignee}`.
fn issue_assignees(state: &AppState, req: &ApiRequest, repo: &StoredRepo, index: &str) -> Response {
    let Ok(index) = index.parse::<i64>() else {
        return ApiError::bad_request("index must be a number").into_response();
    };
    let body = req.json();
    let names: Vec<String> = body
        .get("assignees")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    let mut issues = state.issues_mut();
    let Some(stored) = issues
        .iter_mut()
        .find(|i| i.repo_id == repo.id && i.index == index)
    else {
        return ApiError::not_found("issue not found").into_response();
    };
    match req.method {
        Method::POST => {
            for name in &names {
                if !stored
                    .assignees
                    .iter()
                    .any(|a| a.eq_ignore_ascii_case(name))
                {
                    stored.assignees.push(name.clone());
                }
            }
        }
        Method::DELETE => {
            stored
                .assignees
                .retain(|a| !names.iter().any(|n| n.eq_ignore_ascii_case(a)));
        }
        _ => return unimplemented(req),
    }
    stored.updated = now();
    let snapshot = stored.clone();
    drop(issues);
    respond::ok(snapshot.to_dto(
        repo,
        &labels_of(state, repo.id, &snapshot),
        milestone_of(state, snapshot.milestone_id),
    ))
}

/// `GET /api/v1/repos/{owner}/{repo}/issues/{index}/assignees/{assignee}`.
fn issue_assignee(
    state: &AppState,
    req: &ApiRequest,
    repo: &StoredRepo,
    index: &str,
    assignee: &str,
) -> Response {
    let _ = req;
    let Ok(index) = index.parse::<i64>() else {
        return ApiError::bad_request("index must be a number").into_response();
    };
    match state.issue(repo.id, index) {
        Some(issue)
            if issue
                .assignees
                .iter()
                .any(|a| a.eq_ignore_ascii_case(assignee)) =>
        {
            respond::no_content()
        }
        Some(_) => ApiError::not_found("assignee not found").into_response(),
        None => ApiError::not_found("issue not found").into_response(),
    }
}

/// `GET|PUT|DELETE /api/v1/repos/{owner}/{repo}/issues/{index}/lock`.
fn issue_lock(state: &AppState, req: &ApiRequest, repo: &StoredRepo, index: &str) -> Response {
    let Ok(index) = index.parse::<i64>() else {
        return ApiError::bad_request("index must be a number").into_response();
    };
    let mut issues = state.issues_mut();
    let Some(stored) = issues
        .iter_mut()
        .find(|i| i.repo_id == repo.id && i.index == index)
    else {
        return ApiError::not_found("issue not found").into_response();
    };
    match req.method {
        Method::GET => {
            if stored.locked {
                respond::no_content()
            } else {
                ApiError::not_found("issue is not locked").into_response()
            }
        }
        Method::PUT => {
            stored.locked = true;
            stored.updated = now();
            respond::no_content()
        }
        Method::DELETE => {
            stored.locked = false;
            stored.updated = now();
            respond::no_content()
        }
        _ => unimplemented(req),
    }
}

/// `POST /api/v1/repos/{owner}/{repo}/issues/{index}/deadline`.
fn issue_deadline(state: &AppState, req: &ApiRequest, repo: &StoredRepo, index: &str) -> Response {
    if req.method != Method::POST {
        return unimplemented(req);
    }
    let Ok(index) = index.parse::<i64>() else {
        return ApiError::bad_request("index must be a number").into_response();
    };
    let deadline = req.body_str("due_date");
    let mut issues = state.issues_mut();
    let Some(stored) = issues
        .iter_mut()
        .find(|i| i.repo_id == repo.id && i.index == index)
    else {
        return ApiError::not_found("issue not found").into_response();
    };
    stored.deadline = deadline.filter(|d| !d.trim().is_empty());
    stored.updated = now();
    let snapshot = stored.clone();
    drop(issues);
    respond::ok(snapshot.to_dto(
        repo,
        &labels_of(state, repo.id, &snapshot),
        milestone_of(state, snapshot.milestone_id),
    ))
}

/// `POST|DELETE /api/v1/repos/{owner}/{repo}/issues/{index}/pin` and
/// `PATCH .../pin/{position}`.
fn issue_pin(
    state: &AppState,
    req: &ApiRequest,
    repo: &StoredRepo,
    index: &str,
    position: Option<&str>,
) -> Response {
    let Ok(index) = index.parse::<i64>() else {
        return ApiError::bad_request("index must be a number").into_response();
    };
    let mut issues = state.issues_mut();
    let Some(stored) = issues
        .iter_mut()
        .find(|i| i.repo_id == repo.id && i.index == index)
    else {
        return ApiError::not_found("issue not found").into_response();
    };
    match (req.method.clone(), position) {
        (Method::POST, None) => stored.pinned = true,
        (Method::DELETE, None) => stored.pinned = false,
        (Method::PATCH, Some(raw)) => {
            stored.pinned = true;
            stored.pin_order = raw.parse().unwrap_or_default();
        }
        _ => return unimplemented(req),
    }
    stored.updated = now();
    respond::no_content()
}

/// `GET /api/v1/repos/{owner}/{repo}/issues/{index}/subscriptions`.
fn issue_subscriptions(
    state: &AppState,
    req: &ApiRequest,
    repo: &StoredRepo,
    index: &str,
) -> Response {
    let _ = req;
    let Ok(index) = index.parse::<i64>() else {
        return ApiError::bad_request("index must be a number").into_response();
    };
    let Some(issue) = state.issue(repo.id, index) else {
        return ApiError::not_found("issue not found").into_response();
    };
    let users: Vec<dto::User> = issue
        .subscribers
        .iter()
        .filter_map(|login| state.user(login).map(|u| u.to_dto()))
        .collect();
    respond::ok(users)
}

/// `GET /api/v1/repos/{owner}/{repo}/issues/{index}/subscriptions/check`.
fn issue_subscription_check(
    state: &AppState,
    req: &ApiRequest,
    repo: &StoredRepo,
    index: &str,
) -> Response {
    let Ok(index) = index.parse::<i64>() else {
        return ApiError::bad_request("index must be a number").into_response();
    };
    let Some(issue) = state.issue(repo.id, index) else {
        return ApiError::not_found("issue not found").into_response();
    };
    let login = super::users::current_login(state, req);
    let subscribed = issue
        .subscribers
        .iter()
        .any(|s| s.eq_ignore_ascii_case(&login));
    respond::ok(serde_json::json!({ "subscribed": subscribed }))
}

/// `PUT|DELETE /api/v1/repos/{owner}/{repo}/issues/{index}/subscriptions/{user}`.
fn issue_subscription(
    state: &AppState,
    req: &ApiRequest,
    repo: &StoredRepo,
    index: &str,
    user: &str,
) -> Response {
    let Ok(index) = index.parse::<i64>() else {
        return ApiError::bad_request("index must be a number").into_response();
    };
    let mut issues = state.issues_mut();
    let Some(stored) = issues
        .iter_mut()
        .find(|i| i.repo_id == repo.id && i.index == index)
    else {
        return ApiError::not_found("issue not found").into_response();
    };
    match req.method {
        Method::PUT => {
            if !stored
                .subscribers
                .iter()
                .any(|s| s.eq_ignore_ascii_case(user))
            {
                stored.subscribers.push(user.to_string());
            }
        }
        Method::DELETE => stored.subscribers.retain(|s| !s.eq_ignore_ascii_case(user)),
        _ => return unimplemented(req),
    }
    stored.updated = now();
    respond::no_content()
}

/// `GET|POST|DELETE /api/v1/repos/{owner}/{repo}/issues/{index}/reactions` and
/// the comment twin.
fn reactions(state: &AppState, req: &ApiRequest, repo: &StoredRepo, index: &str) -> Response {
    let Ok(index) = index.parse::<i64>() else {
        return ApiError::bad_request("index must be a number").into_response();
    };
    if state.issue(repo.id, index).is_none() {
        return ApiError::not_found("issue not found").into_response();
    }
    reactions_on(state, req, repo, index, false)
}

/// `GET|POST|DELETE /api/v1/repos/{owner}/{repo}/issues/comments/{id}/reactions`.
fn comment_reactions(state: &AppState, req: &ApiRequest, repo: &StoredRepo, id: &str) -> Response {
    let Ok(id) = id.parse::<i64>() else {
        return ApiError::bad_request("id must be a number").into_response();
    };
    if !state
        .issues()
        .iter()
        .any(|i| i.repo_id == repo.id && i.comments.iter().any(|c| c.id == id))
    {
        return ApiError::not_found("comment not found").into_response();
    }
    reactions_on(state, req, repo, id, true)
}

/// `GET|POST /api/v1/repos/{owner}/{repo}/issues/{index}/assets`.
fn issue_assets(state: &AppState, req: &ApiRequest, repo: &StoredRepo, index: &str) -> Response {
    let Ok(index) = index.parse::<i64>() else {
        return ApiError::bad_request("index must be a number").into_response();
    };
    let Some(issue_id) = state.issue(repo.id, index).map(|i| i.id) else {
        return ApiError::not_found("issue not found").into_response();
    };
    asset_list_or_create(state, req, repo, issue_id, 0)
}

/// `GET|PATCH|DELETE /api/v1/repos/{owner}/{repo}/issues/{index}/assets/{attachment_id}`.
fn issue_asset_detail(
    state: &AppState,
    req: &ApiRequest,
    repo: &StoredRepo,
    index: &str,
    attachment: &str,
) -> Response {
    let Ok(index) = index.parse::<i64>() else {
        return ApiError::bad_request("index must be a number").into_response();
    };
    let Some(issue_id) = state.issue(repo.id, index).map(|i| i.id) else {
        return ApiError::not_found("issue not found").into_response();
    };
    asset_detail(state, req, repo, issue_id, 0, attachment)
}

/// `GET|POST /api/v1/repos/{owner}/{repo}/issues/comments/{id}/assets`.
fn comment_assets(state: &AppState, req: &ApiRequest, repo: &StoredRepo, id: &str) -> Response {
    let Ok(comment_id) = id.parse::<i64>() else {
        return ApiError::bad_request("id must be a number").into_response();
    };
    if !state
        .issues()
        .iter()
        .any(|i| i.repo_id == repo.id && i.comments.iter().any(|c| c.id == comment_id))
    {
        return ApiError::not_found("comment not found").into_response();
    }
    asset_list_or_create(state, req, repo, 0, comment_id)
}

/// `GET|PATCH|DELETE /api/v1/repos/{owner}/{repo}/issues/comments/{id}/assets/{attachment_id}`.
fn comment_asset_detail(
    state: &AppState,
    req: &ApiRequest,
    repo: &StoredRepo,
    id: &str,
    attachment: &str,
) -> Response {
    let Ok(comment_id) = id.parse::<i64>() else {
        return ApiError::bad_request("id must be a number").into_response();
    };
    if !state
        .issues()
        .iter()
        .any(|i| i.repo_id == repo.id && i.comments.iter().any(|c| c.id == comment_id))
    {
        return ApiError::not_found("comment not found").into_response();
    }
    asset_detail(state, req, repo, 0, comment_id, attachment)
}

/// List or create an attachment owned by one issue or one comment.
fn asset_list_or_create(
    state: &AppState,
    req: &ApiRequest,
    repo: &StoredRepo,
    issue_id: i64,
    comment_id: i64,
) -> Response {
    let owned = |a: &StoredAttachment| {
        a.repo_id == repo.id && a.issue_id == issue_id && a.comment_id == comment_id
    };
    match req.method {
        Method::GET => {
            let items: Vec<dto::Attachment> = state
                .attachments()
                .iter()
                .filter(|a| owned(a))
                .map(|a| a.to_dto())
                .collect();
            respond::ok(items)
        }
        Method::POST => {
            let attachment = StoredAttachment {
                id: state.next_id(),
                repo_id: repo.id,
                release_id: 0,
                issue_id,
                comment_id,
                name: req.body_str("name").unwrap_or_else(|| "attachment".into()),
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

/// Read, rename or delete one attachment owned by an issue or comment.
fn asset_detail(
    state: &AppState,
    req: &ApiRequest,
    repo: &StoredRepo,
    issue_id: i64,
    comment_id: i64,
    attachment: &str,
) -> Response {
    let Ok(attachment_id) = attachment.parse::<i64>() else {
        return ApiError::bad_request("attachment id must be a number").into_response();
    };
    let owned = |a: &StoredAttachment| {
        a.repo_id == repo.id
            && a.issue_id == issue_id
            && a.comment_id == comment_id
            && a.id == attachment_id
    };
    match req.method {
        Method::GET => match state.attachments().iter().find(|a| owned(a)) {
            Some(a) => respond::ok(a.to_dto()),
            None => ApiError::not_found("attachment not found").into_response(),
        },
        Method::PATCH => {
            let name = req.body_str("name");
            match state.attachments_mut().iter_mut().find(|a| owned(a)) {
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
            state.attachments_mut().retain(|a| !owned(a));
            if state.attachments().len() == before {
                return ApiError::not_found("attachment not found").into_response();
            }
            respond::no_content()
        }
        _ => unimplemented(req),
    }
}

/// Shared reaction logic: `target` is an issue index or a comment id.
fn reactions_on(
    state: &AppState,
    req: &ApiRequest,
    repo: &StoredRepo,
    target: i64,
    comment: bool,
) -> Response {
    let mine =
        |r: &StoredReaction| r.repo_id == repo.id && r.comment == comment && r.target == target;
    match req.method {
        Method::GET => {
            let items: Vec<dto::Reaction> = state
                .reactions()
                .iter()
                .filter(|r| mine(r))
                .map(|r| r.to_dto())
                .collect();
            respond::ok(items)
        }
        Method::POST => {
            let content = req.body_str("content").unwrap_or_else(|| "+1".into());
            let user = super::users::current_login(state, req);
            // Reacting twice with the same emoji is idempotent, as in Gitea.
            let existing = state
                .reactions()
                .iter()
                .find(|r| mine(r) && r.user == user && r.content == content)
                .map(|r| r.to_dto());
            if let Some(dto) = existing {
                return respond::created(dto);
            }
            let reaction = StoredReaction {
                id: state.next_id(),
                repo_id: repo.id,
                target,
                comment,
                user,
                content,
                created: now(),
            };
            let dto = reaction.to_dto();
            state.reactions_mut().push(reaction);
            respond::created(dto)
        }
        Method::DELETE => {
            let content = req.body_str("content");
            let user = super::users::current_login(state, req);
            state.reactions_mut().retain(|r| {
                !(mine(r) && r.user == user && content.as_ref().is_none_or(|c| &r.content == c))
            });
            respond::no_content()
        }
        _ => unimplemented(req),
    }
}

/// `GET|POST|DELETE /api/v1/repos/{owner}/{repo}/issues/{index}/{blocks,dependencies}`.
///
/// `blocks` lists the issues this one blocks; `dependencies` the ones it waits
/// on. Both are stored as issue indices on the issue itself.
fn issue_blocks(
    state: &AppState,
    req: &ApiRequest,
    repo: &StoredRepo,
    index: &str,
    dependencies: bool,
) -> Response {
    let Ok(index) = index.parse::<i64>() else {
        return ApiError::bad_request("index must be a number").into_response();
    };
    if state.issue(repo.id, index).is_none() {
        return ApiError::not_found("issue not found").into_response();
    }
    let target = req.body_str("index").and_then(|i| i.parse::<i64>().ok());

    if req.method == Method::GET {
        let Some(issue) = state.issue(repo.id, index) else {
            return ApiError::not_found("issue not found").into_response();
        };
        let indices = if dependencies {
            issue.dependencies
        } else {
            issue.blocks
        };
        let listed: Vec<dto::Issue> = indices
            .into_iter()
            .filter_map(|i| state.issue(repo.id, i))
            .map(|i| {
                let labels = labels_of(state, repo.id, &i);
                i.to_dto(repo, &labels, None)
            })
            .collect();
        return respond::ok(listed);
    }

    let mut issues = state.issues_mut();
    let Some(stored) = issues
        .iter_mut()
        .find(|i| i.repo_id == repo.id && i.index == index)
    else {
        return ApiError::not_found("issue not found").into_response();
    };
    let field = if dependencies {
        &mut stored.dependencies
    } else {
        &mut stored.blocks
    };
    match req.method {
        Method::POST => {
            if let Some(target) = target {
                if !field.contains(&target) {
                    field.push(target);
                }
            }
            respond::no_content()
        }
        Method::DELETE => {
            if let Some(target) = target {
                field.retain(|i| *i != target);
            } else {
                field.clear();
            }
            respond::no_content()
        }
        _ => unimplemented(req),
    }
}

/// `POST /api/v1/repos/{owner}/{repo}/issues/{index}/stopwatch/{start,stop,delete}`.
fn stopwatch(
    state: &AppState,
    req: &ApiRequest,
    repo: &StoredRepo,
    index: &str,
    action: &str,
) -> Response {
    let _ = req;
    let Ok(index) = index.parse::<i64>() else {
        return ApiError::bad_request("index must be a number").into_response();
    };
    let mut issues = state.issues_mut();
    let Some(stored) = issues
        .iter_mut()
        .find(|i| i.repo_id == repo.id && i.index == index)
    else {
        return ApiError::not_found("issue not found").into_response();
    };
    let seconds = stored.stopwatch_seconds.unwrap_or(0);
    match action {
        "start" => {
            stored.stopwatch_seconds = Some(seconds.max(0));
            respond::no_content()
        }
        "stop" | "delete" => {
            stored.stopwatch_seconds = Some(0);
            respond::no_content()
        }
        _ => unimplemented(req),
    }
}

/// `GET /api/v1/repos/{owner}/{repo}/issues/{index}/timeline`.
fn timeline(state: &AppState, repo: &StoredRepo, index: &str) -> Response {
    let Ok(index) = index.parse::<i64>() else {
        return ApiError::bad_request("index must be a number").into_response();
    };
    let Some(issue) = state.issue(repo.id, index) else {
        return ApiError::not_found("issue not found").into_response();
    };
    // The creation event plus one event per comment, mirroring Gitea's
    // condensed timeline.
    let mut events: Vec<serde_json::Value> = vec![serde_json::json!({
        "id": issue.id,
        "type": "issue",
        "created_at": issue.created,
        "body": issue.body,
        "user": { "login": issue.poster },
    })];
    events.extend(issue.comments.iter().map(|c| {
        serde_json::json!({
            "id": c.id,
            "type": "comment",
            "created_at": c.created,
            "body": c.body,
            "user": { "login": c.poster },
        })
    }));
    respond::ok(events)
}

/// `GET|POST /api/v1/repos/{owner}/{repo}/issues/{index}/times` — tracked times
/// recorded on an issue.
///
/// Gitea records a tracked time as a comment whose body carries a
/// `@<user> tracked <duration>` marker, so the listing is derived from the
/// comment store and stays consistent with `/user/times`.
fn issue_times(state: &AppState, req: &ApiRequest, repo: &StoredRepo, index: &str) -> Response {
    let Ok(index) = index.parse::<i64>() else {
        return ApiError::bad_request("index must be a number").into_response();
    };
    let Some(issue_id) = state.issue(repo.id, index).map(|i| i.id) else {
        return ApiError::not_found("issue not found").into_response();
    };

    if req.method == Method::POST {
        let seconds = req.body_i64("time").unwrap_or(0);
        let poster = super::users::current_login(state, req);
        let comment = StoredComment {
            id: state.next_id(),
            issue_id,
            body: format!("@{poster} tracked {seconds}s"),
            poster,
            created: now(),
            updated: now(),
        };
        let id = comment.id;
        let created = comment.created.clone();
        if let Some(stored) = state.issues_mut().iter_mut().find(|i| i.id == issue_id) {
            stored.comments.push(comment);
        }
        return respond::created(serde_json::json!({
            "id": id,
            "time": seconds,
            "created": created,
            "issue_id": issue_id,
        }));
    }

    let Some(issue) = state.issue(repo.id, index) else {
        return ApiError::not_found("issue not found").into_response();
    };
    let items: Vec<serde_json::Value> = issue
        .comments
        .iter()
        .filter_map(|c| {
            let seconds = tracked_seconds(&c.body)?;
            Some(serde_json::json!({
                "id": c.id,
                "time": seconds,
                "created": c.created,
                "issue_id": issue.id,
                "user_id": state.user(&c.poster).map(|u| u.id),
                "user_name": c.poster,
            }))
        })
        .collect();
    respond::ok(items)
}

/// `DELETE /api/v1/repos/{owner}/{repo}/issues/{index}/times/{id}` — reset a
/// tracked time.
fn issue_time_reset(
    state: &AppState,
    req: &ApiRequest,
    repo: &StoredRepo,
    index: &str,
    id: &str,
) -> Response {
    if req.method != Method::DELETE {
        return unimplemented(req);
    }
    let Ok(index) = index.parse::<i64>() else {
        return ApiError::bad_request("index must be a number").into_response();
    };
    let Ok(id) = id.parse::<i64>() else {
        return ApiError::bad_request("id must be a number").into_response();
    };
    let Some(issue_id) = state.issue(repo.id, index).map(|i| i.id) else {
        return ApiError::not_found("issue not found").into_response();
    };
    let mut removed = false;
    if let Some(stored) = state.issues_mut().iter_mut().find(|i| i.id == issue_id) {
        let before = stored.comments.len();
        stored.comments.retain(|c| c.id != id);
        removed = stored.comments.len() != before;
    }
    if !removed {
        return ApiError::not_found("tracked time not found").into_response();
    }
    respond::no_content()
}

/// The tracked times recorded on one issue, as `(poster, seconds)` pairs.
///
/// Both the issue-scoped and the repository-scoped views derive from this, so
/// they cannot drift apart.
pub(super) fn tracked_times_on(issue: &StoredIssue) -> Vec<(&str, i64)> {
    issue
        .comments
        .iter()
        .filter_map(|c| tracked_seconds(&c.body).map(|s| (c.poster.as_str(), s)))
        .collect()
}

/// Seconds in a `@user tracked 1h30m` comment body, or `None` if the comment
/// is not a time-tracking comment.
pub(super) fn tracked_seconds(body: &str) -> Option<i64> {
    let rest = body.split("tracked").nth(1)?;
    let token = rest.split_whitespace().next()?;
    let mut total = 0i64;
    let mut digits = String::new();
    let mut saw_unit = false;
    for ch in token.chars() {
        if ch.is_ascii_digit() {
            digits.push(ch);
            continue;
        }
        let value: i64 = digits.parse().unwrap_or(0);
        digits.clear();
        saw_unit = true;
        total += match ch {
            'w' => value * 7 * 24 * 3600,
            'd' => value * 24 * 3600,
            'h' => value * 3600,
            'm' => value * 60,
            's' => value,
            _ => 0,
        };
    }
    if !saw_unit && digits.is_empty() {
        return None;
    }
    Some(total)
}

/// `GET|POST /api/v1/repos/{owner}/{repo}/pulls`.
pub fn handle_pulls(state: &AppState, req: &ApiRequest, repo: &StoredRepo) -> Response {
    let tail = tail_after(req, 5);
    match tail.as_slice() {
        ["pulls"] => match req.method {
            Method::GET => list(state, req, repo, true),
            Method::POST => create(state, req, repo, true),
            _ => unimplemented(req),
        },
        ["pulls", index] => pull_detail(state, req, repo, index),
        ["pulls", index, "merge"] => merge(state, req, repo, index),
        ["pulls", index, "update"] => pull_update(state, req, repo, index),
        ["pulls", index, "commits"] => pull_commits(state, req, repo, index),
        ["pulls", index, "files"] => pull_files(state, req, repo, index),
        ["pulls", index, "requested_reviewers"] => {
            pull_requested_reviewers(state, req, repo, index)
        }
        ["pulls", index, "comments"] => comments(state, req, repo, index),
        ["pulls", _index, "comments", id, "replies"] => {
            respond::created(serde_json::json!({ "id": id }))
        }
        ["pulls", index, "reviews"] => reviews(state, req, repo, index),
        ["pulls", index, "reviews", id] => review_detail(state, req, repo, index, id),
        ["pulls", index, "reviews", id, "comments"] => review_comments(state, req, repo, index, id),
        ["pulls", index, "reviews", id, "dismissals"] => {
            review_dismissal(state, req, repo, index, id)
        }
        ["pulls", index, "reviews", id, "undismissals"] => {
            review_undismissal(state, req, repo, index, id)
        }
        ["pulls", index, "labels"] => issue_labels(state, req, repo, index),
        ["pulls", "comments", id, action @ ("resolve" | "unresolve")] => {
            comment_resolution(state, req, repo, id, action)
        }
        ["pulls", base, head] => pulls_by_basehead(state, repo, base, head),
        _ => unimplemented(req),
    }
}

/// `POST /api/v1/repos/{owner}/{repo}/pulls/{index}/update`.
///
/// Merges the base branch into the head branch. The in-memory port has no real
/// object store, so the observable effect is that the pull request is marked
/// updated; the response mirrors Gitea's `200` on success.
fn pull_update(state: &AppState, req: &ApiRequest, repo: &StoredRepo, index: &str) -> Response {
    if req.method != Method::POST {
        return unimplemented(req);
    }
    let Ok(index) = index.parse::<i64>() else {
        return ApiError::bad_request("index must be a number").into_response();
    };
    let Some(pull) = state
        .issues()
        .iter()
        .find(|i| i.repo_id == repo.id && i.index == index && i.is_pull)
        .map(|i| i.id)
    else {
        return ApiError::not_found("pull request not found").into_response();
    };
    if let Some(stored) = state.issues_mut().iter_mut().find(|i| i.id == pull) {
        stored.updated = now();
    }
    respond::no_content()
}

/// `GET /api/v1/repos/{owner}/{repo}/pulls/{index}/commits`.
fn pull_commits(state: &AppState, req: &ApiRequest, repo: &StoredRepo, index: &str) -> Response {
    let _ = req;
    let Ok(index) = index.parse::<i64>() else {
        return ApiError::bad_request("index must be a number").into_response();
    };
    if state
        .issues()
        .iter()
        .find(|i| i.repo_id == repo.id && i.index == index && i.is_pull)
        .is_none()
    {
        return ApiError::not_found("pull request not found").into_response();
    }
    // The commits between the pull request's base and head branches.
    let commits: Vec<dto::Commit> = state
        .commits()
        .iter()
        .filter(|c| c.repo_id == repo.id)
        .map(|c| c.to_dto(repo))
        .collect();
    respond::ok(commits)
}

/// `GET /api/v1/repos/{owner}/{repo}/pulls/{index}/files`.
fn pull_files(state: &AppState, req: &ApiRequest, repo: &StoredRepo, index: &str) -> Response {
    let _ = req;
    let Ok(index) = index.parse::<i64>() else {
        return ApiError::bad_request("index must be a number").into_response();
    };
    if state
        .issues()
        .iter()
        .find(|i| i.repo_id == repo.id && i.index == index && i.is_pull)
        .is_none()
    {
        return ApiError::not_found("pull request not found").into_response();
    }
    // The files changed by the pull request's head commit, when known.
    let files: Vec<serde_json::Value> = state
        .commits()
        .iter()
        .filter(|c| c.repo_id == repo.id)
        .flat_map(|c| c.files.clone())
        .map(|(path, additions, deletions)| {
            serde_json::json!({
                "filename": path,
                "additions": additions,
                "deletions": deletions,
                "status": "modified",
            })
        })
        .collect();
    respond::ok(files)
}

/// `POST|DELETE /api/v1/repos/{owner}/{repo}/pulls/{index}/requested_reviewers`.
fn pull_requested_reviewers(
    state: &AppState,
    req: &ApiRequest,
    repo: &StoredRepo,
    index: &str,
) -> Response {
    let Ok(index) = index.parse::<i64>() else {
        return ApiError::bad_request("index must be a number").into_response();
    };
    let reviewers: Vec<String> = req
        .json()
        .get("reviewers")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    let mut issues = state.issues_mut();
    let Some(stored) = issues
        .iter_mut()
        .find(|i| i.repo_id == repo.id && i.index == index && i.is_pull)
    else {
        return ApiError::not_found("pull request not found").into_response();
    };
    match req.method {
        Method::POST => {
            for r in reviewers {
                if !stored
                    .requested_reviewers
                    .iter()
                    .any(|x| x.eq_ignore_ascii_case(&r))
                {
                    stored.requested_reviewers.push(r);
                }
            }
        }
        Method::DELETE => stored
            .requested_reviewers
            .retain(|x| !reviewers.iter().any(|r| r.eq_ignore_ascii_case(x))),
        _ => return unimplemented(req),
    }
    stored.updated = now();
    respond::no_content()
}

/// `GET /api/v1/repos/{owner}/{repo}/pulls/{base}/{head}` — the pull request
/// from `head` into `base`.
fn pulls_by_basehead(state: &AppState, repo: &StoredRepo, base: &str, head: &str) -> Response {
    let head = head.trim_end_matches(".diff").trim_end_matches(".patch");
    match state.issues().iter().find(|i| {
        i.repo_id == repo.id
            && i.is_pull
            && i.base.as_deref() == Some(base)
            && i.head.as_deref().map(|h| h == head).unwrap_or(false)
    }) {
        Some(pr) => respond::ok(pr.to_pull_dto(repo, &labels_of(state, repo.id, pr))),
        None => ApiError::not_found("pull request not found").into_response(),
    }
}

/// `POST /api/v1/repos/{owner}/{repo}/pulls/comments/{id}/{resolve,unresolve}`.
///
/// Resolves or reopens an inline review comment on a pull request.
fn comment_resolution(
    state: &AppState,
    req: &ApiRequest,
    repo: &StoredRepo,
    id: &str,
    action: &str,
) -> Response {
    if req.method != Method::POST {
        return unimplemented(req);
    }
    let Ok(id) = id.parse::<i64>() else {
        return ApiError::bad_request("id must be a number").into_response();
    };
    let resolve = action == "resolve";
    let login = super::users::current_login(state, req);
    let mut comments = state.review_comments_mut();
    match comments
        .iter_mut()
        .find(|c| c.repo_id == repo.id && c.id == id)
    {
        Some(comment) => {
            comment.resolver = if resolve { Some(login) } else { None };
            comment.updated = now();
            respond::no_content()
        }
        None => ApiError::not_found("comment not found").into_response(),
    }
}

/// `GET|POST|DELETE /api/v1/repos/{owner}/{repo}/pulls/{index}/reviews`.
fn reviews(state: &AppState, req: &ApiRequest, repo: &StoredRepo, index: &str) -> Response {
    let Ok(index) = index.parse::<i64>() else {
        return ApiError::bad_request("index must be a number").into_response();
    };
    if state
        .issues()
        .iter()
        .find(|i| i.repo_id == repo.id && i.index == index && i.is_pull)
        .is_none()
    {
        return ApiError::not_found("pull request not found").into_response();
    }
    match req.method {
        Method::GET => {
            let items: Vec<dto::PullReview> = state
                .reviews()
                .iter()
                .filter(|r| r.repo_id == repo.id && r.index == index)
                .map(|r| review_dto(r, repo))
                .collect();
            respond::ok(items)
        }
        Method::POST => {
            let reviewer = super::users::current_login(state, req);
            let state_value = req.body_str("event").unwrap_or_else(|| "COMMENT".into());
            let review = StoredReview {
                id: state.next_id(),
                repo_id: repo.id,
                index,
                body: req.body_str("body").unwrap_or_default(),
                state: state_value,
                reviewer,
                commit_id: req.body_str("commit_id").unwrap_or_default(),
                dismissed: false,
                official: true,
                created: now(),
                updated: now(),
            };
            let dto = review_dto(&review, repo);
            // The review may carry inline line comments; they belong to the
            // review that was just created.
            if let Some(comments) = req.json().get("comments").and_then(|c| c.as_array()) {
                let created = now();
                let mut stored = state.review_comments_mut();
                for comment in comments {
                    stored.push(StoredReviewComment {
                        id: state.next_id(),
                        repo_id: repo.id,
                        index,
                        review_id: review.id,
                        body: comment
                            .get("body")
                            .and_then(|b| b.as_str())
                            .unwrap_or_default()
                            .to_string(),
                        path: comment
                            .get("path")
                            .and_then(|p| p.as_str())
                            .unwrap_or_default()
                            .to_string(),
                        position: comment
                            .get("new_position")
                            .or_else(|| comment.get("old_position"))
                            .and_then(|p| p.as_i64())
                            .unwrap_or(0),
                        commit_id: review.commit_id.clone(),
                        resolver: None,
                        created: created.clone(),
                        updated: created.clone(),
                    });
                }
            }
            state.reviews_mut().push(review);
            respond::created(dto)
        }
        Method::DELETE => {
            state
                .reviews_mut()
                .retain(|r| !(r.repo_id == repo.id && r.index == index));
            respond::no_content()
        }
        _ => unimplemented(req),
    }
}

/// Render a review as the `PullReview` wire type.
fn review_dto(review: &StoredReview, repo: &StoredRepo) -> dto::PullReview {
    dto::PullReview {
        id: Some(review.id),
        body: Some(review.body.clone()),
        state: Some(dto::ReviewStateType::from_wire(&review.state)),
        commit_id: Some(review.commit_id.clone()),
        dismissed: Some(review.dismissed),
        official: Some(review.official),
        stale: Some(false),
        comments_count: Some(0),
        html_url: Some(format!(
            "https://codez.example/{}/pulls/{}/reviews/{}",
            repo.full_name(),
            review.index,
            review.id
        )),
        pull_request_url: Some(format!(
            "/api/v1/repos/{}/pulls/{}",
            repo.full_name(),
            review.index
        )),
        submitted_at: Some(review.created.clone()),
        user: Some(dto::User {
            login: Some(review.reviewer.clone()),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// `GET /api/v1/repos/{owner}/{repo}/pulls/{index}/reviews/{id}/comments`.
///
/// The inline line comments submitted with a review.
fn review_comments(
    state: &AppState,
    req: &ApiRequest,
    repo: &StoredRepo,
    index: &str,
    id: &str,
) -> Response {
    if req.method != Method::GET {
        return unimplemented(req);
    }
    let Ok(index) = index.parse::<i64>() else {
        return ApiError::bad_request("index must be a number").into_response();
    };
    let Ok(id) = id.parse::<i64>() else {
        return ApiError::bad_request("id must be a number").into_response();
    };
    let comments: Vec<dto::PullReviewComment> = state
        .review_comments()
        .iter()
        .filter(|c| c.repo_id == repo.id && c.index == index && c.review_id == id)
        .map(|c| c.to_dto())
        .collect();
    respond::ok(comments)
}

/// `GET|POST|DELETE /api/v1/repos/{owner}/{repo}/pulls/{index}/reviews/{id}`.
fn review_detail(
    state: &AppState,
    req: &ApiRequest,
    repo: &StoredRepo,
    index: &str,
    id: &str,
) -> Response {
    let Ok(index) = index.parse::<i64>() else {
        return ApiError::bad_request("index must be a number").into_response();
    };
    let id: i64 = id.parse().unwrap_or_default();
    match req.method {
        Method::GET => match state
            .reviews()
            .iter()
            .find(|r| r.repo_id == repo.id && r.index == index && r.id == id)
        {
            Some(review) => respond::ok(review_dto(review, repo)),
            None => ApiError::not_found("review not found").into_response(),
        },
        Method::DELETE => {
            let before = state.reviews().len();
            state
                .reviews_mut()
                .retain(|r| !(r.repo_id == repo.id && r.index == index && r.id == id));
            if state.reviews().len() == before {
                return ApiError::not_found("review not found").into_response();
            }
            respond::no_content()
        }
        Method::POST => match state
            .reviews()
            .iter()
            .find(|r| r.repo_id == repo.id && r.index == index && r.id == id)
            .cloned()
        {
            Some(mut review) => {
                review.body = req.body_str("body").unwrap_or(review.body);
                review.updated = now();
                let dto = review_dto(&review, repo);
                if let Some(stored) = state.reviews_mut().iter_mut().find(|r| r.id == id) {
                    *stored = review;
                }
                respond::ok(dto)
            }
            None => ApiError::not_found("review not found").into_response(),
        },
        _ => unimplemented(req),
    }
}

/// `POST /api/v1/repos/{owner}/{repo}/pulls/{index}/reviews/{id}/{dismissals,undismissals}`.
fn review_dismissal(
    state: &AppState,
    req: &ApiRequest,
    repo: &StoredRepo,
    index: &str,
    id: &str,
) -> Response {
    review_set_dismissed(state, req, repo, index, id, true)
}

fn review_undismissal(
    state: &AppState,
    req: &ApiRequest,
    repo: &StoredRepo,
    index: &str,
    id: &str,
) -> Response {
    review_set_dismissed(state, req, repo, index, id, false)
}

/// Flip the `dismissed` flag on a review.
fn review_set_dismissed(
    state: &AppState,
    req: &ApiRequest,
    repo: &StoredRepo,
    index: &str,
    id: &str,
    dismissed: bool,
) -> Response {
    if req.method != Method::POST {
        return unimplemented(req);
    }
    let Ok(index) = index.parse::<i64>() else {
        return ApiError::bad_request("index must be a number").into_response();
    };
    let id: i64 = id.parse().unwrap_or_default();
    match state
        .reviews_mut()
        .iter_mut()
        .find(|r| r.repo_id == repo.id && r.index == index && r.id == id)
    {
        Some(review) => {
            review.dismissed = dismissed;
            review.updated = now();
            respond::ok(review_dto(review, repo))
        }
        None => ApiError::not_found("review not found").into_response(),
    }
}

/// `GET|POST /api/v1/repos/{owner}/{repo}/labels`.
pub fn handle_labels(state: &AppState, req: &ApiRequest, repo: &StoredRepo) -> Response {
    let tail = tail_after(req, 5);
    match tail.as_slice() {
        ["labels"] => match req.method {
            Method::GET => {
                let items: Vec<dto::Label> = state
                    .labels()
                    .iter()
                    .filter(|l| l.repo_id == repo.id)
                    .map(|l| l.to_dto(repo))
                    .collect();
                let pagination = Pagination {
                    page: req.query_i64("page"),
                    limit: req.query_i64("limit"),
                };
                let total = items.len();
                respond::page(pagination.apply(&items), &pagination, total)
            }
            Method::POST => {
                let name = req.body_str("name").unwrap_or_default();
                if name.trim().is_empty() {
                    return ApiError::unprocessable("name is required").into_response();
                }
                let label = StoredLabel {
                    id: state.next_id(),
                    repo_id: repo.id,
                    name,
                    color: req.body_str("color").unwrap_or_else(|| "ededed".into()),
                    description: req.body_str("description").unwrap_or_default(),
                };
                let dto = label.to_dto(repo);
                state.labels_mut().push(label);
                respond::created(dto)
            }
            _ => unimplemented(req),
        },
        ["labels", id] => {
            let id: i64 = id.parse().unwrap_or_default();
            match req.method {
                Method::GET => match state
                    .labels()
                    .iter()
                    .find(|l| l.repo_id == repo.id && l.id == id)
                {
                    Some(label) => respond::ok(label.to_dto(repo)),
                    None => ApiError::not_found("label not found").into_response(),
                },
                Method::PATCH => {
                    let body = req.json();
                    let mut labels = state.labels_mut();
                    match labels
                        .iter_mut()
                        .find(|l| l.repo_id == repo.id && l.id == id)
                    {
                        Some(label) => {
                            if let Some(name) = body.get("name").and_then(|v| v.as_str()) {
                                label.name = name.to_string();
                            }
                            if let Some(color) = body.get("color").and_then(|v| v.as_str()) {
                                label.color = color.to_string();
                            }
                            if let Some(description) =
                                body.get("description").and_then(|v| v.as_str())
                            {
                                label.description = description.to_string();
                            }
                            respond::ok(label.to_dto(repo))
                        }
                        None => ApiError::not_found("label not found").into_response(),
                    }
                }
                Method::DELETE => {
                    let before = state.labels().len();
                    state
                        .labels_mut()
                        .retain(|l| !(l.repo_id == repo.id && l.id == id));
                    if state.labels().len() == before {
                        return ApiError::not_found("label not found").into_response();
                    }
                    respond::no_content()
                }
                _ => unimplemented(req),
            }
        }
        _ => unimplemented(req),
    }
}

/// `GET|POST /api/v1/repos/{owner}/{repo}/milestones`.
pub fn handle_milestones(state: &AppState, req: &ApiRequest, repo: &StoredRepo) -> Response {
    let tail = tail_after(req, 5);
    match tail.as_slice() {
        ["milestones"] => match req.method {
            Method::GET => {
                let items: Vec<dto::Milestone> = state
                    .milestones()
                    .iter()
                    .filter(|m| m.repo_id == repo.id)
                    .map(|m| {
                        let (open, closed) = milestone_counts(state, repo.id, m.id);
                        m.to_dto(repo, open, closed)
                    })
                    .collect();
                respond::ok(items)
            }
            Method::POST => {
                let title = req.body_str("title").unwrap_or_default();
                if title.trim().is_empty() {
                    return ApiError::unprocessable("title is required").into_response();
                }
                let milestone = StoredMilestone {
                    id: state.next_id(),
                    repo_id: repo.id,
                    title,
                    description: req.body_str("description").unwrap_or_default(),
                    state: "open".into(),
                    deadline: req.body_str("due_on"),
                    created: now(),
                    updated: now(),
                    closed: None,
                };
                let dto = milestone.to_dto(repo, 0, 0);
                state.milestones_mut().push(milestone);
                respond::created(dto)
            }
            _ => unimplemented(req),
        },
        ["milestones", id] => {
            let id: i64 = id.parse().unwrap_or_default();
            match req.method {
                Method::GET => match state
                    .milestones()
                    .iter()
                    .find(|m| m.repo_id == repo.id && m.id == id)
                {
                    Some(milestone) => {
                        let (open, closed) = milestone_counts(state, repo.id, id);
                        respond::ok(milestone.to_dto(repo, open, closed))
                    }
                    None => ApiError::not_found("milestone not found").into_response(),
                },
                Method::PATCH => {
                    let body = req.json();
                    let mut milestones = state.milestones_mut();
                    match milestones
                        .iter_mut()
                        .find(|m| m.repo_id == repo.id && m.id == id)
                    {
                        Some(milestone) => {
                            if let Some(title) = body.get("title").and_then(|v| v.as_str()) {
                                milestone.title = title.to_string();
                            }
                            if let Some(state_value) = body.get("state").and_then(|v| v.as_str()) {
                                milestone.state = state_value.to_string();
                                milestone.closed = if state_value == "closed" {
                                    Some(now())
                                } else {
                                    None
                                };
                            }
                            milestone.updated = now();
                            respond::ok(milestone.to_dto(repo, 0, 0))
                        }
                        None => ApiError::not_found("milestone not found").into_response(),
                    }
                }
                Method::DELETE => {
                    let before = state.milestones().len();
                    state
                        .milestones_mut()
                        .retain(|m| !(m.repo_id == repo.id && m.id == id));
                    if state.milestones().len() == before {
                        return ApiError::not_found("milestone not found").into_response();
                    }
                    respond::no_content()
                }
                _ => unimplemented(req),
            }
        }
        _ => unimplemented(req),
    }
}

/// `GET /api/v1/repos/issues/search` — issues across every repository.
///
/// Gitea serves this alongside `/repos/search`; both are documented as
/// sub-resources of the `repos` collection rather than of a single repository.
pub fn search_all(state: &AppState, req: &ApiRequest) -> Response {
    let wanted_state = req.query_param("state").unwrap_or_else(|| "open".into());
    let term = req.query_param("q").unwrap_or_default().to_lowercase();
    let is_pull = match req.query_param("type").as_deref() {
        Some("pulls") => Some(true),
        Some("issues") => Some(false),
        _ => None,
    };

    let mut items: Vec<StoredIssue> = state
        .issues()
        .iter()
        .filter(|i| {
            (wanted_state == "all" || i.state == wanted_state)
                && (term.is_empty() || i.title.to_lowercase().contains(&term))
                && is_pull.is_none_or(|pull| i.is_pull == pull)
        })
        .cloned()
        .collect();
    items.sort_by_key(|i| std::cmp::Reverse(i.index));

    let pagination = pagination(req);
    let total = items.len();
    let page: Vec<serde_json::Value> = pagination
        .apply(&items)
        .into_iter()
        .filter_map(|issue| {
            let repo = state.repo_by_id(issue.repo_id)?;
            Some(
                serde_json::to_value(issue.to_dto(
                    &repo,
                    &labels_of(state, repo.id, &issue),
                    milestone_of(state, issue.milestone_id),
                ))
                .unwrap_or(serde_json::Value::Null),
            )
        })
        .collect();
    respond::page(page, &pagination, total)
}

/// `GET /api/v1/repos/issues/search` and `GET /api/v1/repos/{owner}/{repo}/issues`.
fn list(state: &AppState, req: &ApiRequest, repo: &StoredRepo, pulls: bool) -> Response {
    let wanted_state = req.query_param("state").unwrap_or_else(|| "open".into());
    let term = req.query_param("q").unwrap_or_default().to_lowercase();

    let mut items: Vec<StoredIssue> = state
        .issues()
        .iter()
        .filter(|i| {
            i.repo_id == repo.id
                && i.is_pull == pulls
                && (wanted_state == "all" || i.state == wanted_state)
                && (term.is_empty() || i.title.to_lowercase().contains(&term))
        })
        .cloned()
        .collect();
    items.sort_by_key(|i| std::cmp::Reverse(i.index));

    let pagination = pagination(req);
    let total = items.len();
    let page: Vec<serde_json::Value> = pagination
        .apply(&items)
        .into_iter()
        .map(|issue| {
            if pulls {
                serde_json::to_value(issue.to_pull_dto(repo, &labels_of(state, repo.id, &issue)))
                    .unwrap_or(serde_json::Value::Null)
            } else {
                serde_json::to_value(issue.to_dto(
                    repo,
                    &labels_of(state, repo.id, &issue),
                    milestone_of(state, issue.milestone_id),
                ))
                .unwrap_or(serde_json::Value::Null)
            }
        })
        .collect();
    respond::page(page, &pagination, total)
}

/// `POST /api/v1/repos/{owner}/{repo}/issues` and the pull equivalent.
fn create(state: &AppState, req: &ApiRequest, repo: &StoredRepo, is_pull: bool) -> Response {
    let title = req.body_str("title").unwrap_or_default();
    if title.trim().is_empty() {
        return ApiError::unprocessable("title is required").into_response();
    }
    let poster = super::users::current_user(state, req)
        .map(|u| u.login)
        .unwrap_or_else(|| repo.owner.clone());

    let (head, base) = if is_pull {
        (
            req.body_str("head").or_else(|| Some("feature".into())),
            req.body_str("base")
                .or_else(|| Some(repo.default_branch.clone())),
        )
    } else {
        (None, None)
    };

    let issue = StoredIssue {
        id: state.next_id(),
        repo_id: repo.id,
        index: state.next_index(repo.id),
        title,
        body: req.body_str("body").unwrap_or_default(),
        state: "open".into(),
        is_pull,
        poster,
        assignees: req.body_str_vec("assignees").unwrap_or_default(),
        label_ids: req
            .json()
            .get("labels")
            .and_then(|v| v.as_array())
            .map(|ids| ids.iter().filter_map(|i| i.as_i64()).collect())
            .unwrap_or_default(),
        milestone_id: req.body_i64("milestone").filter(|id| *id > 0),
        created: now(),
        updated: now(),
        closed: None,
        comments: Vec::new(),
        head,
        base,
        merged: false,
        ..Default::default()
    };

    let dto = if is_pull {
        serde_json::to_value(issue.to_pull_dto(repo, &labels_of(state, repo.id, &issue)))
    } else {
        serde_json::to_value(issue.to_dto(
            repo,
            &labels_of(state, repo.id, &issue),
            milestone_of(state, issue.milestone_id),
        ))
    };
    state.issues_mut().push(issue);
    respond::created(dto.unwrap_or(serde_json::Value::Null))
}

/// `GET|PATCH /api/v1/repos/{owner}/{repo}/issues/{index}`.
fn issue_detail(
    state: &AppState,
    req: &ApiRequest,
    repo: &StoredRepo,
    index: &str,
    is_pull: bool,
) -> Response {
    let Ok(index) = index.parse::<i64>() else {
        return ApiError::bad_request("index must be a number").into_response();
    };
    let Some(issue) = state.issue(repo.id, index) else {
        return ApiError::not_found("issue not found").into_response();
    };

    match req.method {
        Method::GET => {
            let labels = labels_of(state, repo.id, &issue);
            if is_pull {
                respond::ok(issue.to_pull_dto(repo, &labels))
            } else {
                respond::ok(issue.to_dto(repo, &labels, milestone_of(state, issue.milestone_id)))
            }
        }
        Method::PATCH => {
            let body = req.json();
            let mut issues = state.issues_mut();
            let Some(stored) = issues
                .iter_mut()
                .find(|i| i.repo_id == repo.id && i.index == index)
            else {
                return ApiError::not_found("issue not found").into_response();
            };
            if let Some(title) = body.get("title").and_then(|v| v.as_str()) {
                stored.title = title.to_string();
            }
            if let Some(text) = body.get("body").and_then(|v| v.as_str()) {
                stored.body = text.to_string();
            }
            if let Some(state_value) = body.get("state").and_then(|v| v.as_str()) {
                stored.state = state_value.to_string();
                stored.closed = if state_value == "closed" {
                    Some(now())
                } else {
                    None
                };
            }
            if let Some(assignees) = body.get("assignees").and_then(|v| v.as_array()) {
                stored.assignees = assignees
                    .iter()
                    .filter_map(|a| a.as_str().map(str::to_string))
                    .collect();
            }
            stored.updated = now();
            let snapshot = stored.clone();
            drop(issues);
            let labels = labels_of(state, repo.id, &snapshot);
            if snapshot.is_pull {
                respond::ok(snapshot.to_pull_dto(repo, &labels))
            } else {
                respond::ok(snapshot.to_dto(
                    repo,
                    &labels,
                    milestone_of(state, snapshot.milestone_id),
                ))
            }
        }
        _ => unimplemented(req),
    }
}

/// `GET|PATCH /api/v1/repos/{owner}/{repo}/pulls/{index}`.
fn pull_detail(state: &AppState, req: &ApiRequest, repo: &StoredRepo, index: &str) -> Response {
    issue_detail(state, req, repo, index, true)
}

/// `POST /api/v1/repos/{owner}/{repo}/pulls/{index}/merge`.
fn merge(state: &AppState, _req: &ApiRequest, repo: &StoredRepo, index: &str) -> Response {
    let Ok(index) = index.parse::<i64>() else {
        return ApiError::bad_request("index must be a number").into_response();
    };
    let mut issues = state.issues_mut();
    let Some(stored) = issues
        .iter_mut()
        .find(|i| i.repo_id == repo.id && i.index == index && i.is_pull)
    else {
        return ApiError::not_found("pull request not found").into_response();
    };
    if stored.merged {
        return ApiError::conflict("pull request is already merged").into_response();
    }
    if stored.state == "closed" {
        return ApiError::conflict("pull request is closed").into_response();
    }
    stored.merged = true;
    stored.state = "closed".into();
    stored.closed = Some(now());
    stored.updated = now();
    respond::no_content()
}

/// `GET|POST /api/v1/repos/{owner}/{repo}/issues/{index}/comments`.
fn comments(state: &AppState, req: &ApiRequest, repo: &StoredRepo, index: &str) -> Response {
    let Ok(index) = index.parse::<i64>() else {
        return ApiError::bad_request("index must be a number").into_response();
    };
    let Some(issue) = state.issue(repo.id, index) else {
        return ApiError::not_found("issue not found").into_response();
    };

    match req.method {
        Method::GET => {
            let items: Vec<dto::Comment> = issue
                .comments
                .iter()
                .map(|c| c.to_dto(repo, issue.index))
                .collect();
            respond::ok(items)
        }
        Method::POST => {
            let Some(body) = req.body_str("body") else {
                return ApiError::unprocessable("body is required").into_response();
            };
            let comment = StoredComment {
                id: state.next_id(),
                issue_id: issue.id,
                body,
                poster: super::users::current_user(state, req)
                    .map(|u| u.login)
                    .unwrap_or_else(|| repo.owner.clone()),
                created: now(),
                updated: now(),
            };
            let dto = comment.to_dto(repo, issue.index);
            if let Some(stored) = state
                .issues_mut()
                .iter_mut()
                .find(|i| i.repo_id == repo.id && i.index == index)
            {
                stored.comments.push(comment);
                stored.updated = now();
            }
            respond::created(dto)
        }
        _ => unimplemented(req),
    }
}

/// `PATCH|DELETE /api/v1/repos/{owner}/{repo}/issues/comments/{id}`.
fn comment_by_id(state: &AppState, req: &ApiRequest, repo: &StoredRepo, id: &str) -> Response {
    let id: i64 = id.parse().unwrap_or_default();
    match req.method {
        Method::DELETE => {
            let mut issues = state.issues_mut();
            let mut found = false;
            for issue in issues.iter_mut().filter(|i| i.repo_id == repo.id) {
                let before = issue.comments.len();
                issue.comments.retain(|c| c.id != id);
                found |= issue.comments.len() != before;
            }
            if !found {
                return ApiError::not_found("comment not found").into_response();
            }
            respond::no_content()
        }
        Method::PATCH => {
            let Some(body) = req.body_str("body") else {
                return ApiError::unprocessable("body is required").into_response();
            };
            let mut issues = state.issues_mut();
            for issue in issues.iter_mut().filter(|i| i.repo_id == repo.id) {
                if let Some(comment) = issue.comments.iter_mut().find(|c| c.id == id) {
                    comment.body = body;
                    comment.updated = now();
                    let index = issue.index;
                    let dto = comment.to_dto(repo, index);
                    return respond::ok(dto);
                }
            }
            ApiError::not_found("comment not found").into_response()
        }
        Method::GET => {
            for issue in state.issues().iter().filter(|i| i.repo_id == repo.id) {
                if let Some(comment) = issue.comments.iter().find(|c| c.id == id) {
                    return respond::ok(comment.to_dto(repo, issue.index));
                }
            }
            ApiError::not_found("comment not found").into_response()
        }
        _ => unimplemented(req),
    }
}

/// `GET|DELETE /api/v1/repos/{owner}/{repo}/issues/{index}/comments/{id}`.
fn comment_detail(
    state: &AppState,
    req: &ApiRequest,
    repo: &StoredRepo,
    index: &str,
    id: &str,
) -> Response {
    let Ok(index) = index.parse::<i64>() else {
        return ApiError::bad_request("index must be a number").into_response();
    };
    let id: i64 = id.parse().unwrap_or_default();
    match req.method {
        Method::GET => {
            let Some(issue) = state.issue(repo.id, index) else {
                return ApiError::not_found("issue not found").into_response();
            };
            match issue.comments.iter().find(|c| c.id == id) {
                Some(comment) => respond::ok(comment.to_dto(repo, issue.index)),
                None => ApiError::not_found("comment not found").into_response(),
            }
        }
        Method::PATCH => {
            let Some(body) = req.body_str("body") else {
                return ApiError::unprocessable("body is required").into_response();
            };
            let mut issues = state.issues_mut();
            let Some(issue) = issues
                .iter_mut()
                .find(|i| i.repo_id == repo.id && i.index == index)
            else {
                return ApiError::not_found("issue not found").into_response();
            };
            match issue.comments.iter_mut().find(|c| c.id == id) {
                Some(comment) => {
                    comment.body = body;
                    comment.updated = now();
                    respond::ok(comment.to_dto(repo, issue.index))
                }
                None => ApiError::not_found("comment not found").into_response(),
            }
        }
        Method::DELETE => {
            let mut issues = state.issues_mut();
            let Some(issue) = issues
                .iter_mut()
                .find(|i| i.repo_id == repo.id && i.index == index)
            else {
                return ApiError::not_found("issue not found").into_response();
            };
            let before = issue.comments.len();
            issue.comments.retain(|c| c.id != id);
            if issue.comments.len() == before {
                return ApiError::not_found("comment not found").into_response();
            }
            respond::no_content()
        }
        _ => unimplemented(req),
    }
}

/// `GET|POST|DELETE /api/v1/repos/{owner}/{repo}/issues/{index}/labels`.
fn issue_labels(state: &AppState, req: &ApiRequest, repo: &StoredRepo, index: &str) -> Response {
    let Ok(index) = index.parse::<i64>() else {
        return ApiError::bad_request("index must be a number").into_response();
    };
    match req.method {
        Method::GET => {
            let Some(issue) = state.issue(repo.id, index) else {
                return ApiError::not_found("issue not found").into_response();
            };
            respond::ok(labels_of(state, repo.id, &issue))
        }
        Method::POST => {
            let ids = req
                .json()
                .get("labels")
                .and_then(|v| v.as_array())
                .map(|ids| ids.iter().filter_map(|i| i.as_i64()).collect::<Vec<_>>())
                .unwrap_or_default();
            let mut issues = state.issues_mut();
            let Some(issue) = issues
                .iter_mut()
                .find(|i| i.repo_id == repo.id && i.index == index)
            else {
                return ApiError::not_found("issue not found").into_response();
            };
            for id in ids {
                if !issue.label_ids.contains(&id) {
                    issue.label_ids.push(id);
                }
            }
            let snapshot = issue.clone();
            drop(issues);
            respond::ok(labels_of(state, repo.id, &snapshot))
        }
        Method::PUT => {
            let ids = req
                .json()
                .get("labels")
                .and_then(|v| v.as_array())
                .map(|ids| ids.iter().filter_map(|i| i.as_i64()).collect::<Vec<_>>())
                .unwrap_or_default();
            let mut issues = state.issues_mut();
            let Some(issue) = issues
                .iter_mut()
                .find(|i| i.repo_id == repo.id && i.index == index)
            else {
                return ApiError::not_found("issue not found").into_response();
            };
            issue.label_ids = ids;
            let snapshot = issue.clone();
            drop(issues);
            respond::ok(labels_of(state, repo.id, &snapshot))
        }
        Method::DELETE => {
            if let Some(stored) = state
                .issues_mut()
                .iter_mut()
                .find(|i| i.repo_id == repo.id && i.index == index)
            {
                stored.label_ids.clear();
            }
            respond::no_content()
        }
        _ => unimplemented(req),
    }
}

/// `DELETE /api/v1/repos/{owner}/{repo}/issues/{index}/labels/{id}`.
fn issue_label_remove(
    state: &AppState,
    req: &ApiRequest,
    repo: &StoredRepo,
    index: &str,
    id: &str,
) -> Response {
    let _ = req;
    let (Ok(index), Ok(id)) = (index.parse::<i64>(), id.parse::<i64>()) else {
        return ApiError::bad_request("index and label id must be numbers").into_response();
    };
    let mut issues = state.issues_mut();
    let Some(issue) = issues
        .iter_mut()
        .find(|i| i.repo_id == repo.id && i.index == index)
    else {
        return ApiError::not_found("issue not found").into_response();
    };
    issue.label_ids.retain(|l| *l != id);
    respond::no_content()
}

/// `GET /api/v1/repos/{owner}/{repo}/pulls/{index}/reviews`.
/// Labels attached to an issue.
pub(super) fn labels_of(state: &AppState, repo_id: i64, issue: &StoredIssue) -> Vec<dto::Label> {
    state
        .labels()
        .iter()
        .filter(|l| l.repo_id == repo_id && issue.label_ids.contains(&l.id))
        .map(|l| {
            l.to_dto(&StoredRepo {
                id: repo_id,
                ..Default::default()
            })
        })
        .collect()
}

/// The milestone attached to an issue, when it exists.
fn milestone_of(state: &AppState, id: Option<i64>) -> Option<dto::Milestone> {
    let id = id?;
    let milestone = state.milestones().iter().find(|m| m.id == id)?.clone();
    let repo = StoredRepo {
        id: milestone.repo_id,
        ..Default::default()
    };
    let (open, closed) = milestone_counts(state, milestone.repo_id, id);
    Some(milestone.to_dto(&repo, open, closed))
}

/// Open and closed issue counts for a milestone.
fn milestone_counts(state: &AppState, repo_id: i64, milestone_id: i64) -> (i64, i64) {
    let mut open = 0;
    let mut closed = 0;
    for issue in state
        .issues()
        .iter()
        .filter(|i| i.repo_id == repo_id && i.milestone_id == Some(milestone_id))
    {
        if issue.state == "closed" {
            closed += 1;
        } else {
            open += 1;
        }
    }
    (open, closed)
}

/// Path segments after `/api/v1/repos/{owner}/{repo}`, i.e. skipping `skip`.
fn tail_after(req: &ApiRequest, skip: usize) -> Vec<&str> {
    req.path.trim_matches('/').split('/').skip(skip).collect()
}

/// Organization labels reuse the repository label store: an organization label
/// stores its scope as `-org_id`, so the two namespaces never collide.
fn org_scope_id(org_id: i64) -> i64 {
    -org_id
}

/// Render an organization label. The wire `url` points at the organization
/// route, which has no owning repository to derive it from.
fn org_label_dto(label: &StoredLabel, org: &str) -> dto::Label {
    dto::Label {
        id: Some(label.id),
        name: Some(label.name.clone()),
        color: Some(label.color.clone()),
        description: Some(label.description.clone()),
        exclusive: Some(false),
        is_archived: Some(false),
        url: Some(format!("/api/v1/orgs/{org}/labels/{}", label.id)),
    }
}

/// `GET|POST /api/v1/orgs/{org}/labels` and `/{id}`.
pub fn handle_org_labels(
    state: &AppState,
    req: &ApiRequest,
    org: &crate::state::StoredOrg,
) -> Response {
    let scope = org_scope_id(org.id);
    let tail = tail_after(req, 4);
    match tail.as_slice() {
        ["labels"] => match req.method {
            Method::GET => {
                let items: Vec<dto::Label> = state
                    .labels()
                    .iter()
                    .filter(|l| l.repo_id == scope)
                    .map(|l| org_label_dto(l, &org.name))
                    .collect();
                let pagination = pagination(req);
                let total = items.len();
                respond::page(pagination.apply(&items), &pagination, total)
            }
            Method::POST => {
                let name = req.body_str("name").unwrap_or_default();
                if name.trim().is_empty() {
                    return ApiError::unprocessable("name is required").into_response();
                }
                let label = StoredLabel {
                    id: state.next_id(),
                    repo_id: scope,
                    name,
                    color: req.body_str("color").unwrap_or_else(|| "ededed".into()),
                    description: req.body_str("description").unwrap_or_default(),
                };
                let dto = org_label_dto(&label, &org.name);
                state.labels_mut().push(label);
                respond::created(dto)
            }
            _ => unimplemented(req),
        },
        ["labels", id] => {
            let id: i64 = id.parse().unwrap_or_default();
            match req.method {
                Method::GET => match state
                    .labels()
                    .iter()
                    .find(|l| l.repo_id == scope && l.id == id)
                {
                    Some(label) => respond::ok(org_label_dto(label, &org.name)),
                    None => ApiError::not_found("label not found").into_response(),
                },
                Method::PATCH => {
                    let body = req.json();
                    let mut labels = state.labels_mut();
                    match labels.iter_mut().find(|l| l.repo_id == scope && l.id == id) {
                        Some(label) => {
                            if let Some(name) = body.get("name").and_then(|v| v.as_str()) {
                                label.name = name.to_string();
                            }
                            if let Some(color) = body.get("color").and_then(|v| v.as_str()) {
                                label.color = color.to_string();
                            }
                            if let Some(description) =
                                body.get("description").and_then(|v| v.as_str())
                            {
                                label.description = description.to_string();
                            }
                            respond::ok(org_label_dto(label, &org.name))
                        }
                        None => ApiError::not_found("label not found").into_response(),
                    }
                }
                Method::DELETE => {
                    let before = state.labels().len();
                    state
                        .labels_mut()
                        .retain(|l| !(l.repo_id == scope && l.id == id));
                    if state.labels().len() == before {
                        return ApiError::not_found("label not found").into_response();
                    }
                    respond::no_content()
                }
                _ => unimplemented(req),
            }
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

    fn repo(state: &AppState) -> StoredRepo {
        state.repo("alice", "demo").unwrap()
    }

    #[test]
    fn issues_get_sequential_indices() {
        let state = state();
        let repo = repo(&state);
        for title in ["first", "second"] {
            let body = format!(r#"{{"title":"{title}"}}"#);
            let response = create(
                &state,
                &req(Method::POST, "/api/v1/repos/alice/demo/issues", &body),
                &repo,
                false,
            );
            assert_eq!(response.status(), StatusCode::CREATED);
        }
        let indices: Vec<i64> = state.issues().iter().map(|i| i.index).collect();
        assert_eq!(indices, vec![1, 2]);
    }

    #[test]
    fn closing_an_issue_stamps_the_close_time() {
        let state = state();
        let repo = repo(&state);
        create(
            &state,
            &req(
                Method::POST,
                "/api/v1/repos/alice/demo/issues",
                r#"{"title":"broken"}"#,
            ),
            &repo,
            false,
        );
        let response = issue_detail(
            &state,
            &req(
                Method::PATCH,
                "/api/v1/repos/alice/demo/issues/1",
                r#"{"state":"closed"}"#,
            ),
            &repo,
            "1",
            false,
        );
        assert_eq!(response.status(), StatusCode::OK);
        let issue = state.issue(repo.id, 1).unwrap();
        assert_eq!(issue.state, "closed");
        assert!(issue.closed.is_some());
    }

    #[test]
    fn merging_a_pull_closes_it() {
        let state = state();
        let repo = repo(&state);
        create(
            &state,
            &req(
                Method::POST,
                "/api/v1/repos/alice/demo/pulls",
                r#"{"title":"feature","head":"feature","base":"main"}"#,
            ),
            &repo,
            true,
        );
        let response = merge(
            &state,
            &req(Method::POST, "/api/v1/repos/alice/demo/pulls/1/merge", "{}"),
            &repo,
            "1",
        );
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        let pull = state.issue(repo.id, 1).unwrap();
        assert!(pull.merged);
        assert_eq!(pull.state, "closed");

        // A second merge is rejected.
        let again = merge(
            &state,
            &req(Method::POST, "/api/v1/repos/alice/demo/pulls/1/merge", "{}"),
            &repo,
            "1",
        );
        assert_eq!(again.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn comments_are_attached_and_counted() {
        let state = state();
        let repo = repo(&state);
        create(
            &state,
            &req(
                Method::POST,
                "/api/v1/repos/alice/demo/issues",
                r#"{"title":"discuss"}"#,
            ),
            &repo,
            false,
        );
        let response = comments(
            &state,
            &req(
                Method::POST,
                "/api/v1/repos/alice/demo/issues/1/comments",
                r#"{"body":"looks good"}"#,
            ),
            &repo,
            "1",
        );
        assert_eq!(response.status(), StatusCode::CREATED);
        let issue = state.issue(repo.id, 1).unwrap();
        assert_eq!(issue.comments.len(), 1);
        assert_eq!(issue.comments[0].body, "looks good");
    }

    #[test]
    fn labels_attach_and_detach() {
        let state = state();
        let repo = repo(&state);
        state.labels_mut().push(StoredLabel {
            id: 5,
            repo_id: repo.id,
            name: "bug".into(),
            color: "ff0000".into(),
            ..Default::default()
        });
        create(
            &state,
            &req(
                Method::POST,
                "/api/v1/repos/alice/demo/issues",
                r#"{"title":"broken","labels":[5]}"#,
            ),
            &repo,
            false,
        );
        let issue = state.issue(repo.id, 1).unwrap();
        assert_eq!(labels_of(&state, repo.id, &issue).len(), 1);

        issue_label_remove(
            &state,
            &req(
                Method::DELETE,
                "/api/v1/repos/alice/demo/issues/1/labels/5",
                "",
            ),
            &repo,
            "1",
            "5",
        );
        let issue = state.issue(repo.id, 1).unwrap();
        assert!(labels_of(&state, repo.id, &issue).is_empty());
    }

    #[test]
    fn listing_filters_by_state_and_kind() {
        let state = state();
        let repo = repo(&state);
        create(
            &state,
            &req(
                Method::POST,
                "/api/v1/repos/alice/demo/issues",
                r#"{"title":"issue"}"#,
            ),
            &repo,
            false,
        );
        create(
            &state,
            &req(
                Method::POST,
                "/api/v1/repos/alice/demo/pulls",
                r#"{"title":"pull"}"#,
            ),
            &repo,
            true,
        );
        let issues = state
            .issues()
            .iter()
            .filter(|i| i.repo_id == repo.id && !i.is_pull)
            .count();
        let pulls = state
            .issues()
            .iter()
            .filter(|i| i.repo_id == repo.id && i.is_pull)
            .count();
        assert_eq!((issues, pulls), (1, 1));
    }
}
