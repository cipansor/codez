//! Administrative dashboard, user management and instance notices.

use crate::api::{get, local_resource};
use crate::components::Loading;
use leptos::prelude::*;
use shared::{Cron, Organization, User};

/// The `GET /admin/stats` document (`AdminStats`).
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct AdminStats {
    #[serde(default)]
    pub issue_count: i64,
    #[serde(default)]
    pub organization_count: i64,
    #[serde(default)]
    pub pull_request_count: i64,
    #[serde(default)]
    pub repo_count: i64,
    #[serde(default)]
    pub user_count: i64,
}

/// `/admin` — instance statistics and scheduled tasks.
#[component]
pub fn AdminDashboard() -> impl IntoView {
    let stats = local_resource(
        || (),
        |_| async move { get::<AdminStats>("/api/v1/admin/stats").await },
    );
    let cron = local_resource(
        || (),
        |_| async move { get::<Vec<Cron>>("/api/v1/admin/cron").await },
    );

    view! {
        <div class="admin-dashboard">
            <div class="page-header">
                <h2>"Administration"</h2>
            </div>

            <Suspense fallback=|| view! { <p class="text-muted">"Loading statistics…"</p> }>
                {move || stats.get().map(|s| view! {
                    <div class="stat-grid">
                        <div class="stat-card"><span class="stat-value">{s.user_count}</span><span>"Users"</span></div>
                        <div class="stat-card"><span class="stat-value">{s.repo_count}</span><span>"Repositories"</span></div>
                        <div class="stat-card"><span class="stat-value">{s.organization_count}</span><span>"Organizations"</span></div>
                        <div class="stat-card"><span class="stat-value">{s.issue_count}</span><span>"Issues"</span></div>
                        <div class="stat-card"><span class="stat-value">{s.pull_request_count}</span><span>"Pull requests"</span></div>
                    </div>
                })}
            </Suspense>

            <h3 class="mt-3">"Scheduled tasks"</h3>
            <ul class="item-list panel">
                <Suspense fallback=|| view! { <Loading label="Loading…"/> }>
                    {move || cron.get().map(|list| {
                        if list.is_empty() {
                            view! { <li class="text-muted">"No scheduled tasks."</li> }.into_any()
                        } else {
                            view! {
                                <For each=move || list.clone() key=|c| c.name.clone().unwrap_or_default() children=move |c| {
                                    let name = c.name.clone().unwrap_or_default();
                                    let schedule = c.schedule.clone().unwrap_or_default();
                                    view! { <li><strong>{name}</strong> <code>{schedule}</code></li> }
                                }/>
                            }.into_any()
                        }
                    })}
                </Suspense>
            </ul>
        </div>
    }
}

/// `/admin/users` — every account on the instance.
#[component]
pub fn AdminUsers() -> impl IntoView {
    let users = local_resource(
        || (),
        |_| async move { get::<Vec<User>>("/api/v1/admin/users").await },
    );

    view! {
        <div class="admin-users panel">
            <div class="page-header">
                <h2>"Users"</h2>
            </div>
            <ul class="item-list">
                <Suspense fallback=|| view! { <Loading label="Loading users…"/> }>
                    {move || users.get().map(|list| {
                        if list.is_empty() {
                            view! { <li class="text-muted">"No users."</li> }.into_any()
                        } else {
                            view! {
                                <For each=move || list.clone() key=|u| u.id.unwrap_or(0) children=move |u| {
                                    let login = u.login.clone().unwrap_or_default();
                                    let href = format!("/users/{login}");
                                    let email = u.email.clone().unwrap_or_default();
                                    let admin = u.is_admin.unwrap_or(false);
                                    view! {
                                        <li class="flex-between">
                                            <span>
                                                <a href=href><strong>{login}</strong></a>
                                                <span class="text-small text-muted">" " {email}</span>
                                            </span>
                                            {admin.then(|| view! { <span class="label label-accent">"Admin"</span> })}
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

/// `/admin/orgs` — every organization on the instance.
#[component]
pub fn AdminOrgs() -> impl IntoView {
    let orgs = local_resource(
        || (),
        |_| async move { get::<Vec<Organization>>("/api/v1/admin/orgs").await },
    );

    view! {
        <div class="admin-users panel">
            <div class="page-header">
                <h2>"Organizations"</h2>
            </div>
            <ul class="item-list">
                <Suspense fallback=|| view! { <Loading label="Loading organizations…"/> }>
                    {move || orgs.get().map(|list| {
                        if list.is_empty() {
                            view! { <li class="text-muted">"No organizations."</li> }.into_any()
                        } else {
                            view! {
                                <For each=move || list.clone() key=|o| o.id.unwrap_or(0) children=move |o| {
                                    let name = o.name.clone().unwrap_or_default();
                                    let href = format!("/orgs/{name}");
                                    let description = o.description.clone().unwrap_or_default();
                                    view! {
                                        <li>
                                            <a href=href><strong>{name}</strong></a>
                                            <p class="mb-0 text-muted">{description}</p>
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

/// `/admin/emails` — every email address on the instance.
#[component]
pub fn AdminEmails() -> impl IntoView {
    let emails = local_resource(
        || (),
        |_| async move { get::<Vec<shared::Email>>("/api/v1/admin/emails").await },
    );

    view! {
        <div class="admin-users panel">
            <div class="page-header">
                <h2>"Emails"</h2>
            </div>
            <ul class="item-list">
                <Suspense fallback=|| view! { <Loading label="Loading emails…"/> }>
                    {move || emails.get().map(|list| {
                        if list.is_empty() {
                            view! { <li class="text-muted">"No email addresses."</li> }.into_any()
                        } else {
                            view! {
                                <For each=move || list.clone() key=|e| e.email.clone().unwrap_or_default() children=move |e| {
                                    let address = e.email.clone().unwrap_or_default();
                                    let user = e.username.clone().unwrap_or_default();
                                    let primary = e.primary.unwrap_or(false);
                                    view! {
                                        <li>
                                            <span>{address}</span>
                                            <span class="text-small text-muted">" " {user}</span>
                                            {primary.then(|| view! { <span class="label label-accent">" Primary"</span> })}
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
