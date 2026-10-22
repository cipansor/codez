//! Issue, label, milestone and comment pages.

use crate::api::{get, get_or, local_resource, post_json, spawn, WRITE_ERROR};
use crate::components::{first_line, FormError, LabelChip, Loading, RepoNav, StateChip};
use leptos::prelude::*;
use leptos_router::hooks::*;
use shared::{Comment, CreateIssueOption, CreateLabelOption, Issue, Label, Milestone};

/// `/repos/:owner/:repo/issues` — the issue list with a state filter.
#[component]
pub fn IssueList() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let repo = move || params.with(|p| p.get("repo").unwrap_or_default());
    let (state, set_state) = signal("open".to_string());
    let (refresh, set_refresh) = signal(0u32);
    let (error, set_error) = signal(Option::<String>::None);
    let (title, set_title) = signal(String::new());
    let (body, set_body) = signal(String::new());

    let issues = local_resource(
        move || (owner(), repo(), state.get(), refresh.get()),
        |(o, r, s, _)| async move {
            get::<Vec<Issue>>(&format!("/api/v1/repos/{o}/{r}/issues?state={s}")).await
        },
    );

    let new_issue = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let title_value = title.get();
        if title_value.trim().is_empty() {
            return;
        }
        let body_value = body.get();
        let o = owner();
        let r = repo();
        spawn(async move {
            let option = CreateIssueOption {
                title: title_value,
                body: Some(body_value),
                ..Default::default()
            };
            if post_json(&format!("/api/v1/repos/{o}/{r}/issues"), &option).await {
                set_error.set(None);
                set_title.set(String::new());
                set_body.set(String::new());
                set_refresh.update(|n| *n += 1);
            } else {
                // Keep the typed title and body so the write can be retried.
                set_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    view! {
        <div class="issue-list">
            <RepoNav/>
            <div class="page-header">
                <h3>"Issues"</h3>
                <div class="issue-filters tabs">
                    <button class:active=move || state.get() == "open"
                        on:click=move |_| set_state.set("open".to_string())>"Open"</button>
                    <button class:active=move || state.get() == "closed"
                        on:click=move |_| set_state.set("closed".to_string())>"Closed"</button>
                    <button class:active=move || state.get() == "all"
                        on:click=move |_| set_state.set("all".to_string())>"All"</button>
                </div>
            </div>

            <form class="panel flex-center gap-sm" on:submit=new_issue>
                <input class="input-lg" type="text" placeholder="Issue title"
                    prop:value=title on:input=move |ev| set_title.set(event_target_value(&ev)) />
                <input class="input-lg" type="text" placeholder="Leave a description"
                    prop:value=body on:input=move |ev| set_body.set(event_target_value(&ev)) />
                <button class="btn-primary" type="submit">"New issue"</button>
            </form>
            {move || error.get().map(|msg| view! { <FormError message=msg/> })}

            <ul class="issue-list item-list panel">
                <Suspense fallback=|| view! { <Loading label="Loading issues…"/> }>
                    {move || issues.get().map(|list| {
                        if list.is_empty() {
                            view! { <li class="text-muted">"No issues."</li> }.into_any()
                        } else {
                            let o = owner();
                            let r = repo();
                            view! {
                                <For each=move || list.clone() key=|i| i.id.unwrap_or(0) children=move |i| {
                                    let number = i.number.unwrap_or(0);
                                    let title = i.title.clone().unwrap_or_default();
                                    let href = format!("/repos/{o}/{r}/issues/{number}");
                                    let body = first_line(&i.body, 120);
                                    let labels = i.labels.clone().unwrap_or_default();
                                    let comments = i.comments.unwrap_or(0);
                                    let state = i.state.map(|s| s.as_wire().to_string());
                                    view! {
                                        <li class="issue-row">
                                            <span class="issue-icon">"🐛"</span>
                                            <div>
                                                <a href=href><strong>{title}</strong></a>
                                                <div class="text-small text-muted">{body}</div>
                                                {labels.into_iter().map(|l| view! { <LabelChip label=l/> }).collect_view()}
                                            </div>
                                            <span class="meta text-small text-muted">
                                                "#" {number} " · " {comments} " comments"
                                            </span>
                                            <StateChip state=state merged=false/>
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

/// `/repos/:owner/:repo/issues/:index` — an issue, its comments and its
/// close/reopen toggle.
#[component]
pub fn IssueDetail() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let repo = move || params.with(|p| p.get("repo").unwrap_or_default());
    let index = move || params.with(|p| p.get("index").unwrap_or_default());
    let (refresh, set_refresh) = signal(0u32);
    let (error, set_error) = signal(Option::<String>::None);

    let issue = local_resource(
        move || (owner(), repo(), index(), refresh.get()),
        |(o, r, i, _)| async move {
            get_or::<Option<Issue>>(&format!("/api/v1/repos/{o}/{r}/issues/{i}"), None).await
        },
    );

    let comments = local_resource(
        move || (owner(), repo(), index(), refresh.get()),
        |(o, r, i, _)| async move {
            get::<Vec<Comment>>(&format!("/api/v1/repos/{o}/{r}/issues/{i}/comments")).await
        },
    );

    // Close and reopen are separate endpoints (`PATCH /issues/{index}` with a
    // state body); the toggle reads the current state to pick which one to call.
    let on_toggle = move |_| {
        let o = owner();
        let r = repo();
        let i = index();
        let current = issue.get_untracked().flatten();
        let next = match current.as_ref().and_then(|i| i.state.as_ref()) {
            Some(shared::StateType::Closed) => "open",
            _ => "closed",
        };
        spawn(async move {
            let payload = serde_json::json!({ "state": next });
            if crate::api::patch_json(&format!("/api/v1/repos/{o}/{r}/issues/{i}"), &payload).await
            {
                set_error.set(None);
                set_refresh.update(|n| *n += 1);
            } else {
                set_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    let on_comment = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        use wasm_bindgen::JsCast;
        let form = ev
            .target()
            .and_then(|t| t.dyn_into::<web_sys::HtmlFormElement>().ok());
        let body = form
            .as_ref()
            .and_then(|f| f.elements().named_item("body"))
            .and_then(|el| el.dyn_into::<web_sys::HtmlInputElement>().ok())
            .map(|i| i.value())
            .unwrap_or_default();
        if body.trim().is_empty() {
            return;
        }
        let o = owner();
        let r = repo();
        let i = index();
        spawn(async move {
            let payload = serde_json::json!({ "body": body });
            if post_json(
                &format!("/api/v1/repos/{o}/{r}/issues/{i}/comments"),
                &payload,
            )
            .await
            {
                set_error.set(None);
                set_refresh.update(|n| *n += 1);
            } else {
                set_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    view! {
        <div class="issue-detail">
            <RepoNav/>
            {move || error.get().map(|msg| view! { <FormError message=msg/> })}
            <Suspense fallback=|| view! { <p class="text-muted">"Loading issue…"</p> }>
                {move || issue.get().flatten().map(|i| {
                    let number = i.number.unwrap_or(0);
                    let title = i.title.clone().unwrap_or_default();
                    let body = i.body.clone().unwrap_or_default();
                    let state = i.state.as_ref().map(|s| s.as_wire().to_string());
                    let author = i.user.as_ref().and_then(|u| u.login.clone()).unwrap_or_default();
                    let created = i.created_at.clone().unwrap_or_default();
                    let labels = i.labels.clone().unwrap_or_default();
                    let closed = i.state == Some(shared::StateType::Closed);
                    let toggle = if closed { "Reopen" } else { "Close" };
                    let milestone = i.milestone.as_ref().and_then(|m| m.title.clone());
                    let assignee = i.assignee.as_ref().and_then(|u| u.login.clone());
                    view! {
                        <div class="issue-header page-header">
                            <h3>"#" {number} " " {title}</h3>
                            <div class="pr-actions">
                                <StateChip state=state merged=false/>
                                <button class="btn-sm" on:click=on_toggle>{toggle}</button>
                            </div>
                        </div>
                        <div class="issue-container">
                            <div class="issue-main">
                                <div class="comment">
                                    <div class="comment-header">
                                        <strong>{author}</strong>
                                        <span class="text-small text-muted">" opened on " {created}</span>
                                    </div>
                                    <div class="comment-body">{body}</div>
                                    <div class="comment-reactions">
                                        {labels.into_iter().map(|l| view! { <LabelChip label=l/> }).collect_view()}
                                    </div>
                                </div>

                                <h4>"Comments"</h4>
                                <ul class="comments-section item-list">
                                    <Suspense fallback=|| view! { <Loading label="Loading comments…"/> }>
                                        {move || comments.get().map(|list| {
                                            if list.is_empty() {
                                                view! { <li class="text-muted">"No comments yet."</li> }.into_any()
                                            } else {
                                                view! {
                                                    <For each=move || list.clone() key=|c| c.id.unwrap_or(0) children=move |c| {
                                                        let author = c.user.as_ref().and_then(|u| u.login.clone()).unwrap_or_default();
                                                        let body = c.body.clone().unwrap_or_default();
                                                        let created = c.created_at.clone().unwrap_or_default();
                                                        view! {
                                                            <li class="comment">
                                                                <div class="comment-header">
                                                                    <strong>{author}</strong>
                                                                    <span class="text-small text-muted">" " {created}</span>
                                                                </div>
                                                                <div class="comment-body">{body}</div>
                                                            </li>
                                                        }
                                                    }/>
                                                }.into_any()
                                            }
                                        })}
                                    </Suspense>
                                </ul>

                                <form class="comment-form" on:submit=on_comment>
                                    <input class="input-lg" type="text" name="body" placeholder="Leave a comment" />
                                    <button class="btn-primary" type="submit">"Comment"</button>
                                </form>
                            </div>

                            <aside class="issue-sidebar">
                                <div class="panel">
                                    <h4>"Details"</h4>
                                    <dl class="meta">
                                        <dt>"Assignee"</dt><dd>{assignee.unwrap_or_else(|| "Unassigned".to_string())}</dd>
                                        <dt>"Milestone"</dt><dd>{milestone.unwrap_or_else(|| "None".to_string())}</dd>
                                    </dl>
                                </div>
                            </aside>
                        </div>
                    }
                })}
            </Suspense>
        </div>
    }
}

/// `/repos/:owner/:repo/labels` — labels and their creation form.
#[component]
pub fn LabelList() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let repo = move || params.with(|p| p.get("repo").unwrap_or_default());
    let (refresh, set_refresh) = signal(0u32);
    let (name, set_name) = signal(String::new());
    let (color, set_color) = signal("ededed".to_string());
    let (error, set_error) = signal(Option::<String>::None);

    let labels = local_resource(
        move || (owner(), repo(), refresh.get()),
        |(o, r, _)| async move { get::<Vec<Label>>(&format!("/api/v1/repos/{o}/{r}/labels")).await },
    );

    let on_create = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let payload = CreateLabelOption {
            name: name.get(),
            color: color.get(),
            ..Default::default()
        };
        let o = owner();
        let r = repo();
        spawn(async move {
            if post_json(&format!("/api/v1/repos/{o}/{r}/labels"), &payload).await {
                set_error.set(None);
                set_name.set(String::new());
                set_refresh.update(|n| *n += 1);
            } else {
                // Keep the typed name and colour so the write can be retried.
                set_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    view! {
        <div class="label-list">
            <RepoNav/>
            <h3>"Labels"</h3>
            {move || error.get().map(|msg| view! { <FormError message=msg/> })}
            <form class="panel flex-center gap-sm" on:submit=on_create>
                <input class="input-lg" type="text" placeholder="Label name"
                    prop:value=name on:input=move |ev| set_name.set(event_target_value(&ev)) />
                <input type="text" placeholder="ededed"
                    prop:value=color on:input=move |ev| set_color.set(event_target_value(&ev)) />
                <button class="btn-primary" type="submit">"New label"</button>
            </form>
            <ul class="labels-list item-list panel">
                <Suspense fallback=|| view! { <Loading label="Loading labels…"/> }>
                    {move || labels.get().map(|list| {
                        if list.is_empty() {
                            view! { <li class="text-muted">"No labels."</li> }.into_any()
                        } else {
                            view! {
                                <For each=move || list.clone() key=|l| l.id.unwrap_or(0) children=move |l| {
                                    let description = l.description.clone().unwrap_or_default();
                                    view! {
                                        <li class="flex-between">
                                            <span><LabelChip label=l.clone()/> " " <span class="text-muted">{description}</span></span>
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

/// `/repos/:owner/:repo/milestones` — milestones and their creation form.
#[component]
pub fn MilestoneList() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let repo = move || params.with(|p| p.get("repo").unwrap_or_default());
    let (refresh, set_refresh) = signal(0u32);
    let (title, set_title) = signal(String::new());
    let (description, set_description) = signal(String::new());
    let (error, set_error) = signal(Option::<String>::None);

    let milestones = local_resource(
        move || (owner(), repo(), refresh.get()),
        |(o, r, _)| async move {
            get::<Vec<Milestone>>(&format!("/api/v1/repos/{o}/{r}/milestones")).await
        },
    );

    let on_create = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let payload = shared::CreateMilestoneOption {
            title: Some(title.get()),
            description: Some(description.get()),
            ..Default::default()
        };
        let o = owner();
        let r = repo();
        spawn(async move {
            if post_json(&format!("/api/v1/repos/{o}/{r}/milestones"), &payload).await {
                set_error.set(None);
                set_title.set(String::new());
                set_description.set(String::new());
                set_refresh.update(|n| *n += 1);
            } else {
                set_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    view! {
        <div class="milestone-list">
            <RepoNav/>
            <h3>"Milestones"</h3>
            {move || error.get().map(|msg| view! { <FormError message=msg/> })}
            <form class="panel flex-center gap-sm" on:submit=on_create>
                <input class="input-lg" type="text" placeholder="Milestone title"
                    prop:value=title on:input=move |ev| set_title.set(event_target_value(&ev)) />
                <input class="input-lg" type="text" placeholder="Description"
                    prop:value=description on:input=move |ev| set_description.set(event_target_value(&ev)) />
                <button class="btn-primary" type="submit">"New milestone"</button>
            </form>
            <ul class="milestone-list item-list panel">
                <Suspense fallback=|| view! { <Loading label="Loading milestones…"/> }>
                    {move || milestones.get().map(|list| {
                        if list.is_empty() {
                            view! { <li class="text-muted">"No milestones."</li> }.into_any()
                        } else {
                            let o = owner();
                            let r = repo();
                            view! {
                                <For each=move || list.clone() key=|m| m.id.unwrap_or(0) children=move |m| {
                                    let id = m.id.unwrap_or(0);
                                    let title = m.title.clone().unwrap_or_default();
                                    let description = m.description.clone().unwrap_or_default();
                                    let open = m.open_issues.unwrap_or(0);
                                    let closed = m.closed_issues.unwrap_or(0);
                                    let href = format!("/repos/{o}/{r}/milestones/{id}");
                                    view! {
                                        <li class="flex-between">
                                            <span>
                                                <a href=href><strong>{title}</strong></a>
                                                <p class="mb-0 text-muted">{description}</p>
                                            </span>
                                            <span class="text-small text-muted">{open} " open · " {closed} " closed"</span>
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

/// `/repos/:owner/:repo/milestones/:index` — one milestone.
#[component]
pub fn MilestoneDetail() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let repo = move || params.with(|p| p.get("repo").unwrap_or_default());
    let index = move || params.with(|p| p.get("index").unwrap_or_default());

    let milestone = local_resource(
        move || (owner(), repo(), index()),
        |(o, r, i)| async move {
            get_or::<Option<Milestone>>(&format!("/api/v1/repos/{o}/{r}/milestones/{i}"), None)
                .await
        },
    );

    view! {
        <div class="milestone-detail">
            <RepoNav/>
            <Suspense fallback=|| view! { <p class="text-muted">"Loading milestone…"</p> }>
                {move || milestone.get().flatten().map(|m| {
                    let title = m.title.clone().unwrap_or_default();
                    let description = m.description.clone().unwrap_or_default();
                    let due = m.due_on.clone().unwrap_or_default();
                    let open = m.open_issues.unwrap_or(0);
                    let closed = m.closed_issues.unwrap_or(0);
                    let state = m.state.map(|s| s.as_wire().to_string());
                    view! {
                        <div class="page-header">
                            <h3>{title} <StateChip state=state merged=false/></h3>
                        </div>
                        <p class="text-muted">{description}</p>
                        <p class="text-small text-muted">"Due " {due}</p>
                        <div class="stat-grid">
                            <div class="stat-card"><span class="stat-value">{open}</span><span>"Open"</span></div>
                            <div class="stat-card"><span class="stat-value">{closed}</span><span>"Closed"</span></div>
                        </div>
                    }
                })}
            </Suspense>
        </div>
    }
}
