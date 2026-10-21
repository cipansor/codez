//! Repository overview, source browser, commit history, branches and tags.

use crate::api::{decode_base64, get, get_or, local_resource, put_json, spawn, WRITE_ERROR};
use crate::components::{FormError, Loading, RepoNav, RepoRefresh};
use leptos::prelude::*;
use leptos_router::hooks::*;
use shared::{Branch, Commit, ContentsResponse, RepoTopicOptions, Repository, Tag};

/// `/repos/:owner/:repo` — the repository landing page: description, topics,
/// language breakdown and the latest commits.
#[component]
pub fn RepoDetail() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let repo_name = move || params.with(|p| p.get("repo").unwrap_or_default());
    let refresh = RwSignal::new(0u32);
    // Share the invalidation trigger with RepoNav so a star/watch action (which
    // changes the server-side counts) refreshes both the header and this
    // overview instead of leaving the overview stats stale.
    provide_context(RepoRefresh(refresh));

    let repo = local_resource(
        move || (owner(), repo_name(), refresh.get()),
        |(o, r, _)| async move {
            get_or::<Option<Repository>>(&format!("/api/v1/repos/{o}/{r}"), None).await
        },
    );

    let commits = local_resource(
        move || (owner(), repo_name()),
        |(o, r)| async move { get::<Vec<Commit>>(&format!("/api/v1/repos/{o}/{r}/commits")).await },
    );

    view! {
        <div class="repo-detail">
            <RepoNav/>
            <Suspense fallback=|| view! { <p class="text-muted">"Loading repository…"</p> }>
                {move || repo.get().flatten().map(|r| {
                    let description = r.description.clone()
                        .filter(|d| !d.is_empty())
                        .unwrap_or_else(|| "No description.".to_string());
                    let default_branch = r.default_branch.clone().unwrap_or_else(|| "main".to_string());
                    let clone_url = r.clone_url.clone().unwrap_or_default();
                    let website = r.website.clone().unwrap_or_default();
                    let topics = r.topics.clone().unwrap_or_default();
                    let size = r.size.unwrap_or(0);
                    view! {
                        <div class="repo-stats panel">
                            <p class="mb-1">{description}</p>
                            <div class="text-small text-muted">
                                <span>"Default branch: " <code>{default_branch}</code></span>
                                " · " <span>{size} " KB"</span>
                            </div>
                            {(!website.is_empty()).then(|| view! {
                                <p class="text-small"><a href=website.clone()>{website.clone()}</a></p>
                            })}
                            <div class="install-instructions">
                                <label>"Clone"</label>
                                <input class="input-lg" type="text" readonly value=clone_url />
                            </div>
                            <TopicEditor topics=topics/>
                        </div>
                    }
                })}
            </Suspense>

            <div class="repo-shell">
                <div class="repo-main">
                    <div class="panel">
                        <h3>"Languages"</h3>
                        <LanguageBar/>
                    </div>
                    <div class="panel">
                        <h3>"Latest commits"</h3>
                        <ul class="commit-list item-list">
                            <Suspense fallback=|| view! { <Loading label="Loading commits…"/> }>
                                {move || commits.get().map(|list| {
                                    if list.is_empty() {
                                        view! { <li class="text-muted">"No commits yet."</li> }.into_any()
                                    } else {
                                        view! {
                                            <For each=move || list.clone() key=|c| c.sha.clone().unwrap_or_default() children=move |c| {
                                                let sha = c.sha.clone().unwrap_or_default();
                                                let short = sha.chars().take(8).collect::<String>();
                                                let message = c.commit.as_ref()
                                                    .and_then(|cm| cm.message.clone())
                                                    .unwrap_or_default();
                                                let author = c.commit.as_ref()
                                                    .and_then(|cm| cm.author.as_ref())
                                                    .and_then(|a| a.name.clone())
                                                    .unwrap_or_default();
                                                view! {
                                                    <li>
                                                        <span class="commit-sha"><code>{short}</code></span>
                                                        <span class="commit-message">{message}</span>
                                                        <span class="commit-author text-small text-muted">{author}</span>
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
        </div>
    }
}

/// Language breakdown as a proportional bar plus a legend.
#[component]
fn LanguageBar() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let repo = move || params.with(|p| p.get("repo").unwrap_or_default());
    let languages = local_resource(
        move || (owner(), repo()),
        |(o, r)| async move {
            get::<std::collections::BTreeMap<String, i64>>(&format!(
                "/api/v1/repos/{o}/{r}/languages"
            ))
            .await
        },
    );

    view! {
        <Suspense fallback=|| view! { <p class="text-muted">"Loading languages…"</p> }>
            {move || languages.get().map(|map| {
                let total: i64 = map.values().sum();
                if total == 0 {
                    return view! { <p class="text-muted">"No language data."</p> }.into_any();
                }
                let entries: Vec<(String, i64)> = map.into_iter().collect();
                let bar = entries.iter().map(|(name, bytes)| {
                    let pct = (*bytes as f64 / total as f64) * 100.0;
                    let style = format!("width:{pct:.1}%;background:{}", language_color(name));
                    view! { <span class="language-bar" style=style title=name.clone()></span> }
                }).collect_view();
                let legend = entries.iter().map(|(name, bytes)| {
                    let pct = (*bytes as f64 / total as f64) * 100.0;
                    view! {
                        <li class="language-legend-item">
                            <span class="badge" style=format!("background:{}", language_color(name))></span>
                            <strong>{name.clone()}</strong> " " {format!("{pct:.1}%")}
                        </li>
                    }
                }).collect_view();
                view! {
                    <div class="language-bar-wrap">{bar}</div>
                    <ul class="language-legend">{legend}</ul>
                }.into_any()
            })}
        </Suspense>
    }
}

/// A stable colour per language name, derived from a small palette.
fn language_color(name: &str) -> String {
    let palette = [
        "#dea584", "#3572A5", "#2b7489", "#f1e05a", "#b07219", "#563d7c", "#e34c26",
    ];
    let index = name.bytes().fold(0usize, |acc, b| acc + b as usize) % palette.len();
    palette[index].to_string()
}

/// The topic chips plus an inline editor that `PUT`s `/topics`.
#[component]
fn TopicEditor(topics: Vec<String>) -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let repo = move || params.with(|p| p.get("repo").unwrap_or_default());
    let (current, set_current) = signal(topics);
    let (editing, set_editing) = signal(false);
    let (input, set_input) = signal(String::new());
    let (error, set_error) = signal(Option::<String>::None);

    let start_edit = move |_| {
        set_input.set(current.get().join(", "));
        set_editing.set(true);
    };

    let save = move |_| {
        let names: Vec<String> = input
            .get()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let payload = RepoTopicOptions {
            topics: Some(names.clone()),
        };
        let o = owner();
        let r = repo();
        spawn(async move {
            // Keep the editor open on a rejected write so the typed topics are
            // not discarded.
            if put_json(&format!("/api/v1/repos/{o}/{r}/topics"), &payload).await {
                set_current.set(names);
                set_editing.set(false);
                set_error.set(None);
            } else {
                set_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    view! {
        <div class="repo-topics mt-1">
            {move || error.get().map(|msg| view! { <FormError message=msg/> })}
            {move || if editing.get() {
                view! {
                    <div class="flex-center gap-sm">
                        <input class="input-lg" type="text" placeholder="rust, axum, leptos"
                            prop:value=input
                            on:input=move |ev| set_input.set(event_target_value(&ev)) />
                        <button class="btn-primary" on:click=save>"Save"</button>
                        <button on:click=move |_| set_editing.set(false)>"Cancel"</button>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="flex-center flex-wrap gap-sm">
                        {current.get().into_iter().map(|t| view! {
                            <span class="label label-accent">{t}</span>
                        }).collect_view()}
                        <button class="btn-sm" on:click=start_edit>"Edit topics"</button>
                    </div>
                }.into_any()
            }}
        </div>
    }
}

/// `/repos/:owner/:repo/src/*path` — the file browser and file viewer.
#[component]
pub fn RepoCode() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let repo = move || params.with(|p| p.get("repo").unwrap_or_default());
    let path = move || params.with(|p| p.get("path").unwrap_or_default());

    // A directory listing comes back as an array, a single file as an object;
    // both are fetched as raw JSON and discriminated on the shape.
    let entry = local_resource(
        move || (owner(), repo(), path()),
        |(o, r, p)| async move {
            let url = if p.is_empty() {
                format!("/api/v1/repos/{o}/{r}/contents")
            } else {
                format!("/api/v1/repos/{o}/{r}/contents/{p}")
            };
            get::<serde_json::Value>(&url).await
        },
    );

    view! {
        <div class="repo-code">
            <RepoNav/>
            <Suspense fallback=|| view! { <p class="text-muted">"Loading…"</p> }>
                {move || {
                    let value = entry.get()?;
                    let o = owner();
                    let r = repo();
                    if value.is_array() {
                        let files: Vec<ContentsResponse> =
                            serde_json::from_value(value).unwrap_or_default();
                        Some(view! {
                            <ul class="item-list panel">
                                {files.into_iter().map(|f| {
                                    let name = f.name.clone().unwrap_or_default();
                                    let file_path = f.path.clone().unwrap_or_default();
                                    let kind = f.type_.clone().unwrap_or_default();
                                    let size = f.size.unwrap_or(0);
                                    let href = format!("/repos/{o}/{r}/src/{file_path}");
                                    let is_dir = kind == "dir";
                                    view! {
                                        <li>
                                            <a href=href>
                                                {if is_dir { "📁 " } else { "📄 " }}
                                                <strong>{name}</strong>
                                            </a>
                                            {(!is_dir).then(|| view! {
                                                <span class="text-small text-muted">" " {size} " bytes"</span>
                                            })}
                                        </li>
                                    }
                                }).collect_view()}
                            </ul>
                        }.into_any())
                    } else if value.is_object() {
                        let file: ContentsResponse = serde_json::from_value(value).unwrap_or_default();
                        let name = file.name.clone().unwrap_or_default();
                        let content = decode_base64(file.content.as_deref().unwrap_or_default());
                        let size = file.size.unwrap_or(0);
                        let sha = file.sha.clone().unwrap_or_default();
                        let edit_href = format!("/repos/{o}/{r}/edit/{}", file.path.clone().unwrap_or_default());
                        Some(view! {
                            <div class="file-header">
                                <strong>{name}</strong>
                                <span class="text-small text-muted">" " {size} " bytes · " <code>{sha}</code></span>
                                <a class="btn-sm" href=edit_href>"Edit"</a>
                            </div>
                            <pre class="file-diff"><code>{content}</code></pre>
                        }.into_any())
                    } else {
                        Some(view! {
                            <div class="empty-state">"This path does not exist."</div>
                        }.into_any())
                    }
                }}
            </Suspense>
        </div>
    }
}

/// `/repos/:owner/:repo/edit/*path` — edit a file and commit the change.
#[component]
pub fn FileEdit() -> impl IntoView {
    let params = use_params_map();
    let navigate = use_navigate();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let repo = move || params.with(|p| p.get("repo").unwrap_or_default());
    let path = move || params.with(|p| p.get("path").unwrap_or_default());

    let (content, set_content) = signal(String::new());
    let (message, set_message) = signal(String::new());
    let (error, set_error) = signal(Option::<String>::None);

    // Load the current file body so the editor starts from its contents.
    spawn(async move {
        let o = owner();
        let r = repo();
        let p = path();
        if !p.is_empty() {
            let file = get_or::<Option<ContentsResponse>>(
                &format!("/api/v1/repos/{o}/{r}/contents/{p}"),
                None,
            )
            .await;
            if let Some(file) = file {
                set_content.set(decode_base64(file.content.as_deref().unwrap_or_default()));
            }
        }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let o = owner();
        let r = repo();
        let p = path();
        let payload = serde_json::json!({
            "content": crate::api::encode_base64(&content.get()),
            "message": message.get(),
        });
        let navigate = navigate.clone();
        spawn(async move {
            if crate::api::post_json(&format!("/api/v1/repos/{o}/{r}/contents/{p}"), &payload).await
            {
                set_error.set(None);
                navigate(&format!("/repos/{o}/{r}/src/{p}"), Default::default());
            } else {
                // Keep the edited body so a rejected commit is not lost.
                set_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    view! {
        <div class="file-edit">
            <RepoNav/>
            <h3>"Editing " <code>{path}</code></h3>
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
                    <button class="btn-primary" type="submit">"Commit changes"</button>
                </div>
            </form>
        </div>
    }
}

/// `/repos/:owner/:repo/commits` — the commit history.
#[component]
pub fn CommitList() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let repo = move || params.with(|p| p.get("repo").unwrap_or_default());

    let commits = local_resource(
        move || (owner(), repo()),
        |(o, r)| async move { get::<Vec<Commit>>(&format!("/api/v1/repos/{o}/{r}/commits")).await },
    );

    view! {
        <div class="commit-list">
            <RepoNav/>
            <ul class="item-list panel">
                <Suspense fallback=|| view! { <Loading label="Loading commits…"/> }>
                    {move || commits.get().map(|list| {
                        if list.is_empty() {
                            view! { <li class="text-muted">"No commits yet."</li> }.into_any()
                        } else {
                            let o = owner();
                            let r = repo();
                            view! {
                                <For each=move || list.clone() key=|c| c.sha.clone().unwrap_or_default() children=move |c| {
                                    let sha = c.sha.clone().unwrap_or_default();
                                    let short = sha.chars().take(8).collect::<String>();
                                    let message = c.commit.as_ref()
                                        .and_then(|cm| cm.message.clone())
                                        .unwrap_or_default();
                                    let author = c.commit.as_ref()
                                        .and_then(|cm| cm.author.as_ref())
                                        .and_then(|a| a.name.clone())
                                        .unwrap_or_default();
                                    let date = c.created.clone().unwrap_or_default();
                                    let href = format!("/repos/{o}/{r}/commits/{sha}");
                                    view! {
                                        <li>
                                            <a href=href class="commit-sha"><code>{short}</code></a>
                                            <span class="commit-message">{message}</span>
                                            <span class="commit-author text-small text-muted">{author}</span>
                                            <span class="commit-date text-small text-muted">{date}</span>
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

/// `/repos/:owner/:repo/commits/:sha` — one commit, with the files it touched.
#[component]
pub fn CommitDiff() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let repo = move || params.with(|p| p.get("repo").unwrap_or_default());
    let sha = move || params.with(|p| p.get("sha").unwrap_or_default());

    let commit = local_resource(
        move || (owner(), repo(), sha()),
        |(o, r, s)| async move {
            get_or::<Option<Commit>>(&format!("/api/v1/repos/{o}/{r}/commits/{s}"), None).await
        },
    );

    view! {
        <div class="commit-diff">
            <RepoNav/>
            <Suspense fallback=|| view! { <p class="text-muted">"Loading commit…"</p> }>
                {move || commit.get().flatten().map(|c| {
                    let full_sha = c.sha.clone().unwrap_or_default();
                    let message = c.commit.as_ref()
                        .and_then(|cm| cm.message.clone())
                        .unwrap_or_default();
                    let author = c.commit.as_ref()
                        .and_then(|cm| cm.author.as_ref())
                        .and_then(|a| a.name.clone())
                        .unwrap_or_default();
                    let date = c.created.clone().unwrap_or_default();
                    let files = c.files.clone().unwrap_or_default();
                    let stats = c.stats.clone();
                    view! {
                        <div class="page-header">
                            <h3>{message}</h3>
                            <span class="text-small text-muted">{author} " · " {date}</span>
                        </div>
                        <p><code>{full_sha}</code></p>
                        {stats.map(|s| view! {
                            <p class="diff-stats">
                                <span class="text-success">"+" {s.additions.unwrap_or(0)}</span>
                                " "
                                <span class="text-danger">"-" {s.deletions.unwrap_or(0)}</span>
                            </p>
                        })}
                        <ul class="commit-list item-list panel">
                            {if files.is_empty() {
                                view! { <li class="text-muted">"No file changes recorded."</li> }.into_any()
                            } else {
                                files.into_iter().map(|f| {
                                    let name = f.filename.clone().unwrap_or_default();
                                    let status = f.status.clone().unwrap_or_default();
                                    view! {
                                        <li>
                                            <span class="label">{status}</span>
                                            <code>{name}</code>
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

/// `/repos/:owner/:repo/branches` — branch listing.
#[component]
pub fn BranchList() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let repo = move || params.with(|p| p.get("repo").unwrap_or_default());

    let branches = local_resource(
        move || (owner(), repo()),
        |(o, r)| async move { get::<Vec<Branch>>(&format!("/api/v1/repos/{o}/{r}/branches")).await },
    );

    view! {
        <div class="branch-list">
            <RepoNav/>
            <ul class="item-list panel">
                <Suspense fallback=|| view! { <Loading label="Loading branches…"/> }>
                    {move || branches.get().map(|list| {
                        if list.is_empty() {
                            view! { <li class="text-muted">"No branches."</li> }.into_any()
                        } else {
                            view! {
                                <For each=move || list.clone() key=|b| b.name.clone().unwrap_or_default() children=move |b| {
                                    let name = b.name.clone().unwrap_or_default();
                                    let protected = b.protected.unwrap_or(false);
                                    let sha = b.commit.as_ref()
                                        .and_then(|c| c.id.clone())
                                        .map(|s| s.chars().take(8).collect::<String>())
                                        .unwrap_or_default();
                                    let message = b.commit.as_ref()
                                        .and_then(|c| c.message.clone())
                                        .unwrap_or_default();
                                    view! {
                                        <li class="flex-between">
                                            <span>
                                                <strong>{name}</strong>
                                                {protected.then(|| view! { <span class="label label-accent">" Protected"</span> })}
                                                <span class="text-small text-muted">" " {message}</span>
                                            </span>
                                            <code class="text-small">{sha}</code>
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

/// `/repos/:owner/:repo/tags` — tag listing.
#[component]
pub fn TagList() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let repo = move || params.with(|p| p.get("repo").unwrap_or_default());

    let tags = local_resource(
        move || (owner(), repo()),
        |(o, r)| async move { get::<Vec<Tag>>(&format!("/api/v1/repos/{o}/{r}/tags")).await },
    );

    view! {
        <div class="tag-list">
            <RepoNav/>
            <ul class="item-list panel">
                <Suspense fallback=|| view! { <Loading label="Loading tags…"/> }>
                    {move || tags.get().map(|list| {
                        if list.is_empty() {
                            view! { <li class="text-muted">"No tags."</li> }.into_any()
                        } else {
                            view! {
                                <For each=move || list.clone() key=|t| t.name.clone().unwrap_or_default() children=move |t| {
                                    let name = t.name.clone().unwrap_or_default();
                                    let sha = t.commit.as_ref()
                                        .and_then(|c| c.sha.clone())
                                        .map(|s| s.chars().take(8).collect::<String>())
                                        .unwrap_or_default();
                                    let zip = t.zipball_url.clone().unwrap_or_default();
                                    let tarball = t.tarball_url.clone().unwrap_or_default();
                                    view! {
                                        <li class="flex-between">
                                            <span><strong>{name}</strong> <code class="text-small">{sha}</code></span>
                                            <span>
                                                <a class="btn-sm" href=zip>"zip"</a>
                                                <a class="btn-sm" href=tarball>"tar.gz"</a>
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
    }
}

/// `/repos/:owner/:repo/pulse` — recent activity counters.
///
/// Gitea has no `/pulse` endpoint in its OpenAPI document, so the counters are
/// derived from the resources the API does expose: the issues, pull requests
/// and commits the repository already lists.
#[component]
pub fn RepoPulse() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let repo = move || params.with(|p| p.get("repo").unwrap_or_default());

    let issues = local_resource(
        move || (owner(), repo()),
        |(o, r)| async move {
            get::<Vec<shared::Issue>>(&format!("/api/v1/repos/{o}/{r}/issues?state=all")).await
        },
    );
    let pulls = local_resource(
        move || (owner(), repo()),
        |(o, r)| async move {
            get::<Vec<shared::PullRequest>>(&format!("/api/v1/repos/{o}/{r}/pulls?state=all")).await
        },
    );
    let commits = local_resource(
        move || (owner(), repo()),
        |(o, r)| async move { get::<Vec<Commit>>(&format!("/api/v1/repos/{o}/{r}/commits")).await },
    );

    view! {
        <div class="repo-pulse">
            <RepoNav/>
            <h3>"Pulse"</h3>
            <div class="pulse-grid stat-grid">
                <Suspense fallback=|| view! { <p class="text-muted">"Loading…"</p> }>
                    {move || {
                        let open_issues = issues.get().map(|l| l.iter()
                            .filter(|i| i.state == Some(shared::StateType::Open)).count()).unwrap_or(0);
                        let closed_issues = issues.get().map(|l| l.iter()
                            .filter(|i| i.state == Some(shared::StateType::Closed)).count()).unwrap_or(0);
                        let merged = pulls.get().map(|l| l.iter()
                            .filter(|p| p.merged.unwrap_or(false)).count()).unwrap_or(0);
                        let open_pulls = pulls.get().map(|l| l.iter()
                            .filter(|p| p.state == Some(shared::StateType::Open)).count()).unwrap_or(0);
                        let commit_count = commits.get().map(|l| l.len()).unwrap_or(0);
                        view! {
                            <div class="stat-card"><span class="stat-value">{open_issues}</span><span>"Open issues"</span></div>
                            <div class="stat-card"><span class="stat-value">{closed_issues}</span><span>"Closed issues"</span></div>
                            <div class="stat-card"><span class="stat-value">{open_pulls}</span><span>"Open pull requests"</span></div>
                            <div class="stat-card"><span class="stat-value">{merged}</span><span>"Merged pull requests"</span></div>
                            <div class="stat-card"><span class="stat-value">{commit_count}</span><span>"Commits"</span></div>
                        }
                    }}
                </Suspense>
            </div>
        </div>
    }
}

/// `/repos/:owner/:repo/search` — search code within the repository.
///
/// Gitea exposes code search only through the web UI's internal endpoint, which
/// the OpenAPI document does not cover. The page searches the paths the
/// contents API returns instead of pretending a full-text index exists.
#[component]
pub fn RepoCodeSearch() -> impl IntoView {
    let params = use_params_map();
    let query_map = use_query_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let repo = move || params.with(|p| p.get("repo").unwrap_or_default());
    let (query, set_query) = signal(query_map.with(|q| q.get("q").unwrap_or_default()));

    let files = local_resource(
        move || (owner(), repo(), query.get()),
        |(o, r, q)| async move {
            let entries =
                get::<Vec<ContentsResponse>>(&format!("/api/v1/repos/{o}/{r}/contents")).await;
            if q.is_empty() {
                return entries;
            }
            let needle = q.to_lowercase();
            entries
                .into_iter()
                .filter(|f| {
                    f.name
                        .as_deref()
                        .unwrap_or_default()
                        .to_lowercase()
                        .contains(&needle)
                        || f.path
                            .as_deref()
                            .unwrap_or_default()
                            .to_lowercase()
                            .contains(&needle)
                })
                .collect()
        },
    );

    view! {
        <div class="repo-search">
            <RepoNav/>
            <div class="panel flex-center gap-sm">
                <input class="search-input-flex" type="text" placeholder="Search code…"
                    prop:value=query
                    on:input=move |ev| set_query.set(event_target_value(&ev)) />
            </div>
            <ul class="item-list panel">
                <Suspense fallback=|| view! { <Loading label="Searching…"/> }>
                    {move || files.get().map(|list| {
                        if list.is_empty() {
                            view! { <li class="text-muted">"No matching files."</li> }.into_any()
                        } else {
                            let o = owner();
                            let r = repo();
                            view! {
                                <For each=move || list.clone() key=|f| f.path.clone().unwrap_or_default() children=move |f| {
                                    let path = f.path.clone().unwrap_or_default();
                                    let href = format!("/repos/{o}/{r}/src/{path}");
                                    view! {
                                        <li><a href=href class="code-search-link"><code>{path}</code></a></li>
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
