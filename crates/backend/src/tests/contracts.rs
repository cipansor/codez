//! Contract tests: the JSON on the wire must match Gitea's field names.

use axum::http::StatusCode;

use super::{delete, get, patch, post};

#[tokio::test]
async fn version_document_shape() {
    let (status, body) = get("/api/v1/version").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["version"].is_string());
}

#[tokio::test]
async fn repository_uses_gitea_field_names() {
    let (status, body) = get("/api/v1/repos/codez-org/codez").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["full_name"], "codez-org/codez");
    assert_eq!(body["default_branch"], "main");
    assert_eq!(body["empty"], false);
    assert_eq!(body["owner"]["login"], "codez-org");
    // No camelCase leakage.
    assert!(body.get("fullName").is_none());
    assert!(body.get("defaultBranch").is_none());
}

#[tokio::test]
async fn missing_repository_returns_gitea_error_shape() {
    let (status, body) = get("/api/v1/repos/codez-org/nope").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body["message"].is_string());
    assert!(body["url"].is_string());
}

#[tokio::test]
async fn user_dto_never_exposes_the_password() {
    let (status, body) = get("/api/v1/users/alice").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["login"], "alice");
    let serialized = body.to_string();
    assert!(!serialized.contains("password"), "got {serialized}");
    assert!(!serialized.contains("hunter2"));
}

#[tokio::test]
async fn list_endpoints_attach_pagination_headers() {
    // `/repos/search` answers with `SearchResults`, not a bare array.
    let (status, headers, body) = super::call("GET", "/api/v1/repos/search", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["ok"], true);
    assert!(body["data"].is_array(), "got {body}");
    assert!(headers.get("X-Total-Count").is_some());
    assert!(headers.get("X-HasMore").is_some());

    // A bare-array list endpoint carries the same headers.
    let (status, headers, body) =
        super::call("GET", "/api/v1/repos/codez-org/codez/labels", None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_array());
    assert!(headers.get("X-Total-Count").is_some());
}

#[tokio::test]
async fn issues_expose_the_documented_fields() {
    let (status, body) = get("/api/v1/repos/codez-org/codez/issues/1").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["number"], 1);
    assert_eq!(body["state"], "open");
    assert!(body["title"].is_string());
    assert_eq!(body["repository"]["full_name"], "codez-org/codez");
    assert!(body["user"]["login"].is_string());
}

#[tokio::test]
async fn pull_requests_expose_head_and_base() {
    let (status, body) = get("/api/v1/repos/codez-org/codez/pulls/2").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["head"]["ref"], "feat/leptos-ui");
    assert_eq!(body["base"]["ref"], "main");
    assert_eq!(body["merged"], false);
    assert_eq!(body["mergeable"], true);
}

#[tokio::test]
async fn organization_and_team_shapes() {
    let (status, body) = get("/api/v1/orgs/codez-org").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], "codez-org");
    assert_eq!(body["visibility"], "public");

    let (status, body) = get("/api/v1/orgs/codez-org/teams").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body[0]["permission"], "owner");
    assert_eq!(body[0]["organization"]["name"], "codez-org");
}

#[tokio::test]
async fn branches_and_commits_are_listed() {
    let (status, body) = get("/api/v1/repos/codez-org/codez/branches").await;
    assert_eq!(status, StatusCode::OK);
    let names: Vec<&str> = body
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|b| b["name"].as_str())
        .collect();
    assert!(names.contains(&"main"), "got {names:?}");
    assert!(names.contains(&"feat/gitea-port"), "got {names:?}");

    let (status, body) = get("/api/v1/repos/codez-org/codez/commits").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.as_array().unwrap().len() >= 2);
    assert!(body[0]["sha"].is_string());
    assert!(body[0]["commit"]["message"].is_string());
}

#[tokio::test]
async fn file_contents_are_base64_encoded() {
    let (status, body) = get("/api/v1/repos/codez-org/codez/contents/README.md").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], "README.md");
    assert_eq!(body["path"], "README.md");
    assert_eq!(body["type"], "file");
    assert_eq!(body["encoding"], "base64");
    let encoded = body["content"].as_str().unwrap();
    assert_eq!(
        crate::api::repos::decode_base64(encoded),
        "# codez\n\nA Gitea-compatible platform in Rust.\n"
    );
}

#[tokio::test]
async fn labels_milestones_and_releases() {
    let (status, body) = get("/api/v1/repos/codez-org/codez/labels").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 2);

    let (status, body) = get("/api/v1/repos/codez-org/codez/milestones").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body[0]["title"], "v1.0");
    assert_eq!(body[0]["state"], "open");
    assert_eq!(body[0]["open_issues"], 1);

    let (status, body) = get("/api/v1/repos/codez-org/codez/releases").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body[0]["tag_name"], "v0.1.0");
    assert_eq!(body[0]["draft"], false);
}

#[tokio::test]
async fn notifications_carry_subject_and_repository() {
    let (status, body) = get("/api/v1/notifications").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body[0]["unread"], true);
    assert_eq!(body[0]["subject"]["type"], "Issue");
    assert_eq!(body[0]["repository"]["full_name"], "codez-org/codez");
}

#[tokio::test]
async fn admin_stats_report_the_seeded_counts() {
    // `GET /api/v1/admin/stats` is a documented extension, not a Gitea path.
    let (status, body) = get("/api/v1/admin/stats").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["user_count"], 3);
    assert_eq!(body["repo_count"], 2);
    assert_eq!(body["organization_count"], 1);
}

#[tokio::test]
async fn search_endpoints_answer_with_documents() {
    // `/repos/search` and `/users/search` answer with `SearchResults`; the
    // issue search returns a bare array of issues.
    for path in ["/api/v1/repos/search", "/api/v1/users/search"] {
        let (status, body) = get(path).await;
        assert_eq!(status, StatusCode::OK, "for {path}");
        assert_eq!(body["ok"], true, "for {path}: {body}");
        assert!(body["data"].is_array(), "for {path}: {body}");
    }

    let (status, body) = get("/api/v1/repos/issues/search").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_array(), "got {body}");
}

#[tokio::test]
async fn markdown_renders_and_escapes() {
    let (status, body) = post(
        "/api/v1/markdown",
        serde_json::json!({ "text": "# Title\n\n<script>bad()</script>" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let html = body.as_str().unwrap();
    assert!(html.contains("<h1>Title</h1>"));
    assert!(!html.contains("<script>"));
}

#[tokio::test]
async fn write_endpoints_round_trip_through_the_api() {
    // Create a repository, then read it back through the same router.
    let (status, created) = post(
        "/api/v1/user/repos",
        serde_json::json!({ "name": "contract-check", "auto_init": true }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["full_name"], "alice/contract-check");

    let (status, fetched) = get("/api/v1/repos/alice/contract-check").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(fetched["full_name"], "alice/contract-check");

    // Patch it.
    let (status, updated) = patch(
        "/api/v1/repos/alice/contract-check",
        serde_json::json!({ "description": "changed" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["description"], "changed");

    // Clean up.
    assert_eq!(
        delete("/api/v1/repos/alice/contract-check").await,
        StatusCode::NO_CONTENT
    );
    let (status, _) = get("/api/v1/repos/alice/contract-check").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn issue_lifecycle_through_http() {
    let (status, created) = post(
        "/api/v1/repos/codez-org/codez/issues",
        serde_json::json!({ "title": "HTTP lifecycle check" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let index = created["number"].as_i64().expect("number");

    let (status, commented) = post(
        &format!("/api/v1/repos/codez-org/codez/issues/{index}/comments"),
        serde_json::json!({ "body": "first" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(commented["body"], "first");

    let (status, listed) = get(&format!(
        "/api/v1/repos/codez-org/codez/issues/{index}/comments"
    ))
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(listed.as_array().unwrap().len(), 1);

    let (status, closed) = patch(
        &format!("/api/v1/repos/codez-org/codez/issues/{index}"),
        serde_json::json!({ "state": "closed" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(closed["state"], "closed");
    assert!(closed["closed_at"].is_string());
}

#[tokio::test]
async fn signin_issues_a_token_that_authenticates() {
    let (status, body) = post(
        "/api/v1/signin",
        serde_json::json!({ "user_name": "bob", "password": "password" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let token = body["sha1"].as_str().expect("token").to_string();

    let (status, me) = get(&format!("/api/v1/user?token={token}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(me["login"], "bob");
}

#[tokio::test]
async fn wrong_password_is_rejected() {
    let (status, body) = post(
        "/api/v1/signin",
        serde_json::json!({ "user_name": "bob", "password": "wrong" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(body["message"].is_string());
}
