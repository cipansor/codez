//! Release listing, detail and creation.

use crate::api::{get, get_or, local_resource, post_json_resp, spawn, WRITE_ERROR};
use crate::components::{FormError, Loading, RepoNav};
use leptos::prelude::*;
use leptos_router::hooks::*;
use shared::{CreateReleaseOption, Release};

/// `/repos/:owner/:repo/releases` — every release, newest first.
#[component]
pub fn ReleaseList() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let repo = move || params.with(|p| p.get("repo").unwrap_or_default());

    let releases = local_resource(
        move || (owner(), repo()),
        |(o, r)| async move { get::<Vec<Release>>(&format!("/api/v1/repos/{o}/{r}/releases")).await },
    );

    view! {
        <div class="release-list">
            <RepoNav/>
            <div class="page-header">
                <h3>"Releases"</h3>
                {move || view! {
                    <a class="btn-primary" href=move || format!("/repos/{}/{}/releases/new", owner(), repo())>
                        "New release"
                    </a>
                }}
            </div>
            <ul class="item-list panel">
                <Suspense fallback=|| view! { <Loading label="Loading releases…"/> }>
                    {move || releases.get().map(|list| {
                        if list.is_empty() {
                            view! { <li class="text-muted">"No releases yet."</li> }.into_any()
                        } else {
                            let o = owner();
                            let r = repo();
                            view! {
                                <For each=move || list.clone() key=|rel| rel.id.unwrap_or(0) children=move |rel| {
                                    let id = rel.id.unwrap_or(0);
                                    let tag = rel.tag_name.clone().unwrap_or_default();
                                    let name = rel.name.clone().filter(|n| !n.is_empty()).unwrap_or_else(|| tag.clone());
                                    let href = format!("/repos/{o}/{r}/releases/{id}");
                                    let body = crate::components::first_line(&rel.body, 140);
                                    let draft = rel.draft.unwrap_or(false);
                                    let prerelease = rel.prerelease.unwrap_or(false);
                                    view! {
                                        <li>
                                            <a href=href><strong>{name}</strong></a>
                                            <span class="label">{tag}</span>
                                            {draft.then(|| view! { <span class="label label-warning">" Draft"</span> })}
                                            {prerelease.then(|| view! { <span class="label label-attention">" Pre-release"</span> })}
                                            <p class="mb-0 text-muted">{body}</p>
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

/// `/repos/:owner/:repo/releases/:id` — one release and its assets.
#[component]
pub fn ReleaseDetail() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let repo = move || params.with(|p| p.get("repo").unwrap_or_default());
    let id = move || params.with(|p| p.get("id").unwrap_or_default());

    let release = local_resource(
        move || (owner(), repo(), id()),
        |(o, r, i)| async move {
            get_or::<Option<Release>>(&format!("/api/v1/repos/{o}/{r}/releases/{i}"), None).await
        },
    );

    view! {
        <div class="release-detail">
            <RepoNav/>
            <Suspense fallback=|| view! { <p class="text-muted">"Loading release…"</p> }>
                {move || release.get().flatten().map(|rel| {
                    let tag = rel.tag_name.clone().unwrap_or_default();
                    let name = rel.name.clone().filter(|n| !n.is_empty()).unwrap_or_else(|| tag.clone());
                    let body = rel.body.clone().unwrap_or_default();
                    let published = rel.published_at.clone().or(rel.created_at.clone()).unwrap_or_default();
                    let assets = rel.assets.clone().unwrap_or_default();
                    view! {
                        <div class="page-header">
                            <h3>{name} <span class="label">{tag}</span></h3>
                            <span class="text-small text-muted">{published}</span>
                        </div>
                        <div class="release-body">{body}</div>
                        <h4>"Assets"</h4>
                        <ul class="item-list assets-section">
                            {if assets.is_empty() {
                                view! { <li class="text-muted">"No assets attached."</li> }.into_any()
                            } else {
                                assets.into_iter().map(|a| {
                                    let name = a.name.clone().unwrap_or_default();
                                    let size = a.size.unwrap_or(0);
                                    let url = a.browser_download_url.clone().unwrap_or_default();
                                    view! {
                                        <li>
                                            <a href=url><strong>{name}</strong></a>
                                            <span class="text-small text-muted">" " {size} " bytes"</span>
                                        </li>
                                    }
                                }).collect_view().into_any()
                            }}
                        </ul>
                    }
                })}
            </Suspense>
        </div>
    }
}

/// `/repos/:owner/:repo/releases/new` — publish a release.
#[component]
pub fn ReleaseCreate() -> impl IntoView {
    let params = use_params_map();
    let navigate = use_navigate();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let repo = move || params.with(|p| p.get("repo").unwrap_or_default());

    let (tag_name, set_tag_name) = signal(String::new());
    let (name, set_name) = signal(String::new());
    let (body, set_body) = signal(String::new());
    let (target, set_target) = signal(String::new());
    let (draft, set_draft) = signal(false);
    let (prerelease, set_prerelease) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let payload = CreateReleaseOption {
            tag_name: tag_name.get(),
            name: Some(name.get()),
            body: Some(body.get()),
            target_commitish: Some(target.get()),
            draft: Some(draft.get()),
            prerelease: Some(prerelease.get()),
            ..Default::default()
        };
        let o = owner();
        let r = repo();
        let navigate = navigate.clone();
        spawn(async move {
            match post_json_resp::<_, Release>(&format!("/api/v1/repos/{o}/{r}/releases"), &payload)
                .await
            {
                Some(release) => {
                    set_error.set(None);
                    let id = release.id.unwrap_or(0);
                    navigate(&format!("/repos/{o}/{r}/releases/{id}"), Default::default());
                }
                // Keep the typed fields so the user can fix them.
                None => set_error.set(Some(WRITE_ERROR.to_string())),
            }
        });
    };

    view! {
        <div class="release-create">
            <RepoNav/>
            <h3>"New release"</h3>
            {move || error.get().map(|msg| view! { <FormError message=msg/> })}
            <form class="panel" on:submit=on_submit>
                <div class="form-group">
                    <label>"Tag name"</label>
                    <input type="text" prop:value=tag_name
                        on:input=move |ev| set_tag_name.set(event_target_value(&ev)) />
                </div>
                <div class="form-group">
                    <label>"Release title"</label>
                    <input type="text" prop:value=name
                        on:input=move |ev| set_name.set(event_target_value(&ev)) />
                </div>
                <div class="form-group">
                    <label>"Target branch"</label>
                    <input type="text" prop:value=target
                        on:input=move |ev| set_target.set(event_target_value(&ev)) />
                </div>
                <div class="form-group">
                    <label>"Description"</label>
                    <textarea class="textarea-tall" prop:value=body
                        on:input=move |ev| set_body.set(event_target_value(&ev))></textarea>
                </div>
                <div class="checkbox-group">
                    <label>
                        <input type="checkbox" prop:checked=draft
                            on:change=move |ev| set_draft.set(event_target_checked(&ev)) />
                        " Draft"
                    </label>
                    <label>
                        <input type="checkbox" prop:checked=prerelease
                            on:change=move |ev| set_prerelease.set(event_target_checked(&ev)) />
                        " Pre-release"
                    </label>
                </div>
                <div class="form-actions">
                    <button class="btn-primary" type="submit">"Publish release"</button>
                </div>
            </form>
        </div>
    }
}
