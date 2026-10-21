//! Pull request list and detail, including merge, reviews and the commit
//! status rollup.

use crate::api::{get, get_or, local_resource, patch_json, post_json_resp, spawn, WRITE_ERROR};
use crate::components::{first_line, FormError, Loading, RepoNav, StateChip};
use leptos::prelude::*;
use leptos_router::hooks::*;
use shared::{CombinedStatus, CommitStatus, MergePullRequestOption, PullRequest, PullReview};

/// `/repos/:owner/:repo/pulls` — the pull request list.
#[component]
pub fn PullRequestList() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let repo = move || params.with(|p| p.get("repo").unwrap_or_default());
    let (state, set_state) = signal("open".to_string());

    let pulls = local_resource(
        move || (owner(), repo(), state.get()),
        |(o, r, s)| async move {
            get::<Vec<PullRequest>>(&format!("/api/v1/repos/{o}/{r}/pulls?state={s}")).await
        },
    );

    view! {
        <div class="pull-list">
            <RepoNav/>
            <div class="page-header">
                <h3>"Pull requests"</h3>
                <a class="btn-primary" href=move || format!("/repos/{}/{}/compare", owner(), repo())>
                    "New pull request"
                </a>
            </div>
            <div class="tabs">
                <button class:active=move || state.get() == "open"
                    on:click=move |_| set_state.set("open".to_string())>"Open"</button>
                <button class:active=move || state.get() == "closed"
                    on:click=move |_| set_state.set("closed".to_string())>"Closed"</button>
                <button class:active=move || state.get() == "all"
                    on:click=move |_| set_state.set("all".to_string())>"All"</button>
            </div>
            <ul class="pull-list item-list panel">
                <Suspense fallback=|| view! { <Loading label="Loading pull requests…"/> }>
                    {move || pulls.get().map(|list| {
                        if list.is_empty() {
                            view! { <li class="text-muted">"No pull requests."</li> }.into_any()
                        } else {
                            let o = owner();
                            let r = repo();
                            view! {
                                <For each=move || list.clone() key=|p| p.id.unwrap_or(0) children=move |p| {
                                    let number = p.number.unwrap_or(0);
                                    let title = p.title.clone().unwrap_or_default();
                                    let href = format!("/repos/{o}/{r}/pulls/{number}");
                                    let body = first_line(&p.body, 120);
                                    let merged = p.merged.unwrap_or(false);
                                    let state = p.state.map(|s| s.as_wire().to_string());
                                    let head = p.head.as_ref().and_then(|h| h.ref_.clone()).unwrap_or_default();
                                    let base = p.base.as_ref().and_then(|b| b.ref_.clone()).unwrap_or_default();
                                    let draft = p.draft.unwrap_or(false);
                                    view! {
                                        <li class="issue-row">
                                            <span class="issue-icon">"🔀"</span>
                                            <div>
                                                <a href=href><strong>{title}</strong></a>
                                                <div class="text-small text-muted">
                                                    <code>{head}</code> " → " <code>{base}</code>
                                                </div>
                                                <div class="text-small text-muted">{body}</div>
                                            </div>
                                            <span class="meta text-small text-muted">"#" {number}</span>
                                            {draft.then(|| view! { <span class="label label-warning">"Draft"</span> })}
                                            <StateChip state=state merged=merged/>
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

/// `/repos/:owner/:repo/pulls/:index` — one pull request: description, reviews,
/// commit statuses and the merge action.
#[component]
pub fn PullRequestDetail() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let repo = move || params.with(|p| p.get("repo").unwrap_or_default());
    let index = move || params.with(|p| p.get("index").unwrap_or_default());
    let (refresh, set_refresh) = signal(0u32);
    let (error, set_error) = signal(Option::<String>::None);

    let pull = local_resource(
        move || (owner(), repo(), index(), refresh.get()),
        |(o, r, i, _)| async move {
            get_or::<Option<PullRequest>>(&format!("/api/v1/repos/{o}/{r}/pulls/{i}"), None).await
        },
    );

    let reviews = local_resource(
        move || (owner(), repo(), index(), refresh.get()),
        |(o, r, i, _)| async move {
            get::<Vec<PullReview>>(&format!("/api/v1/repos/{o}/{r}/pulls/{i}/reviews")).await
        },
    );

    // The merge endpoint is a `POST` with a merge strategy body; `merge` is the
    // default and what the button offers.
    let on_merge = move |_| {
        let o = owner();
        let r = repo();
        let i = index();
        spawn(async move {
            let payload = MergePullRequestOption::default();
            match post_json_resp::<_, serde_json::Value>(
                &format!("/api/v1/repos/{o}/{r}/pulls/{i}/merge"),
                &payload,
            )
            .await
            {
                Some(_) => {
                    set_error.set(None);
                    set_refresh.update(|n| *n += 1);
                }
                None => set_error.set(Some(WRITE_ERROR.to_string())),
            }
        });
    };

    // Closing a pull request is the issue endpoint with a `closed` state, since
    // Gitea models pull requests as issues for the state transition.
    let on_close = move |_| {
        let o = owner();
        let r = repo();
        let i = index();
        spawn(async move {
            let payload = serde_json::json!({ "state": "closed" });
            if patch_json(&format!("/api/v1/repos/{o}/{r}/issues/{i}"), &payload).await {
                set_error.set(None);
                set_refresh.update(|n| *n += 1);
            } else {
                set_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    view! {
        <div class="pull-detail">
            <RepoNav/>
            {move || error.get().map(|msg| view! { <FormError message=msg/> })}
            <Suspense fallback=|| view! { <p class="text-muted">"Loading pull request…"</p> }>
                {move || pull.get().flatten().map(|p| {
                    let number = p.number.unwrap_or(0);
                    let title = p.title.clone().unwrap_or_default();
                    let body = p.body.clone().unwrap_or_default();
                    let merged = p.merged.unwrap_or(false);
                    let state = p.state.as_ref().map(|s| s.as_wire().to_string());
                    let author = p.user.as_ref().and_then(|u| u.login.clone()).unwrap_or_default();
                    let created = p.created_at.clone().unwrap_or_default();
                    let head = p.head.as_ref().and_then(|h| h.ref_.clone()).unwrap_or_default();
                    let base = p.base.as_ref().and_then(|b| b.ref_.clone()).unwrap_or_default();
                    let mergeable = p.mergeable.unwrap_or(false);
                    let additions = p.additions.unwrap_or(0);
                    let deletions = p.deletions.unwrap_or(0);
                    let changed = p.changed_files.unwrap_or(0);
                    let closed = p.state == Some(shared::StateType::Closed);
                    view! {
                        <div class="pr-header page-header">
                            <h3>"#" {number} " " {title}</h3>
                            <div class="pr-actions">
                                <StateChip state=state merged=merged/>
                                {if !merged && !closed {
                                    view! {
                                        <button class="btn-merge" on:click=on_merge
                                            disabled=!mergeable>"Merge"</button>
                                    }.into_any()
                                } else {
                                    view! { <span></span> }.into_any()
                                }}
                                {(!closed && !merged).then(|| view! {
                                    <button class="btn-sm" on:click=on_close>"Close"</button>
                                })}
                            </div>
                        </div>

                        <p class="text-small text-muted">
                            <strong>{author}</strong> " wants to merge "
                            <code>{head}</code> " into " <code>{base}</code>
                            " · " {created}
                        </p>
                        <p class="diff-stats text-small">
                            <span class="text-success">"+" {additions}</span> " "
                            <span class="text-danger">"-" {deletions}</span> " · "
                            {changed} " files changed"
                        </p>
                        {(!mergeable && !merged).then(|| view! {
                            <p class="form-error" role="alert">
                                "This pull request cannot be merged automatically."
                            </p>
                        })}

                        <div class="pr-body comment">
                            <div class="comment-body">{body}</div>
                        </div>

                        <CommitStatusList owner=owner() repo=repo() sha=p.head.as_ref().and_then(|h| h.sha.clone()).unwrap_or_default()/>

                        <h4>"Reviews"</h4>
                        <ul class="review-list item-list panel">
                            <Suspense fallback=|| view! { <Loading label="Loading reviews…"/> }>
                                {move || reviews.get().map(|list| {
                                    if list.is_empty() {
                                        view! { <li class="text-muted">"No reviews yet."</li> }.into_any()
                                    } else {
                                        view! {
                                            <For each=move || list.clone() key=|rv| rv.id.unwrap_or(0) children=move |rv| {
                                                let author = rv.user.as_ref().and_then(|u| u.login.clone()).unwrap_or_default();
                                                let state = rv.state.as_ref().map(|s| s.as_wire().to_string()).unwrap_or_default();
                                                let body = rv.body.clone().unwrap_or_default();
                                                view! {
                                                    <li class="review-item">
                                                        <div class="review-header">
                                                            <strong>{author}</strong>
                                                            <span class="label">{state}</span>
                                                        </div>
                                                        <div class="review-body">{body}</div>
                                                    </li>
                                                }
                                            }/>
                                        }.into_any()
                                    }
                                })}
                            </Suspense>
                        </ul>
                    }
                })}
            </Suspense>
        </div>
    }
}

/// The commit status rollup for a SHA, rendered as a list of checks.
#[component]
pub fn CommitStatusList(owner: String, repo: String, sha: String) -> impl IntoView {
    let status = local_resource(
        move || (owner.clone(), repo.clone(), sha.clone()),
        |(o, r, s)| async move {
            get_or::<Option<CombinedStatus>>(
                &format!("/api/v1/repos/{o}/{r}/commits/{s}/status"),
                None,
            )
            .await
        },
    );

    view! {
        <div class="commit-statuses">
            <h4>"Checks"</h4>
            <Suspense fallback=|| view! { <p class="text-muted">"Loading checks…"</p> }>
                {move || {
                    let value = status.get().flatten();
                    match value {
                        Some(status) => {
                            let overall = status.state.map(|s| s.as_wire().to_string()).unwrap_or_default();
                            let statuses: Vec<CommitStatus> = status.statuses.unwrap_or_default();
                            view! {
                                <p class="text-small text-muted">"Overall: " <span class="label">{overall}</span></p>
                                <ul class="item-list">
                                    {statuses.into_iter().map(|s| {
                                        let context = s.context.clone().unwrap_or_default();
                                        let state = s.status.map(|st| st.as_wire().to_string()).unwrap_or_default();
                                        let description = s.description.clone().unwrap_or_default();
                                        view! {
                                            <li class="flex-between">
                                                <span><strong>{context}</strong> " " <span class="text-muted">{description}</span></span>
                                                <span class="label">{state}</span>
                                            </li>
                                        }
                                    }).collect_view()}
                                </ul>
                            }.into_any()
                        }
                        None => view! {
                            <p class="text-muted">"No checks reported for this commit."</p>
                        }.into_any(),
                    }
                }}
            </Suspense>
        </div>
    }
}
