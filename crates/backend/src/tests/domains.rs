//! Domain integration tests for the ported Gitea surface.
//!
//! These drive the real router over the seeded in-memory state and assert the
//! behaviour the SPA depends on: the exact field names Gitea publishes, the
//! list/detail conventions (`null` for a missing detail, `404` for a missing
//! repository), and the write paths the pages exercise.

use axum::http::StatusCode;
use serde_json::Value;

use super::{delete, delete_with_body, get, patch, post, put};

#[tokio::test]
async fn issue_list_reports_seeded_open_issue() {
    let (status, body) = get("/api/v1/repos/codez-org/codez/issues").await;
    assert_eq!(status, StatusCode::OK);
    let issues = body.as_array().unwrap();
    assert_eq!(issues.len(), 1, "got {issues:?}");
    assert_eq!(issues[0]["number"], 1);
    assert_eq!(issues[0]["title"], "Serve the full /api/v1 surface");
    assert_eq!(issues[0]["state"], "open");
    // The seeded issue is opened by bob and assigned to alice.
    assert_eq!(issues[0]["user"]["login"], "bob");
    assert_eq!(issues[0]["assignee"]["login"], "alice");
}

#[tokio::test]
async fn missing_issue_returns_gitea_error_shape() {
    // A missing issue is a `404` with Gitea's error body, unlike the
    // `200`-with-`null` convention some detail endpoints use.
    let (status, body) = get("/api/v1/repos/codez-org/codez/issues/9999").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body["message"].is_string(), "got {body}");
}

#[tokio::test]
async fn issue_can_be_closed_and_reopened_over_http() {
    let uri = "/api/v1/repos/codez-org/codez/issues/1";

    let (status, body) = patch(uri, serde_json::json!({ "state": "closed" })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["state"], "closed");

    let (status, body) = patch(uri, serde_json::json!({ "state": "open" })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["state"], "open");
}

#[tokio::test]
async fn created_issue_appears_in_the_list() {
    let (status, created) = post(
        "/api/v1/repos/codez-org/codez/issues",
        serde_json::json!({ "title": "Integration: created issue", "body": "from cargo" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["title"], "Integration: created issue");

    let (_, body) = get("/api/v1/repos/codez-org/codez/issues?state=all").await;
    let titles: Vec<&str> = body
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|i| i["title"].as_str())
        .collect();
    assert!(
        titles.contains(&"Integration: created issue"),
        "got {titles:?}"
    );
}

#[tokio::test]
async fn issue_comment_is_listed_after_posting() {
    let uri = "/api/v1/repos/codez-org/codez/issues/1/comments";
    let (status, created) = post(uri, serde_json::json!({ "body": "a cargo comment" })).await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, body) = get(uri).await;
    assert_eq!(status, StatusCode::OK);
    let bodies: Vec<&str> = body
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|c| c["body"].as_str())
        .collect();
    assert!(bodies.contains(&"a cargo comment"), "got {bodies:?}");
    assert_eq!(created["body"], "a cargo comment");
}

#[tokio::test]
async fn label_create_then_list() {
    let uri = "/api/v1/repos/codez-org/codez/labels";
    let (status, _) = post(
        uri,
        serde_json::json!({ "name": "integration-label", "color": "abcdef" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let (_, body) = get(uri).await;
    let names: Vec<&str> = body
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|l| l["name"].as_str())
        .collect();
    assert!(names.contains(&"integration-label"), "got {names:?}");
}

#[tokio::test]
async fn pull_list_and_detail_agree_on_number() {
    let (status, list) = get("/api/v1/repos/codez-org/codez/pulls").await;
    assert_eq!(status, StatusCode::OK);
    let pulls = list.as_array().unwrap();
    assert_eq!(pulls.len(), 1, "got {pulls:?}");
    let number = pulls[0]["number"].as_i64().unwrap();

    let (status, detail) = get(&format!("/api/v1/repos/codez-org/codez/pulls/{number}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(detail["number"], number);
}

#[tokio::test]
async fn milestone_detail_matches_the_listed_id() {
    let (status, list) = get("/api/v1/repos/codez-org/codez/milestones").await;
    assert_eq!(status, StatusCode::OK);
    let id = list.as_array().unwrap()[0]["id"].as_i64().unwrap();

    let (status, detail) = get(&format!("/api/v1/repos/codez-org/codez/milestones/{id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(detail["id"], id);
    assert_eq!(detail["title"], "v1.0");
}

#[tokio::test]
async fn release_detail_uses_the_listed_id() {
    let (status, list) = get("/api/v1/repos/codez-org/codez/releases").await;
    assert_eq!(status, StatusCode::OK);
    let id = list.as_array().unwrap()[0]["id"].as_i64().unwrap();

    let (status, detail) = get(&format!("/api/v1/repos/codez-org/codez/releases/{id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(detail["tag_name"], "v0.1.0");
}

#[tokio::test]
async fn branch_and_tag_lists_render_the_seeded_git_data() {
    let (status, branches) = get("/api/v1/repos/codez-org/codez/branches").await;
    assert_eq!(status, StatusCode::OK);
    let names: Vec<&str> = branches
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|b| b["name"].as_str())
        .collect();
    assert!(names.contains(&"main"), "got {names:?}");
    assert!(names.contains(&"feat/gitea-port"), "got {names:?}");

    let (status, tags) = get("/api/v1/repos/codez-org/codez/tags").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(tags.as_array().unwrap()[0]["name"], "v0.1.0");
}

#[tokio::test]
async fn file_contents_round_trip_through_base64() {
    let uri = "/api/v1/repos/codez-org/codez/contents/README.md";
    let (status, body) = get(uri).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], "README.md");
    assert_eq!(body["encoding"], "base64");
    assert!(body["content"].is_string());
}

#[tokio::test]
async fn organization_document_lists_repositories() {
    let (status, body) = get("/api/v1/orgs/codez-org").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], "codez-org");
    assert_eq!(body["full_name"], "Codez");

    let (status, repos) = get("/api/v1/orgs/codez-org/repos").await;
    assert_eq!(status, StatusCode::OK);
    let names: Vec<&str> = repos
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|r| r["name"].as_str())
        .collect();
    assert!(names.contains(&"codez"), "got {names:?}");
}

#[tokio::test]
async fn user_document_and_repository_list() {
    let (status, body) = get("/api/v1/users/alice").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["login"], "alice");
    assert_eq!(body["full_name"], "Alice Liddell");

    let (status, repos) = get("/api/v1/users/alice/repos").await;
    assert_eq!(status, StatusCode::OK);
    let names: Vec<&str> = repos
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|r| r["name"].as_str())
        .collect();
    assert!(names.contains(&"notes"), "got {names:?}");
}

#[tokio::test]
async fn admin_user_list_includes_the_seeded_accounts() {
    let (status, body) = get("/api/v1/admin/users").await;
    assert_eq!(status, StatusCode::OK);
    let logins: Vec<&str> = body
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|u| u["login"].as_str())
        .collect();
    assert!(logins.contains(&"alice"), "got {logins:?}");
    assert!(logins.contains(&"bob"), "got {logins:?}");
}

#[tokio::test]
async fn search_returns_matching_repositories() {
    let (status, body) = get("/api/v1/repos/search?q=codez").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["ok"], true);
    let names: Vec<&str> = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|r| r["name"].as_str())
        .collect();
    assert!(names.contains(&"codez"), "got {names:?}");
}

#[tokio::test]
async fn starred_repository_lists_the_starring_user() {
    let (status, _) = put(
        "/api/v1/user/starred/codez-org/codez",
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, body) = get("/api/v1/repos/codez-org/codez/stargazers").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body
        .as_array()
        .unwrap()
        .iter()
        .any(|u| u["login"] == "alice"));

    let status = delete("/api/v1/user/starred/codez-org/codez").await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn notification_list_answers_with_an_array() {
    let (status, body) = get("/api/v1/notifications").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_array(), "got {body}");
}

#[tokio::test]
async fn creating_a_repository_returns_created_and_is_listable() {
    let (status, created) = post(
        "/api/v1/user/repos",
        serde_json::json!({ "name": "integration-repo", "auto_init": true }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["name"], "integration-repo");
    assert_eq!(created["full_name"], "alice/integration-repo");

    let (status, body) = get("/api/v1/repos/alice/integration-repo").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], "integration-repo");
}

#[tokio::test]
async fn editing_a_comment_updates_it_in_the_listing() {
    let base = "/api/v1/repos/codez-org/codez/issues/1/comments";
    let body = format!("original {}", crate::state::now());
    let (status, created) = post(base, serde_json::json!({ "body": body })).await;
    assert_eq!(status, StatusCode::CREATED);
    let id = created["id"].as_i64().unwrap();

    let edited = format!("edited {}", crate::state::now());
    let (status, patched) = super::patch(
        &format!("{base}/{id}"),
        serde_json::json!({ "body": edited }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(patched["body"], edited);

    // The edit must be visible without a reload.
    let (_, listed) = get(base).await;
    let bodies: Vec<&str> = listed
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|c| c["body"].as_str())
        .collect();
    assert!(bodies.contains(&edited.as_str()), "got {bodies:?}");

    // A rejected edit keeps the previous value.
    let (status, _) = super::patch(&format!("{base}/{id}"), serde_json::json!({})).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let (_, listed) = get(base).await;
    let bodies: Vec<&str> = listed
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|c| c["body"].as_str())
        .collect();
    assert!(bodies.contains(&edited.as_str()), "got {bodies:?}");
}

#[tokio::test]
async fn labels_can_be_replaced_wholesale_on_an_issue() {
    let labels = "/api/v1/repos/codez-org/codez/labels";
    let (status, created) = post(
        labels,
        serde_json::json!({ "name": "replace-me", "color": "00ff00" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let id = created["id"].as_i64().unwrap();

    let uri = "/api/v1/repos/codez-org/codez/issues/1/labels";
    let (status, applied) = put(uri, serde_json::json!({ "labels": [id] })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(applied.as_array().unwrap().len(), 1);
    assert_eq!(applied[0]["id"].as_i64(), Some(id));

    // `PUT` replaces the set: a second call with no labels clears it.
    let (status, cleared) = put(uri, serde_json::json!({ "labels": [] })).await;
    assert_eq!(status, StatusCode::OK);
    assert!(cleared.as_array().unwrap().is_empty(), "got {cleared}");
}

#[tokio::test]
async fn reactions_persist_on_issues_and_comments() {
    // Reactions used to be accepted and dropped, so a GET always answered `[]`.
    // The listing must reflect the POST.
    let issue = "/api/v1/repos/codez-org/codez/issues/1/reactions";
    let (status, created) = post(issue, serde_json::json!({ "content": "rocket" })).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["content"], "rocket");

    let (status, list) = get(issue).await;
    assert_eq!(status, StatusCode::OK);
    let list = list.as_array().unwrap();
    assert!(
        list.iter().any(|r| r["content"] == "rocket"),
        "issue reaction was not persisted: {list:?}"
    );

    // Reacting twice with the same emoji must not duplicate the row.
    post(issue, serde_json::json!({ "content": "rocket" })).await;
    let (_, list) = get(issue).await;
    let rockets = list
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["content"] == "rocket")
        .count();
    assert_eq!(rockets, 1, "duplicate reaction row: {list}");

    // Comments carry their own, independent reaction list.
    let (_, comment) = post(
        "/api/v1/repos/codez-org/codez/issues/1/comments",
        serde_json::json!({ "body": "reaction target" }),
    )
    .await;
    let comment_id = comment["id"].as_i64().unwrap();
    let comment_reactions =
        format!("/api/v1/repos/codez-org/codez/issues/comments/{comment_id}/reactions");
    let (status, _) = post(
        comment_reactions.as_str(),
        serde_json::json!({ "content": "heart" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let (_, comment_list) = get(comment_reactions.as_str()).await;
    assert_eq!(comment_list.as_array().unwrap().len(), 1);
    // The comment reaction must not leak into the issue listing.
    let (_, issue_list) = get(issue).await;
    assert!(
        !issue_list
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["content"] == "heart"),
        "comment reaction leaked into the issue listing: {issue_list}"
    );
}

#[tokio::test]
async fn issue_and_comment_assets_round_trip() {
    // Assets are scoped to their owner; a listing that ignores the owner would
    // show another issue's attachment.
    let issue_assets = "/api/v1/repos/codez-org/codez/issues/1/assets";
    let (status, created) = post(
        issue_assets,
        serde_json::json!({ "name": "diagram.png", "size": 128 }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let id = created["id"].as_i64().unwrap();
    assert_eq!(created["name"], "diagram.png");

    let (status, list) = get(issue_assets).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);

    // A different issue must not see it.
    let (_, other) = get("/api/v1/repos/codez-org/codez/issues/2/assets").await;
    assert!(
        other.as_array().unwrap().is_empty(),
        "attachment leaked across issues: {other}"
    );

    let detail = format!("{issue_assets}/{id}");
    let (status, renamed) = patch(&detail, serde_json::json!({ "name": "renamed.png" })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(renamed["name"], "renamed.png");

    assert_eq!(delete(&detail).await, StatusCode::NO_CONTENT);
    let (_, after) = get(issue_assets).await;
    assert!(after.as_array().unwrap().is_empty(), "delete left {after}");
}

#[tokio::test]
async fn user_emails_persist_add_and_remove() {
    // POST/DELETE used to answer success and change nothing, so GET only ever
    // showed the primary address.
    let emails = "/api/v1/user/emails";
    let address = format!("alias-{}@example.com", super::unique());

    let (status, _) = post(emails, serde_json::json!({ "emails": [address] })).await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, list) = get(emails).await;
    assert_eq!(status, StatusCode::OK);
    let list = list.as_array().unwrap();
    assert!(
        list.iter().any(|e| e["email"] == address.as_str()),
        "added email missing from listing: {list:?}"
    );
    // The account's own address is always present and marked primary.
    assert!(list.iter().any(|e| e["primary"] == true));

    assert_eq!(
        delete_with_body(emails, serde_json::json!({ "emails": [address] })).await,
        StatusCode::NO_CONTENT
    );
    let (_, after) = get(emails).await;
    assert!(
        !after
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["email"] == address.as_str()),
        "removed email still listed: {after}"
    );
}

#[tokio::test]
async fn git_hook_edits_are_readable_back() {
    // PATCH returned the posted body without storing it, so a follow-up GET
    // always answered an empty hook.
    let hook = "/api/v1/repos/codez-org/codez/hooks/git/pre-receive";
    let body = "#!/bin/sh\necho port\n";

    let (status, patched) = patch(
        hook,
        serde_json::json!({ "content": body, "is_active": true }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(patched["content"], body);

    let (status, fetched) = get(hook).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        fetched["content"], body,
        "git hook content was not persisted"
    );
    assert_eq!(fetched["is_active"], true);

    assert_eq!(delete(hook).await, StatusCode::NO_CONTENT);
    let (_, cleared) = get(hook).await;
    assert_eq!(cleared["content"], "", "delete did not clear the hook");
}

#[tokio::test]
async fn collaborators_round_trip_through_the_listing() {
    // The settings page adds a collaborator and re-lists; the listing was a
    // constant `[]`, so an added collaborator never appeared.
    let base = "/api/v1/repos/codez-org/codez/collaborators";
    let target = "bob".to_string();

    let (status, _) = put(
        &format!("{base}/{target}"),
        serde_json::json!({ "permission": "read" }),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, list) = get(base).await;
    assert_eq!(status, StatusCode::OK);
    let logins: Vec<&str> = list
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|u| u["login"].as_str())
        .collect();
    assert!(
        logins.contains(&target.as_str()),
        "added collaborator is missing from the listing: {logins:?}"
    );

    // The permission recorded by the PUT is what the permission endpoint
    // reports back.
    let (_, permission) = get(&format!("{base}/{target}/permission")).await;
    assert_eq!(permission["permission"], "read");

    let status = delete(&format!("{base}/{target}")).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, list) = get(base).await;
    let logins: Vec<&str> = list
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|u| u["login"].as_str())
        .collect();
    assert!(!logins.contains(&target.as_str()), "delete was dropped");
}

#[tokio::test]
async fn org_labels_create_and_list_through_one_route() {
    // The listing arm shadowed the create route, so `POST` answered `200 []`
    // and no label was ever stored.
    let base = "/api/v1/orgs/codez-org/labels";
    let name = format!("label-{}", super::unique());

    let (status, created) =
        post(base, serde_json::json!({ "name": name, "color": "abcdef" })).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["name"], name);

    let (status, list) = get(base).await;
    assert_eq!(status, StatusCode::OK);
    let names: Vec<&str> = list
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|l| l["name"].as_str())
        .collect();
    assert!(
        names.contains(&name.as_str()),
        "created label is missing from the listing: {names:?}"
    );
}

#[tokio::test]
async fn diffpatch_records_the_patched_paths_as_a_commit() {
    // The endpoint answered a hand-built object with a repo count in
    // `status`, matching neither the spec nor the store.
    let path = "/api/v1/repos/codez-org/codez/diffpatch";
    let (status, body) = post(
        path,
        serde_json::json!({
            "content": "diff --git a/src/a.rs b/src/a.rs\n--- a/src/a.rs\n+++ b/src/a.rs\n@@ -1 +1,2 @@\n x\n+y\n",
            "message": "apply the patch"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["commit"]["message"], "apply the patch");
    let sha = body["commit"]["sha"].as_str().expect("commit sha");
    assert!(!sha.is_empty());

    // The commit is visible through the git API, which reads the same store.
    let (status, commit) = get(&format!("/api/v1/repos/codez-org/codez/git/commits/{sha}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(commit["commit"]["message"], "apply the patch");
}

#[tokio::test]
async fn review_comments_round_trip_and_resolve() {
    // The review endpoint accepted `comments` in the body and dropped them, so
    // the listing always answered `[]` and resolve had nothing to act on.
    let reviews = "/api/v1/repos/codez-org/codez/pulls/2/reviews";
    let (status, review) = post(
        reviews,
        serde_json::json!({
            "event": "COMMENT",
            "body": "looks close",
            "commit_id": "abc123",
            "comments": [
                { "body": "rename this", "path": "src/lib.rs", "new_position": 12 },
                { "body": "and this", "path": "src/main.rs", "new_position": 3 }
            ]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let review_id = review["id"].as_i64().unwrap();

    let (status, comments) = get(&format!("{reviews}/{review_id}/comments")).await;
    assert_eq!(status, StatusCode::OK);
    let comments = comments.as_array().unwrap();
    assert_eq!(
        comments.len(),
        2,
        "review comments were not persisted: {comments:?}"
    );
    assert_eq!(comments[0]["path"], "src/lib.rs");
    assert_eq!(comments[0]["position"], 12);

    // Resolving marks the comment with a resolver; unresolving clears it.
    let comment_id = comments[0]["id"].as_i64().unwrap();
    let resolve = format!("/api/v1/repos/codez-org/codez/pulls/comments/{comment_id}/resolve");
    assert_eq!(
        post(&resolve, serde_json::json!({})).await.0,
        StatusCode::NO_CONTENT
    );

    let (_, resolved) = get(&format!("{reviews}/{review_id}/comments")).await;
    assert!(
        resolved[0]["resolver"].is_object(),
        "resolve did not set a resolver: {}",
        resolved[0]
    );

    let unresolve = format!("/api/v1/repos/codez-org/codez/pulls/comments/{comment_id}/unresolve");
    assert_eq!(
        post(&unresolve, serde_json::json!({})).await.0,
        StatusCode::NO_CONTENT
    );
    let (_, reopened) = get(&format!("{reviews}/{review_id}/comments")).await;
    assert!(
        reopened[0]["resolver"].is_null(),
        "unresolve did not clear the resolver: {}",
        reopened[0]
    );
}

#[tokio::test]
async fn team_repos_list_the_organizations_repositories() {
    // The team repo listing answered a constant `[]` while the organization
    // owned repositories, so a team page showed nothing.
    let (status, teams) = get("/api/v1/orgs/codez-org/teams").await;
    assert_eq!(status, StatusCode::OK);
    let team_id = teams
        .as_array()
        .and_then(|t| t.first())
        .and_then(|t| t["id"].as_i64())
        .expect("seed should provide an organization team");

    let (status, repos) = get(&format!("/api/v1/teams/{team_id}/repos")).await;
    assert_eq!(status, StatusCode::OK);
    let repos = repos.as_array().unwrap();
    assert!(
        !repos.is_empty(),
        "team listing ignored the organization's repositories"
    );
    assert!(
        repos.iter().all(|r| r["owner"]["login"] == "codez-org"),
        "team listing leaked a foreign repository: {repos:?}"
    );
}

#[tokio::test]
async fn branch_protection_priority_reorders_the_listing() {
    // The priority endpoint used to accept the ids and return 204 without
    // changing anything, so the listing kept insertion order.
    let base = "/api/v1/repos/codez-org/codez/branch_protections";
    let first = format!("first-{}", super::unique());
    let second = format!("second-{}", super::unique());

    let (_, _a) = post(base, serde_json::json!({ "branch_name": first })).await;
    let (_, _b) = post(base, serde_json::json!({ "branch_name": second })).await;

    let names = |list: &Value| -> Vec<String> {
        list.as_array()
            .unwrap()
            .iter()
            .map(|p| p["branch_name"].as_str().unwrap().to_string())
            .collect()
    };
    // The ids the priority endpoint takes are the rules' `priority` values,
    // which the listing publishes.
    let id_of = |list: &Value, name: &str| -> i64 {
        list.as_array()
            .unwrap()
            .iter()
            .find(|p| p["branch_name"] == name)
            .and_then(|p| p["priority"].as_i64())
            .unwrap()
    };

    let (_, list) = get(base).await;
    let before = names(&list);
    assert!(
        before.iter().position(|n| n == &first) < before.iter().position(|n| n == &second),
        "unexpected seed order: {before:?}"
    );
    let (a_id, b_id) = (id_of(&list, &first), id_of(&list, &second));

    // Put the second rule first.
    let (status, _) = post(
        &format!("{base}/priority"),
        serde_json::json!({ "ids": [b_id, a_id] }),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, list) = get(base).await;
    let after = names(&list);
    assert!(
        after.iter().position(|n| n == &second) < after.iter().position(|n| n == &first),
        "priority reorder had no effect: {after:?}"
    );
}

#[tokio::test]
async fn issue_stopwatch_accepts_the_documented_verbs() {
    // The spec spells `start` and `stop` as `POST`, but `delete` as `DELETE`;
    // none of the three may fall through to the generic fallback.
    let base = "/api/v1/repos/codez-org/codez/issues/1/stopwatch";
    for action in ["start", "stop"] {
        let (status, _) = post(&format!("{base}/{action}"), serde_json::json!({})).await;
        assert_eq!(status, StatusCode::NO_CONTENT, "action {action}");
    }
    assert_eq!(
        delete(&format!("{base}/delete")).await,
        StatusCode::NO_CONTENT
    );
}

#[tokio::test]
async fn commit_status_write_is_visible_on_both_status_endpoints() {
    // `/statuses/{sha}` and `/commits/{sha}/status(es)` read the same store, so a
    // POST to one must show up in the others. A hardcoded `[]` on either read
    // endpoint is invisible to a test that only checks the write returns 2xx.
    let (status, branch) = get("/api/v1/repos/codez-org/codez/branches/main").await;
    assert_eq!(status, StatusCode::OK);
    let sha = branch["commit"]["id"].as_str().unwrap().to_string();
    let base = "/api/v1/repos/codez-org/codez";

    let (status, created) = post(
        &format!("{base}/statuses/{sha}"),
        serde_json::json!({ "state": "success", "context": "ci/port" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["context"], "ci/port");

    let (status, list) = get(&format!("{base}/commits/{sha}/statuses")).await;
    assert_eq!(status, StatusCode::OK);
    let list = list.as_array().unwrap();
    assert!(
        list.iter().any(|s| s["context"] == "ci/port"),
        "commit status list dropped the write: {list:?}"
    );

    let (status, combined) = get(&format!("{base}/commits/{sha}/status")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(combined["total_count"].as_u64(), Some(list.len() as u64));
    assert_eq!(combined["state"], "success");
}

#[tokio::test]
async fn tracked_time_round_trips_and_shows_in_every_times_view() {
    // A tracked time is one record; the issue, repository and user views must
    // all agree on it. Any of the three answering from a constant is a bug the
    // write-only assertion would miss.
    let issue = "/api/v1/repos/codez-org/codez/issues/1";
    let (status, created) = post(
        &format!("{issue}/times"),
        serde_json::json!({ "time": 3600 }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let id = created["id"].as_i64().unwrap();

    let (status, times) = get(&format!("{issue}/times")).await;
    assert_eq!(status, StatusCode::OK);
    let times = times.as_array().unwrap();
    let mine = times
        .iter()
        .find(|t| t["id"].as_i64() == Some(id))
        .unwrap_or_else(|| panic!("tracked time missing from issue view: {times:?}"));
    assert_eq!(mine["time"].as_i64(), Some(3600));

    let (status, repo_times) = get("/api/v1/repos/codez-org/codez/times").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        !repo_times.as_array().unwrap().is_empty(),
        "repository times view is empty despite a tracked time: {repo_times}"
    );

    assert_eq!(
        delete(&format!("{issue}/times/{id}")).await,
        StatusCode::NO_CONTENT
    );
    let (_, times) = get(&format!("{issue}/times")).await;
    assert!(
        !times
            .as_array()
            .unwrap()
            .iter()
            .any(|t| t["id"].as_i64() == Some(id)),
        "tracked time survived deletion: {times}"
    );
}

#[tokio::test]
async fn repository_notifications_come_from_the_store() {
    // The demo dataset seeds a notification for `alice` on the `codez` repo, so
    // a repository-scoped listing that returns `[]` is answering from a constant.
    let (status, list) = get("/api/v1/repos/codez-org/codez/notifications").await;
    assert_eq!(status, StatusCode::OK);
    let list = list.as_array().unwrap();
    assert!(
        !list.is_empty(),
        "seeded notification was not returned by the repository view"
    );
    assert_eq!(list[0]["repository"]["name"], "codez");
}

#[tokio::test]
async fn user_stopwatches_reflect_a_started_stopwatch() {
    // `/user/stopwatches` derives from the issue store, so it must not be a
    // constant: start one and it has to appear.
    let base = "/api/v1/repos/codez-org/codez/issues/1/stopwatch";
    post(&format!("{base}/start"), serde_json::json!({})).await;

    let (status, list) = get("/api/v1/user/stopwatches").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        !list.as_array().unwrap().is_empty(),
        "started stopwatch did not surface in the user listing"
    );

    delete(&format!("{base}/delete")).await;
}
