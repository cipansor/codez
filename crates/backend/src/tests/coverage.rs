//! Coverage test: every documented operation must reach a handler.
//!
//! The dispatcher's fallback still answers, so this asserts that no path is
//! silently unrouted — the failure mode a hand-written router would hide.

use axum::http::StatusCode;

use crate::api::dispatch::concrete_path;
use crate::api::ops;

use super::{call, call_fresh, get};

#[tokio::test]
async fn every_documented_path_is_routed() {
    let mut checked = 0;
    for op in ops::OPS {
        // Sample the `GET` variant of every distinct path; parameterised
        // writes are covered by the resource tests.
        if op.method != "GET" {
            continue;
        }
        let concrete = concrete_path(op.path);

        let (status, _, _) = call(op.method, &concrete, None).await;
        assert_ne!(
            status,
            StatusCode::METHOD_NOT_ALLOWED,
            "{} {} is not routed",
            op.method,
            op.path
        );
        checked += 1;
    }
    assert!(
        checked > 150,
        "expected to sample most GET operations, got {checked}"
    );
}

#[tokio::test]
async fn unknown_api_path_is_a_json_404() {
    let (status, body) = get("/api/v1/definitely-not-a-route").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(
        body.get("message").is_some(),
        "expected a message, got {body}"
    );
}

#[tokio::test]
async fn version_endpoints_agree() {
    let (status, body) = get("/api/v1/version").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["version"], "1.23.0");

    let (status, body) = get("/-/version").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["version"], "1.23.0");
}

#[tokio::test]
async fn settings_report_the_enforced_limits() {
    let (status, body) = get("/api/v1/settings/api").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["max_response_items"], crate::pagination::MAX_LIMIT);
}

#[tokio::test]
async fn operation_table_matches_the_specification() {
    // 537 operations across 342 paths, straight from the OpenAPI document.
    assert_eq!(ops::OPS.len(), 537);
    let paths: std::collections::HashSet<&str> = ops::OPS.iter().map(|o| o.path).collect();
    assert_eq!(paths.len(), 342);

    let tags = ops::tag_counts();
    let repository = tags
        .iter()
        .find(|(tag, _)| *tag == "repository")
        .expect("repository tag");
    assert_eq!(repository.1, 222);
}

/// Every documented operation must reach a bespoke handler, not the generic
/// fallback.
///
/// The fallback still answers with a plausible shape (an empty list, a `null`
/// detail, an "accepted" echo for writes), so routing alone cannot be observed
/// from the status code. The fallback therefore stamps its replies with
/// `x-codez-fallback`, and this test samples every operation and asserts the
/// marker never appears. A resource that regresses to a stub fails here.
///
/// Path parameters are filled with values that exist in the seed rather than a
/// placeholder: handlers commonly resolve the entity first and `404` before
/// reaching their sub-resource match arms, so a fake id would let an
/// unimplemented sub-resource hide behind the early return.
#[tokio::test]
async fn no_documented_operation_is_answered_by_the_fallback() {
    let mut checked = 0;
    let mut stubs = Vec::new();
    for op in ops::OPS {
        let concrete = concrete_path_for(op.path);
        let body = matches!(op.method, "POST" | "PUT" | "PATCH").then(|| serde_json::json!({}));
        let (_, headers, _) = call_fresh(op.method, &concrete, body).await;
        checked += 1;
        if headers.contains_key("x-codez-fallback") {
            stubs.push(format!("{} {}", op.method, op.path));
        }
    }
    assert_eq!(checked, ops::OPS.len());
    assert!(
        stubs.is_empty(),
        "{} operation(s) still hit the fallback:\n  {}",
        stubs.len(),
        stubs.join("\n  ")
    );
}

/// Like [`concrete_path`], but substitutes seed values for the path parameters
/// so a handler resolves its entity and runs its real dispatch.
fn concrete_path_for(pattern: &str) -> String {
    pattern
        .split('/')
        .map(|segment| {
            if !segment.contains('{') {
                return segment.to_string();
            }
            let mut out = String::new();
            let mut rest = segment;
            while let Some(open) = rest.find('{') {
                out.push_str(&rest[..open]);
                let Some(close) = rest[open..].find('}') else {
                    return segment.to_string();
                };
                let name = &rest[open + 1..open + close];
                out.push_str(seed_value(name));
                rest = &rest[open + close + 1..];
            }
            out.push_str(rest);
            out
        })
        .collect::<Vec<_>>()
        .join("/")
}

/// A value from the seed that satisfies the named path parameter.
fn seed_value(name: &str) -> &'static str {
    match name {
        "owner" | "username" | "user" | "login" => "alice",
        "repo" | "repo_name" => "notes",
        "org" => "codez-org",
        "team" => "owners",
        "index" | "issue_id" | "id" | "artifact_id" | "attachment_id" | "job_id" | "runner_id"
        | "column_id" => "1",
        "branch" | "ref" | "base" | "head" | "basehead" | "sha" | "tag" => "main",
        "name" | "pageName" | "topic" | "secretname" | "task" | "type" | "position" => "home",
        "template_owner" => "alice",
        "template_repo" => "notes",
        "token" => "none",
        "collaborator" | "assignee" | "target" => "bob",
        "filepath" => "README.md",
        "attempt" => "1",
        "run" => "1",
        "diffType" => "diff",
        _ => "x",
    }
}

#[tokio::test]
async fn operation_lookup_by_method_and_path() {
    assert!(ops::find("GET", "/api/v1/version").is_some());
    assert!(ops::find("GET", "/api/v1/repos/{owner}/{repo}").is_some());
    assert!(ops::find("DELETE", "/api/v1/repos/{owner}/{repo}").is_some());
    assert!(ops::find("GET", "/api/v1/not-a-path").is_none());
}
