//! Repository creation and migration.

use crate::api::{get, local_resource, post_json_resp, spawn, WRITE_ERROR};
use crate::components::FormError;
use leptos::prelude::*;
use leptos_router::hooks::*;
use shared::{CreateRepoOption, MigrateRepoOptions, Repository};

/// `/repo/create` — create a repository in the signed-in user's account.
#[component]
pub fn CreateRepo() -> impl IntoView {
    let navigate = use_navigate();
    let (name, set_name) = signal(String::new());
    let (description, set_description) = signal(String::new());
    let (private, set_private) = signal(false);
    let (auto_init, set_auto_init) = signal(true);
    let (gitignore, set_gitignore) = signal(String::new());
    let (license, set_license) = signal(String::new());
    let (error, set_error) = signal(Option::<String>::None);

    let gitignores = local_resource(
        || (),
        |_| async move { get::<Vec<String>>("/api/v1/gitignore/templates").await },
    );
    let licenses = local_resource(
        || (),
        |_| async move { get::<Vec<String>>("/api/v1/licenses").await },
    );

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let body = CreateRepoOption {
            name: name.get(),
            description: Some(description.get()),
            private: Some(private.get()),
            auto_init: Some(auto_init.get()),
            gitignores: Some(gitignore.get()).filter(|g| !g.is_empty()),
            license: Some(license.get()).filter(|l| !l.is_empty()),
            ..Default::default()
        };
        let navigate = navigate.clone();
        spawn(async move {
            match post_json_resp::<_, Repository>("/api/v1/user/repos", &body).await {
                Some(repo) => {
                    set_error.set(None);
                    let path = crate::components::repo_path(&repo);
                    navigate(&format!("/repos/{path}"), Default::default());
                }
                // Keep the typed name and description on a rejected write.
                None => set_error.set(Some(WRITE_ERROR.to_string())),
            }
        });
    };

    view! {
        <div class="create-repo container-narrow">
            <h2>"New repository"</h2>
            {move || error.get().map(|msg| view! { <FormError message=msg/> })}
            <form class="panel" on:submit=on_submit>
                <div class="form-group">
                    <label>"Repository name"</label>
                    <input type="text" prop:value=name
                        on:input=move |ev| set_name.set(event_target_value(&ev)) />
                </div>
                <div class="form-group">
                    <label>"Description"</label>
                    <textarea class="textarea-tall" prop:value=description
                        on:input=move |ev| set_description.set(event_target_value(&ev))></textarea>
                </div>
                <div class="form-group">
                    <label>".gitignore template"</label>
                    <select on:change=move |ev| set_gitignore.set(event_target_value(&ev))>
                        <option value="">"None"</option>
                        {move || gitignores.get().unwrap_or_default().into_iter().map(|name| {
                            view! { <option value=name.clone()>{name.clone()}</option> }
                        }).collect_view()}
                    </select>
                </div>
                <div class="form-group">
                    <label>"License"</label>
                    <select on:change=move |ev| set_license.set(event_target_value(&ev))>
                        <option value="">"None"</option>
                        {move || licenses.get().unwrap_or_default().into_iter().map(|name| {
                            view! { <option value=name.clone()>{name.clone()}</option> }
                        }).collect_view()}
                    </select>
                </div>
                <div class="checkbox-group">
                    <label>
                        <input type="checkbox" prop:checked=private
                            on:change=move |ev| set_private.set(event_target_checked(&ev)) />
                        " Private"
                    </label>
                    <label>
                        <input type="checkbox" prop:checked=auto_init
                            on:change=move |ev| set_auto_init.set(event_target_checked(&ev)) />
                        " Initialize with a README"
                    </label>
                </div>
                <div class="form-actions">
                    <button class="btn-primary" type="submit">"Create repository"</button>
                </div>
            </form>
        </div>
    }
}

/// `/repo/migrate` — import a repository from another host.
#[component]
pub fn MigrateRepo() -> impl IntoView {
    let navigate = use_navigate();
    let (clone_addr, set_clone_addr) = signal(String::new());
    let (repo_name, set_repo_name) = signal(String::new());
    let (description, set_description) = signal(String::new());
    let (private, set_private) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let body = MigrateRepoOptions {
            clone_addr: clone_addr.get(),
            repo_name: repo_name.get(),
            description: Some(description.get()),
            private: Some(private.get()),
            ..Default::default()
        };
        let navigate = navigate.clone();
        spawn(async move {
            match post_json_resp::<_, Repository>("/api/v1/repos/migrate", &body).await {
                Some(repo) => {
                    set_error.set(None);
                    let path = crate::components::repo_path(&repo);
                    navigate(&format!("/repos/{path}"), Default::default());
                }
                None => set_error.set(Some(WRITE_ERROR.to_string())),
            }
        });
    };

    view! {
        <div class="migrate-repo container-narrow">
            <h2>"Migrate repository"</h2>
            {move || error.get().map(|msg| view! { <FormError message=msg/> })}
            <form class="panel" on:submit=on_submit>
                <div class="form-group">
                    <label>"Clone address"</label>
                    <input type="text" placeholder="https://example.com/owner/repo.git"
                        prop:value=clone_addr
                        on:input=move |ev| set_clone_addr.set(event_target_value(&ev)) />
                </div>
                <div class="form-group">
                    <label>"Repository name"</label>
                    <input type="text" prop:value=repo_name
                        on:input=move |ev| set_repo_name.set(event_target_value(&ev)) />
                </div>
                <div class="form-group">
                    <label>"Description"</label>
                    <textarea class="textarea-tall" prop:value=description
                        on:input=move |ev| set_description.set(event_target_value(&ev))></textarea>
                </div>
                <div class="checkbox-group">
                    <label>
                        <input type="checkbox" prop:checked=private
                            on:change=move |ev| set_private.set(event_target_checked(&ev)) />
                        " Private"
                    </label>
                </div>
                <div class="form-actions">
                    <button class="btn-primary" type="submit">"Migrate repository"</button>
                </div>
            </form>
        </div>
    }
}
