//! Small presentational helpers shared by the pages.
//!
//! They exist so pages do not each re-implement "render an empty state", "show
//! a failed write", or "turn an optional wire string into something printable".
//! Every accessor tolerates the optional fields the generated DTOs carry,
//! because Gitea marks almost every response field optional.

use leptos::prelude::*;
use shared::{Label, Repository, User};

/// A placeholder for a field the server left absent.
pub const NONE: &str = "—";

/// `Some(text)` when non-empty, otherwise `NONE`.
pub fn text(value: &Option<String>) -> String {
    value
        .as_deref()
        .filter(|v| !v.is_empty())
        .unwrap_or(NONE)
        .to_string()
}

/// The login of a user, falling back to `login_name`.
pub fn login_of(user: &Option<User>) -> String {
    user.as_ref()
        .and_then(|u| u.login.clone().or_else(|| u.login_name.clone()))
        .unwrap_or_else(|| NONE.to_string())
}

/// `owner/name` for a repository, falling back to the `owner` document.
pub fn repo_path(repo: &Repository) -> String {
    repo.full_name.clone().unwrap_or_else(|| {
        let owner = login_of(&repo.owner);
        format!("{owner}/{}", text(&repo.name))
    })
}

/// Route to a repository's overview.
pub fn repo_href(repo: &Repository) -> String {
    format!("/repos/{}", repo_path(repo))
}

/// The first line of a body, trimmed for list display.
pub fn first_line(body: &Option<String>, limit: usize) -> String {
    let value = body.as_deref().unwrap_or_default().trim();
    let line = value.lines().next().unwrap_or_default();
    let mut out: String = line.chars().take(limit).collect();
    if line.chars().count() > limit {
        out.push('…');
    }
    out
}

/// Render a failed-write alert. `role="alert"` so assistive tech announces it.
#[component]
pub fn FormError(message: String) -> impl IntoView {
    view! { <p class="form-error" role="alert">{message}</p> }
}

/// Render a "loading…" line inside a list.
#[component]
pub fn Loading(label: &'static str) -> impl IntoView {
    view! { <li class="text-muted">{label}</li> }
}

/// Render a state chip ("open" / "closed" / "merged").
#[component]
pub fn StateChip(state: Option<String>, merged: bool) -> impl IntoView {
    let (label, class) = if merged {
        ("merged".to_string(), "label label-success")
    } else {
        match state.as_deref().unwrap_or_default() {
            "closed" => ("closed".to_string(), "label label-danger"),
            other => (other.to_string(), "label label-success"),
        }
    };
    view! { <span class=class>{label}</span> }
}

/// Colour swatch for a label, falling back to a neutral tone.
#[component]
pub fn LabelChip(label: Label) -> impl IntoView {
    let name = text(&label.name);
    let color = label.color.clone().unwrap_or_else(|| "dddddd".to_string());
    let style = format!("background:#{color}");
    view! { <span class="label" style=style>{name}</span> }
}
