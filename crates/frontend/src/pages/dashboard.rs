//! Dashboard, explore and global search.

use crate::api::{encode_query, get, local_resource, patch, spawn, WRITE_ERROR};
use crate::components::{repo_href, FormError, Loading};
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use shared::{Issue, NotificationThread, Repository, SearchResults};

/// `/` — the signed-in user's dashboard: their repositories, notifications and
/// the issues and pull requests assigned to them.
#[component]
pub fn UserDashboard() -> impl IntoView {
    let repos = local_resource(
        || (),
        |_| async move { get::<Vec<Repository>>("/api/v1/user/repos").await },
    );

    let notifications = local_resource(
        || (),
        |_| async move { get::<Vec<NotificationThread>>("/api/v1/notifications").await },
    );

    // Gitea has no `/user/issues`; the assigned-issue view is the repository
    // search endpoint filtered by `created`/`assigned`. `/repos/issues/search`
    // is the one list that spans every repository the caller can see.
    let my_issues = local_resource(
        || (),
        |_| async move { get::<Vec<Issue>>("/api/v1/repos/issues/search?state=open&type=issues").await },
    );

    view! {
        <div class="dashboard">
            <div class="page-header">
                <h2>"Dashboard"</h2>
                <div class="flex gap-sm">
                    <a class="btn" href="/repo/create">"New repository"</a>
                    <a class="btn" href="/org/create">"New organization"</a>
                </div>
            </div>

            <div class="repo-shell">
                <aside class="repo-sidebar">
                    <div class="panel">
                        <h3>"Your repositories"</h3>
                        <ul class="item-list">
                            <Suspense fallback=|| view! { <Loading label="Loading repositories…"/> }>
                                {move || repos.get().map(|list| {
                                    if list.is_empty() {
                                        view! { <li class="text-muted">"No repositories yet."</li> }.into_any()
                                    } else {
                                        view! {
                                            <For each=move || list.clone() key=|r| r.id.unwrap_or(0) children=move |r| {
                                                let href = repo_href(&r);
                                                let path = crate::components::repo_path(&r);
                                                let description = r.description.clone().unwrap_or_default();
                                                view! {
                                                    <li>
                                                        <a href=href><strong>{path}</strong></a>
                                                        <div class="text-small text-muted">{description}</div>
                                                    </li>
                                                }
                                            }/>
                                        }.into_any()
                                    }
                                })}
                            </Suspense>
                        </ul>
                    </div>
                </aside>

                <div class="repo-main">
                    <div class="notifications panel">
                        <h3>"Notifications"</h3>
                        <ul class="item-list">
                            <Suspense fallback=|| view! { <Loading label="Loading…"/> }>
                                {move || notifications.get().map(|list| {
                                    if list.is_empty() {
                                        view! { <li class="text-muted">"You have no notifications."</li> }.into_any()
                                    } else {
                                        view! {
                                            <For each=move || list.clone() key=|n| n.id.unwrap_or(0) children=move |n| {
                                                let title = n.subject.as_ref()
                                                    .and_then(|s| s.title.clone())
                                                    .unwrap_or_else(|| "(no subject)".to_string());
                                                let href = n.subject.as_ref()
                                                    .and_then(|s| s.html_url.clone())
                                                    .unwrap_or_default();
                                                let unread = n.unread.unwrap_or(false);
                                                view! {
                                                    <li class="flex-between">
                                                        <span>
                                                            <a href=href><strong>{title}</strong></a>
                                                            {if unread {
                                                                view! { <span class="label label-accent">" Unread"</span> }.into_any()
                                                            } else {
                                                                view! { <span class="text-small text-muted">" Read"</span> }.into_any()
                                                            }}
                                                        </span>
                                                    </li>
                                                }
                                            }/>
                                        }.into_any()
                                    }
                                })}
                            </Suspense>
                        </ul>
                    </div>

                    <h3 class="mt-3">"Issues assigned across repositories"</h3>
                    <ul class="item-list panel">
                        <Suspense fallback=|| view! { <Loading label="Loading issues…"/> }>
                            {move || my_issues.get().map(|list| {
                                if list.is_empty() {
                                    view! { <li class="text-muted">"No open issues."</li> }.into_any()
                                } else {
                                    view! {
                                        <For each=move || list.clone() key=|i| i.id.unwrap_or(0) children=move |i| {
                                            let title = i.title.clone().unwrap_or_default();
                                            let number = i.number.unwrap_or(0);
                                            let href = issue_href(&i);
                                            let state = i.state.map(|s| s.as_wire().to_string());
                                            view! {
                                                <li>
                                                    <a href=href><strong>"#" {number}</strong> " " {title}</a>
                                                    <span class="label">{state.unwrap_or_default()}</span>
                                                </li>
                                            }
                                        }/>
                                }.into_any()
                                }
                            })}
                        </Suspense>
                    </ul>
                </div>
            </div>
        </div>
    }
}

/// Best-effort link back to an issue from a cross-repository list, where the
/// issue carries a `repository` reference but no owning route parameters.
fn issue_href(issue: &Issue) -> String {
    match issue.repository.as_ref().and_then(|r| r.full_name.clone()) {
        Some(full_name) => format!("/repos/{full_name}/issues/{}", issue.number.unwrap_or(0)),
        None => "#".to_string(),
    }
}

/// `/explore` — recently updated repositories.
#[component]
pub fn Explore() -> impl IntoView {
    let repos = local_resource(
        || (),
        |_| async move {
            get::<SearchResults>("/api/v1/repos/search?sort=updated&limit=20")
                .await
                .data
                .unwrap_or_default()
        },
    );

    view! {
        <div class="explore">
            <div class="page-header">
                <h2>"Explore Codez"</h2>
            </div>
            <Search/>
            <h3 class="mt-3">"Recent repositories"</h3>
            <div class="explore-list">
                <Suspense fallback=|| view! { <p class="text-muted">"Loading…"</p> }>
                    {move || repos.get().map(|list| {
                        if list.is_empty() {
                            view! { <div class="empty-state">"No repositories yet."</div> }.into_any()
                        } else {
                            view! {
                                <For each=move || list.clone() key=|r| r.id.unwrap_or(0) children=move |r| {
                                    let href = repo_href(&r);
                                    let path = crate::components::repo_path(&r);
                                    let description = r.description.clone()
                                        .filter(|d| !d.is_empty())
                                        .unwrap_or_else(|| "No description".to_string());
                                    let stars = r.stars_count.unwrap_or(0);
                                    let forks = r.forks_count.unwrap_or(0);
                                    view! {
                                        <div class="card">
                                            <a href=href><strong>{path}</strong></a>
                                            <p class="text-muted mb-0">{description}</p>
                                            <span class="text-small text-muted">"⭐ " {stars} " · 🍴 " {forks}</span>
                                        </div>
                                    }
                                }/>
                            }.into_any()
                        }
                    })}
                </Suspense>
            </div>
        </div>
    }
}

/// `/search` — repository and issue search driven by the `q` query parameter.
#[component]
pub fn Search() -> impl IntoView {
    let query_map = use_query_map();
    let initial = query_map.with(|q| q.get("q").unwrap_or_default());
    let (query, set_query) = signal(initial);
    let (search_type, set_search_type) = signal("repos".to_string());

    let (repo_results, set_repo_results) = signal(Vec::<Repository>::new());
    let (issue_results, set_issue_results) = signal(Vec::<Issue>::new());

    // Monotonic request generation. A slow earlier request must not overwrite
    // the results of a newer one, so each response is applied only if no newer
    // search has started since it was issued.
    let generation = RwSignal::new(0u64);

    let run_search = move |q: String, t: String| {
        let current = generation.get_untracked() + 1;
        generation.set(current);
        spawn(async move {
            // Drop stale responses: a newer search has already been issued.
            let is_current = move || current == generation.get_untracked();

            if t == "repos" {
                let url = format!("/api/v1/repos/search?q={}&limit=20", encode_query(&q));
                let results = get::<SearchResults>(&url).await.data.unwrap_or_default();
                if is_current() {
                    set_repo_results.set(results);
                    set_issue_results.set(Vec::new());
                }
            } else {
                let url = format!(
                    "/api/v1/repos/issues/search?q={}&state=all",
                    encode_query(&q)
                );
                let results = get::<Vec<Issue>>(&url).await;
                if is_current() {
                    set_issue_results.set(results);
                    set_repo_results.set(Vec::new());
                }
            }
        });
    };

    let on_search = move |_| run_search(query.get(), search_type.get());

    // Keep the box and results in sync when `q` changes, including the initial
    // load (from the global header navigating to `/search?q=…`) and an empty
    // `q`. This single effect drives every load, so there is no duplicate
    // initial request racing the mount-time one.
    Effect::new(move |_| {
        let q = query_map.with(|q| q.get("q").unwrap_or_default());
        set_query.set(q.clone());
        run_search(q, search_type.get_untracked());
    });

    view! {
        <div class="search-page">
            <div class="panel">
                <div class="flex-center flex-wrap">
                    <input type="text" placeholder="Search…"
                        prop:value=query
                        on:input=move |ev| set_query.set(event_target_value(&ev))
                        class="search-input-flex" />
                    <select on:change=move |ev| {
                        // Changing the type must start a fresh search for the
                        // current query. Otherwise the panel keeps the previous
                        // type's results until Search is clicked.
                        let t = event_target_value(&ev);
                        set_search_type.set(t.clone());
                        run_search(query.get(), t);
                    } class="w-auto">
                        <option value="repos">"Repositories"</option>
                        <option value="issues">"Issues"</option>
                    </select>
                    <button class="btn-primary" on:click=on_search>"Search"</button>
                </div>
            </div>

            <div class="search-results mt-2">
                {move || if search_type.get() == "repos" {
                    view! {
                        <ul class="item-list panel">
                            <For each=move || repo_results.get() key=|r| r.id.unwrap_or(0) children=move |r| {
                                let href = repo_href(&r);
                                let path = crate::components::repo_path(&r);
                                let description = r.description.clone().unwrap_or_default();
                                let stars = r.stars_count.unwrap_or(0);
                                view! {
                                    <li>
                                        <a href=href><strong>{path}</strong></a>
                                        <p class="mb-0 text-muted">{description}</p>
                                        <small class="text-muted">"⭐ " {stars}</small>
                                    </li>
                                }
                            }/>
                        </ul>
                    }.into_any()
                } else {
                    view! {
                        <ul class="item-list panel">
                            <For each=move || issue_results.get() key=|i| i.id.unwrap_or(0) children=move |i| {
                                let title = i.title.clone().unwrap_or_default();
                                let number = i.number.unwrap_or(0);
                                let href = issue_href(&i);
                                let state = i.state.map(|s| s.as_wire().to_string());
                                let body = crate::components::first_line(&i.body, 120);
                                view! {
                                    <li>
                                        <a href=href><strong>"#" {number}</strong> " " {title}</a>
                                        <span class="label">{state.unwrap_or_default()}</span>
                                        <p class="mb-0 text-muted">{body}</p>
                                    </li>
                                }
                            }/>
                        </ul>
                    }.into_any()
                }}
            </div>
        </div>
    }
}

/// `/notifications` — the notification inbox with a mark-as-read action.
#[component]
pub fn NotificationList() -> impl IntoView {
    let (refresh, set_refresh) = signal(0);
    let (action_error, set_action_error) = signal(Option::<String>::None);
    let notifs = local_resource(
        move || refresh.get(),
        |_| async move { get::<Vec<NotificationThread>>("/api/v1/notifications").await },
    );

    let on_mark_read = move |id: i64| {
        spawn(async move {
            if patch(&format!("/api/v1/notifications/threads/{id}")).await {
                set_action_error.set(None);
                set_refresh.update(|n| *n += 1);
            } else {
                set_action_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    view! {
        <div class="notifications panel">
            <div class="page-header">
                <h3 class="mb-0">"Notifications"</h3>
            </div>
            {move || action_error.get().map(|msg| view! { <FormError message=msg/> })}
            <ul class="item-list">
                <Suspense fallback=|| view! { <Loading label="Loading…"/> }>
                    {move || notifs.get().map(|list| {
                        if list.is_empty() {
                            view! { <li class="text-muted">"You have no notifications."</li> }.into_any()
                        } else {
                            view! {
                                <For each=move || list.clone() key=|n| n.id.unwrap_or(0) children=move |n| {
                                    let unread = n.unread.unwrap_or(false);
                                    let id = n.id.unwrap_or(0);
                                    let title = n.subject.as_ref()
                                        .and_then(|s| s.title.clone())
                                        .unwrap_or_else(|| "(no subject)".to_string());
                                    let repo = n.repository.as_ref().map(crate::components::repo_path).unwrap_or_default();
                                    let href = n.subject.as_ref()
                                        .and_then(|s| s.html_url.clone())
                                        .unwrap_or_default();
                                    view! {
                                        <li class="flex-between">
                                            <span>
                                                <a href=href><strong>{title}</strong></a>
                                                <span class="text-small text-muted">" " {repo}</span>
                                                {if unread {
                                                    view! { <span class="label label-accent">" (Unread)"</span> }.into_any()
                                                } else {
                                                    view! { <span class="text-small text-muted">" (Read)"</span> }.into_any()
                                                }}
                                            </span>
                                            {if unread {
                                                view! {
                                                    <button class="btn-sm" on:click=move |_| on_mark_read(id)>"Mark Read"</button>
                                                }.into_any()
                                            } else {
                                                view! { <span class="text-small text-muted">"Read"</span> }.into_any()
                                            }}
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
