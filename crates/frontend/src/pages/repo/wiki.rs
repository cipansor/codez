//! Repository wiki: page listing, viewing and editing.
//!
//! Gitea's wiki is git-backed, so the API only exposes page metadata and
//! content. This port renders what the endpoints return and, when a wiki has no
//! pages yet, offers to create the first one.

use crate::api::{
    decode_base64, encode_base64, get, get_or, local_resource, post_json, spawn, WRITE_ERROR,
};
use crate::components::{FormError, Loading, RepoNav};
use leptos::prelude::*;
use leptos_router::hooks::*;
use shared::WikiPage;

/// `/repos/:owner/:repo/wiki` and `/wiki/pages/:page_name` — the wiki sidebar
/// and the selected page.
#[component]
pub fn Wiki() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let repo = move || params.with(|p| p.get("repo").unwrap_or_default());
    let page_name = move || params.with(|p| p.get("page_name").unwrap_or_default());

    let pages = local_resource(
        move || (owner(), repo()),
        |(o, r)| async move { get::<Vec<WikiPage>>(&format!("/api/v1/repos/{o}/{r}/wiki/pages")).await },
    );

    let page = local_resource(
        move || (owner(), repo(), page_name()),
        |(o, r, name)| async move {
            if name.is_empty() {
                return None;
            }
            get_or::<Option<WikiPage>>(&format!("/api/v1/repos/{o}/{r}/wiki/page/{name}"), None)
                .await
        },
    );

    view! {
        <div class="wiki-container">
            <RepoNav/>
            <div class="wiki-sidebar panel">
                <h4>"Pages"</h4>
                <ul class="item-list">
                    <Suspense fallback=|| view! { <Loading label="Loading…"/> }>
                        {move || pages.get().map(|list| {
                            if list.is_empty() {
                                view! { <li class="text-muted">"No wiki pages yet."</li> }.into_any()
                            } else {
                                let o = owner();
                                let r = repo();
                                view! {
                                    <For each=move || list.clone() key=|p| p.title.clone().unwrap_or_default() children=move |p| {
                                        let title = p.title.clone().unwrap_or_default();
                                        let href = format!("/repos/{o}/{r}/wiki/pages/{title}");
                                        view! { <li><a href=href>{title.clone()}</a></li> }
                                    }/>
                                }.into_any()
                            }
                        })}
                    </Suspense>
                </ul>
                {move || {
                    let o = owner();
                    let r = repo();
                    let name = page_name();
                    if name.is_empty() {
                        view! {
                            <a class="btn-sm" href=format!("/repos/{o}/{r}/wiki/pages/Home/edit")>"New page"</a>
                        }.into_any()
                    } else {
                        view! {
                            <a class="btn-sm" href=format!("/repos/{o}/{r}/wiki/pages/{name}/edit")>"Edit this page"</a>
                        }.into_any()
                    }
                }}
            </div>

            <div class="wiki-content panel">
                <Suspense fallback=|| view! { <p class="text-muted">"Loading page…"</p> }>
                    {move || match page.get().flatten() {
                        Some(p) => {
                            let title = p.title.clone().unwrap_or_default();
                            let content = decode_base64(p.content_base64.as_deref().unwrap_or_default());
                            let commits = p.commit_count.unwrap_or(0);
                            view! {
                                <h3>{title}</h3>
                                <p class="text-small text-muted">{commits} " revisions"</p>
                                <pre class="wiki-view">{content}</pre>
                            }.into_any()
                        }
                        None => view! {
                            <div class="empty-state">
                                "Select a page from the sidebar, or create the first one."
                            </div>
                        }.into_any(),
                    }}
                </Suspense>
            </div>
        </div>
    }
}

/// `/repos/:owner/:repo/wiki/pages/:page_name/edit` — create or update a page.
#[component]
pub fn WikiEdit() -> impl IntoView {
    let params = use_params_map();
    let navigate = use_navigate();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let repo = move || params.with(|p| p.get("repo").unwrap_or_default());
    let page_name = move || params.with(|p| p.get("page_name").unwrap_or_default());

    let (content, set_content) = signal(String::new());
    let (message, set_message) = signal(String::new());
    let (error, set_error) = signal(Option::<String>::None);

    // Pre-fill from the existing page, if any.
    spawn(async move {
        let o = owner();
        let r = repo();
        let name = page_name();
        let existing =
            get_or::<Option<WikiPage>>(&format!("/api/v1/repos/{o}/{r}/wiki/page/{name}"), None)
                .await;
        if let Some(page) = existing {
            set_content.set(decode_base64(
                page.content_base64.as_deref().unwrap_or_default(),
            ));
        }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let o = owner();
        let r = repo();
        let name = page_name();
        let payload = shared::CreateWikiPageOptions {
            title: Some(name.clone()),
            content_base64: Some(encode_base64(&content.get())),
            message: Some(message.get()),
        };
        let navigate = navigate.clone();
        spawn(async move {
            if post_json(&format!("/api/v1/repos/{o}/{r}/wiki/new"), &payload).await {
                set_error.set(None);
                navigate(
                    &format!("/repos/{o}/{r}/wiki/pages/{name}"),
                    Default::default(),
                );
            } else {
                // Keep the typed body so a rejected save is not lost.
                set_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    view! {
        <div class="wiki-edit">
            <RepoNav/>
            <h3>"Editing " <code>{page_name}</code></h3>
            {move || error.get().map(|msg| view! { <FormError message=msg/> })}
            <form on:submit=on_submit>
                <div class="form-group">
                    <textarea class="textarea-tall input-title" prop:value=content
                        on:input=move |ev| set_content.set(event_target_value(&ev))></textarea>
                </div>
                <div class="form-group">
                    <label>"Commit message"</label>
                    <input type="text" prop:value=message
                        on:input=move |ev| set_message.set(event_target_value(&ev)) />
                </div>
                <div class="form-actions">
                    <button class="btn-primary" type="submit">"Save page"</button>
                </div>
            </form>
        </div>
    }
}
