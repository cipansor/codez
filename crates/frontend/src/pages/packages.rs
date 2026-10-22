//! Package registry listings and per-package detail.

use crate::api::{get, get_or, local_resource};
use crate::components::Loading;
use leptos::prelude::*;
use leptos_router::hooks::*;
use shared::{Package, PackageFile};

/// `/packages/:owner` — the packages an owner publishes.
#[component]
pub fn PackageList() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());

    let packages = local_resource(owner, |name| async move {
        get::<Vec<Package>>(&format!("/api/v1/packages/{name}")).await
    });

    view! {
        <div class="package-list">
            <div class="page-header">
                <h3>"Packages for " {owner}</h3>
            </div>
            <p class="text-muted">
                "Packages published to this instance, one entry per version. "
                "New versions arrive through each ecosystem's own client."
            </p>
            <ul class="item-list panel">
                <Suspense fallback=|| view! { <Loading label="Loading packages…"/> }>
                    {move || packages.get().map(|list| {
                        if list.is_empty() {
                            view! { <li class="text-muted">"No packages published yet."</li> }.into_any()
                        } else {
                            view! {
                                <For each=move || list.clone() key=|p| p.id.unwrap_or(0) children=move |p| {
                                    let name = p.name.clone().unwrap_or_default();
                                    let kind = p.type_.clone().unwrap_or_default();
                                    let version = p.version.clone().unwrap_or_default();
                                    let owner = p.owner.as_ref().and_then(|o| o.login.clone()).unwrap_or_default();
                                    let href = format!("/packages/{owner}/{kind}/{name}/{version}");
                                    view! {
                                        <li>
                                            <a href=href><strong>{name}</strong></a>
                                            <span class="text-muted">" v" {version} " (" {kind} ")"</span>
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

/// `/packages/:owner/:type/:name/:version` — one package and its files.
#[component]
pub fn PackageDetail() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let kind = move || params.with(|p| p.get("type").unwrap_or_default());
    let name = move || params.with(|p| p.get("name").unwrap_or_default());
    let version = move || params.with(|p| p.get("version").unwrap_or_default());

    let package = local_resource(
        move || (owner(), kind(), name(), version()),
        |(o, t, n, v)| async move {
            get_or::<Option<Package>>(&format!("/api/v1/packages/{o}/{t}/{n}/{v}"), None).await
        },
    );

    let files = local_resource(
        move || (owner(), kind(), name(), version()),
        |(o, t, n, v)| async move {
            get::<Vec<PackageFile>>(&format!("/api/v1/packages/{o}/{t}/{n}/{v}/files")).await
        },
    );

    view! {
        <div class="package-detail">
            <div class="page-header">
                <h3>{name} " v" {version} <span class="text-muted text-lg">{kind}</span></h3>
            </div>

            <Suspense fallback=|| view! { <p class="text-muted">"Loading package…"</p> }>
                {move || package.get().flatten().map(|p| {
                    let created = p.created_at.clone().unwrap_or_default();
                    let url = p.html_url.clone().unwrap_or_default();
                    view! {
                        <dl class="meta">
                            <dt>"Published"</dt><dd>{created}</dd>
                            <dt>"Page"</dt><dd><a href=url.clone()>{url.clone()}</a></dd>
                        </dl>
                    }
                })}
            </Suspense>

            <h4>"Files"</h4>
            <ul class="item-list panel">
                <Suspense fallback=|| view! { <Loading label="Loading files…"/> }>
                    {move || files.get().map(|list| {
                        if list.is_empty() {
                            view! { <li class="text-muted">"No files recorded."</li> }.into_any()
                        } else {
                            view! {
                                <For each=move || list.clone() key=|f| f.id.unwrap_or(0) children=move |f| {
                                    let name = f.name.clone().unwrap_or_default();
                                    let size = f.size.unwrap_or(0);
                                    let sha256 = f.sha256.clone().unwrap_or_default();
                                    view! {
                                        <li>
                                            <strong>{name}</strong>
                                            <span class="text-small text-muted">" " {size} " bytes"</span>
                                            <div class="text-small text-muted"><code>{sha256}</code></div>
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
