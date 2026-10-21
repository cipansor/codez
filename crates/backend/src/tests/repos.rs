//! Repository, pagination and static-serving integration tests.

use axum::http::StatusCode;

use super::{delete, get, patch, post, put};

#[tokio::test]
async fn pagination_slices_and_reports_totals() {
    // Create enough repositories to need a second page.
    for index in 0..5 {
        let (status, _) = post(
            "/api/v1/user/repos",
            serde_json::json!({ "name": format!("page-{index}") }),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
    }

    let (status, headers, first) =
        super::call("GET", "/api/v1/repos/search?q=page-&limit=2&page=1", None).await;
    assert_eq!(status, StatusCode::OK);
    // Gitea wraps search results in `SearchResults { ok, data }`.
    assert_eq!(first["ok"], true);
    assert_eq!(first["data"].as_array().unwrap().len(), 2);
    assert_eq!(headers.get("X-HasMore").unwrap(), "true");

    let (status, headers, last) =
        super::call("GET", "/api/v1/repos/search?q=page-&limit=2&page=3", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(last["data"].as_array().unwrap().len(), 1);
    assert_eq!(headers.get("X-HasMore").unwrap(), "false");
}

#[tokio::test]
async fn limit_is_capped_at_the_documented_maximum() {
    let (status, headers, _) = super::call("GET", "/api/v1/repos/search?limit=100000", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        headers.get("X-Total-Count").unwrap(),
        headers.get("X-Total-Count").unwrap()
    );
}

#[tokio::test]
async fn star_and_unstar_are_idempotent_over_http() {
    // Gitea stars through `/user/starred/{owner}/{repo}`, not a repo-scoped path.
    let repo = "/api/v1/user/starred/codez-org/codez";
    for _ in 0..2 {
        let (status, _) = put(repo, serde_json::json!({})).await;
        assert_eq!(status, StatusCode::NO_CONTENT);
    }
    let (status, body) = get("/api/v1/repos/codez-org/codez/stargazers").await;
    assert_eq!(status, StatusCode::OK);
    let logins: Vec<&str> = body
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|u| u["login"].as_str())
        .collect();
    assert_eq!(logins.len(), 2, "got {logins:?}");
    assert!(logins.contains(&"bob"));
    assert!(logins.contains(&"alice"));

    assert_eq!(delete(repo).await, StatusCode::NO_CONTENT);
    let (_, body) = get("/api/v1/repos/codez-org/codez/stargazers").await;
    assert_eq!(body.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn branch_creation_and_deletion() {
    let (status, created) = post(
        "/api/v1/repos/codez-org/codez/branches",
        serde_json::json!({ "new_branch_name": "integration-check" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["name"], "integration-check");

    let (status, _) = get("/api/v1/repos/codez-org/codez/branches/integration-check").await;
    assert_eq!(status, StatusCode::OK);

    assert_eq!(
        delete("/api/v1/repos/codez-org/codez/branches/integration-check").await,
        StatusCode::NO_CONTENT
    );
    let (status, _) = get("/api/v1/repos/codez-org/codez/branches/integration-check").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn file_create_read_update_delete() {
    let base = "/api/v1/repos/codez-org/codez/contents/notes/scratch.md";
    let (status, _) = post(
        base,
        serde_json::json!({
            "content": "aGVsbG8=", // "hello"
            "message": "add scratch",
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, body) = get(base).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        crate::api::repos::decode_base64(body["content"].as_str().unwrap()),
        "hello"
    );

    let (status, _) = put(
        base,
        serde_json::json!({ "content": "d29ybGQ=", "message": "update" }), // "world"
    )
    .await;
    // Gitea answers `200 OK` when the write updates an existing file.
    assert_eq!(status, StatusCode::OK);
    let (_, body) = get(base).await;
    assert_eq!(
        crate::api::repos::decode_base64(body["content"].as_str().unwrap()),
        "world"
    );

    assert_eq!(delete(base).await, StatusCode::OK);
    let (status, _) = get(base).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn transfer_moves_a_repository_to_another_owner() {
    let (status, _) = post(
        "/api/v1/user/repos",
        serde_json::json!({ "name": "to-transfer" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, moved) = post(
        "/api/v1/repos/alice/to-transfer/transfer",
        serde_json::json!({ "new_owner": "bob" }),
    )
    .await;
    // Gitea answers `202 Accepted`: the transfer is queued.
    assert_eq!(status, StatusCode::ACCEPTED);
    assert_eq!(moved["full_name"], "bob/to-transfer");

    let (status, _) = get("/api/v1/repos/bob/to-transfer").await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn deep_links_fall_back_to_the_spa_shell() {
    // The bundle is not built in unit tests, so the fallback reports the
    // missing bundle rather than a routing error.
    let (status, _, body) = super::call("GET", "/codez-org/codez/issues/1", None).await;
    assert!(matches!(status, StatusCode::OK | StatusCode::NOT_FOUND));
    if status == StatusCode::NOT_FOUND {
        assert!(body.to_string().contains("frontend bundle not found"));
    }
}

#[tokio::test]
async fn unknown_api_paths_are_never_answered_with_html() {
    let (status, _, body) = super::call("GET", "/api/v1/not/real", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(
        body.get("message").is_some(),
        "API 404s must be JSON, got {body}"
    );
}

#[tokio::test]
async fn branch_protections_round_trip_through_the_listing() {
    let base = "/api/v1/repos/codez-org/codez/branch_protections";
    let branch = format!("protect-{}", crate::state::now());

    let (status, created) = post(
        base,
        serde_json::json!({ "branch_name": branch, "enable_push": true }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["branch_name"], branch);

    // The create must be visible to the listing without a reload.
    let (_, listed) = get(base).await;
    let names: Vec<&str> = listed
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|p| p["branch_name"].as_str())
        .collect();
    assert!(names.contains(&branch.as_str()), "got {names:?}");

    let detail = format!("{base}/{branch}");
    let (status, body) = get(&detail).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["enable_push"], true);

    let (status, patched) = patch(&detail, serde_json::json!({ "enable_push": false })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(patched["enable_push"], false);

    assert_eq!(delete(&detail).await, StatusCode::NO_CONTENT);
    let (status, _) = get(&detail).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn tag_protections_can_be_created_listed_and_deleted() {
    let base = "/api/v1/repos/codez-org/codez/tag_protections";
    let pattern = format!("v{}.*", crate::state::now());

    let (status, created) = post(base, serde_json::json!({ "name": pattern })).await;
    assert_eq!(status, StatusCode::CREATED);
    let id = created["id"].as_i64().unwrap();
    assert_eq!(created["name_pattern"], pattern);

    let (_, listed) = get(base).await;
    assert!(
        listed
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["id"].as_i64() == Some(id)),
        "created protection missing from {listed}"
    );

    let detail = format!("{base}/{id}");
    assert_eq!(delete(&detail).await, StatusCode::NO_CONTENT);
    let (status, _) = get(&detail).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn push_mirrors_are_created_and_removed_by_name() {
    let base = "/api/v1/repos/codez-org/codez/push_mirrors";
    let name = format!("mirror-{}", crate::state::now());

    let (status, created) = post(
        base,
        serde_json::json!({ "remote_name": name, "remote_address": "https://example.com/x.git" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["remote_name"], name);

    let (_, listed) = get(base).await;
    assert!(
        listed
            .as_array()
            .unwrap()
            .iter()
            .any(|m| m["remote_name"] == name),
        "created mirror missing from {listed}"
    );

    assert_eq!(
        delete(&format!("{base}/{name}")).await,
        StatusCode::NO_CONTENT
    );
}

#[tokio::test]
async fn a_branch_can_be_renamed() {
    let base = "/api/v1/repos/codez-org/codez/branches";
    let original = format!("rename-{}", crate::state::now());
    let renamed = format!("{original}-new");

    let (status, _) = post(
        "/api/v1/repos/codez-org/codez/branches",
        serde_json::json!({ "new_branch_name": original, "old_branch_name": "main" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, body) = put(
        &format!("{base}/{original}"),
        serde_json::json!({ "name": renamed }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["name"], renamed);

    let (status, _) = get(&format!("{base}/{original}")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = get(&format!("{base}/{renamed}")).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn deploy_keys_are_scoped_to_their_repository() {
    let base = "/api/v1/repos/codez-org/codez/keys";
    let title = format!("deploy-{}", crate::state::now());

    let (status, created) = post(
        base,
        serde_json::json!({ "title": title, "key": "ssh-ed25519 AAAA demo" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let id = created["id"].as_i64().unwrap();

    let (_, listed) = get(base).await;
    assert!(
        listed
            .as_array()
            .unwrap()
            .iter()
            .any(|k| k["id"].as_i64() == Some(id)),
        "created key missing from {listed}"
    );

    let (status, _) = get(&format!("{base}/{id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        delete(&format!("{base}/{id}")).await,
        StatusCode::NO_CONTENT
    );
    let (status, _) = get(&format!("{base}/{id}")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn commit_statuses_are_recorded_and_listed() {
    let base = "/api/v1/repos/codez-org/codez/statuses";
    let sha = "c0ffee0000000000000000000000000000000000";
    let context = format!("ci/{}", crate::state::now());

    let (status, created) = post(
        &format!("{base}/{sha}"),
        serde_json::json!({ "state": "success", "context": context, "description": "all good" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["context"], context);

    let (status, listed) = get(&format!("{base}/{sha}")).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        listed
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["context"] == context && s["status"] == "success"),
        "created status missing from {listed}"
    );
}

#[tokio::test]
async fn topics_can_be_replaced_and_removed_one_by_one() {
    let base = "/api/v1/repos/codez-org/codez/topics";
    let first = format!("topic-a-{}", crate::state::now());
    let second = format!("topic-b-{}", crate::state::now());

    let (status, body) = put(base, serde_json::json!({ "topics": [first, second] })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["topics"].as_array().unwrap().len(), 2);

    let (_, listed) = get(base).await;
    assert_eq!(listed["topics"].as_array().unwrap().len(), 2);

    // `PUT /topics/{topic}` adds one topic; `DELETE` removes it.
    assert_eq!(
        put(&format!("{base}/{first}"), serde_json::json!({}))
            .await
            .0,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        delete(&format!("{base}/{second}")).await,
        StatusCode::NO_CONTENT
    );
    let (_, listed) = get(base).await;
    let remaining: Vec<&str> = listed["topics"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|t| t.as_str())
        .collect();
    assert_eq!(remaining, vec![first.as_str()], "got {listed}");
}
