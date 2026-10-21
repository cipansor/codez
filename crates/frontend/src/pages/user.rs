//! User profile, followers, settings and the sign-in / register forms.

use crate::api::{
    delete, get, get_or, get_then, local_resource, post_json, post_json_resp, probe, put, spawn,
    WRITE_ERROR,
};
use crate::components::{repo_href, FormError, Loading};
use leptos::prelude::*;
use leptos_router::hooks::*;
use shared::{AccessToken, CreateUserOption, Email, GPGKey, PublicKey, Repository, User};

/// `/login` — exchange credentials for an access token.
#[component]
pub fn Login() -> impl IntoView {
    let (username, set_username) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (error, set_error) = signal(Option::<String>::None);
    let (token, set_token) = signal(Option::<String>::None);
    let navigate = use_navigate();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let name = username.get();
        let pass = password.get();
        let navigate = navigate.clone();
        spawn(async move {
            // `/signin` takes the credentials as a form document and answers
            // with the token's `sha1`, which clients then send as `?token=`.
            let body = serde_json::json!({ "user_name": name, "password": pass });
            match post_json_resp::<_, serde_json::Value>("/api/v1/signin", &body).await {
                Some(value) => {
                    let sha1 = value
                        .get("sha1")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string();
                    set_error.set(None);
                    set_token.set(Some(sha1));
                    navigate("/", Default::default());
                }
                None => {
                    // Keep the typed credentials: a rejected sign-in must not
                    // clear the form.
                    set_error.set(Some(
                        "Sign-in failed. Check the username and password.".to_string(),
                    ));
                }
            }
        });
    };

    view! {
        <div class="login panel container-narrow">
            <h2>"Sign in"</h2>
            {move || error.get().map(|msg| view! { <FormError message=msg/> })}
            {move || token.get().map(|t| view! {
                <p class="flash-success">"Signed in. Access token: " <code>{t}</code></p>
            })}
            <form on:submit=on_submit>
                <div class="form-group">
                    <label>"Username"</label>
                    <input type="text" prop:value=username
                        on:input=move |ev| set_username.set(event_target_value(&ev)) />
                </div>
                <div class="form-group">
                    <label>"Password"</label>
                    <input type="password" prop:value=password
                        on:input=move |ev| set_password.set(event_target_value(&ev)) />
                </div>
                <div class="form-actions">
                    <button class="btn-primary" type="submit">"Sign in"</button>
                    <a href="/register">"Register"</a>
                </div>
            </form>
        </div>
    }
}

/// `/register` — create an account.
#[component]
pub fn Register() -> impl IntoView {
    let (username, set_username) = signal(String::new());
    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (error, set_error) = signal(Option::<String>::None);
    let (done, set_done) = signal(false);
    let navigate = use_navigate();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let body = CreateUserOption {
            username: username.get(),
            email: email.get(),
            password: Some(password.get()),
            ..Default::default()
        };
        let navigate = navigate.clone();
        spawn(async move {
            // `/admin/users` is the only account-creation endpoint Gitea
            // documents; a self-service registration posts there too.
            match post_json_resp::<_, User>("/api/v1/admin/users", &body).await {
                Some(_) => {
                    set_error.set(None);
                    set_done.set(true);
                    navigate("/login", Default::default());
                }
                // Keep the typed values so the user can correct them.
                None => set_error.set(Some(
                    "Registration failed. The username or email may already be taken.".to_string(),
                )),
            }
        });
    };

    view! {
        <div class="register panel container-narrow">
            <h2>"Register"</h2>
            {move || error.get().map(|msg| view! { <FormError message=msg/> })}
            {move || done.get().then(|| view! {
                <p class="flash-success">"Account created. You can sign in now."</p>
            })}
            <form on:submit=on_submit>
                <div class="form-group">
                    <label>"Username"</label>
                    <input type="text" prop:value=username
                        on:input=move |ev| set_username.set(event_target_value(&ev)) />
                </div>
                <div class="form-group">
                    <label>"Email"</label>
                    <input type="email" prop:value=email
                        on:input=move |ev| set_email.set(event_target_value(&ev)) />
                </div>
                <div class="form-group">
                    <label>"Password"</label>
                    <input type="password" prop:value=password
                        on:input=move |ev| set_password.set(event_target_value(&ev)) />
                </div>
                <div class="form-actions">
                    <button class="btn-primary" type="submit">"Create account"</button>
                </div>
            </form>
        </div>
    }
}

/// `/users/:username` — a user's profile and public repositories.
#[component]
pub fn UserProfile() -> impl IntoView {
    let params = use_params_map();
    let username = move || params.with(|p| p.get("username").unwrap_or_default());
    let refresh = RwSignal::new(0u32);

    let user = local_resource(
        move || (username(), refresh.get()),
        |(name, _)| async move { get_or::<Option<User>>(&format!("/api/v1/users/{name}"), None).await },
    );

    let repos = local_resource(username, |name| async move {
        get::<Vec<Repository>>(&format!("/api/v1/users/{name}/repos")).await
    });

    let orgs = local_resource(username, |name| async move {
        get::<Vec<shared::Organization>>(&format!("/api/v1/users/{name}/orgs")).await
    });

    // The follow edge is a status probe: `204` when the edge exists, `404`
    // when it does not, so the resource is a boolean rather than a document.
    let following = local_resource(
        move || (username(), refresh.get()),
        |(name, _)| async move { probe(&format!("/api/v1/user/following/{name}")).await },
    );

    let (error, set_error) = signal(Option::<String>::None);

    let on_follow = move |_| {
        let name = username();
        let currently = following.get_untracked().unwrap_or(false);
        spawn(async move {
            let url = format!("/api/v1/user/following/{name}");
            let ok = if currently {
                delete(&url).await
            } else {
                put(&url).await
            };
            if ok {
                set_error.set(None);
                refresh.update(|n| *n += 1);
            } else {
                set_error.set(Some(WRITE_ERROR.to_string()));
            }
        });
    };

    view! {
        <div class="user-profile">
            <Suspense fallback=|| view! { <p class="text-muted">"Loading profile…"</p> }>
                {move || user.get().flatten().map(|u| {
                    let login = u.login.clone().unwrap_or_default();
                    let full_name = u.full_name.clone().unwrap_or_default();
                    let bio = u.description.clone().unwrap_or_default();
                    let location = u.location.clone().unwrap_or_default();
                    let website = u.website.clone().unwrap_or_default();
                    let followers = u.followers_count.unwrap_or(0);
                    let following_count = u.following_count.unwrap_or(0);
                    let login_followers = login.clone();
                    let login_following = login.clone();
                    let label = move || if following.get().unwrap_or(false) { "Unfollow" } else { "Follow" };
                    view! {
                        <div class="page-header">
                            <h2>{login.clone()} <span class="text-muted text-lg">{full_name}</span></h2>
                            <div class="user-links">
                                <a href=format!("/users/{login_followers}/followers")>{followers} " followers"</a>
                                <a href=format!("/users/{login_following}/following")>{following_count} " following"</a>
                                <button class="btn btn-sm" on:click=on_follow>{label}</button>
                            </div>
                        </div>
                        <p class="text-muted">{bio}</p>
                        <p class="text-small text-muted">{location} " " {website}</p>
                    }
                })}
            </Suspense>
            {move || error.get().map(|msg| view! { <FormError message=msg/> })}

            <h3 class="mt-3">"Repositories"</h3>
            <ul class="item-list panel">
                <Suspense fallback=|| view! { <Loading label="Loading repositories…"/> }>
                    {move || repos.get().map(|list| {
                        if list.is_empty() {
                            view! { <li class="text-muted">"No public repositories."</li> }.into_any()
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

            <h3 class="mt-3">"Organizations"</h3>
            <ul class="item-list panel">
                <Suspense fallback=|| view! { <Loading label="Loading organizations…"/> }>
                    {move || orgs.get().map(|list| {
                        if list.is_empty() {
                            view! { <li class="text-muted">"Not a member of any organization."</li> }.into_any()
                        } else {
                            view! {
                                <For each=move || list.clone() key=|o| o.id.unwrap_or(0) children=move |o| {
                                    let name = o.name.clone().unwrap_or_default();
                                    let href = format!("/orgs/{name}");
                                    view! { <li><a href=href><strong>{name}</strong></a></li> }
                                }/>
                            }.into_any()
                        }
                    })}
                </Suspense>
            </ul>
        </div>
    }
}

/// `/users/:username/followers`.
#[component]
pub fn UserFollowers() -> impl IntoView {
    let params = use_params_map();
    let username = move || params.with(|p| p.get("username").unwrap_or_default());
    let users = local_resource(username, |name| async move {
        get::<Vec<User>>(&format!("/api/v1/users/{name}/followers")).await
    });

    view! {
        <div class="followers panel">
            <h2>"Followers"</h2>
            <ul class="item-list">
                <Suspense fallback=|| view! { <Loading label="Loading…"/> }>
                    {move || users.get().map(|list| {
                        if list.is_empty() {
                            view! { <li class="text-muted">"No followers."</li> }.into_any()
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
        </div>
    }
}

/// `/users/:username/following`.
#[component]
pub fn UserFollowing() -> impl IntoView {
    let params = use_params_map();
    let username = move || params.with(|p| p.get("username").unwrap_or_default());
    let users = local_resource(username, |name| async move {
        get::<Vec<User>>(&format!("/api/v1/users/{name}/following")).await
    });

    view! {
        <div class="following panel">
            <h2>"Following"</h2>
            <ul class="item-list">
                <Suspense fallback=|| view! { <Loading label="Loading…"/> }>
                    {move || users.get().map(|list| {
                        if list.is_empty() {
                            view! { <li class="text-muted">"Not following anyone."</li> }.into_any()
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
        </div>
    }
}

/// `/settings/profile` — the account settings, keys and access tokens.
#[component]
pub fn UserSettings() -> impl IntoView {
    let (refresh, set_refresh) = signal(0);
    let (full_name, set_full_name) = signal(String::new());
    let (description, set_description) = signal(String::new());
    let (location, set_location) = signal(String::new());
    let (website, set_website) = signal(String::new());
    let (error, set_error) = signal(Option::<String>::None);
    let (saved, set_saved) = signal(false);

    // Pre-fill from the server once, so a reload does not wipe the form.
    spawn(async move {
        get_then::<shared::UserSettings, _>("/api/v1/user/settings", move |settings| {
            set_full_name.set(settings.full_name.clone().unwrap_or_default());
            set_description.set(settings.description.clone().unwrap_or_default());
            set_location.set(settings.location.clone().unwrap_or_default());
            set_website.set(settings.website.clone().unwrap_or_default());
        })
        .await;
    });

    let keys = local_resource(
        move || refresh.get(),
        |_| async move { get::<Vec<PublicKey>>("/api/v1/user/keys").await },
    );
    let gpg_keys = local_resource(
        move || refresh.get(),
        |_| async move { get::<Vec<GPGKey>>("/api/v1/user/gpg_keys").await },
    );
    let emails = local_resource(
        move || refresh.get(),
        |_| async move { get::<Vec<Email>>("/api/v1/user/emails").await },
    );
    let tokens = local_resource(
        move || refresh.get(),
        |_| async move { get::<Vec<AccessToken>>("/api/v1/user/tokens").await },
    );

    let on_save = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let body = shared::UserSettingsOptions {
            full_name: Some(full_name.get()),
            description: Some(description.get()),
            location: Some(location.get()),
            website: Some(website.get()),
            ..Default::default()
        };
        spawn(async move {
            // `PATCH /user/settings` is not implemented upstream either; the
            // write is attempted and its rejection surfaced rather than
            // pretending the profile was saved.
            if post_json("/api/v1/user/settings", &body).await {
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
        <div class="user-settings">
            <div class="page-header">
                <h2>"Profile settings"</h2>
            </div>
            {move || error.get().map(|msg| view! { <FormError message=msg/> })}
            {move || saved.get().then(|| view! { <p class="flash-success">"Saved."</p> })}

            <form class="panel" on:submit=on_save>
                <div class="form-group">
                    <label>"Full name"</label>
                    <input type="text" prop:value=full_name
                        on:input=move |ev| set_full_name.set(event_target_value(&ev)) />
                </div>
                <div class="form-group">
                    <label>"Biography"</label>
                    <textarea class="textarea-tall" prop:value=description
                        on:input=move |ev| set_description.set(event_target_value(&ev))></textarea>
                </div>
                <div class="form-group">
                    <label>"Location"</label>
                    <input type="text" prop:value=location
                        on:input=move |ev| set_location.set(event_target_value(&ev)) />
                </div>
                <div class="form-group">
                    <label>"Website"</label>
                    <input type="text" prop:value=website
                        on:input=move |ev| set_website.set(event_target_value(&ev)) />
                </div>
                <div class="form-actions">
                    <button class="btn-primary" type="submit">"Save"</button>
                </div>
            </form>

            <section class="panel">
                <h3>"Emails"</h3>
                <ul class="item-list">
                    <Suspense fallback=|| view! { <Loading label="Loading…"/> }>
                        {move || emails.get().map(|list| list.into_iter().map(|e| {
                            let address = e.email.clone().unwrap_or_default();
                            let primary = e.primary.unwrap_or(false);
                            view! {
                                <li>
                                    <span>{address}</span>
                                    {primary.then(|| view! { <span class="label label-accent">" Primary"</span> })}
                                </li>
                            }
                        }).collect_view())}
                    </Suspense>
                </ul>
            </section>

            <section class="panel ssh-keys">
                <h3>"SSH keys"</h3>
                <ul class="item-list">
                    <Suspense fallback=|| view! { <Loading label="Loading…"/> }>
                        {move || keys.get().map(|list| {
                            if list.is_empty() {
                                view! { <li class="text-muted">"No SSH keys."</li> }.into_any()
                            } else {
                                view! {
                                    <For each=move || list.clone() key=|k| k.id.unwrap_or(0) children=move |k| {
                                        let title = k.title.clone().unwrap_or_default();
                                        let fingerprint = k.fingerprint.clone().unwrap_or_default();
                                        view! { <li><strong>{title}</strong> <code>{fingerprint}</code></li> }
                                    }/>
                                }.into_any()
                            }
                        })}
                    </Suspense>
                </ul>
            </section>

            <section class="panel gpg-keys">
                <h3>"GPG keys"</h3>
                <ul class="item-list">
                    <Suspense fallback=|| view! { <Loading label="Loading…"/> }>
                        {move || gpg_keys.get().map(|list| {
                            if list.is_empty() {
                                view! { <li class="text-muted">"No GPG keys."</li> }.into_any()
                            } else {
                                view! {
                                    <For each=move || list.clone() key=|k| k.id.unwrap_or(0) children=move |k| {
                                        let key_id = k.key_id.clone().unwrap_or_default();
                                        view! { <li><code>{key_id}</code></li> }
                                    }/>
                                }.into_any()
                            }
                        })}
                    </Suspense>
                </ul>
            </section>

            <section class="panel">
                <h3>"Access tokens"</h3>
                <ul class="item-list">
                    <Suspense fallback=|| view! { <Loading label="Loading…"/> }>
                        {move || tokens.get().map(|list| {
                            if list.is_empty() {
                                view! { <li class="text-muted">"No access tokens."</li> }.into_any()
                            } else {
                                view! {
                                    <For each=move || list.clone() key=|t| t.id.unwrap_or(0) children=move |t| {
                                        let name = t.name.clone().unwrap_or_default();
                                        view! { <li><strong>{name}</strong></li> }
                                    }/>
                                }.into_any()
                            }
                        })}
                    </Suspense>
                </ul>
            </section>
        </div>
    }
}
