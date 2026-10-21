//! Actions (CI/CD) pages: repository workflows and their runs.
//!
//! Workflows are discovered from the repository's workflow directory and runs
//! are recorded by the backend, so these pages list real state. A workflow can
//! be dispatched from here, which queues the run the runner fleet would pick
//! up; executing the job itself still needs an actual runner.

use crate::api::{get, local_resource, post_json, spawn, WRITE_ERROR};
use crate::components::{FormError, Loading, RepoNav};
use leptos::prelude::*;
use leptos_router::hooks::*;
use shared::{ActionWorkflow, ActionWorkflowRun};

/// The `ActionsListWorkflowRuns` document: a page of workflow runs.
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct WorkflowRunsPage {
    #[serde(default)]
    pub workflow_runs: Vec<ActionWorkflowRun>,
}

/// The `ActionsListWorkflows` document: the workflows of one repository.
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct WorkflowsPage {
    #[serde(default)]
    pub workflows: Vec<ActionWorkflow>,
}

/// `/repos/:owner/:repo/actions` — the workflows defined in the repository.
#[component]
pub fn ActionsList() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let repo = move || params.with(|p| p.get("repo").unwrap_or_default());

    // Bumped after a dispatch so the run list refreshes without a reload.
    let (refresh, set_refresh) = signal(0u32);
    let (error, set_error) = signal(Option::<String>::None);

    let workflows = local_resource(
        move || (owner(), repo()),
        |(o, r)| async move {
            get::<WorkflowsPage>(&format!("/api/v1/repos/{o}/{r}/actions/workflows"))
                .await
                .workflows
        },
    );

    let runs = local_resource(
        move || (owner(), repo(), refresh.get()),
        |(o, r, _)| async move {
            get::<WorkflowRunsPage>(&format!("/api/v1/repos/{o}/{r}/actions/runs"))
                .await
                .workflow_runs
        },
    );

    let dispatch = move |id: String| {
        let o = owner();
        let r = repo();
        spawn(async move {
            let url = format!("/api/v1/repos/{o}/{r}/actions/workflows/{id}/dispatches");
            if post_json(&url, &serde_json::json!({})).await {
                set_error.set(None);
                set_refresh.update(|n| *n += 1);
            } else {
                set_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    view! {
        <div class="actions-list">
            <RepoNav/>
            <h3>"Workflows"</h3>
            {move || error.get().map(|msg| view! { <FormError message=msg/> })}
            <p class="text-muted">
                "Workflows are read from the repository's "
                <code>".gitea/workflows"</code>
                " directory. Dispatching one queues a run; executing the job itself "
                "needs a runner to pick it up."
            </p>
            <ul class="item-list panel">
                <Suspense fallback=|| view! { <Loading label="Loading workflows…"/> }>
                    {move || workflows.get().map(|list| {
                        if list.is_empty() {
                            view! { <li class="text-muted">"No workflows defined."</li> }.into_any()
                        } else {
                            view! {
                                <For each=move || list.clone() key=|w| w.id.clone().unwrap_or_default() children=move |w| {
                                    let name = w.name.clone().or_else(|| w.id.clone()).unwrap_or_default();
                                    let path = w.path.clone().unwrap_or_default();
                                    let state = w.state.clone().unwrap_or_default();
                                    let dispatch_id = w.id.clone().unwrap_or_default();
                                    view! {
                                        <li class="flex-between">
                                            <span><strong>{name}</strong> <code>{path}</code></span>
                                            <span class="run-actions">
                                                <span class="label">{state}</span>
                                                <button class="btn-sm" type="button"
                                                    on:click=move |_| dispatch(dispatch_id.clone())>
                                                    "Dispatch"
                                                </button>
                                            </span>
                                        </li>
                                    }
                                }/>
                            }.into_any()
                        }
                    })}
                </Suspense>
            </ul>

            <h3 class="mt-3">"Recent runs"</h3>
            <ul class="runs-list item-list panel">
                <Suspense fallback=|| view! { <Loading label="Loading runs…"/> }>
                    {move || runs.get().map(|list| {
                        if list.is_empty() {
                            view! { <li class="text-muted">"No workflow runs."</li> }.into_any()
                        } else {
                            view! {
                                <For each=move || list.clone() key=|r| r.id.unwrap_or(0) children=move |run| {
                                    let title = run.display_title.clone().unwrap_or_default();
                                    let status = run.status.clone().unwrap_or_default();
                                    let conclusion = run.conclusion.clone().unwrap_or_default();
                                    view! {
                                        <li>
                                            <span class="run-status">{status}</span>
                                            <strong>{title}</strong>
                                            <span class="text-small text-muted">" " {conclusion}</span>
                                        </li>
                                    }
                                }/>
                            }.into_any()
                        }
                    })}
                </Suspense>
            </ul>
        </div>
    }
}

/// `/repos/:owner/:repo/actions/workflows/:id` — the runs of one workflow.
#[component]
pub fn WorkflowRunsList() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let repo = move || params.with(|p| p.get("repo").unwrap_or_default());
    let workflow = move || params.with(|p| p.get("id").unwrap_or_default());

    let runs = local_resource(
        move || (owner(), repo(), workflow()),
        |(o, r, w)| async move {
            get::<WorkflowRunsPage>(&format!("/api/v1/repos/{o}/{r}/actions/workflows/{w}/runs"))
                .await
                .workflow_runs
        },
    );

    view! {
        <div class="workflow-runs">
            <RepoNav/>
            <div class="page-header">
                <h3>"Runs for " <code>{workflow}</code></h3>
            </div>
            {move || runs.get().is_some_and(|r| r.is_empty()).then(|| view! {
                <FormError message="No runs for this workflow yet. Dispatch it from the repository Actions page.".to_string()/>
            })}
            <ul class="runs-list item-list panel">
                <Suspense fallback=|| view! { <Loading label="Loading runs…"/> }>
                    {move || runs.get().map(|list| {
                        if list.is_empty() {
                            view! { <li class="text-muted">"No runs for this workflow."</li> }.into_any()
                        } else {
                            view! {
                                <For each=move || list.clone() key=|r| r.id.unwrap_or(0) children=move |run| {
                                    let title = run.display_title.clone().unwrap_or_default();
                                    let status = run.status.clone().unwrap_or_default();
                                    let number = run.run_number.unwrap_or(0);
                                    view! {
                                        <li>
                                            <span class="run-status">{status}</span>
                                            <strong>"#" {number}</strong> " " {title}
                                        </li>
                                    }
                                }/>
                            }.into_any()
                        }
                    })}
                </Suspense>
            </ul>
        </div>
    }
}
