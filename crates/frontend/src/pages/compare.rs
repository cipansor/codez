//! Branch comparison and pull-request creation.

use crate::api::{get, local_resource, post_json_resp, spawn, WRITE_ERROR};
use crate::components::{FormError, RepoNav};
use leptos::prelude::*;
use leptos_router::hooks::*;
use shared::{Branch, CreatePullRequestOption, PullRequest};

/// `/repos/:owner/:repo/compare` — pick a base and head branch, then open a
/// pull request from them.
#[component]
pub fn CompareView() -> impl IntoView {
    let params = use_params_map();
    let navigate = use_navigate();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let repo = move || params.with(|p| p.get("repo").unwrap_or_default());

    let (base_branch, set_base_branch) = signal(String::new());
    let (head_branch, set_head_branch) = signal(String::new());
    let (title, set_title) = signal(String::new());
    let (body, set_body) = signal(String::new());
    let (error, set_error) = signal(Option::<String>::None);

    let branches = local_resource(
        move || (owner(), repo()),
        |(o, r)| async move { get::<Vec<Branch>>(&format!("/api/v1/repos/{o}/{r}/branches")).await },
    );

    // Seed the selectors once the branch list arrives: the repository's default
    // branch (or `main`) as the base, the first other branch as the head.
    Effect::new(move |_| {
        if let Some(list) = branches.get() {
            if base_branch.get_untracked().is_empty() {
                if let Some(name) = list
                    .iter()
                    .find(|b| b.name.as_deref() == Some("main"))
                    .or_else(|| list.first())
                    .and_then(|b| b.name.clone())
                {
                    set_base_branch.set(name);
                }
            }
            if head_branch.get_untracked().is_empty() {
                if let Some(name) = list
                    .iter()
                    .find(|b| b.name.as_deref() != Some(base_branch.get_untracked().as_str()))
                    .and_then(|b| b.name.clone())
                {
                    set_head_branch.set(name);
                }
            }
        }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let base = base_branch.get();
        let head = head_branch.get();
        if base == head {
            set_error.set(Some(
                "Base and compare branches must be different.".to_string(),
            ));
            return;
        }
        let payload = CreatePullRequestOption {
            title: Some(title.get()),
            body: Some(body.get()),
            head: Some(head),
            base: Some(base),
            ..Default::default()
        };
        let o = owner();
        let r = repo();
        let navigate = navigate.clone();
        spawn(async move {
            match post_json_resp::<_, PullRequest>(
                &format!("/api/v1/repos/{o}/{r}/pulls"),
                &payload,
            )
            .await
            {
                Some(pr) => {
                    set_error.set(None);
                    let number = pr.number.unwrap_or(0);
                    navigate(
                        &format!("/repos/{o}/{r}/pulls/{number}"),
                        Default::default(),
                    );
                }
                // Keep the typed title and body on a rejected write.
                None => set_error.set(Some(WRITE_ERROR.to_string())),
            }
        });
    };

    view! {
        <div class="compare-view">
            <RepoNav/>
            <h3>"Compare changes"</h3>
            <p class="text-muted">"Choose two branches to see what's changed, or open a pull request."</p>
            {move || error.get().map(|msg| view! { <FormError message=msg/> })}

            <form class="panel" on:submit=on_submit>
                <div class="form-group">
                    <label>"Base branch"</label>
                    <select on:change=move |ev| set_base_branch.set(event_target_value(&ev))>
                        {move || branches.get().unwrap_or_default().into_iter().map(|b| {
                            let name = b.name.clone().unwrap_or_default();
                            let selected = name == base_branch.get();
                            view! { <option value=name.clone() selected=selected>{name.clone()}</option> }
                        }).collect_view()}
                    </select>
                </div>
                <div class="form-group">
                    <label>"Compare branch"</label>
                    <select on:change=move |ev| set_head_branch.set(event_target_value(&ev))>
                        {move || branches.get().unwrap_or_default().into_iter().map(|b| {
                            let name = b.name.clone().unwrap_or_default();
                            let selected = name == head_branch.get();
                            view! { <option value=name.clone() selected=selected>{name.clone()}</option> }
                        }).collect_view()}
                    </select>
                </div>
                <div class="form-group">
                    <label>"Title"</label>
                    <input type="text" prop:value=title
                        on:input=move |ev| set_title.set(event_target_value(&ev)) />
                </div>
                <div class="form-group">
                    <label>"Description"</label>
                    <textarea class="textarea-tall" prop:value=body
                        on:input=move |ev| set_body.set(event_target_value(&ev))></textarea>
                </div>
                <div class="form-actions">
                    <button class="btn-primary" type="submit">"Create pull request"</button>
                </div>
            </form>
        </div>
    }
}
