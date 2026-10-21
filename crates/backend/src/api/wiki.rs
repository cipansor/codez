//! Repository wiki endpoints.
//!
//! Gitea stores a wiki as a second git repository; the port keeps the same
//! shape of data in memory: pages, each with an append-only list of revisions.
//! Creating a page starts its history, editing appends a revision, and deleting
//! removes the page and its history. The list and detail endpoints report the
//! stored pages, so the wiki UI reflects what a caller wrote.

use axum::http::Method;
use axum::response::{IntoResponse, Response};

use shared::models as dto;

use crate::respond::{self, ApiError};
use crate::state::{
    base64_decode, now, short_sha, AppState, StoredRepo, StoredWikiPage, StoredWikiRevision,
};

use super::ApiRequest;

/// Route a `/api/v1/repos/{owner}/{repo}/wiki/*` request.
pub fn handle(state: &AppState, req: &ApiRequest, repo: &StoredRepo) -> Response {
    let path = req.path.as_str();
    let marker = "/wiki/";
    let Some((_, tail)) = path.split_once(marker) else {
        return unimplemented(req);
    };
    let tail: Vec<&str> = tail.split('/').filter(|s| !s.is_empty()).collect();

    match tail.as_slice() {
        ["pages"] => list(state, repo),
        ["new"] => match req.method {
            Method::POST => create(state, req, repo),
            _ => unimplemented(req),
        },
        ["page", name] => page_detail(state, req, repo, name),
        ["revisions", name] => revisions(state, repo, name),
        _ => unimplemented(req),
    }
}

/// `GET /wiki/pages` — every page, metadata only.
fn list(state: &AppState, repo: &StoredRepo) -> Response {
    let items: Vec<dto::WikiPage> = state
        .repo_wiki_pages(repo.id)
        .iter()
        .map(|page| page.to_dto(repo))
        .collect();
    respond::ok(items)
}

/// `GET|PATCH|DELETE /wiki/page/{pageName}`.
fn page_detail(state: &AppState, req: &ApiRequest, repo: &StoredRepo, name: &str) -> Response {
    let name = decode_segment(name);
    match req.method {
        Method::GET => match state.repo_wiki_page(repo.id, &name) {
            Some(page) => respond::ok(page.to_dto(repo)),
            None => respond::ok(dto::WikiPage::default()),
        },
        Method::PATCH => {
            let Some(page) = state.repo_wiki_page(repo.id, &name) else {
                return ApiError::not_found("wiki page not found").into_response();
            };
            let content = req
                .body_str("content_base64")
                .map(|c| base64_decode(&c))
                .unwrap_or_else(|| page.content().to_string());
            let title = req.body_str("title").unwrap_or_else(|| page.title.clone());
            let message = req
                .body_str("message")
                .unwrap_or_else(|| format!("Update {title}"));

            append_revision(state, repo.id, &name, &title, &content, &message);
            match state.repo_wiki_page(repo.id, &title) {
                Some(updated) => respond::ok(updated.to_dto(repo)),
                None => ApiError::not_found("wiki page not found").into_response(),
            }
        }
        Method::DELETE => {
            let before = state.wiki_pages().len();
            state
                .wiki_pages_mut()
                .retain(|p| !(p.repo_id == repo.id && p.title.eq_ignore_ascii_case(&name)));
            if state.wiki_pages().len() == before {
                ApiError::not_found("wiki page not found").into_response()
            } else {
                respond::no_content()
            }
        }
        _ => unimplemented(req),
    }
}

/// `POST /wiki/new` — create a page, or add a revision to an existing one.
fn create(state: &AppState, req: &ApiRequest, repo: &StoredRepo) -> Response {
    let title = req.body_str("title").unwrap_or_else(|| "Home".into());
    let content = req
        .body_str("content_base64")
        .map(|c| base64_decode(&c))
        .unwrap_or_default();
    let message = req
        .body_str("message")
        .unwrap_or_else(|| format!("Create {title}"));

    append_revision(state, repo.id, &title, &title, &content, &message);
    match state.repo_wiki_page(repo.id, &title) {
        Some(page) => respond::created(page.to_dto(repo)),
        None => ApiError::bad_request("could not create wiki page").into_response(),
    }
}

/// `GET /wiki/revisions/{pageName}` — the page's history.
fn revisions(state: &AppState, repo: &StoredRepo, name: &str) -> Response {
    let name = decode_segment(name);
    match state.repo_wiki_page(repo.id, &name) {
        Some(page) => {
            let mut items: Vec<dto::WikiCommit> =
                page.revisions.iter().map(|r| r.to_dto()).collect();
            items.reverse();
            respond::ok(items)
        }
        None => ApiError::not_found("wiki page not found").into_response(),
    }
}

/// Append a revision, creating the page when it does not exist yet.
///
/// When `title` renames an existing page the row keeps its identity so its
/// history survives.
fn append_revision(
    state: &AppState,
    repo_id: i64,
    lookup: &str,
    title: &str,
    content: &str,
    message: &str,
) {
    let revision = StoredWikiRevision {
        sha: short_sha(&format!("wiki:{repo_id}:{title}:{}", now())),
        message: message.into(),
        author: "alice".into(),
        created: now(),
        content: content.into(),
    };
    let mut pages = state.wiki_pages_mut();
    match pages
        .iter_mut()
        .find(|p| p.repo_id == repo_id && p.title.eq_ignore_ascii_case(lookup))
    {
        Some(page) => {
            page.title = title.into();
            page.updated = revision.created.clone();
            page.revisions.push(revision);
        }
        None => pages.push(StoredWikiPage {
            repo_id,
            title: title.into(),
            updated: revision.created.clone(),
            revisions: vec![revision],
        }),
    }
}

/// Percent-decode a wiki page name from the path.
///
/// Titles are stored verbatim; the frontend encodes them into the path segment.
fn decode_segment(segment: &str) -> String {
    let bytes = segment.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[index + 1..index + 3]).ok();
            if let Some(byte) = hex.and_then(|h| u8::from_str_radix(h, 16).ok()) {
                out.push(byte);
                index += 3;
                continue;
            }
        }
        out.push(bytes[index]);
        index += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| segment.to_string())
}

/// Documented fallback for operations this module does not implement yet.
fn unimplemented(req: &ApiRequest) -> Response {
    super::dispatch::fallback(req)
}

/// Re-export for the router, which needs the base64 helper in tests.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::QueryParams;
    use crate::state::{base64_encode, StoredRepo, StoredUser};
    use axum::body::Bytes;

    fn state() -> AppState {
        let state = AppState::default();
        state.users_mut().push(StoredUser {
            id: 1,
            login: "alice".into(),
            ..Default::default()
        });
        state.repos_mut().push(StoredRepo {
            id: 100,
            owner: "alice".into(),
            name: "demo".into(),
            default_branch: "main".into(),
            ..Default::default()
        });
        state
    }

    fn req(method: Method, path: &str, body: serde_json::Value) -> ApiRequest {
        let pattern = super::super::dispatch::match_pattern(path);
        let op = pattern
            .as_deref()
            .and_then(|p| super::super::ops::find(method.as_str(), p));
        ApiRequest {
            method,
            uri: path.parse().unwrap(),
            path: path.to_string(),
            query: QueryParams::new(),
            body: Bytes::from(body.to_string()),
            op,
        }
    }

    #[test]
    fn creating_a_page_then_listing_it() {
        let state = state();
        let repo = state.repo("alice", "demo").unwrap();
        let response = create(
            &state,
            &req(
                Method::POST,
                "/api/v1/repos/alice/demo/wiki/new",
                serde_json::json!({
                    "title": "Home",
                    "content_base64": base64_encode("hello"),
                }),
            ),
            &repo,
        );
        assert_eq!(response.status(), axum::http::StatusCode::CREATED);
        assert_eq!(state.repo_wiki_pages(100).len(), 1);
        assert_eq!(
            state.repo_wiki_page(100, "Home").unwrap().content(),
            "hello"
        );
    }

    #[test]
    fn editing_appends_a_revision() {
        let state = state();
        append_revision(&state, 100, "Home", "Home", "one", "first");
        append_revision(&state, 100, "Home", "Home", "two", "second");
        let page = state.repo_wiki_page(100, "Home").unwrap();
        assert_eq!(page.revisions.len(), 2);
        assert_eq!(page.content(), "two");
    }

    #[test]
    fn deleting_a_page_removes_it() {
        let state = state();
        let repo = state.repo("alice", "demo").unwrap();
        append_revision(&state, 100, "Home", "Home", "hi", "first");
        let response = page_detail(
            &state,
            &req(
                Method::DELETE,
                "/api/v1/repos/alice/demo/wiki/page/Home",
                serde_json::Value::Null,
            ),
            &repo,
            "Home",
        );
        assert_eq!(response.status(), axum::http::StatusCode::NO_CONTENT);
        assert!(state.repo_wiki_page(100, "Home").is_none());
    }

    #[test]
    fn base64_round_trips() {
        for value in ["", "a", "ab", "abc", "hello world", "line\nbreak"] {
            assert_eq!(base64_decode(&base64_encode(value)), value);
        }
    }

    #[test]
    fn percent_decoding_a_title() {
        assert_eq!(decode_segment("Hello%20World"), "Hello World");
        assert_eq!(decode_segment("plain"), "plain");
    }
}
