use crate::api::{delete, get_or, local_resource, post_json_resp, probe, put, WRITE_ERROR};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::*;
use shared::Repository;

/// Shared invalidation trigger for repository data.
///
/// `RepoNav` and the page it heads each fetch the repository independently, so a
/// star/watch action (which mutates the server-side counts) must refresh both.
/// The heading page provides this signal; `RepoNav` falls back to a private one
/// when rendered without a provider (e.g. sub-pages with no overview).
#[derive(Copy, Clone)]
pub struct RepoRefresh(pub RwSignal<u32>);

/// Read the shared refresh signal, or create a page-local one when absent.
pub fn use_repo_refresh() -> RwSignal<u32> {
    use_context::<RepoRefresh>()
        .map(|r| r.0)
        .unwrap_or_else(|| RwSignal::new(0))
}

/// Shared repository chrome: the repository header (name + star/watch/fork
/// actions) followed by the section navigation.
///
/// Rendered on every repository sub-page so the chrome is identical everywhere.
/// The header fetches its own repository/status data from the route params, so
/// callers only render `<RepoNav/>` and do not have to thread props through.
#[component]
pub fn RepoNav() -> impl IntoView {
    let params = use_params_map();
    let navigate = use_navigate();
    let location = use_location();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let repo = move || params.with(|p| p.get("repo").unwrap_or_default());
    let base = move || format!("/repos/{}/{}", owner(), repo());
    let refresh = use_repo_refresh();

    let repo_data = local_resource(
        move || (owner(), repo(), refresh.get()),
        |(o, r, _)| async move {
            get_or::<Option<Repository>>(&format!("/api/v1/repos/{}/{}", o, r), None).await
        },
    );

    // Gitea exposes starring and watching as status probes: a `204` means yes,
    // a `404` means no, so the two resources are booleans rather than documents.
    let starred = local_resource(
        move || (owner(), repo(), refresh.get()),
        |(o, r, _)| async move { probe(&format!("/api/v1/user/starred/{}/{}", o, r)).await },
    );

    let watching = local_resource(
        move || (owner(), repo(), refresh.get()),
        |(o, r, _)| async move {
            get_or::<shared::WatchInfo>(
                &format!("/api/v1/repos/{}/{}/subscription", o, r),
                shared::WatchInfo::default(),
            )
            .await
            .subscribed
            .unwrap_or(false)
        },
    );

    let (action_error, set_action_error) = signal(Option::<String>::None);
    let bump = move || refresh.update(|n| *n += 1);

    // Star and watch are toggles: refreshing only after a confirmed 2xx keeps the
    // button label and count from flipping on a rejected request.
    let on_star = move |_| {
        let o = owner();
        let r = repo();
        let currently = starred.get_untracked().unwrap_or(false);
        spawn_local(async move {
            let url = format!("/api/v1/user/starred/{}/{}", o, r);
            let ok = if currently {
                delete(&url).await
            } else {
                put(&url).await
            };
            if ok {
                set_action_error.set(None);
                bump();
            } else {
                set_action_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    let on_watch = move |_| {
        let o = owner();
        let r = repo();
        let currently = watching.get_untracked().unwrap_or(false);
        spawn_local(async move {
            let url = format!("/api/v1/repos/{}/{}/subscription", o, r);
            let ok = if currently {
                delete(&url).await
            } else {
                put(&url).await
            };
            if ok {
                set_action_error.set(None);
                bump();
            } else {
                set_action_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    let on_fork = move |_| {
        let o = owner();
        let r = repo();
        let navigate = navigate.clone();
        spawn_local(async move {
            let body = shared::CreateForkOption::default();
            match post_json_resp::<_, Repository>(
                &format!("/api/v1/repos/{}/{}/forks", o, r),
                &body,
            )
            .await
            {
                Some(new_repo) => {
                    let target = new_repo
                        .full_name
                        .clone()
                        .unwrap_or_else(|| format!("{}/{}", o, r));
                    navigate(&format!("/repos/{target}"), Default::default());
                }
                None => set_action_error.set(Some(WRITE_ERROR.to_string())),
            }
        });
    };

    let links = [
        ("", "Code"),
        ("/issues", "Issues"),
        ("/pulls", "Pull requests"),
        ("/commits", "Commits"),
        ("/branches", "Branches"),
        ("/tags", "Tags"),
        ("/releases", "Releases"),
        ("/labels", "Labels"),
        ("/milestones", "Milestones"),
        ("/wiki", "Wiki"),
        ("/projects", "Projects"),
        ("/discussions", "Discussions"),
        ("/actions", "Actions"),
        ("/security", "Security"),
        ("/pulse", "Pulse"),
        ("/settings", "Settings"),
    ];

    let is_active = move |suffix: &str| {
        let path = location.pathname.get();
        let root = base();
        let at = |sub: &str| {
            path == format!("{root}{sub}") || path.starts_with(&format!("{root}{sub}/"))
        };
        if suffix.is_empty() {
            // Code covers the root browser, /src, file edit and code search —
            // these are all sub-views of the repository's code, not tabs of
            // their own.
            path == root || path == format!("{root}/") || at("/src") || at("/edit") || at("/search")
        } else if suffix == "/pulls" {
            // The compare view opens a pull request, so it belongs under Pulls.
            at(suffix) || path == format!("{root}/compare")
        } else {
            at(suffix)
        }
    };

    view! {
        <div class="repo-chrome">
            <div class="repo-header">
                <h3 class="mb-0">"Repository: " {owner} " / " {repo}</h3>
                <div class="repo-actions">
                    <Transition fallback=move || view! { <span class="text-muted">"Loading…"</span> }>
                        {move || {
                            let repo_data = repo_data.get()??;
                            let star_label = if starred.get().unwrap_or(false) { "Unstar" } else { "Star" };
                            let watch_label = if watching.get().unwrap_or(false) { "Unwatch" } else { "Watch" };
                            Some(view! {
                                <button class="btn-primary" on:click=on_star>{star_label} " (" {repo_data.stars_count.unwrap_or(0)} ")"</button>
                                <button on:click=on_watch>{watch_label} " (" {repo_data.watchers_count.unwrap_or(0)} ")"</button>
                            })
                        }}
                    </Transition>
                    <button on:click=on_fork>"Fork"</button>
                </div>
                {move || action_error.get().map(|msg| view! {
                    <p class="form-error" role="alert">{msg}</p>
                })}
            </div>

            <nav class="repo-nav">
                {links.iter().map(|(suffix, label)| {
                    let (suffix, label) = (*suffix, *label);
                    let href = move || format!("{}{}", base(), suffix);
                    let active = move || is_active(suffix);
                    view! {
                        <a href=href class:active=active aria-current=move || active().then_some("page")>
                            {label}
                        </a>
                    }
                }).collect_view()}
            </nav>
        </div>
    }
}
