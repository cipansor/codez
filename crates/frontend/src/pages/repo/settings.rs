//! Repository settings: general options, collaborators, webhooks, secrets,
//! deploy keys, protected branches and LFS locks.

use crate::api::{get, get_or, local_resource, patch_json, put_json, spawn, WRITE_ERROR};
use crate::components::{FormError, Loading, RepoNav};
use leptos::prelude::*;
use leptos_router::hooks::*;
use shared::{AddCollaboratorOption, DeployKey, EditRepoOption, Hook, Repository, Secret, User};

/// `/repos/:owner/:repo/settings` — the repository's general options.
#[component]
pub fn RepoSettings() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let repo = move || params.with(|p| p.get("repo").unwrap_or_default());

    let (description, set_description) = signal(String::new());
    let (website, set_website) = signal(String::new());
    let (private, set_private) = signal(false);
    let (has_issues, set_has_issues) = signal(true);
    let (has_wiki, set_has_wiki) = signal(true);
    let (has_projects, set_has_projects) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);
    let (saved, set_saved) = signal(false);
    let (refresh, set_refresh) = signal(0u32);

    // Seed the form from the current repository once it has loaded.
    let current = local_resource(
        move || (owner(), repo(), refresh.get()),
        |(o, r, _)| async move {
            get_or::<Option<Repository>>(&format!("/api/v1/repos/{o}/{r}"), None).await
        },
    );
    Effect::new(move |_| {
        if let Some(Some(r)) = current.get() {
            set_description.set(r.description.clone().unwrap_or_default());
            set_website.set(r.website.clone().unwrap_or_default());
            set_private.set(r.private.unwrap_or(false));
            set_has_issues.set(r.has_issues.unwrap_or(true));
            set_has_wiki.set(r.has_wiki.unwrap_or(true));
            set_has_projects.set(r.has_projects.unwrap_or(true));
        }
    });

    let on_save = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let body = EditRepoOption {
            description: Some(description.get()),
            website: Some(website.get()),
            private: Some(private.get()),
            has_issues: Some(has_issues.get()),
            has_wiki: Some(has_wiki.get()),
            has_projects: Some(has_projects.get()),
            ..Default::default()
        };
        let o = owner();
        let r = repo();
        spawn(async move {
            if patch_json(&format!("/api/v1/repos/{o}/{r}"), &body).await {
                set_error.set(None);
                set_saved.set(true);
                set_refresh.update(|n| *n += 1);
            } else {
                set_saved.set(false);
                set_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    view! {
        <div class="repo-settings">
            <RepoNav/>
            <h3>"Settings"</h3>
            {move || error.get().map(|msg| view! { <FormError message=msg/> })}
            {move || saved.get().then(|| view! { <p class="flash-success">"Settings saved."</p> })}

            <form class="panel" on:submit=on_save>
                <div class="form-group">
                    <label>"Description"</label>
                    <textarea class="textarea-tall" prop:value=description
                        on:input=move |ev| set_description.set(event_target_value(&ev))></textarea>
                </div>
                <div class="form-group">
                    <label>"Website"</label>
                    <input type="text" prop:value=website
                        on:input=move |ev| set_website.set(event_target_value(&ev)) />
                </div>
                <div class="checkbox-group">
                    <label>
                        <input type="checkbox" prop:checked=private
                            on:change=move |ev| set_private.set(event_target_checked(&ev)) />
                        " Private"
                    </label>
                    <label>
                        <input type="checkbox" prop:checked=has_issues
                            on:change=move |ev| set_has_issues.set(event_target_checked(&ev)) />
                        " Issues"
                    </label>
                    <label>
                        <input type="checkbox" prop:checked=has_wiki
                            on:change=move |ev| set_has_wiki.set(event_target_checked(&ev)) />
                        " Wiki"
                    </label>
                    <label>
                        <input type="checkbox" prop:checked=has_projects
                            on:change=move |ev| set_has_projects.set(event_target_checked(&ev)) />
                        " Projects"
                    </label>
                </div>
                <div class="form-actions">
                    <button class="btn-primary" type="submit">"Save settings"</button>
                </div>
            </form>

            <CollabList/>
            <WebhookList/>
            <SecretList/>
            <DeployKeyList/>
            <ProtectedBranchList/>
        </div>
    }
}

/// Collaborators with their permission level.
#[component]
fn CollabList() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let repo = move || params.with(|p| p.get("repo").unwrap_or_default());
    let (refresh, set_refresh) = signal(0u32);
    let (login, set_login) = signal(String::new());
    let (permission, set_permission) = signal("write".to_string());
    let (error, set_error) = signal(Option::<String>::None);

    let collaborators = local_resource(
        move || (owner(), repo(), refresh.get()),
        |(o, r, _)| async move {
            get::<Vec<User>>(&format!("/api/v1/repos/{o}/{r}/collaborators")).await
        },
    );

    let on_add = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let body = AddCollaboratorOption {
            permission: Some(shared::RepoWritePermission::from_wire(&permission.get())),
        };
        let o = owner();
        let r = repo();
        let target = login.get();
        spawn(async move {
            if put_json(
                &format!("/api/v1/repos/{o}/{r}/collaborators/{target}"),
                &body,
            )
            .await
            {
                set_error.set(None);
                set_login.set(String::new());
                set_refresh.update(|n| *n += 1);
            } else {
                // Keep the typed username so the add can be retried.
                set_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    view! {
        <section class="panel collaborators">
            <h4>"Collaborators"</h4>
            {move || error.get().map(|msg| view! { <FormError message=msg/> })}
            <form class="flex-center gap-sm" on:submit=on_add>
                <input class="input-lg" type="text" placeholder="Username"
                    prop:value=login on:input=move |ev| set_login.set(event_target_value(&ev)) />
                <select on:change=move |ev| set_permission.set(event_target_value(&ev))>
                    <option value="read">"Read"</option>
                    <option value="write">"Write"</option>
                    <option value="admin">"Admin"</option>
                </select>
                <button class="btn-primary" type="submit">"Add collaborator"</button>
            </form>
            <ul class="item-list">
                <Suspense fallback=|| view! { <Loading label="Loading collaborators…"/> }>
                    {move || collaborators.get().map(|list| {
                        if list.is_empty() {
                            view! { <li class="text-muted">"No collaborators."</li> }.into_any()
                        } else {
                            view! {
                                <For each=move || list.clone() key=|u| u.id.unwrap_or(0) children=move |u| {
                                    let login = u.login.clone().unwrap_or_default();
                                    view! { <li><strong>{login}</strong></li> }
                                }/>
                            }.into_any()
                        }
                    })}
                </Suspense>
            </ul>
        </section>
    }
}

/// Repository webhooks.
#[component]
fn WebhookList() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let repo = move || params.with(|p| p.get("repo").unwrap_or_default());

    let hooks = local_resource(
        move || (owner(), repo()),
        |(o, r)| async move { get::<Vec<Hook>>(&format!("/api/v1/repos/{o}/{r}/hooks")).await },
    );

    view! {
        <section class="panel webhook-list">
            <h4>"Webhooks"</h4>
            <ul class="item-list">
                <Suspense fallback=|| view! { <Loading label="Loading webhooks…"/> }>
                    {move || hooks.get().map(|list| {
                        if list.is_empty() {
                            view! { <li class="text-muted">"No webhooks configured."</li> }.into_any()
                        } else {
                            view! {
                                <For each=move || list.clone() key=|h| h.id.unwrap_or(0) children=move |h| {
                                    let kind = h.type_.clone().unwrap_or_default();
                                    let active = h.active.unwrap_or(false);
                                    view! {
                                        <li class="flex-between">
                                            <span><strong>{kind}</strong></span>
                                            <span class="label">{if active { "active" } else { "inactive" }}</span>
                                        </li>
                                    }
                                }/>
                            }.into_any()
                        }
                    })}
                </Suspense>
            </ul>
        </section>
    }
}

/// Repository actions secrets.
#[component]
fn SecretList() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let repo = move || params.with(|p| p.get("repo").unwrap_or_default());
    let (refresh, set_refresh) = signal(0u32);
    let (name, set_name) = signal(String::new());
    let (data, set_data) = signal(String::new());
    let (error, set_error) = signal(Option::<String>::None);

    let secrets = local_resource(
        move || (owner(), repo(), refresh.get()),
        |(o, r, _)| async move {
            get::<Vec<Secret>>(&format!("/api/v1/repos/{o}/{r}/actions/secrets")).await
        },
    );

    let on_add = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let body = shared::CreateOrUpdateSecretOption {
            data: data.get(),
            description: Some(String::new()),
        };
        let o = owner();
        let r = repo();
        let target = name.get();
        spawn(async move {
            if put_json(
                &format!("/api/v1/repos/{o}/{r}/actions/secrets/{target}"),
                &body,
            )
            .await
            {
                set_error.set(None);
                set_name.set(String::new());
                set_data.set(String::new());
                set_refresh.update(|n| *n += 1);
            } else {
                set_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    view! {
        <section class="panel secret-list">
            <h4>"Actions secrets"</h4>
            {move || error.get().map(|msg| view! { <FormError message=msg/> })}
            <form class="flex-center gap-sm" on:submit=on_add>
                <input class="input-lg" type="text" placeholder="Secret name"
                    prop:value=name on:input=move |ev| set_name.set(event_target_value(&ev)) />
                <input class="input-lg" type="password" placeholder="Value"
                    prop:value=data on:input=move |ev| set_data.set(event_target_value(&ev)) />
                <button class="btn-primary" type="submit">"Add secret"</button>
            </form>
            <ul class="item-list">
                <Suspense fallback=|| view! { <Loading label="Loading secrets…"/> }>
                    {move || secrets.get().map(|list| {
                        if list.is_empty() {
                            view! { <li class="text-muted">"No secrets configured."</li> }.into_any()
                        } else {
                            view! {
                                <For each=move || list.clone() key=|s| s.name.clone().unwrap_or_default() children=move |s| {
                                    let name = s.name.clone().unwrap_or_default();
                                    let created = s.created_at.clone().unwrap_or_default();
                                    view! { <li><strong>{name}</strong> <span class="text-small text-muted">{created}</span></li> }
                                }/>
                            }.into_any()
                        }
                    })}
                </Suspense>
            </ul>
        </section>
    }
}

/// Deploy keys.
#[component]
fn DeployKeyList() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let repo = move || params.with(|p| p.get("repo").unwrap_or_default());

    let keys = local_resource(
        move || (owner(), repo()),
        |(o, r)| async move { get::<Vec<DeployKey>>(&format!("/api/v1/repos/{o}/{r}/keys")).await },
    );

    view! {
        <section class="panel deploy-key-list">
            <h4>"Deploy keys"</h4>
            <ul class="item-list">
                <Suspense fallback=|| view! { <Loading label="Loading keys…"/> }>
                    {move || keys.get().map(|list| {
                        if list.is_empty() {
                            view! { <li class="text-muted">"No deploy keys."</li> }.into_any()
                        } else {
                            view! {
                                <For each=move || list.clone() key=|k| k.id.unwrap_or(0) children=move |k| {
                                    let title = k.title.clone().unwrap_or_default();
                                    let fingerprint = k.fingerprint.clone().unwrap_or_default();
                                    let read_only = k.read_only.unwrap_or(false);
                                    view! {
                                        <li class="flex-between">
                                            <span><strong>{title}</strong> <code class="text-small">{fingerprint}</code></span>
                                            <span class="label">{if read_only { "read-only" } else { "read-write" }}</span>
                                        </li>
                                    }
                                }/>
                            }.into_any()
                        }
                    })}
                </Suspense>
            </ul>
        </section>
    }
}

/// Branch protection rules.
#[component]
fn ProtectedBranchList() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let repo = move || params.with(|p| p.get("repo").unwrap_or_default());

    let protections = local_resource(
        move || (owner(), repo()),
        |(o, r)| async move {
            get::<Vec<shared::BranchProtection>>(&format!(
                "/api/v1/repos/{o}/{r}/branch_protections"
            ))
            .await
        },
    );

    view! {
        <section class="panel protected-branches">
            <h4>"Protected branches"</h4>
            <ul class="item-list">
                <Suspense fallback=|| view! { <Loading label="Loading rules…"/> }>
                    {move || protections.get().map(|list| {
                        if list.is_empty() {
                            view! { <li class="text-muted">"No protected branches."</li> }.into_any()
                        } else {
                            view! {
                                <For each=move || list.clone() key=|p| p.branch_name.clone().unwrap_or_default() children=move |p| {
                                    let name = p.branch_name.clone().unwrap_or_default();
                                    let approvals = p.required_approvals.unwrap_or(0);
                                    let push = p.enable_push.unwrap_or(false);
                                    view! {
                                        <li class="flex-between">
                                            <span><strong>{name}</strong></span>
                                            <span class="text-small text-muted">
                                                {approvals} " required approvals · "
                                                {if push { "push enabled" } else { "push disabled" }}
                                            </span>
                                        </li>
                                    }
                                }/>
                            }.into_any()
                        }
                    })}
                </Suspense>
            </ul>
        </section>
    }
}
