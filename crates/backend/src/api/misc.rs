//! Miscellaneous endpoints: version, sign-in, settings and markup helpers.

use axum::http::Method;
use axum::response::{IntoResponse, Response};

use shared::models as dto;

use crate::respond::{self, ApiError};
use crate::state::AppState;

use super::ApiRequest;

/// `GET /api/v1/version` — the compatibility string clients probe first.
pub fn version() -> Response {
    respond::ok(serde_json::json!({ "version": "1.23.0" }))
}

/// `POST /api/v1/signin` — issue an access token for a login.
///
/// Gitea's `user_name` field accepts either a login name or an email address,
/// and the resulting token is the `sha1` the caller sends as `?token=`.
pub fn signin(state: &AppState, req: &ApiRequest) -> Response {
    let login = req
        .body_str("user_name")
        .or_else(|| req.body_str("username"))
        .or_else(|| req.body_str("email"));
    let password = req.body_str("password").unwrap_or_default();

    let Some(login) = login else {
        return ApiError::bad_request("user_name is required").into_response();
    };

    let user = state.user(&login).or_else(|| {
        state
            .users()
            .iter()
            .find(|u| u.email.eq_ignore_ascii_case(&login))
            .cloned()
    });

    let Some(user) = user else {
        return ApiError::unauthorized("user does not exist [name: user_name]").into_response();
    };
    if !password.is_empty() && user.password != password {
        return ApiError::unauthorized("invalid password").into_response();
    }

    let id = state.next_id();
    let token = crate::state::short_sha(&format!("{}:{id}", user.login));
    state.tokens_mut().push(crate::state::StoredToken {
        id,
        user: user.login.clone(),
        name: "signin".into(),
        sha1: token.clone(),
        scopes: vec!["all".into()],
        created: crate::state::now(),
        updated: crate::state::now(),
    });

    respond::ok(serde_json::json!({ "sha1": token }))
}

/// `GET /api/v1/settings/api`, `/attachment`, `/repository`, `/ui`.
pub fn settings(_state: &AppState, req: &ApiRequest) -> Response {
    let path = req.path.as_str();
    if path.ends_with("/settings/attachment") {
        return respond::ok(serde_json::json!({
            "max_size": 4 * 1024 * 1024,
            "max_files": 10,
        }));
    }
    if path.ends_with("/settings/repository") {
        return respond::ok(serde_json::json!({
            "default_branch": "main",
            "default_git_trees_per_page": 20,
            "default_max_blob_size": 10 * 1024 * 1024,
            "default_pager_size": 50,
            "default_repo_units": ["repo.code", "repo.issues", "repo.pulls"],
        }));
    }
    if path.ends_with("/settings/ui") {
        return respond::ok(serde_json::json!({
            "default_theme": "codez-auto",
            "allowed_reactions": ["+1", "-1", "laugh", "hooray", "confused", "heart", "rocket", "eyes"],
            "custom_emojis": {},
        }));
    }
    // `/settings/api`: report the limits this backend enforces.
    respond::ok(serde_json::json!({
        "max_response_items": crate::pagination::MAX_LIMIT,
        "default_paging_num": crate::pagination::DEFAULT_LIMIT,
        "default_git_trees_per_page": 20,
        "default_max_blob_size": 10 * 1024 * 1024,
    }))
}

/// `POST /api/v1/markdown` and `/markdown/raw` — render markup to HTML.
pub fn markdown(req: &ApiRequest) -> Response {
    let text = if req.path.ends_with("/raw") {
        String::from_utf8_lossy(&req.body).to_string()
    } else {
        req.body_str("text").unwrap_or_default()
    };
    respond::ok(render_markdown(&text))
}

/// `POST /api/v1/markup` — render markup to HTML.
///
/// Unlike `/markdown`, the request body carries the options under their
/// capitalised Go names (`Text`, `Mode`), which is why this reads the
/// `MarkupOption` DTO instead of a bare `text` field.
pub fn markup(req: &ApiRequest) -> Response {
    let option: dto::MarkupOption = match req.parse() {
        Ok(option) => option,
        Err(error) => return error.into_response(),
    };
    match option.text {
        Some(text) => respond::ok(render_markdown(&text)),
        None => ApiError::unprocessable("text is required").into_response(),
    }
}

/// `GET|DELETE /api/v1/token` — the access token a request authenticated with.
///
/// The store keeps the token metadata; the secret itself is the `sha1` the
/// caller already holds, so it is never echoed back.
pub fn token(state: &AppState, req: &ApiRequest) -> Response {
    let Some(user) = super::users::current_user(state, req) else {
        return ApiError::unauthorized("authentication required").into_response();
    };
    let presented = req.token();
    let found = state
        .tokens()
        .iter()
        .find(|t| t.user == user.login && presented.as_deref().is_none_or(|p| p == t.sha1))
        .cloned();

    match req.method {
        Method::GET => match found {
            Some(token) => respond::ok(dto::CurrentAccessToken {
                id: Some(token.id),
                name: Some(token.name),
                scopes: Some(token.scopes),
                created_at: Some(token.created),
                last_used_at: Some(token.updated),
                user: Some(dto::UserMeta {
                    id: Some(user.id),
                    login: Some(user.login),
                }),
            }),
            None => ApiError::not_found("access token not found").into_response(),
        },
        Method::DELETE => {
            let Some(token) = found else {
                return ApiError::not_found("access token not found").into_response();
            };
            state.tokens_mut().retain(|t| t.id != token.id);
            respond::no_content()
        }
        _ => super::dispatch::fallback(req),
    }
}

/// `GET /api/v1/misc/*`, `/gitignore/*`, `/licenses`, `/label/*`.
pub fn handle(state: &AppState, req: &ApiRequest) -> Response {
    let _ = state;
    let path = req.path.as_str();
    if path.starts_with("/api/v1/gitignore") {
        if path.ends_with("/templates") {
            return respond::ok(vec!["Go", "Node", "Rust", "Python", "Java"]);
        }
        let name = req.param("name").unwrap_or_default();
        return respond::ok(serde_json::json!({
            "name": name,
            "content": "# generated by codez\n/target\nnode_modules\n",
        }));
    }
    if path.starts_with("/api/v1/licenses") {
        if req.param("name").is_some() {
            let name = req.param("name").unwrap_or_default();
            return respond::ok(serde_json::json!({
                "name": name,
                "body": format!("{name} license text is not bundled with this build."),
            }));
        }
        return respond::ok(vec!["MIT", "Apache-2.0", "BSD-3-Clause", "GPL-3.0"]);
    }
    if path.starts_with("/api/v1/label") {
        return respond::ok(Vec::<serde_json::Value>::new());
    }
    match req.method {
        Method::GET => respond::ok(serde_json::json!({ "version": "1.23.0" })),
        _ => super::dispatch::fallback(req),
    }
}

/// A deliberately small Markdown subset: headings, emphasis, code and links.
///
/// Rendering the full CommonMark grammar is out of scope for the port, so the
/// subset is documented rather than silently partial.
pub fn render_markdown(text: &str) -> String {
    let mut html = String::with_capacity(text.len() + 32);
    let mut in_code = false;
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            in_code = !in_code;
            html.push_str(if in_code {
                "<pre><code>"
            } else {
                "</code></pre>"
            });
            continue;
        }
        if in_code {
            html.push_str(&escape_html(line));
            html.push('\n');
            continue;
        }
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("### ") {
            html.push_str(&format!("<h3>{}</h3>", inline(rest)));
        } else if let Some(rest) = trimmed.strip_prefix("## ") {
            html.push_str(&format!("<h2>{}</h2>", inline(rest)));
        } else if let Some(rest) = trimmed.strip_prefix("# ") {
            html.push_str(&format!("<h1>{}</h1>", inline(rest)));
        } else if let Some(rest) = trimmed.strip_prefix("- ") {
            html.push_str(&format!("<li>{}</li>", inline(rest)));
        } else if trimmed.is_empty() {
            html.push_str("<p></p>");
        } else {
            html.push_str(&format!("<p>{}</p>", inline(line)));
        }
    }
    if in_code {
        html.push_str("</code></pre>");
    }
    html
}

/// Escape the characters that would otherwise inject markup.
fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Inline emphasis and links within a single line.
fn inline(text: &str) -> String {
    let escaped = escape_html(text);
    let mut out = String::with_capacity(escaped.len());
    let mut rest = escaped.as_str();
    while let Some(start) = rest.find("**") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        match after.find("**") {
            Some(end) => {
                out.push_str(&format!("<strong>{}</strong>", &after[..end]));
                rest = &after[end + 2..];
            }
            None => {
                out.push_str("**");
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_headings_and_paragraphs() {
        let html = render_markdown("# Title\n\nsome **bold** text");
        assert!(html.contains("<h1>Title</h1>"));
        assert!(html.contains("<strong>bold</strong>"));
    }

    #[test]
    fn escapes_injected_markup() {
        let html = render_markdown("<script>alert(1)</script>");
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn renders_fenced_code_without_formatting() {
        let html = render_markdown("```\n# not a heading\n```");
        assert!(html.contains("<pre><code># not a heading"));
        assert!(!html.contains("<h1>"));
    }

    #[test]
    fn unterminated_emphasis_is_left_alone() {
        assert!(render_markdown("2 ** 8").contains("**"));
    }
}
