//! Generated Axum wiring: one route per Gitea `/api/v1` operation.
//!
//! 537 operations across 340 paths.
//! Regenerate with `python3 scripts/gen_gitea_api.py`.

use axum::routing::any;
use axum::Router;

use super::dispatch::dispatch;
use crate::state::AppState;

/// Every `/api/v1` path Gitea exposes, served by the shared dispatcher.
pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/admin/actions/jobs", any(dispatch))
        .route("/api/v1/admin/actions/runners", any(dispatch))
        .route(
            "/api/v1/admin/actions/runners/registration-token",
            any(dispatch),
        )
        .route("/api/v1/admin/actions/runners/{p0}", any(dispatch))
        .route("/api/v1/admin/actions/runs", any(dispatch))
        .route("/api/v1/admin/cron", any(dispatch))
        .route("/api/v1/admin/cron/{p0}", any(dispatch))
        .route("/api/v1/admin/emails", any(dispatch))
        .route("/api/v1/admin/emails/search", any(dispatch))
        .route("/api/v1/admin/hooks", any(dispatch))
        .route("/api/v1/admin/hooks/{p0}", any(dispatch))
        .route("/api/v1/admin/orgs", any(dispatch))
        .route("/api/v1/admin/packages", any(dispatch))
        .route("/api/v1/admin/unadopted", any(dispatch))
        .route("/api/v1/admin/unadopted/{p0}/{p1}", any(dispatch))
        .route("/api/v1/admin/users", any(dispatch))
        .route("/api/v1/admin/users/{p0}", any(dispatch))
        .route("/api/v1/admin/users/{p0}/badges", any(dispatch))
        .route("/api/v1/admin/users/{p0}/keys", any(dispatch))
        .route("/api/v1/admin/users/{p0}/keys/{p1}", any(dispatch))
        .route("/api/v1/admin/users/{p0}/orgs", any(dispatch))
        .route("/api/v1/admin/users/{p0}/rename", any(dispatch))
        .route("/api/v1/admin/users/{p0}/repos", any(dispatch))
        .route("/api/v1/gitignore/templates", any(dispatch))
        .route("/api/v1/gitignore/templates/{p0}", any(dispatch))
        .route("/api/v1/label/templates", any(dispatch))
        .route("/api/v1/label/templates/{p0}", any(dispatch))
        .route("/api/v1/licenses", any(dispatch))
        .route("/api/v1/licenses/{p0}", any(dispatch))
        .route("/api/v1/markdown", any(dispatch))
        .route("/api/v1/markdown/raw", any(dispatch))
        .route("/api/v1/markup", any(dispatch))
        .route("/api/v1/notifications", any(dispatch))
        .route("/api/v1/notifications/new", any(dispatch))
        .route("/api/v1/notifications/threads/{p0}", any(dispatch))
        .route("/api/v1/org/{p0}/repos", any(dispatch))
        .route("/api/v1/orgs", any(dispatch))
        .route("/api/v1/orgs/{p0}", any(dispatch))
        .route("/api/v1/orgs/{p0}/actions/jobs", any(dispatch))
        .route("/api/v1/orgs/{p0}/actions/runners", any(dispatch))
        .route(
            "/api/v1/orgs/{p0}/actions/runners/registration-token",
            any(dispatch),
        )
        .route("/api/v1/orgs/{p0}/actions/runners/{p1}", any(dispatch))
        .route("/api/v1/orgs/{p0}/actions/runs", any(dispatch))
        .route("/api/v1/orgs/{p0}/actions/secrets", any(dispatch))
        .route("/api/v1/orgs/{p0}/actions/secrets/{p1}", any(dispatch))
        .route("/api/v1/orgs/{p0}/actions/variables", any(dispatch))
        .route("/api/v1/orgs/{p0}/actions/variables/{p1}", any(dispatch))
        .route("/api/v1/orgs/{p0}/activities/feeds", any(dispatch))
        .route("/api/v1/orgs/{p0}/avatar", any(dispatch))
        .route("/api/v1/orgs/{p0}/blocks", any(dispatch))
        .route("/api/v1/orgs/{p0}/blocks/{p1}", any(dispatch))
        .route("/api/v1/orgs/{p0}/hooks", any(dispatch))
        .route("/api/v1/orgs/{p0}/hooks/{p1}", any(dispatch))
        .route("/api/v1/orgs/{p0}/labels", any(dispatch))
        .route("/api/v1/orgs/{p0}/labels/{p1}", any(dispatch))
        .route("/api/v1/orgs/{p0}/members", any(dispatch))
        .route("/api/v1/orgs/{p0}/members/{p1}", any(dispatch))
        .route("/api/v1/orgs/{p0}/projects", any(dispatch))
        .route("/api/v1/orgs/{p0}/projects/{p1}", any(dispatch))
        .route("/api/v1/orgs/{p0}/projects/{p1}/columns", any(dispatch))
        .route(
            "/api/v1/orgs/{p0}/projects/{p1}/columns/move",
            any(dispatch),
        )
        .route(
            "/api/v1/orgs/{p0}/projects/{p1}/columns/{p2}",
            any(dispatch),
        )
        .route(
            "/api/v1/orgs/{p0}/projects/{p1}/columns/{p2}/default",
            any(dispatch),
        )
        .route(
            "/api/v1/orgs/{p0}/projects/{p1}/columns/{p2}/issues",
            any(dispatch),
        )
        .route(
            "/api/v1/orgs/{p0}/projects/{p1}/columns/{p2}/issues/{p3}",
            any(dispatch),
        )
        .route(
            "/api/v1/orgs/{p0}/projects/{p1}/issues/{p2}/move",
            any(dispatch),
        )
        .route("/api/v1/orgs/{p0}/public_members", any(dispatch))
        .route("/api/v1/orgs/{p0}/public_members/{p1}", any(dispatch))
        .route("/api/v1/orgs/{p0}/rename", any(dispatch))
        .route("/api/v1/orgs/{p0}/repos", any(dispatch))
        .route("/api/v1/orgs/{p0}/teams", any(dispatch))
        .route("/api/v1/orgs/{p0}/teams/search", any(dispatch))
        .route("/api/v1/packages/{p0}", any(dispatch))
        .route("/api/v1/packages/{p0}/{p1}/{p2}", any(dispatch))
        .route("/api/v1/packages/{p0}/{p1}/{p2}/-/latest", any(dispatch))
        .route("/api/v1/packages/{p0}/{p1}/{p2}/-/link/{p3}", any(dispatch))
        .route("/api/v1/packages/{p0}/{p1}/{p2}/-/unlink", any(dispatch))
        .route("/api/v1/packages/{p0}/{p1}/{p2}/{p3}", any(dispatch))
        .route("/api/v1/packages/{p0}/{p1}/{p2}/{p3}/files", any(dispatch))
        .route("/api/v1/repos/issues/search", any(dispatch))
        .route("/api/v1/repos/migrate", any(dispatch))
        .route("/api/v1/repos/search", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/actions/artifacts", any(dispatch))
        .route(
            "/api/v1/repos/{p0}/{p1}/actions/artifacts/{p2}",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/actions/artifacts/{p2}/zip",
            any(dispatch),
        )
        .route("/api/v1/repos/{p0}/{p1}/actions/jobs", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/actions/jobs/{p2}", any(dispatch))
        .route(
            "/api/v1/repos/{p0}/{p1}/actions/jobs/{p2}/logs",
            any(dispatch),
        )
        .route("/api/v1/repos/{p0}/{p1}/actions/runners", any(dispatch))
        .route(
            "/api/v1/repos/{p0}/{p1}/actions/runners/registration-token",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/actions/runners/{p2}",
            any(dispatch),
        )
        .route("/api/v1/repos/{p0}/{p1}/actions/runs", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/actions/runs/{p2}", any(dispatch))
        .route(
            "/api/v1/repos/{p0}/{p1}/actions/runs/{p2}/approve",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/actions/runs/{p2}/artifacts",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/actions/runs/{p2}/attempts/{p3}",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/actions/runs/{p2}/attempts/{p3}/jobs",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/actions/runs/{p2}/cancel",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/actions/runs/{p2}/force-cancel",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/actions/runs/{p2}/jobs",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/actions/runs/{p2}/jobs/{p3}/rerun",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/actions/runs/{p2}/logs",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/actions/runs/{p2}/rerun",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/actions/runs/{p2}/rerun-failed-jobs",
            any(dispatch),
        )
        .route("/api/v1/repos/{p0}/{p1}/actions/secrets", any(dispatch))
        .route(
            "/api/v1/repos/{p0}/{p1}/actions/secrets/{p2}",
            any(dispatch),
        )
        .route("/api/v1/repos/{p0}/{p1}/actions/tasks", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/actions/variables", any(dispatch))
        .route(
            "/api/v1/repos/{p0}/{p1}/actions/variables/{p2}",
            any(dispatch),
        )
        .route("/api/v1/repos/{p0}/{p1}/actions/workflows", any(dispatch))
        .route(
            "/api/v1/repos/{p0}/{p1}/actions/workflows/{p2}",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/actions/workflows/{p2}/disable",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/actions/workflows/{p2}/dispatches",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/actions/workflows/{p2}/enable",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/actions/workflows/{p2}/runs",
            any(dispatch),
        )
        .route("/api/v1/repos/{p0}/{p1}/activities/feeds", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/archive/{*p2}", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/assignees", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/assignees/{p2}", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/avatar", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/branch_protections", any(dispatch))
        .route(
            "/api/v1/repos/{p0}/{p1}/branch_protections/priority",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/branch_protections/{p2}",
            any(dispatch),
        )
        .route("/api/v1/repos/{p0}/{p1}/branches", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/branches/{p2}", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/collaborators", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/collaborators/{p2}", any(dispatch))
        .route(
            "/api/v1/repos/{p0}/{p1}/collaborators/{p2}/permission",
            any(dispatch),
        )
        .route("/api/v1/repos/{p0}/{p1}/commits", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/commits/{p2}/pull", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/commits/{p2}/status", any(dispatch))
        .route(
            "/api/v1/repos/{p0}/{p1}/commits/{p2}/statuses",
            any(dispatch),
        )
        .route("/api/v1/repos/{p0}/{p1}/compare/{*p2}", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/contents", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/contents-ext/{*p2}", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/contents/{*p2}", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/diffpatch", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/editorconfig/{*p2}", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/file-contents", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/forks", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/generate", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/git/blobs/{p2}", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/git/commits/{p2}", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/git/notes/{p2}", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/git/refs", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/git/refs/{*p2}", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/git/tags/{p2}", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/git/trees/{p2}", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/hooks", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/hooks/git", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/hooks/git/{p2}", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/hooks/{p2}", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/hooks/{p2}/tests", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/issue_config", any(dispatch))
        .route(
            "/api/v1/repos/{p0}/{p1}/issue_config/validate",
            any(dispatch),
        )
        .route("/api/v1/repos/{p0}/{p1}/issue_templates", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/issues", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/issues/comments", any(dispatch))
        .route(
            "/api/v1/repos/{p0}/{p1}/issues/comments/{p2}",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/issues/comments/{p2}/assets",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/issues/comments/{p2}/assets/{p3}",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/issues/comments/{p2}/reactions",
            any(dispatch),
        )
        .route("/api/v1/repos/{p0}/{p1}/issues/pinned", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/issues/{p2}", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/issues/{p2}/assets", any(dispatch))
        .route(
            "/api/v1/repos/{p0}/{p1}/issues/{p2}/assets/{p3}",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/issues/{p2}/assignees",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/issues/{p2}/assignees/{p3}",
            any(dispatch),
        )
        .route("/api/v1/repos/{p0}/{p1}/issues/{p2}/blocks", any(dispatch))
        .route(
            "/api/v1/repos/{p0}/{p1}/issues/{p2}/comments",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/issues/{p2}/comments/{p3}",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/issues/{p2}/deadline",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/issues/{p2}/dependencies",
            any(dispatch),
        )
        .route("/api/v1/repos/{p0}/{p1}/issues/{p2}/labels", any(dispatch))
        .route(
            "/api/v1/repos/{p0}/{p1}/issues/{p2}/labels/{p3}",
            any(dispatch),
        )
        .route("/api/v1/repos/{p0}/{p1}/issues/{p2}/lock", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/issues/{p2}/pin", any(dispatch))
        .route(
            "/api/v1/repos/{p0}/{p1}/issues/{p2}/pin/{p3}",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/issues/{p2}/reactions",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/issues/{p2}/stopwatch/delete",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/issues/{p2}/stopwatch/start",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/issues/{p2}/stopwatch/stop",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/issues/{p2}/subscriptions",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/issues/{p2}/subscriptions/check",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/issues/{p2}/subscriptions/{p3}",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/issues/{p2}/timeline",
            any(dispatch),
        )
        .route("/api/v1/repos/{p0}/{p1}/issues/{p2}/times", any(dispatch))
        .route(
            "/api/v1/repos/{p0}/{p1}/issues/{p2}/times/{p3}",
            any(dispatch),
        )
        .route("/api/v1/repos/{p0}/{p1}/keys", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/keys/tokens", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/keys/{p2}", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/labels", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/labels/{p2}", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/languages", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/licenses", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/media/{*p2}", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/merge-upstream", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/milestones", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/milestones/{p2}", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/mirror-sync", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/new_pin_allowed", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/notifications", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/projects", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/projects/{p2}", any(dispatch))
        .route(
            "/api/v1/repos/{p0}/{p1}/projects/{p2}/columns",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/projects/{p2}/columns/move",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/projects/{p2}/columns/{p3}",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/projects/{p2}/columns/{p3}/default",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/projects/{p2}/columns/{p3}/issues",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/projects/{p2}/columns/{p3}/issues/{p4}",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/projects/{p2}/issues/{p3}/move",
            any(dispatch),
        )
        .route("/api/v1/repos/{p0}/{p1}/pulls", any(dispatch))
        .route(
            "/api/v1/repos/{p0}/{p1}/pulls/comments/{p2}/resolve",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/pulls/comments/{p2}/unresolve",
            any(dispatch),
        )
        .route("/api/v1/repos/{p0}/{p1}/pulls/pinned", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/pulls/{p2}", any(dispatch))
        .route(
            "/api/v1/repos/{p0}/{p1}/pulls/{p2}/comments/{p3}/replies",
            any(dispatch),
        )
        .route("/api/v1/repos/{p0}/{p1}/pulls/{p2}/commits", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/pulls/{p2}/files", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/pulls/{p2}/merge", any(dispatch))
        .route(
            "/api/v1/repos/{p0}/{p1}/pulls/{p2}/requested_reviewers",
            any(dispatch),
        )
        .route("/api/v1/repos/{p0}/{p1}/pulls/{p2}/reviews", any(dispatch))
        .route(
            "/api/v1/repos/{p0}/{p1}/pulls/{p2}/reviews/{p3}",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/pulls/{p2}/reviews/{p3}/comments",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/pulls/{p2}/reviews/{p3}/dismissals",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/pulls/{p2}/reviews/{p3}/undismissals",
            any(dispatch),
        )
        .route("/api/v1/repos/{p0}/{p1}/pulls/{p2}/update", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/pulls/{p2}/{*p3}", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/push_mirrors", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/push_mirrors-sync", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/push_mirrors/{p2}", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/raw/{*p2}", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/releases", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/releases/latest", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/releases/tags/{p2}", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/releases/{p2}", any(dispatch))
        .route(
            "/api/v1/repos/{p0}/{p1}/releases/{p2}/assets",
            any(dispatch),
        )
        .route(
            "/api/v1/repos/{p0}/{p1}/releases/{p2}/assets/{p3}",
            any(dispatch),
        )
        .route("/api/v1/repos/{p0}/{p1}/reviewers", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/signing-key.gpg", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/signing-key.pub", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/stargazers", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/statuses/{p2}", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/subscribers", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/subscription", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/tag_protections", any(dispatch))
        .route(
            "/api/v1/repos/{p0}/{p1}/tag_protections/{p2}",
            any(dispatch),
        )
        .route("/api/v1/repos/{p0}/{p1}/tags", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/tags/{p2}", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/teams", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/teams/{p2}", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/times", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/times/{p2}", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/topics", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/topics/{p2}", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/transfer", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/transfer/accept", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/transfer/reject", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/wiki/new", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/wiki/page/{*p2}", any(dispatch))
        .route("/api/v1/repos/{p0}/{p1}/wiki/pages", any(dispatch))
        .route(
            "/api/v1/repos/{p0}/{p1}/wiki/revisions/{*p2}",
            any(dispatch),
        )
        .route("/api/v1/repositories/{p0}", any(dispatch))
        .route("/api/v1/settings/api", any(dispatch))
        .route("/api/v1/settings/attachment", any(dispatch))
        .route("/api/v1/settings/repository", any(dispatch))
        .route("/api/v1/settings/ui", any(dispatch))
        .route("/api/v1/signing-key.gpg", any(dispatch))
        .route("/api/v1/signing-key.pub", any(dispatch))
        .route("/api/v1/teams/{p0}", any(dispatch))
        .route("/api/v1/teams/{p0}/activities/feeds", any(dispatch))
        .route("/api/v1/teams/{p0}/members", any(dispatch))
        .route("/api/v1/teams/{p0}/members/{p1}", any(dispatch))
        .route("/api/v1/teams/{p0}/repos", any(dispatch))
        .route("/api/v1/teams/{p0}/repos/{p1}/{p2}", any(dispatch))
        .route("/api/v1/token", any(dispatch))
        .route("/api/v1/topics/search", any(dispatch))
        .route("/api/v1/user", any(dispatch))
        .route("/api/v1/user/actions/jobs", any(dispatch))
        .route("/api/v1/user/actions/runners", any(dispatch))
        .route(
            "/api/v1/user/actions/runners/registration-token",
            any(dispatch),
        )
        .route("/api/v1/user/actions/runners/{p0}", any(dispatch))
        .route("/api/v1/user/actions/runs", any(dispatch))
        .route("/api/v1/user/actions/secrets/{p0}", any(dispatch))
        .route("/api/v1/user/actions/variables", any(dispatch))
        .route("/api/v1/user/actions/variables/{p0}", any(dispatch))
        .route("/api/v1/user/applications/oauth2", any(dispatch))
        .route("/api/v1/user/applications/oauth2/{p0}", any(dispatch))
        .route("/api/v1/user/avatar", any(dispatch))
        .route("/api/v1/user/blocks", any(dispatch))
        .route("/api/v1/user/blocks/{p0}", any(dispatch))
        .route("/api/v1/user/emails", any(dispatch))
        .route("/api/v1/user/followers", any(dispatch))
        .route("/api/v1/user/following", any(dispatch))
        .route("/api/v1/user/following/{p0}", any(dispatch))
        .route("/api/v1/user/gpg_key_token", any(dispatch))
        .route("/api/v1/user/gpg_key_verify", any(dispatch))
        .route("/api/v1/user/gpg_keys", any(dispatch))
        .route("/api/v1/user/gpg_keys/{p0}", any(dispatch))
        .route("/api/v1/user/hooks", any(dispatch))
        .route("/api/v1/user/hooks/{p0}", any(dispatch))
        .route("/api/v1/user/keys", any(dispatch))
        .route("/api/v1/user/keys/{p0}", any(dispatch))
        .route("/api/v1/user/orgs", any(dispatch))
        .route("/api/v1/user/projects", any(dispatch))
        .route("/api/v1/user/projects/{p0}", any(dispatch))
        .route("/api/v1/user/projects/{p0}/columns", any(dispatch))
        .route("/api/v1/user/projects/{p0}/columns/move", any(dispatch))
        .route("/api/v1/user/projects/{p0}/columns/{p1}", any(dispatch))
        .route(
            "/api/v1/user/projects/{p0}/columns/{p1}/default",
            any(dispatch),
        )
        .route(
            "/api/v1/user/projects/{p0}/columns/{p1}/issues",
            any(dispatch),
        )
        .route(
            "/api/v1/user/projects/{p0}/columns/{p1}/issues/{p2}",
            any(dispatch),
        )
        .route("/api/v1/user/projects/{p0}/issues/{p1}/move", any(dispatch))
        .route("/api/v1/user/repos", any(dispatch))
        .route("/api/v1/user/settings", any(dispatch))
        .route("/api/v1/user/starred", any(dispatch))
        .route("/api/v1/user/starred/{p0}/{p1}", any(dispatch))
        .route("/api/v1/user/stopwatches", any(dispatch))
        .route("/api/v1/user/subscriptions", any(dispatch))
        .route("/api/v1/user/teams", any(dispatch))
        .route("/api/v1/user/times", any(dispatch))
        .route("/api/v1/users/search", any(dispatch))
        .route("/api/v1/users/{p0}", any(dispatch))
        .route("/api/v1/users/{p0}/activities/feeds", any(dispatch))
        .route("/api/v1/users/{p0}/followers", any(dispatch))
        .route("/api/v1/users/{p0}/following", any(dispatch))
        .route("/api/v1/users/{p0}/following/{p1}", any(dispatch))
        .route("/api/v1/users/{p0}/gpg_keys", any(dispatch))
        .route("/api/v1/users/{p0}/heatmap", any(dispatch))
        .route("/api/v1/users/{p0}/keys", any(dispatch))
        .route("/api/v1/users/{p0}/orgs", any(dispatch))
        .route("/api/v1/users/{p0}/orgs/{p1}/permissions", any(dispatch))
        .route("/api/v1/users/{p0}/projects", any(dispatch))
        .route("/api/v1/users/{p0}/repos", any(dispatch))
        .route("/api/v1/users/{p0}/starred", any(dispatch))
        .route("/api/v1/users/{p0}/subscriptions", any(dispatch))
        .route("/api/v1/users/{p0}/tokens", any(dispatch))
        .route("/api/v1/users/{p0}/tokens/{p1}", any(dispatch))
        .route("/api/v1/version", any(dispatch))
}
