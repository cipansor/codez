//! Integration tests for the operations ported after the initial coverage
//! sweep: follows and blocks, project boards, OAuth2 applications, user
//! webhooks, the admin surface, and repository lookup by id / template
//! generation.
//!
//! Each test drives the real router over the seeded in-memory state, so the
//! assertions describe the observable follow-up (a write shows up in the next
//! list, a delete makes the detail 404) rather than merely the status code.

use axum::http::StatusCode;

use super::{call, delete, get, patch, post, put};

// --- Follows and blocks ---------------------------------------------------

#[tokio::test]
async fn follow_and_unfollow_round_trip_over_http() {
    let uri = "/api/v1/user/following/bob";

    // Not followed yet: the probe answers 404.
    let (status, _) = get(uri).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, _) = put(uri, serde_json::json!({})).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // The probe answers `204` once the follow edge exists.
    let (status, _) = get(uri).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // The follow shows up on both the actor's and the target's list.
    let (_, following) = get("/api/v1/user/following").await;
    assert!(following
        .as_array()
        .unwrap()
        .iter()
        .any(|u| u["login"] == "bob"));
    let (_, followers) = get("/api/v1/users/bob/followers").await;
    assert!(followers
        .as_array()
        .unwrap()
        .iter()
        .any(|u| u["login"] == "alice"));

    // The profile counts are derived from the same edges.
    let (_, bob) = get("/api/v1/users/bob").await;
    assert_eq!(bob["followers_count"], 1);
    let (_, alice) = get("/api/v1/users/alice").await;
    assert_eq!(alice["following_count"], 1);

    let status = delete(uri).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = get(uri).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (_, bob) = get("/api/v1/users/bob").await;
    assert_eq!(bob["followers_count"], 0);
}

#[tokio::test]
async fn blocking_a_user_is_visible_in_the_block_list() {
    let uri = "/api/v1/user/blocks/bob";

    let (status, _) = put(uri, serde_json::json!({})).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // The probe answers `204` once the block edge exists.
    let (status, _) = get(uri).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, blocked) = get("/api/v1/user/blocks").await;
    assert!(blocked
        .as_array()
        .unwrap()
        .iter()
        .any(|u| u["login"] == "bob"));

    let status = delete(uri).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

// --- Projects and their columns ------------------------------------------

#[tokio::test]
async fn project_columns_lifecycle_over_http() {
    let (status, project) = post(
        "/api/v1/user/projects",
        serde_json::json!({ "title": "Integration board" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let project_id = project["id"].as_i64().unwrap();
    assert_eq!(project["title"], "Integration board");

    // A created project appears in the list without a reload.
    let (_, projects) = get("/api/v1/user/projects").await;
    assert!(projects
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["id"] == project_id));

    let (status, column) = post(
        &format!("/api/v1/user/projects/{project_id}/columns"),
        serde_json::json!({ "title": "To do" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let column_id = column["id"].as_i64().unwrap();

    let (_, columns) = get(&format!("/api/v1/user/projects/{project_id}/columns")).await;
    assert_eq!(columns.as_array().unwrap().len(), 1);

    // Marking the only column default succeeds with an empty body.
    let (status, body) = post(
        &format!("/api/v1/user/projects/{project_id}/columns/{column_id}/default"),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(body.is_null());

    let (status, _) = patch(
        &format!("/api/v1/user/projects/{project_id}/columns/{column_id}"),
        serde_json::json!({ "title": "Doing" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (_, columns) = get(&format!("/api/v1/user/projects/{project_id}/columns")).await;
    assert_eq!(columns[0]["title"], "Doing");

    let status = delete(&format!(
        "/api/v1/user/projects/{project_id}/columns/{column_id}"
    ))
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, columns) = get(&format!("/api/v1/user/projects/{project_id}/columns")).await;
    assert!(columns.as_array().unwrap().is_empty());

    let status = delete(&format!("/api/v1/user/projects/{project_id}")).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn moving_a_project_issue_between_columns() {
    let (_, project) = post(
        "/api/v1/user/projects",
        serde_json::json!({ "title": "Move board" }),
    )
    .await;
    let project_id = project["id"].as_i64().unwrap();

    let (_, first) = post(
        &format!("/api/v1/user/projects/{project_id}/columns"),
        serde_json::json!({ "title": "Backlog" }),
    )
    .await;
    let (_, second) = post(
        &format!("/api/v1/user/projects/{project_id}/columns"),
        serde_json::json!({ "title": "Done" }),
    )
    .await;
    let first_id = first["id"].as_i64().unwrap();
    let second_id = second["id"].as_i64().unwrap();

    let (status, _) = post(
        &format!("/api/v1/user/projects/{project_id}/issues/1/move"),
        serde_json::json!({ "column_id": second_id }),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, in_second) = get(&format!(
        "/api/v1/user/projects/{project_id}/columns/{second_id}/issues"
    ))
    .await;
    assert_eq!(in_second.as_array().unwrap().len(), 1);

    let (_, in_first) = get(&format!(
        "/api/v1/user/projects/{project_id}/columns/{first_id}/issues"
    ))
    .await;
    assert!(in_first.as_array().unwrap().is_empty());
}

// --- OAuth2 applications --------------------------------------------------

#[tokio::test]
async fn oauth2_application_lifecycle() {
    let (status, app) = post(
        "/api/v1/user/applications/oauth2",
        serde_json::json!({
            "name": "integration-app",
            "redirect_uris": ["https://example.com/cb"],
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let app_id = app["id"].as_i64().unwrap();
    assert_eq!(app["name"], "integration-app");

    let (_, apps) = get("/api/v1/user/applications/oauth2").await;
    assert!(apps.as_array().unwrap().iter().any(|a| a["id"] == app_id));

    let (status, fetched) = get(&format!("/api/v1/user/applications/oauth2/{app_id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(fetched["name"], "integration-app");

    let status = delete(&format!("/api/v1/user/applications/oauth2/{app_id}")).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = get(&format!("/api/v1/user/applications/oauth2/{app_id}")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// --- User hooks -----------------------------------------------------------

#[tokio::test]
async fn user_webhook_lifecycle_over_http() {
    let (status, hook) = post(
        "/api/v1/user/hooks",
        serde_json::json!({
            "type": "gitea",
            "name": "integration-hook",
            "active": true,
            "config": { "url": "https://example.com/hook", "content_type": "json" },
            "events": ["push"],
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let hook_id = hook["id"].as_i64().unwrap();

    let (_, hooks) = get("/api/v1/user/hooks").await;
    assert!(hooks.as_array().unwrap().iter().any(|h| h["id"] == hook_id));

    let (status, updated) = patch(
        &format!("/api/v1/user/hooks/{hook_id}"),
        serde_json::json!({ "active": false }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["active"], false);

    let status = delete(&format!("/api/v1/user/hooks/{hook_id}")).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

// --- Admin surface --------------------------------------------------------

#[tokio::test]
async fn admin_cron_lists_tasks_and_run_answers_no_content() {
    let (status, body) = get("/api/v1/admin/cron").await;
    assert_eq!(status, StatusCode::OK);
    let tasks = body.as_array().unwrap();
    assert!(!tasks.is_empty());
    assert!(tasks[0]["name"].is_string());

    let (status, _) = post(
        "/api/v1/admin/cron/repo_health_check",
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn admin_emails_include_seeded_accounts() {
    // `admin/emails` answers with a bare array, not a `data` envelope.
    let (status, body) = get("/api/v1/admin/emails").await;
    assert_eq!(status, StatusCode::OK);
    let emails = body.as_array().unwrap();
    assert!(emails.iter().any(|e| e["email"] == "alice@example.com"));
}

#[tokio::test]
async fn admin_badges_are_added_and_removed() {
    let uri = "/api/v1/admin/users/alice/badges";
    let slug = format!("badge-{}", std::process::id());

    let (status, created) = post(uri, serde_json::json!({ "badge_slugs": [slug] })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(created.as_array().unwrap().len(), 1);
    assert_eq!(created[0]["slug"], slug);

    let (_, badges) = get(uri).await;
    assert!(badges.as_array().unwrap().iter().any(|b| b["slug"] == slug));

    let (status, _, _) = call(
        "DELETE",
        uri,
        Some(serde_json::json!({ "badge_slugs": [slug] })),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, badges) = get(uri).await;
    assert!(!badges.as_array().unwrap().iter().any(|b| b["slug"] == slug));
}

#[tokio::test]
async fn admin_runners_can_be_registered_and_removed() {
    let (status, runner) = post(
        "/api/v1/admin/actions/runners",
        serde_json::json!({ "name": "integration-runner", "labels": ["self-hosted"] }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let runner_id = runner["id"].as_i64().unwrap();
    assert_eq!(runner["name"], "integration-runner");

    // The list reports the new runner (the spec names the field `runners`).
    let (_, list) = get("/api/v1/admin/actions/runners").await;
    assert!(list["runners"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["id"] == runner_id));

    // A single runner can be fetched by id.
    let (status, fetched) = get(&format!("/api/v1/admin/actions/runners/{runner_id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(fetched["name"], "integration-runner");

    let (status, renamed) = patch(
        &format!("/api/v1/admin/actions/runners/{runner_id}"),
        serde_json::json!({ "name": "renamed-runner", "disabled": true }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(renamed["name"], "renamed-runner");
    assert_eq!(renamed["disabled"], true);

    let status = delete(&format!("/api/v1/admin/actions/runners/{runner_id}")).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // The removed runner is gone from the list.
    let (_, list) = get("/api/v1/admin/actions/runners").await;
    assert!(!list["runners"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["id"] == runner_id));
}

#[tokio::test]
async fn admin_packages_endpoint_answers_with_a_page() {
    // `admin/packages` answers with a bare array, not a `data` envelope.
    let (status, body) = get("/api/v1/admin/packages").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_array(), "got {body}");
}

// --- Repository-by-id and template generation -----------------------------

#[tokio::test]
async fn repository_lookup_by_id_matches_the_full_name() {
    let (status, body) = get("/api/v1/repositories/101").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["full_name"], "alice/notes");

    let (status, _) = get("/api/v1/repositories/999999").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn generating_from_a_template_creates_a_new_repository() {
    let name = format!("generated-{}", std::process::id());
    let (status, created) = post(
        "/api/v1/repos/alice/notes/generate",
        serde_json::json!({ "name": name, "description": "from template" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["name"], name);
    assert_eq!(created["description"], "from template");

    let (status, fetched) = get(&format!("/api/v1/repos/alice/{name}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(fetched["name"], name);

    // Generating the same name twice is a conflict, as Gitea reports it.
    let (status, _) = post(
        "/api/v1/repos/alice/notes/generate",
        serde_json::json!({ "name": name }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
}

// --- Wiki ------------------------------------------------------------------

#[tokio::test]
async fn wiki_pages_persist_their_content_and_history() {
    // The seeded repository ships one page.
    let (status, pages) = get("/api/v1/repos/codez-org/codez/wiki/pages").await;
    assert_eq!(status, StatusCode::OK);
    assert!(pages
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["title"] == "Home"));

    let title = format!("Notes-{}", std::process::id());
    let (status, created) = post(
        "/api/v1/repos/codez-org/codez/wiki/new",
        serde_json::json!({
            "title": title,
            "content_base64": "aGVsbG8gd29ybGQ=", // "hello world"
            "message": "first revision",
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["title"], title);
    assert_eq!(created["commit_count"], 1);

    // The new page is listed without a reload.
    let (_, pages) = get("/api/v1/repos/codez-org/codez/wiki/pages").await;
    assert!(pages
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["title"] == title));

    // Its content decodes back to what was sent.
    let (status, page) = get(&format!("/api/v1/repos/codez-org/codez/wiki/page/{title}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(page["content_base64"], "aGVsbG8gd29ybGQ=");

    // Editing appends a revision rather than replacing the page.
    let (status, edited) = patch(
        &format!("/api/v1/repos/codez-org/codez/wiki/page/{title}"),
        serde_json::json!({ "content_base64": "c2Vjb25k", "message": "second" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(edited["commit_count"], 2);

    let (_, revisions) = get(&format!(
        "/api/v1/repos/codez-org/codez/wiki/revisions/{title}"
    ))
    .await;
    assert_eq!(revisions.as_array().unwrap().len(), 2);

    // Deleting removes it from the listing.
    let status = delete(&format!("/api/v1/repos/codez-org/codez/wiki/page/{title}")).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, pages) = get("/api/v1/repos/codez-org/codez/wiki/pages").await;
    assert!(!pages
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["title"] == title));
}

// --- Packages --------------------------------------------------------------

#[tokio::test]
async fn package_registry_reports_seeded_packages() {
    let (status, list) = get("/api/v1/packages/codez-org").await;
    assert_eq!(status, StatusCode::OK);
    let items = list.as_array().unwrap();
    assert!(items.iter().any(|p| p["name"] == "codez-backend"));
    assert!(items.iter().all(|p| p["owner"]["login"] == "codez-org"));

    // One package, all versions.
    let (_, versions) = get("/api/v1/packages/codez-org/container/codez-backend").await;
    assert_eq!(versions.as_array().unwrap().len(), 2);

    // `-/latest` picks the newest version.
    let (status, latest) = get("/api/v1/packages/codez-org/container/codez-backend/-/latest").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(latest["version"], "1.1.0");

    // A version's files are listed.
    let (status, files) =
        get("/api/v1/packages/codez-org/container/codez-backend/1.1.0/files").await;
    assert_eq!(status, StatusCode::OK);
    assert!(files
        .as_array()
        .unwrap()
        .iter()
        .any(|f| f["name"] == "codez-backend-1.1.0.tar.gz"));

    // Deleting a version drops it from the list.
    let status = delete("/api/v1/packages/codez-org/container/codez-backend/1.1.0").await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, versions) = get("/api/v1/packages/codez-org/container/codez-backend").await;
    assert_eq!(versions.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn unknown_package_is_a_404() {
    let (status, _) = get("/api/v1/packages/codez-org/container/nope/1.0.0").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// --- Actions ---------------------------------------------------------------

#[tokio::test]
async fn dispatching_a_workflow_queues_a_run_and_job() {
    let (status, workflows) = get("/api/v1/repos/codez-org/codez/actions/workflows").await;
    assert_eq!(status, StatusCode::OK);
    let list = workflows["workflows"].as_array().unwrap();
    assert!(!list.is_empty(), "expected seeded workflows: {workflows}");
    let workflow_id = list[0]["id"].as_str().unwrap().to_string();

    let (status, dispatched) = post(
        &format!("/api/v1/repos/codez-org/codez/actions/workflows/{workflow_id}/dispatches"),
        serde_json::json!({ "ref": "main" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let run_id = dispatched["id"].as_i64().unwrap();

    // The run is listed under the repository.
    let (_, runs) = get("/api/v1/repos/codez-org/codez/actions/runs").await;
    assert!(runs["workflow_runs"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["id"] == run_id));
    assert!(runs["total_count"].as_i64().unwrap() >= 1);

    // The dispatch created the job a runner would pick up.
    let (status, jobs) = get(&format!(
        "/api/v1/repos/codez-org/codez/actions/runs/{run_id}/jobs"
    ))
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(jobs["jobs"].as_array().unwrap().len(), 1);

    // Cancelling the run flips its status.
    let (status, cancelled) = post(
        &format!("/api/v1/repos/codez-org/codez/actions/runs/{run_id}/cancel"),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(cancelled["status"], "cancelled");

    // Deleting the run removes it, along with its jobs.
    let status = delete(&format!(
        "/api/v1/repos/codez-org/codez/actions/runs/{run_id}"
    ))
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, runs) = get("/api/v1/repos/codez-org/codez/actions/runs").await;
    assert!(!runs["workflow_runs"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["id"] == run_id));
}

#[tokio::test]
async fn repository_secrets_and_variables_round_trip() {
    let secret = format!("TOKEN_{}", std::process::id());
    let (status, _) = put(
        &format!("/api/v1/repos/codez-org/codez/actions/secrets/{secret}"),
        serde_json::json!({ "data": "s3cr3t", "description": "ci token" }),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, secrets) = get("/api/v1/repos/codez-org/codez/actions/secrets").await;
    assert!(secrets
        .as_array()
        .unwrap()
        .iter()
        .any(|s| s["name"] == secret));
    // The value is never echoed back.
    assert!(!secrets.to_string().contains("s3cr3t"));

    let status = delete(&format!(
        "/api/v1/repos/codez-org/codez/actions/secrets/{secret}"
    ))
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, secrets) = get("/api/v1/repos/codez-org/codez/actions/secrets").await;
    assert!(!secrets
        .as_array()
        .unwrap()
        .iter()
        .any(|s| s["name"] == secret));

    let variable = format!("VAR_{}", std::process::id());
    let (status, _) = post(
        &format!("/api/v1/repos/codez-org/codez/actions/variables/{variable}"),
        serde_json::json!({ "value": "v1", "description": "a variable" }),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, fetched) = get(&format!(
        "/api/v1/repos/codez-org/codez/actions/variables/{variable}"
    ))
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(fetched["data"], "v1");

    // Updating replaces the value in place.
    let (status, _) = put(
        &format!("/api/v1/repos/codez-org/codez/actions/variables/{variable}"),
        serde_json::json!({ "value": "v2" }),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, fetched) = get(&format!(
        "/api/v1/repos/codez-org/codez/actions/variables/{variable}"
    ))
    .await;
    assert_eq!(fetched["data"], "v2");
}

#[tokio::test]
async fn org_runners_are_scoped_to_the_organization() {
    let (_, org_runners) = get("/api/v1/orgs/codez-org/actions/runners").await;
    let org_list = org_runners["runners"].as_array().unwrap();
    assert!(org_list.iter().any(|r| r["name"] == "org-runner"));

    // The instance-level list is a different scope, so it does not include the
    // organization's runner.
    let (_, admin_runners) = get("/api/v1/admin/actions/runners").await;
    assert!(!admin_runners["runners"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["name"] == "org-runner"));

    // A registration token is minted per scope.
    let (status, token) = post(
        "/api/v1/orgs/codez-org/actions/runners/registration-token",
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert!(token["token"].as_str().is_some_and(|t| !t.is_empty()));
}

#[tokio::test]
async fn job_logs_report_the_recorded_steps() {
    let (_, workflows) = get("/api/v1/repos/codez-org/codez/actions/workflows").await;
    let workflow_id = workflows["workflows"].as_array().unwrap()[0]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let (_, dispatched) = post(
        &format!("/api/v1/repos/codez-org/codez/actions/workflows/{workflow_id}/dispatches"),
        serde_json::json!({}),
    )
    .await;
    let run_id = dispatched["id"].as_i64().unwrap();

    let (_, jobs) = get(&format!(
        "/api/v1/repos/codez-org/codez/actions/runs/{run_id}/jobs"
    ))
    .await;
    let job_id = jobs["jobs"].as_array().unwrap()[0]["id"].as_i64().unwrap();

    let (status, log) = get(&format!(
        "/api/v1/repos/codez-org/codez/actions/jobs/{job_id}/logs"
    ))
    .await;
    assert_eq!(status, StatusCode::OK);
    let text = log.as_str().unwrap();
    assert!(text.contains("Set up job"), "log was {text}");

    // The run-level log aggregates its jobs.
    let (status, run_log) = get(&format!(
        "/api/v1/repos/codez-org/codez/actions/runs/{run_id}/logs"
    ))
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(run_log.as_str().unwrap().contains("Set up job"));
}

// --- Misc: markup, token, deprecated org route -----------------------------

#[tokio::test]
async fn markup_renders_the_text_it_is_given() {
    let (status, body) = post(
        "/api/v1/markup",
        serde_json::json!({ "Text": "# Heading\n\nbody", "Mode": "markdown" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let html = body.as_str().unwrap();
    assert!(html.contains("Heading"), "rendered {html}");
    assert!(html.contains("<h1>"), "rendered {html}");

    // Without text the request is rejected rather than silently accepted.
    let (status, _) = post("/api/v1/markup", serde_json::json!({ "Mode": "markdown" })).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn current_token_round_trips_and_can_be_revoked() {
    // Sign in to mint a token, then present it back.
    let (status, signed) = post(
        "/api/v1/signin",
        serde_json::json!({ "user_name": "alice", "password": "password" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let sha1 = signed["sha1"].as_str().unwrap().to_string();

    let (status, token) = get(&format!("/api/v1/token?token={sha1}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(token["name"], "signin");
    assert_eq!(token["user"]["login"], "alice");
    assert_eq!(token["scopes"], serde_json::json!(["all"]));

    // The secret itself is never echoed back.
    assert!(token.get("sha1").is_none(), "token leaked {token}");

    let status = delete(&format!("/api/v1/token?token={sha1}")).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Once revoked the same token no longer resolves.
    let (status, _) = get(&format!("/api/v1/token?token={sha1}")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn creating_a_repository_under_an_org_uses_the_org_as_owner() {
    let name = format!("org-owned-{}", std::process::id());
    let (status, created) = post(
        "/api/v1/orgs/codez-org/repos",
        serde_json::json!({ "name": name, "auto_init": true }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["owner"]["login"], "codez-org");
    assert_eq!(created["full_name"], format!("codez-org/{name}"));

    // The deprecated singular alias behaves identically.
    let alias = format!("{name}-alias");
    let (status, created) = post(
        "/api/v1/org/codez-org/repos",
        serde_json::json!({ "name": alias, "auto_init": true }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["full_name"], format!("codez-org/{alias}"));
}

/// The admin views of a user's keys and orgs are the same collections the
/// public endpoints serve. They used to answer a fixed `[]`, so an admin
/// looking at a user saw nothing even though the data was in the store.
#[tokio::test]
async fn admin_user_keys_and_orgs_match_the_public_endpoints() {
    // A key created through the public endpoint must be visible to admin too.
    let title = format!("admin-key-{}", std::process::id());
    let (status, _) = post(
        "/api/v1/user/keys",
        serde_json::json!({ "title": title, "key": "ssh-rsa AAAAadminprobe" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, admin_keys) = get("/api/v1/admin/users/alice/keys").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        admin_keys
            .as_array()
            .unwrap()
            .iter()
            .any(|k| k["title"] == title),
        "admin keys endpoint did not surface the created key: {admin_keys}"
    );

    let (_, public_keys) = get("/api/v1/users/alice/keys").await;
    assert_eq!(
        admin_keys, public_keys,
        "admin and public key views diverge"
    );

    // Same for org membership.
    let (status, admin_orgs) = get("/api/v1/admin/users/alice/orgs").await;
    assert_eq!(status, StatusCode::OK);
    let (_, public_orgs) = get("/api/v1/users/alice/orgs").await;
    assert_eq!(
        admin_orgs, public_orgs,
        "admin and public org views diverge"
    );
    assert!(
        !public_orgs.as_array().unwrap().is_empty(),
        "seed should give alice at least one organization"
    );
}

/// `POST /users/{login}/tokens` used to answer a bare list (the same reply as
/// `GET`), so a token was never created. It must create and then list.
#[tokio::test]
async fn user_tokens_round_trip_over_http() {
    let name = format!("tok-{}", std::process::id());
    let (status, created) = post(
        "/api/v1/users/alice/tokens",
        serde_json::json!({ "name": name, "scopes": ["read:user"] }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "token create should be 201");
    assert_eq!(created["name"], name);
    assert!(
        created["sha1"].is_string(),
        "create must return the token secret: {created}"
    );

    // The new token shows up in the list.
    let (status, listed) = get("/api/v1/users/alice/tokens").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        listed.as_array().unwrap().iter().any(|t| t["name"] == name),
        "created token missing from the list: {listed}"
    );

    // Another user's tokens are private.
    let (status, _) = get("/api/v1/users/bob/tokens").await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}
