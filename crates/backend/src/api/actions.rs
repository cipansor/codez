//! Actions (CI/CD) endpoints: runners, workflow definitions, runs, jobs,
//! artifacts, secrets and variables.
//!
//! Executing workflows needs a runner fleet that can actually run a job, which
//! is out of scope for the in-memory port. Everything a caller can *observe*
//! about the CI state is real, though: runners are registered and removed,
//! workflows are discovered in the repository tree, runs and jobs are recorded
//! with their status, and secrets and variables are stored per scope. A run
//! created through the API is reported by the same endpoints that list it, so
//! the surface behaves like the registry it models rather than returning a
//! fixed empty payload.

use axum::http::Method;
use axum::response::{IntoResponse, Response};

use shared::models as dto;

use crate::pagination::Pagination;
use crate::respond::{self, ApiError};
use crate::state::{
    now, short_sha, AppState, StoredArtifact, StoredRepo, StoredRunner, StoredSecret,
    StoredVariable, StoredWorkflow, StoredWorkflowJob, StoredWorkflowRun,
};

use super::ApiRequest;

/// Where an Actions request is scoped.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Scope {
    /// Instance-wide, reached through `/api/v1/admin/actions/*`.
    Instance,
    Org(String),
    User(String),
    Repo(i64, String),
}

impl Scope {
    /// The `owner` key runners are stored under for this scope.
    fn runner_owner(&self) -> String {
        match self {
            Scope::Instance => String::new(),
            Scope::Org(name) => name.clone(),
            Scope::User(login) => login.clone(),
            Scope::Repo(id, _) => format!("repo:{id}"),
        }
    }

    /// The secrets / variables scope key.
    fn store_scope(&self) -> String {
        match self {
            Scope::Instance => "instance".into(),
            Scope::Org(name) => format!("org:{name}"),
            Scope::User(login) => format!("user:{login}"),
            Scope::Repo(id, _) => format!("repo:{id}"),
        }
    }

    /// Repository ids visible in this scope.
    fn repo_ids(&self, state: &AppState) -> Vec<i64> {
        match self {
            Scope::Instance => state.repos().iter().map(|r| r.id).collect(),
            Scope::Org(name) => state
                .repos()
                .iter()
                .filter(|r| r.owner.eq_ignore_ascii_case(name))
                .map(|r| r.id)
                .collect(),
            Scope::User(login) => state
                .repos()
                .iter()
                .filter(|r| r.owner.eq_ignore_ascii_case(login))
                .map(|r| r.id)
                .collect(),
            Scope::Repo(id, _) => vec![*id],
        }
    }
}

/// Route a `/api/v1/{admin,orgs/{org},user,repos/{owner}/{repo}}/actions/*`
/// request, whatever scope it arrived through.
pub fn handle(state: &AppState, req: &ApiRequest) -> Response {
    let Some((scope, tail)) = resolve(state, req) else {
        return unimplemented(req);
    };
    let tail: Vec<&str> = tail.iter().map(String::as_str).collect();

    match tail.as_slice() {
        ["runners"] => runners(state, req, &scope),
        ["runners", "registration-token"] => registration_token(state, req),
        ["runners", id] => runner_detail(state, req, &scope, id),
        ["runs"] => runs(state, req, &scope, None),
        ["runs", id] => run_detail(state, req, &scope, id),
        ["runs", id, "jobs"] => run_jobs(state, req, &scope, id),
        ["runs", id, "logs"] => logs(state, req, &scope, id),
        ["runs", id, "artifacts"] => run_artifacts(state, req, &scope, id),
        ["runs", id, "cancel"] | ["runs", id, "force-cancel"] => cancel_run(state, req, &scope, id),
        ["runs", id, "approve"] => approve_run(state, req, &scope, id),
        ["runs", id, "rerun"] | ["runs", id, "rerun-failed-jobs"] => {
            rerun_run(state, req, &scope, id)
        }
        ["runs", id, "attempts", attempt] => run_attempt(state, req, &scope, id, attempt),
        ["runs", id, "attempts", attempt, "jobs"] => {
            run_attempt_jobs(state, req, &scope, id, attempt)
        }
        ["runs", id, "jobs", job_id, "rerun"] => rerun_job(state, req, &scope, id, job_id),
        ["jobs"] => jobs(state, req, &scope),
        ["jobs", id] => job_detail(state, req, &scope, id),
        ["jobs", id, "logs"] => job_logs(state, req, &scope, id),
        ["artifacts"] => artifacts(state, req, &scope),
        ["artifacts", id] => artifact_detail(state, req, &scope, id),
        ["artifacts", id, "zip"] => artifact_zip(state, req, &scope, id),
        ["tasks"] => tasks(state, req, &scope),
        ["workflows"] => workflows(state, req, &scope),
        ["workflows", id] => workflow_detail(state, req, &scope, id),
        ["workflows", id, "runs"] => workflow_runs(state, req, &scope, id),
        ["workflows", id, "enable"] => workflow_state(state, req, &scope, id, "active"),
        ["workflows", id, "disable"] => workflow_state(state, req, &scope, id, "disabled_manually"),
        ["workflows", id, "dispatches"] => dispatch_workflow(state, req, &scope, id),
        ["secrets"] => secrets(state, req, &scope),
        ["secrets", name] => secret_detail(state, req, &scope, name),
        ["variables"] => variables(state, req, &scope),
        ["variables", name] => variable_detail(state, req, &scope, name),
        _ => unimplemented(req),
    }
}

/// Work out the scope and the path segments after the `actions` marker.
fn resolve(state: &AppState, req: &ApiRequest) -> Option<(Scope, Vec<String>)> {
    let path = req.path.as_str();
    if let Some(rest) = path.strip_prefix("/api/v1/admin/actions/") {
        return Some((Scope::Instance, segments(rest)));
    }
    if let Some(rest) = path.strip_prefix("/api/v1/user/actions/") {
        let login = super::users::current_user(state, req)?.login;
        return Some((Scope::User(login), segments(rest)));
    }
    if path.starts_with("/api/v1/orgs/") {
        let org = req.param("org")?;
        let marker = format!("/api/v1/orgs/{org}/actions/");
        return Some((Scope::Org(org), segments(path.strip_prefix(&marker)?)));
    }
    if path.starts_with("/api/v1/repos/") {
        let (owner, name) = req.repo_spec()?;
        let repo = state.repo(&owner, &name)?;
        let marker = format!("/api/v1/repos/{owner}/{name}/actions/");
        return Some((
            Scope::Repo(repo.id, repo.full_name()),
            segments(path.strip_prefix(&marker)?),
        ));
    }
    None
}

/// Split a relative path into its non-empty segments.
fn segments(rest: &str) -> Vec<String> {
    rest.split('/')
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

// --- Runners ---------------------------------------------------------------

/// `GET|POST /…/actions/runners`.
fn runners(state: &AppState, req: &ApiRequest, scope: &Scope) -> Response {
    let owner = scope.runner_owner();
    match req.method {
        Method::GET => {
            let disabled = req.query_param("disabled");
            let items: Vec<dto::ActionRunner> = state
                .scoped_runners(&owner)
                .into_iter()
                .filter(|r| {
                    disabled
                        .as_deref()
                        .is_none_or(|flag| r.disabled == (flag == "true"))
                })
                .map(|r| r.to_dto())
                .collect();
            respond::ok(dto::ActionRunnersResponse {
                total_count: Some(items.len() as i64),
                runners: Some(items),
            })
        }
        Method::POST => {
            let name = req
                .body_str("name")
                .unwrap_or_else(|| format!("runner-{}", state.ids.last() + 1));
            let labels = req
                .body_str_vec("labels")
                .unwrap_or_else(|| vec!["ubuntu-latest".into()]);
            let runner = StoredRunner {
                id: state.next_id(),
                owner,
                name,
                labels,
                busy: false,
                disabled: false,
                ephemeral: req.body_bool("ephemeral").unwrap_or(false),
                status: "online".into(),
            };
            let dto = runner.to_dto();
            state.runners_mut().push(runner);
            respond::created(dto)
        }
        _ => unimplemented(req),
    }
}

/// `GET|PATCH|DELETE /…/actions/runners/{runner_id}`.
fn runner_detail(state: &AppState, req: &ApiRequest, scope: &Scope, raw_id: &str) -> Response {
    let id: i64 = raw_id.parse().unwrap_or_default();
    let owner = scope.runner_owner();
    if !state
        .runners()
        .iter()
        .any(|r| r.id == id && r.owner == owner)
    {
        return ApiError::not_found("runner not found").into_response();
    }
    match req.method {
        Method::GET => match find_runner(state, id) {
            Some(runner) => respond::ok(runner.to_dto()),
            None => ApiError::not_found("runner not found").into_response(),
        },
        Method::PATCH => {
            if let Some(runner) = state.runners_mut().iter_mut().find(|r| r.id == id) {
                if let Some(name) = req.body_str("name") {
                    runner.name = name;
                }
                if let Some(labels) = req.body_str_vec("labels") {
                    runner.labels = labels;
                }
                if let Some(disabled) = req.body_bool("disabled") {
                    runner.disabled = disabled;
                }
                if let Some(ephemeral) = req.body_bool("ephemeral") {
                    runner.ephemeral = ephemeral;
                }
            }
            match find_runner(state, id) {
                Some(runner) => respond::ok(runner.to_dto()),
                None => ApiError::not_found("runner not found").into_response(),
            }
        }
        Method::DELETE => {
            state.runners_mut().retain(|r| r.id != id);
            respond::no_content()
        }
        _ => unimplemented(req),
    }
}

/// `POST /…/actions/runners/registration-token`.
fn registration_token(state: &AppState, req: &ApiRequest) -> Response {
    let scope = resolve(state, req).map(|(scope, _)| scope);
    let key = scope.map(|s| s.store_scope()).unwrap_or_default();
    let token = short_sha(&format!("runner-token:{key}:{}", now()));
    respond::created(serde_json::json!({ "token": token }))
}

/// `POST /…/actions/runs/{run}/approve` — the run leaves the approval gate.
fn approve_run(state: &AppState, _req: &ApiRequest, scope: &Scope, id: &str) -> Response {
    transition_run(state, scope, id, "queued", "queued", None)
}

/// `POST /…/actions/runs/{run}/cancel` and `force-cancel`.
fn cancel_run(state: &AppState, req: &ApiRequest, scope: &Scope, id: &str) -> Response {
    let _ = req;
    transition_run(
        state,
        scope,
        id,
        "cancelled",
        "cancelled",
        Some("cancelled"),
    )
}

/// `POST /…/actions/runs/{run}/rerun` and `rerun-failed-jobs`.
fn rerun_run(state: &AppState, req: &ApiRequest, scope: &Scope, id: &str) -> Response {
    let _ = req;
    transition_run(state, scope, id, "queued", "queued", None)
}

/// `POST /…/actions/runs/{run}/jobs/{job_id}/rerun`.
fn rerun_job(
    state: &AppState,
    req: &ApiRequest,
    scope: &Scope,
    _run_id: &str,
    job_id: &str,
) -> Response {
    let _ = req;
    let id: i64 = job_id.parse().unwrap_or_default();
    if let Some(job) = state.workflow_jobs_mut().iter_mut().find(|j| j.id == id) {
        job.status = "queued".into();
        job.conclusion = String::new();
    } else {
        return ApiError::not_found("job not found").into_response();
    }
    match find_repo_for_scope(state, scope) {
        Some(repo) => {
            let dto = state
                .workflow_jobs()
                .iter()
                .find(|j| j.id == id)
                .map(|j| j.to_dto(&repo));
            match dto {
                Some(dto) => respond::created(dto),
                None => ApiError::not_found("job not found").into_response(),
            }
        }
        None => respond::no_content(),
    }
}

/// Apply a status change to a run and report it.
fn transition_run(
    state: &AppState,
    scope: &Scope,
    raw_id: &str,
    status: &str,
    _conclusion: &str,
    set_conclusion: Option<&str>,
) -> Response {
    let id: i64 = raw_id.parse().unwrap_or_default();
    let repo_ids = scope.repo_ids(state);
    if !state
        .workflow_runs()
        .iter()
        .any(|r| r.id == id && repo_ids.contains(&r.repo_id))
    {
        return ApiError::not_found("run not found").into_response();
    }
    if let Some(run) = state.workflow_runs_mut().iter_mut().find(|r| r.id == id) {
        run.status = status.into();
        run.updated = now();
        if let Some(conclusion) = set_conclusion {
            run.conclusion = conclusion.into();
        }
    }
    let run = state.workflow_runs().iter().find(|r| r.id == id).cloned();
    match run.and_then(|run| state.repo_by_id(run.repo_id).map(|repo| (run, repo))) {
        Some((run, repo)) => respond::ok(run.to_dto(&repo, actor_dto(state, &run.actor))),
        None => ApiError::not_found("run not found").into_response(),
    }
}

// --- Runs ------------------------------------------------------------------

/// `GET /…/actions/runs` and `GET /…/workflows/{id}/runs`.
fn runs(state: &AppState, req: &ApiRequest, scope: &Scope, workflow: Option<&str>) -> Response {
    let repo_ids = scope.repo_ids(state);
    let status = req.query_param("status");
    let branch = req.query_param("branch");
    let event = req.query_param("event");
    let actor = req.query_param("actor");

    let mut items: Vec<StoredWorkflowRun> = state
        .workflow_runs()
        .iter()
        .filter(|r| repo_ids.contains(&r.repo_id))
        .filter(|r| workflow.is_none_or(|path| r.workflow_path == path))
        .filter(|r| status.as_deref().is_none_or(|s| r.status == s))
        .filter(|r| branch.as_deref().is_none_or(|b| r.head_branch == b))
        .filter(|r| event.as_deref().is_none_or(|e| r.event == e))
        .filter(|r| actor.as_deref().is_none_or(|a| r.actor == a))
        .cloned()
        .collect();
    items.sort_by_key(|r| std::cmp::Reverse(r.id));

    let total = items.len();
    let pagination = pagination(req);
    let dtos: Vec<dto::ActionWorkflowRun> = pagination
        .apply(&items)
        .into_iter()
        .filter_map(|run| {
            state
                .repo_by_id(run.repo_id)
                .map(|repo| run.to_dto(&repo, actor_dto(state, &run.actor)))
        })
        .collect();

    respond::paged(
        dto::ActionWorkflowRunsResponse {
            total_count: Some(total as i64),
            workflow_runs: Some(dtos),
        },
        &pagination,
        total,
    )
}

/// `GET|DELETE /…/actions/runs/{run}`.
fn run_detail(state: &AppState, req: &ApiRequest, scope: &Scope, raw_id: &str) -> Response {
    let id: i64 = raw_id.parse().unwrap_or_default();
    let repo_ids = scope.repo_ids(state);
    let Some(run) = state
        .workflow_runs()
        .iter()
        .find(|r| r.id == id && repo_ids.contains(&r.repo_id))
        .cloned()
    else {
        return ApiError::not_found("run not found").into_response();
    };

    match req.method {
        Method::DELETE => {
            state.workflow_runs_mut().retain(|r| r.id != id);
            state.workflow_jobs_mut().retain(|j| j.run_id != id);
            state.artifacts_mut().retain(|a| a.run_id != id);
            respond::no_content()
        }
        _ => match state.repo_by_id(run.repo_id) {
            Some(repo) => respond::ok(run.to_dto(&repo, actor_dto(state, &run.actor))),
            None => ApiError::not_found("repository does not exist").into_response(),
        },
    }
}

/// `GET /…/actions/runs/{run}/attempts/{attempt}`.
fn run_attempt(
    state: &AppState,
    req: &ApiRequest,
    scope: &Scope,
    raw_id: &str,
    attempt: &str,
) -> Response {
    let id: i64 = raw_id.parse().unwrap_or_default();
    let attempt: i64 = attempt.parse().unwrap_or(1);
    let repo_ids = scope.repo_ids(state);
    let Some(run) = state
        .workflow_runs()
        .iter()
        .find(|r| r.id == id && repo_ids.contains(&r.repo_id))
        .cloned()
    else {
        return ApiError::not_found("run not found").into_response();
    };
    let _ = req;
    match state.repo_by_id(run.repo_id) {
        Some(repo) => {
            let mut dto = run.to_dto(&repo, actor_dto(state, &run.actor));
            dto.run_attempt = Some(attempt);
            respond::ok(dto)
        }
        None => ApiError::not_found("repository does not exist").into_response(),
    }
}

/// `GET /…/actions/runs/{run}/attempts/{attempt}/jobs`.
fn run_attempt_jobs(
    state: &AppState,
    req: &ApiRequest,
    scope: &Scope,
    raw_id: &str,
    attempt: &str,
) -> Response {
    let _ = attempt;
    run_jobs(state, req, scope, raw_id)
}

/// `GET /…/actions/runs/{run}/jobs`.
fn run_jobs(state: &AppState, req: &ApiRequest, scope: &Scope, raw_id: &str) -> Response {
    let id: i64 = raw_id.parse().unwrap_or_default();
    let repo_ids = scope.repo_ids(state);
    if !state
        .workflow_runs()
        .iter()
        .any(|r| r.id == id && repo_ids.contains(&r.repo_id))
    {
        return ApiError::not_found("run not found").into_response();
    }
    let _ = req;
    jobs_for(state, &repo_ids, Some(id))
}

/// `GET /…/actions/runs/{run}/logs` — the aggregated run log.
fn logs(state: &AppState, req: &ApiRequest, scope: &Scope, raw_id: &str) -> Response {
    let _ = req;
    let id: i64 = raw_id.parse().unwrap_or_default();
    let repo_ids = scope.repo_ids(state);
    if !state
        .workflow_runs()
        .iter()
        .any(|r| r.id == id && repo_ids.contains(&r.repo_id))
    {
        return ApiError::not_found("run not found").into_response();
    }
    respond::ok(run_log(state, id))
}

/// A run's log, assembled from its jobs.
fn run_log(state: &AppState, run_id: i64) -> String {
    let mut out = String::new();
    for job in state.workflow_jobs().iter().filter(|j| j.run_id == run_id) {
        out.push_str(&format!("== job: {} ({})\n", job.name, job.status));
        for (name, status, conclusion) in &job.steps {
            out.push_str(&format!("  - {name}: {status} {conclusion}\n"));
        }
    }
    out
}

/// `GET /…/actions/runs/{run}/artifacts`.
fn run_artifacts(state: &AppState, req: &ApiRequest, scope: &Scope, raw_id: &str) -> Response {
    let _ = req;
    let id: i64 = raw_id.parse().unwrap_or_default();
    let repo_ids = scope.repo_ids(state);
    if !state
        .workflow_runs()
        .iter()
        .any(|r| r.id == id && repo_ids.contains(&r.repo_id))
    {
        return ApiError::not_found("run not found").into_response();
    }
    artifacts_for(state, &repo_ids, Some(id))
}

// --- Jobs ------------------------------------------------------------------

/// `GET /…/actions/jobs` — every job visible in the scope.
fn jobs(state: &AppState, req: &ApiRequest, scope: &Scope) -> Response {
    let repo_ids = scope.repo_ids(state);
    let status = req.query_param("status");
    let mut items: Vec<StoredWorkflowJob> = state
        .workflow_jobs()
        .iter()
        .filter(|j| repo_ids.contains(&j.repo_id))
        .filter(|j| status.as_deref().is_none_or(|s| j.status == s))
        .cloned()
        .collect();
    items.sort_by_key(|j| std::cmp::Reverse(j.id));

    let total = items.len();
    let pagination = pagination(req);
    let dtos: Vec<dto::ActionWorkflowJob> = pagination
        .apply(&items)
        .into_iter()
        .filter_map(|job| state.repo_by_id(job.repo_id).map(|repo| job.to_dto(&repo)))
        .collect();

    respond::paged(
        dto::ActionWorkflowJobsResponse {
            total_count: Some(total as i64),
            jobs: Some(dtos),
        },
        &pagination,
        total,
    )
}

/// Jobs of one run, newest first.
fn jobs_for(state: &AppState, repo_ids: &[i64], run_id: Option<i64>) -> Response {
    let mut items: Vec<StoredWorkflowJob> = state
        .workflow_jobs()
        .iter()
        .filter(|j| repo_ids.contains(&j.repo_id))
        .filter(|j| run_id.is_none_or(|id| j.run_id == id))
        .cloned()
        .collect();
    items.sort_by_key(|j| std::cmp::Reverse(j.id));
    let total = items.len();
    let dtos: Vec<dto::ActionWorkflowJob> = items
        .into_iter()
        .filter_map(|job| state.repo_by_id(job.repo_id).map(|repo| job.to_dto(&repo)))
        .collect();
    respond::ok(dto::ActionWorkflowJobsResponse {
        total_count: Some(total as i64),
        jobs: Some(dtos),
    })
}

/// `GET /…/actions/jobs/{job_id}`.
fn job_detail(state: &AppState, req: &ApiRequest, scope: &Scope, raw_id: &str) -> Response {
    let _ = req;
    let id: i64 = raw_id.parse().unwrap_or_default();
    let repo_ids = scope.repo_ids(state);
    let Some(job) = state
        .workflow_jobs()
        .iter()
        .find(|j| j.id == id && repo_ids.contains(&j.repo_id))
        .cloned()
    else {
        return ApiError::not_found("job not found").into_response();
    };
    match state.repo_by_id(job.repo_id) {
        Some(repo) => respond::ok(job.to_dto(&repo)),
        None => ApiError::not_found("repository does not exist").into_response(),
    }
}

/// `GET /…/actions/jobs/{job_id}/logs`.
fn job_logs(state: &AppState, req: &ApiRequest, scope: &Scope, raw_id: &str) -> Response {
    let _ = req;
    let id: i64 = raw_id.parse().unwrap_or_default();
    let repo_ids = scope.repo_ids(state);
    let Some(job) = state
        .workflow_jobs()
        .iter()
        .find(|j| j.id == id && repo_ids.contains(&j.repo_id))
        .cloned()
    else {
        return ApiError::not_found("job not found").into_response();
    };
    let mut out = format!("== job: {} ({})\n", job.name, job.status);
    for (name, status, conclusion) in &job.steps {
        out.push_str(&format!("  - {name}: {status} {conclusion}\n"));
    }
    respond::ok(out)
}

// --- Artifacts -------------------------------------------------------------

/// `GET /…/actions/artifacts`.
fn artifacts(state: &AppState, req: &ApiRequest, scope: &Scope) -> Response {
    let repo_ids = scope.repo_ids(state);
    let _ = req;
    artifacts_for(state, &repo_ids, None)
}

/// Artifacts in the given repositories, optionally of one run.
fn artifacts_for(state: &AppState, repo_ids: &[i64], run_id: Option<i64>) -> Response {
    let mut items: Vec<StoredArtifact> = state
        .artifacts()
        .iter()
        .filter(|a| repo_ids.contains(&a.repo_id))
        .filter(|a| run_id.is_none_or(|id| a.run_id == id))
        .cloned()
        .collect();
    items.sort_by_key(|a| std::cmp::Reverse(a.id));
    let total = items.len();
    let dtos: Vec<dto::ActionArtifact> = items
        .into_iter()
        .filter_map(|a| state.repo_by_id(a.repo_id).map(|repo| a.to_dto(&repo)))
        .collect();
    respond::ok(dto::ActionArtifactsResponse {
        total_count: Some(total as i64),
        artifacts: Some(dtos),
    })
}

/// `GET|DELETE /…/actions/artifacts/{artifact_id}`.
fn artifact_detail(state: &AppState, req: &ApiRequest, scope: &Scope, raw_id: &str) -> Response {
    let id: i64 = raw_id.parse().unwrap_or_default();
    let repo_ids = scope.repo_ids(state);
    let Some(artifact) = state
        .artifacts()
        .iter()
        .find(|a| a.id == id && repo_ids.contains(&a.repo_id))
        .cloned()
    else {
        return ApiError::not_found("artifact not found").into_response();
    };
    match req.method {
        Method::DELETE => {
            state.artifacts_mut().retain(|a| a.id != id);
            respond::no_content()
        }
        _ => match state.repo_by_id(artifact.repo_id) {
            Some(repo) => respond::ok(artifact.to_dto(&repo)),
            None => ApiError::not_found("repository does not exist").into_response(),
        },
    }
}

/// `GET /…/actions/artifacts/{artifact_id}/zip` — a placeholder archive.
fn artifact_zip(state: &AppState, req: &ApiRequest, scope: &Scope, raw_id: &str) -> Response {
    let _ = req;
    let id: i64 = raw_id.parse().unwrap_or_default();
    let repo_ids = scope.repo_ids(state);
    if !state
        .artifacts()
        .iter()
        .any(|a| a.id == id && repo_ids.contains(&a.repo_id))
    {
        return ApiError::not_found("artifact not found").into_response();
    }
    // The artifact bytes are not stored; Gitea streams a zip, so a client that
    // only wants a download URL gets a well-formed (if empty) archive marker.
    respond::ok(serde_json::json!({
        "message": "artifact contents are not stored by this instance",
        "artifact_id": id,
    }))
}

// --- Tasks -----------------------------------------------------------------

/// `GET /…/actions/tasks` — the instance task queue view of recent runs.
fn tasks(state: &AppState, req: &ApiRequest, scope: &Scope) -> Response {
    let _ = req;
    let repo_ids = scope.repo_ids(state);
    let items: Vec<dto::ActionTask> = state
        .workflow_runs()
        .iter()
        .filter(|r| repo_ids.contains(&r.repo_id))
        .map(|run| dto::ActionTask {
            id: Some(run.id),
            name: Some(run.workflow_path.clone()),
            display_title: Some(run.display_title.clone()),
            event: Some(run.event.clone()),
            head_branch: Some(run.head_branch.clone()),
            head_sha: Some(run.head_sha.clone()),
            run_number: Some(run.run_number),
            status: Some(run.status.clone()),
            created_at: Some(run.created.clone()),
            updated_at: Some(run.updated.clone()),
            run_started_at: Some(run.created.clone()),
            ..Default::default()
        })
        .collect();
    let total = items.len();
    respond::ok(dto::ActionTaskResponse {
        total_count: Some(total as i64),
        workflow_runs: Some(items),
    })
}

// --- Workflows -------------------------------------------------------------

/// `GET /…/actions/workflows`.
fn workflows(state: &AppState, req: &ApiRequest, scope: &Scope) -> Response {
    let _ = req;
    let repo_ids = scope.repo_ids(state);
    let items: Vec<dto::ActionWorkflow> = state
        .workflows()
        .iter()
        .filter(|w| repo_ids.contains(&w.repo_id))
        .filter_map(|w| state.repo_by_id(w.repo_id).map(|repo| w.to_dto(&repo)))
        .collect();
    let total = items.len();
    respond::ok(dto::ActionWorkflowResponse {
        total_count: Some(total as i64),
        workflows: Some(items),
    })
}

/// `GET /…/actions/workflows/{workflow_id}`.
fn workflow_detail(state: &AppState, req: &ApiRequest, scope: &Scope, id: &str) -> Response {
    let _ = req;
    let repo_ids = scope.repo_ids(state);
    match find_workflow(state, &repo_ids, id) {
        Some((workflow, repo)) => respond::ok(workflow.to_dto(&repo)),
        None => ApiError::not_found("workflow not found").into_response(),
    }
}

/// `GET /…/actions/workflows/{workflow_id}/runs`.
fn workflow_runs(state: &AppState, req: &ApiRequest, scope: &Scope, id: &str) -> Response {
    let repo_ids = scope.repo_ids(state);
    match find_workflow(state, &repo_ids, id) {
        Some((workflow, _)) => runs(state, req, scope, Some(&workflow.path)),
        None => ApiError::not_found("workflow not found").into_response(),
    }
}

/// `PUT /…/actions/workflows/{workflow_id}/enable|disable`.
fn workflow_state(
    state: &AppState,
    req: &ApiRequest,
    scope: &Scope,
    id: &str,
    new_state: &str,
) -> Response {
    let _ = req;
    let repo_ids = scope.repo_ids(state);
    let Some((workflow, _)) = find_workflow(state, &repo_ids, id) else {
        return ApiError::not_found("workflow not found").into_response();
    };
    if let Some(stored) = state
        .workflows_mut()
        .iter_mut()
        .find(|w| w.id == workflow.id)
    {
        stored.state = new_state.into();
    }
    respond::no_content()
}

/// `POST /…/actions/workflows/{workflow_id}/dispatches` — queue a run.
fn dispatch_workflow(state: &AppState, req: &ApiRequest, scope: &Scope, id: &str) -> Response {
    let repo_ids = scope.repo_ids(state);
    let Some((workflow, repo)) = find_workflow(state, &repo_ids, id) else {
        return ApiError::not_found("workflow not found").into_response();
    };

    let branch = req
        .body_str("ref")
        .unwrap_or_else(|| repo.default_branch.clone());
    let next_number = state
        .workflow_runs()
        .iter()
        .filter(|r| r.repo_id == repo.id)
        .map(|r| r.run_number)
        .max()
        .unwrap_or(0)
        + 1;
    let actor = super::users::current_user(state, req)
        .map(|u| u.login)
        .unwrap_or_default();
    let created = now();

    let run = StoredWorkflowRun {
        id: state.next_id(),
        repo_id: repo.id,
        workflow_path: workflow.path.clone(),
        run_number: next_number,
        run_attempt: 1,
        display_title: workflow.name.clone(),
        event: "workflow_dispatch".into(),
        head_branch: branch,
        head_sha: short_sha(&format!("dispatch:{}:{created}", workflow.path)),
        status: "queued".into(),
        conclusion: String::new(),
        actor,
        created: created.clone(),
        updated: created,
    };
    let run_id = run.id;
    state.workflow_runs_mut().push(run);

    // The dispatch creates the job the runner would pick up.
    let job = StoredWorkflowJob {
        id: state.next_id(),
        run_id,
        repo_id: repo.id,
        name: workflow.name.clone(),
        status: "queued".into(),
        conclusion: String::new(),
        runner_id: 0,
        runner_name: String::new(),
        started: now(),
        completed: String::new(),
        steps: vec![("Set up job".into(), "queued".into(), String::new())],
    };
    state.workflow_jobs_mut().push(job);

    respond::created(serde_json::json!({
        "id": run_id,
        "run_number": next_number,
        "status": "queued",
    }))
}

/// Find a workflow by numeric id or by its path.
fn find_workflow(
    state: &AppState,
    repo_ids: &[i64],
    id: &str,
) -> Option<(StoredWorkflow, StoredRepo)> {
    let numeric: Option<i64> = id.parse().ok();
    let workflow = state
        .workflows()
        .iter()
        .find(|w| {
            repo_ids.contains(&w.repo_id)
                && (numeric == Some(w.id) || w.path == id || w.file_name() == id)
        })
        .cloned()?;
    let repo = state.repo_by_id(workflow.repo_id)?;
    Some((workflow.clone(), repo))
}

// --- Secrets and variables -------------------------------------------------

/// `GET /…/actions/secrets`.
fn secrets(state: &AppState, req: &ApiRequest, scope: &Scope) -> Response {
    let _ = req;
    let items: Vec<dto::Secret> = state
        .scoped_secrets(&scope.store_scope())
        .into_iter()
        .map(|s| s.to_dto())
        .collect();
    respond::ok(items)
}

/// `PUT|DELETE /…/actions/secrets/{secretname}`.
fn secret_detail(state: &AppState, req: &ApiRequest, scope: &Scope, name: &str) -> Response {
    let key = scope.store_scope();
    match req.method {
        Method::PUT => {
            let description = req.body_str("description").unwrap_or_default();
            let mut secrets = state.secrets_mut();
            match secrets
                .iter_mut()
                .find(|s| s.scope == key && s.name == name)
            {
                Some(existing) => existing.description = description,
                None => secrets.push(StoredSecret {
                    scope: key,
                    name: name.into(),
                    description,
                    created: now(),
                }),
            }
            respond::no_content()
        }
        Method::DELETE => {
            state
                .secrets_mut()
                .retain(|s| !(s.scope == key && s.name == name));
            respond::no_content()
        }
        _ => unimplemented(req),
    }
}

/// `GET /…/actions/variables`.
fn variables(state: &AppState, req: &ApiRequest, scope: &Scope) -> Response {
    let _ = req;
    let items: Vec<dto::ActionVariable> = state
        .scoped_variables(&scope.store_scope())
        .into_iter()
        .map(|v| v.to_dto())
        .collect();
    respond::ok(items)
}

/// `GET|POST|PUT|DELETE /…/actions/variables/{variablename}`.
fn variable_detail(state: &AppState, req: &ApiRequest, scope: &Scope, name: &str) -> Response {
    let key = scope.store_scope();
    match req.method {
        Method::GET => match state
            .variables()
            .iter()
            .find(|v| v.scope == key && v.name == name)
        {
            Some(variable) => respond::ok(variable.to_dto()),
            None => ApiError::not_found("variable not found").into_response(),
        },
        Method::POST | Method::PUT => {
            let value = req.body_str("value").unwrap_or_default();
            let description = req.body_str("description").unwrap_or_default();
            let mut variables = state.variables_mut();
            match variables
                .iter_mut()
                .find(|v| v.scope == key && v.name == name)
            {
                Some(existing) => {
                    existing.value = value;
                    existing.description = description;
                }
                None => variables.push(StoredVariable {
                    scope: key,
                    name: name.into(),
                    value,
                    description,
                }),
            }
            respond::no_content()
        }
        Method::DELETE => {
            state
                .variables_mut()
                .retain(|v| !(v.scope == key && v.name == name));
            respond::no_content()
        }
        _ => unimplemented(req),
    }
}

// --- Shared helpers --------------------------------------------------------

/// A runner by id, if it still exists.
fn find_runner(state: &AppState, id: i64) -> Option<StoredRunner> {
    state.runners().iter().find(|r| r.id == id).cloned()
}

/// The repository a single-repository scope points at.
fn find_repo_for_scope(state: &AppState, scope: &Scope) -> Option<StoredRepo> {
    match scope {
        Scope::Repo(id, _) => state.repo_by_id(*id),
        _ => None,
    }
}

/// Render a run's actor as a `User`, when the login still resolves.
fn actor_dto(state: &AppState, login: &str) -> Option<dto::User> {
    if login.is_empty() {
        return None;
    }
    state.user(login).map(|u| u.to_dto())
}

/// Documented fallback for operations this module does not implement yet.
fn unimplemented(req: &ApiRequest) -> Response {
    super::dispatch::fallback(req)
}

/// The pagination requested by the caller.
fn pagination(req: &ApiRequest) -> Pagination {
    Pagination {
        page: req.query_i64("page"),
        limit: req.query_i64("limit"),
    }
}
