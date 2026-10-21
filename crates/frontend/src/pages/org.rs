//! Organization profile, membership, teams and creation.

use crate::api::{get, get_or, local_resource, post_json_resp, spawn, WRITE_ERROR};
use crate::components::{repo_href, FormError, Loading};
use leptos::prelude::*;
use leptos_router::hooks::*;
use shared::{CreateOrgOption, Organization, Repository, Team, User};

/// `/orgs/:org` — an organization's profile, repositories, members and teams.
#[component]
pub fn OrgProfile() -> impl IntoView {
    let params = use_params_map();
    let org_name = move || params.with(|p| p.get("org").unwrap_or_default());

    let org = local_resource(org_name, |name| async move {
        get_or::<Option<Organization>>(&format!("/api/v1/orgs/{name}"), None).await
    });

    let repos = local_resource(org_name, |name| async move {
        get::<Vec<Repository>>(&format!("/api/v1/orgs/{name}/repos")).await
    });

    let members = local_resource(org_name, |name| async move {
        get::<Vec<User>>(&format!("/api/v1/orgs/{name}/members")).await
    });

    let teams = local_resource(org_name, |name| async move {
        get::<Vec<Team>>(&format!("/api/v1/orgs/{name}/teams")).await
    });

    view! {
        <div class="org-profile">
            <Suspense fallback=|| view! { <p class="text-muted">"Loading organization…"</p> }>
                {move || org.get().flatten().map(|o| {
                    let name = o.name.clone().unwrap_or_default();
                    let full_name = o.full_name.clone().unwrap_or_default();
                    let description = o.description.clone().unwrap_or_default();
                    let location = o.location.clone().unwrap_or_default();
                    let website = o.website.clone().unwrap_or_default();
                    view! {
                        <div class="org-header page-header">
                            <h2>{full_name} <span class="text-muted text-lg">{name}</span></h2>
                            <p class="text-muted">{description}</p>
                            <p class="text-small text-muted">{location} " " {website}</p>
                        </div>
                    }
                })}
            </Suspense>

            <h3 class="mt-3">"Repositories"</h3>
            <ul class="item-list panel">
                <Suspense fallback=|| view! { <Loading label="Loading repositories…"/> }>
                    {move || repos.get().map(|list| {
                        if list.is_empty() {
                            view! { <li class="text-muted">"No repositories."</li> }.into_any()
                        } else {
                            view! {
                                <For each=move || list.clone() key=|r| r.id.unwrap_or(0) children=move |r| {
                                    let href = repo_href(&r);
                                    let path = crate::components::repo_path(&r);
                                    let description = r.description.clone().unwrap_or_default();
                                    view! {
                                        <li>
                                            <a href=href><strong>{path}</strong></a>
                                            <p class="mb-0 text-muted">{description}</p>
                                        </li>
                                    }
                                }/>
                            }.into_any()
                        }
                    })}
                </Suspense>
            </ul>

            <h3 class="mt-3">"Members"</h3>
            <ul class="item-list panel">
                <Suspense fallback=|| view! { <Loading label="Loading members…"/> }>
                    {move || members.get().map(|list| {
                        if list.is_empty() {
                            view! { <li class="text-muted">"No members."</li> }.into_any()
                        } else {
                            view! {
                                <For each=move || list.clone() key=|u| u.id.unwrap_or(0) children=move |u| {
                                    let login = u.login.clone().unwrap_or_default();
                                    let href = format!("/users/{login}");
                                    view! { <li><a href=href><strong>{login}</strong></a></li> }
                                }/>
                            }.into_any()
                        }
                    })}
                </Suspense>
            </ul>

            <h3 class="mt-3">"Teams"</h3>
            <ul class="item-list panel">
                <Suspense fallback=|| view! { <Loading label="Loading teams…"/> }>
                    {move || teams.get().map(|list| {
                        if list.is_empty() {
                            view! { <li class="text-muted">"No teams."</li> }.into_any()
                        } else {
                            view! {
                                <For each=move || list.clone() key=|t| t.id.unwrap_or(0) children=move |t| {
                                    let name = t.name.clone().unwrap_or_default();
                                    let description = t.description.clone().unwrap_or_default();
                                    let permission = t.permission.as_ref().map(|p| p.as_wire().to_string()).unwrap_or_default();
                                    view! {
                                        <li>
                                            <strong>{name}</strong>
                                            <span class="label">{permission}</span>
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

/// `/org/create` — create an organization.
#[component]
pub fn CreateOrg() -> impl IntoView {
    let (username, set_username) = signal(String::new());
    let (full_name, set_full_name) = signal(String::new());
    let (description, set_description) = signal(String::new());
    let (email, set_email) = signal(String::new());
    let (error, set_error) = signal(Option::<String>::None);
    let navigate = use_navigate();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let body = CreateOrgOption {
            username: username.get(),
            full_name: Some(full_name.get()),
            description: Some(description.get()),
            email: Some(email.get()),
            ..Default::default()
        };
        let navigate = navigate.clone();
        spawn(async move {
            match post_json_resp::<_, Organization>("/api/v1/orgs", &body).await {
                Some(org) => {
                    set_error.set(None);
                    let name = org.name.unwrap_or_default();
                    navigate(&format!("/orgs/{name}"), Default::default());
                }
                // Keep the typed values so the user can correct them.
                None => set_error.set(Some(WRITE_ERROR.to_string())),
            }
        });
    };

    view! {
        <div class="create-org container-narrow">
            <h2>"New organization"</h2>
            {move || error.get().map(|msg| view! { <FormError message=msg/> })}
            <form class="panel" on:submit=on_submit>
                <div class="form-group">
                    <label>"Organization name"</label>
                    <input type="text" prop:value=username
                        on:input=move |ev| set_username.set(event_target_value(&ev)) />
                </div>
                <div class="form-group">
                    <label>"Display name"</label>
                    <input type="text" prop:value=full_name
                        on:input=move |ev| set_full_name.set(event_target_value(&ev)) />
                </div>
                <div class="form-group">
                    <label>"Email"</label>
                    <input type="email" prop:value=email
                        on:input=move |ev| set_email.set(event_target_value(&ev)) />
                </div>
                <div class="form-group">
                    <label>"Description"</label>
                    <textarea class="textarea-tall" prop:value=description
                        on:input=move |ev| set_description.set(event_target_value(&ev))></textarea>
                </div>
                <div class="form-actions">
                    <button class="btn-primary" type="submit">"Create organization"</button>
                </div>
            </form>
        </div>
    }
}
