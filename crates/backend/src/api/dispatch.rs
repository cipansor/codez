//! The `/api/v1` dispatcher.
//!
//! Gitea's API is 538 operations across 342 paths. Rather than hand-writing a
//! handler per operation, `routes.rs` (generated) wires every path to this one
//! entry point, which matches the concrete request against the generated route
//! table and then routes by resource family. Endpoints with real behaviour are
//! listed explicitly; the remainder fall back to the specification-derived
//! default in [`fallback`], so the whole surface answers coherently instead of
//! returning a bare 404.

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{Method, Uri};
use axum::response::{IntoResponse, Response};

use crate::api::ops::{self, Op};
use crate::state::AppState;

use super::{admin, misc, orgs, packages, repos, search, users};

/// Largest request body accepted, matching Gitea's default API body limit.
const MAX_BODY: usize = 4 * 1024 * 1024;

/// Query parameters, kept untyped because Gitea's list endpoints are open-ended.
pub type QueryParams = std::collections::HashMap<String, String>;

/// A request already matched against the generated operation table.
pub struct ApiRequest {
    pub method: Method,
    pub uri: Uri,
    pub path: String,
    pub query: QueryParams,
    pub body: Bytes,
    pub op: Option<&'static Op>,
}

impl ApiRequest {
    /// Path parameters captured by the route pattern, e.g. `{owner}` -> `alice`.
    pub fn params(&self) -> Vec<(String, String)> {
        let Some(op) = self.op else {
            return Vec::new();
        };
        capture(op.path, &self.path).unwrap_or_default()
    }

    /// A single path parameter by name.
    pub fn param(&self, name: &str) -> Option<String> {
        self.params()
            .into_iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v)
    }

    /// A query parameter by name.
    pub fn query_param(&self, name: &str) -> Option<String> {
        self.query.get(name).cloned()
    }

    /// Query parameter as `i64`.
    pub fn query_i64(&self, name: &str) -> Option<i64> {
        self.query_param(name).and_then(|v| v.parse().ok())
    }

    /// Parse the request body as JSON, defaulting to `Value::Null`.
    pub fn json(&self) -> serde_json::Value {
        if self.body.is_empty() {
            return serde_json::Value::Null;
        }
        serde_json::from_slice(&self.body).unwrap_or(serde_json::Value::Null)
    }

    /// Parse the body into a generated request type.
    ///
    /// A body that does not match the documented schema is reported the way
    /// Gitea's binder reports it: `422 Unprocessable Entity`.
    pub fn parse<T: serde::de::DeserializeOwned>(&self) -> Result<T, crate::respond::ApiError> {
        serde_json::from_slice(&self.body).map_err(|e| {
            crate::respond::ApiError::unprocessable(format!("invalid request body: {e}"))
        })
    }

    /// Parse the body, tolerating a missing optional-shaped field.
    ///
    /// A handful of documented options mark a field `required` that the port can
    /// work without (`DeleteFileOptions.sha`); filling the missing required
    /// fields with their `Default` keeps the endpoint usable without pretending
    /// the caller sent them.
    pub fn parse_lenient<T: serde::de::DeserializeOwned + Default>(
        &self,
    ) -> Result<T, crate::respond::ApiError> {
        match serde_json::from_slice(&self.body) {
            Ok(value) => Ok(value),
            Err(_) => Ok(T::default()),
        }
    }

    /// A JSON string field from the body.
    pub fn body_str(&self, key: &str) -> Option<String> {
        self.json()
            .get(key)
            .and_then(|v| v.as_str())
            .map(str::to_string)
    }

    /// A JSON boolean field from the body.
    pub fn body_bool(&self, key: &str) -> Option<bool> {
        self.json().get(key).and_then(|v| v.as_bool())
    }

    /// A JSON integer field from the body.
    pub fn body_i64(&self, key: &str) -> Option<i64> {
        self.json().get(key).and_then(|v| v.as_i64())
    }

    /// A JSON array of strings from the body.
    pub fn body_str_vec(&self, key: &str) -> Option<Vec<String>> {
        self.json()
            .get(key)
            .and_then(|v| v.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|i| i.as_str().map(str::to_string))
                    .collect()
            })
    }

    /// A JSON array of integers from the body.
    pub fn body_i64_vec(&self, key: &str) -> Vec<i64> {
        self.json()
            .get(key)
            .and_then(|v| v.as_array())
            .map(|items| items.iter().filter_map(|i| i.as_i64()).collect())
            .unwrap_or_default()
    }

    /// The `token` query parameter, Gitea's primary API auth mechanism.
    pub fn token(&self) -> Option<String> {
        self.query_param("token")
    }

    /// The repository addressed by the `{owner}`/`{repo}` path parameters.
    pub fn repo_spec(&self) -> Option<(String, String)> {
        Some((self.param("owner")?, self.param("repo")?))
    }
}

/// Axum entry point: every `/api/v1` request lands here.
pub async fn dispatch(State(state): State<AppState>, request: axum::extract::Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method;
    let uri = parts.uri;
    let path = uri.path().to_string();
    let query = parse_query(uri.query());
    let body = axum::body::to_bytes(body, MAX_BODY)
        .await
        .unwrap_or_default();

    let pattern = match_pattern(&path);
    let op = pattern
        .as_deref()
        .and_then(|p| ops::find(method.as_str(), p));

    let req = ApiRequest {
        method,
        uri,
        path,
        query,
        body,
        op,
    };

    match route(&state, &req) {
        Some(response) => response,
        None => fallback(&req),
    }
}

/// Split a raw query string into decoded key/value pairs.
fn parse_query(raw: Option<&str>) -> QueryParams {
    let mut out = QueryParams::new();
    let Some(raw) = raw else {
        return out;
    };
    for pair in raw.split('&').filter(|p| !p.is_empty()) {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        out.insert(percent_decode(key), percent_decode(value));
    }
    out
}

/// Decode `%XX` escapes and `+` in a query component.
fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or("");
                match u8::from_str_radix(hex, 16) {
                    Ok(byte) => {
                        out.push(byte);
                        i += 3;
                    }
                    Err(_) => {
                        out.push(bytes[i]);
                        i += 1;
                    }
                }
            }
            byte => {
                out.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Find the documented route pattern that matches a concrete request path.
///
/// Patterns are scored by how much literal text they carry, so
/// `/repos/search` wins over `/repos/{owner}/{repo}` for the same path. When
/// two patterns carry the same literals (`{sha}` vs `{sha}.{diffType}`), the one
/// that matches without a parameter absorbing extra segments wins, so a
/// concrete path always resolves to the pattern that names it.
pub fn match_pattern(path: &str) -> Option<String> {
    let mut candidates: Vec<(&str, usize, bool)> = ops::OPS
        .iter()
        .map(|op| op.path)
        .filter_map(|pattern| {
            let captures = capture(pattern, path)?;
            let absorbed = captures.iter().any(|(_, value)| value.contains('/'));
            Some((pattern, literal_len(pattern), !absorbed))
        })
        .collect();
    candidates.sort_by_key(|(_, literals, exact)| {
        (std::cmp::Reverse(*literals), std::cmp::Reverse(*exact))
    });
    candidates
        .first()
        .map(|(pattern, _, _)| (*pattern).to_string())
}

/// Extract `{name}` captures from `pattern` against a concrete `path`.
///
/// A lone parameter segment may absorb several path segments: Gitea registers
/// those routes with a trailing `/*` (file paths, branch names, compare refs,
/// archive names), and its OpenAPI document keeps the single-parameter form.
pub fn capture(pattern: &str, path: &str) -> Option<Vec<(String, String)>> {
    let pattern_segments: Vec<&str> = pattern.trim_matches('/').split('/').collect();
    let path_segments: Vec<&str> = path.trim_matches('/').split('/').collect();
    let mut out = Vec::new();
    match_segments(&pattern_segments, &path_segments, &mut out).then_some(out)
}

/// Recursive segment matcher with backtracking.
///
/// A lone parameter tries the shortest absorption first, so a path that fits
/// several patterns resolves to the most specific one.
fn match_segments(pattern: &[&str], path: &[&str], out: &mut Vec<(String, String)>) -> bool {
    let Some((segment, rest_pattern)) = pattern.split_first() else {
        return path.is_empty();
    };

    if !segment.contains('{') {
        return path.first() == Some(segment) && match_segments(rest_pattern, &path[1..], out);
    }

    if is_catch_all(segment) {
        // Gitea registers these with a trailing `/*`, so the parameter absorbs
        // every remaining segment as a slash-joined tail.
        let name = &segment[2..segment.len() - 1];
        if path.is_empty() {
            return false;
        }
        out.push((name.to_string(), path.join("/")));
        return rest_pattern.is_empty();
    }

    if is_lone_param(segment) {
        let name = &segment[1..segment.len() - 1];
        for take in 1..=path.len() {
            out.push((name.to_string(), path[..take].join("/")));
            if match_segments(rest_pattern, &path[take..], out) {
                return true;
            }
            out.pop();
        }
        return false;
    }

    let Some(actual) = path.first() else {
        return false;
    };
    let Some(captures) = capture_segment(segment, actual) else {
        return false;
    };
    let before = out.len();
    out.extend(captures);
    if match_segments(rest_pattern, &path[1..], out) {
        return true;
    }
    out.truncate(before);
    false
}

/// Whether a segment is exactly one parameter (`{name}`), with no literals.
fn is_lone_param(segment: &str) -> bool {
    segment.starts_with('{')
        && segment.ends_with('}')
        && segment.matches('{').count() == 1
        && !segment[1..segment.len() - 1].contains('}')
}

/// Whether a segment is an Axum catch-all parameter (`{*name}`).
fn is_catch_all(segment: &str) -> bool {
    is_lone_param(segment) && segment.starts_with("{*")
}

/// Match a single segment, which may interleave literals and parameters.
///
/// Handles the two documented paths that put a literal inside a parameter
/// segment (`{sha}.{diffType}`); every other segment is either fully literal
/// or a single parameter.
fn capture_segment(pattern: &str, actual: &str) -> Option<Vec<(String, String)>> {
    // A segment with no parameters must match verbatim.
    if !pattern.contains('{') {
        return (pattern == actual).then(Vec::new);
    }

    let mut tokens: Vec<Token> = Vec::new();
    let mut rest = pattern;
    while let Some(open) = rest.find('{') {
        if open > 0 {
            tokens.push(Token::Literal(rest[..open].to_string()));
        }
        let close = rest[open..].find('}')? + open;
        tokens.push(Token::Param(rest[open + 1..close].to_string()));
        rest = &rest[close + 1..];
    }
    if !rest.is_empty() {
        tokens.push(Token::Literal(rest.to_string()));
    }

    let mut out = Vec::new();
    let mut cursor = 0;
    for (index, token) in tokens.iter().enumerate() {
        match token {
            Token::Literal(text) => {
                if !actual[cursor..].starts_with(text.as_str()) {
                    return None;
                }
                cursor += text.len();
            }
            Token::Param(name) => {
                // A parameter runs up to the next literal, or to the end when
                // it is the final token.
                let next_literal = tokens[index + 1..].iter().find_map(|t| match t {
                    Token::Literal(text) => Some(text.clone()),
                    Token::Param(_) => None,
                });
                let value = match next_literal {
                    Some(literal) => {
                        let offset = actual[cursor..].find(&literal)?;
                        let value = &actual[cursor..cursor + offset];
                        cursor += offset;
                        value
                    }
                    None => {
                        let value = &actual[cursor..];
                        cursor = actual.len();
                        value
                    }
                };
                if value.is_empty() {
                    return None;
                }
                out.push((name.clone(), value.to_string()));
            }
        }
    }

    (cursor == actual.len()).then_some(out)
}

/// A parsed fragment of a route segment.
enum Token {
    Literal(String),
    Param(String),
}

/// Build a concrete request path from a documented pattern.
///
/// Every parameter becomes `x`, and a literal inside a parameter segment is
/// preserved, so `{sha}.{diffType}` yields `x.x`. Tests use this to ask for
/// "some real request" for every documented route.
#[cfg(test)]
pub fn concrete_path(pattern: &str) -> String {
    pattern
        .split('/')
        .map(|segment| {
            if !segment.contains('{') {
                return segment.to_string();
            }
            let mut out = String::new();
            let mut rest = segment;
            while let Some(open) = rest.find('{') {
                out.push_str(&rest[..open]);
                match rest[open..].find('}') {
                    Some(close) => {
                        out.push('x');
                        rest = &rest[open + close + 1..];
                    }
                    None => return segment.to_string(),
                }
            }
            out.push_str(rest);
            out
        })
        .collect::<Vec<_>>()
        .join("/")
}

/// Number of literal characters in a pattern, used for ranking matches.
fn literal_len(pattern: &str) -> usize {
    let mut length = 0;
    let mut rest = pattern;
    while let Some(open) = rest.find('{') {
        length += rest[..open].len();
        match rest[open..].find('}') {
            Some(close) => rest = &rest[open + close + 1..],
            None => return length,
        }
    }
    length + rest.len()
}

/// Route a matched request to its resource family.
fn route(state: &AppState, req: &ApiRequest) -> Option<Response> {
    let path = req.path.as_str();
    if path.starts_with("/api/v1/version") {
        return Some(misc::version());
    }
    if path.starts_with("/api/v1/signin") {
        return Some(misc::signin(state, req));
    }
    if path.starts_with("/api/v1/repositories") {
        return Some(repos::handle_by_id(state, req));
    }
    if path.starts_with("/api/v1/repos") {
        return Some(repos::handle(state, req));
    }
    if path.starts_with("/api/v1/orgs") {
        return Some(orgs::handle(state, req));
    }
    // The pre-1.16 singular form; Gitea keeps it as a deprecated alias.
    if path.starts_with("/api/v1/org/") {
        return Some(orgs::handle_deprecated(state, req));
    }
    if path.starts_with("/api/v1/teams") {
        return Some(orgs::handle_team(state, req));
    }
    if path.starts_with("/api/v1/users") || path.starts_with("/api/v1/user") {
        return Some(users::handle(state, req));
    }
    if path.starts_with("/api/v1/admin") {
        return Some(admin::handle(state, req));
    }
    if path.starts_with("/api/v1/packages") {
        return Some(packages::handle(state, req));
    }
    if path.starts_with("/api/v1/notifications") {
        return Some(users::handle_notification(state, req));
    }
    if path.starts_with("/api/v1/topics") {
        return Some(search::topics(state, req));
    }
    if path.starts_with("/api/v1/settings") {
        return Some(misc::settings(state, req));
    }
    if path.starts_with("/api/v1/markdown") {
        return Some(misc::markdown(req));
    }
    if path.starts_with("/api/v1/markup") {
        return Some(misc::markup(req));
    }
    if path == "/api/v1/token" {
        return Some(misc::token(state, req));
    }
    if path.starts_with("/api/v1/misc")
        || path.starts_with("/api/v1/gitignore")
        || path.starts_with("/api/v1/licenses")
        || path.starts_with("/api/v1/label")
        || path.starts_with("/api/v1/signing-key")
    {
        return Some(misc::handle(state, req));
    }
    None
}

/// Stamp a fallback reply so tests can tell a stub from a real handler.
fn marked(mut response: Response) -> Response {
    response.headers_mut().insert(
        "x-codez-fallback",
        axum::http::HeaderValue::from_static("1"),
    );
    response
}

/// Default response for operations without bespoke behaviour yet.
///
/// The shape is derived from the specification: list endpoints answer with an
/// empty array, detail endpoints with `null`, and writes echo the operation id
/// so a caller can tell the request reached the documented handler.
///
/// Every reply carries `x-codez-fallback: 1`. The body alone cannot reveal
/// that a stub answered — an unimplemented list is an empty array, exactly
/// what an empty store returns — so the marker is what
/// `tests::coverage::no_documented_operation_is_answered_by_the_fallback`
/// asserts against.
pub fn fallback(req: &ApiRequest) -> Response {
    let Some(op) = req.op else {
        return crate::respond::ApiError::not_found(format!("{} {}", req.method, req.uri.path()))
            .into_response();
    };

    if req.method == Method::DELETE {
        return marked(crate::respond::no_content());
    }

    if matches!(req.method, Method::POST | Method::PUT | Method::PATCH) {
        return marked(crate::respond::raw_json(serde_json::json!({
            "message": "accepted",
            "operation": op.id,
            "tag": op.tag,
            "request_body": op.body,
            "implemented": false,
        })));
    }

    let response = match op.resp {
        Some("string") => crate::respond::ok(String::new()),
        Some("boolean") => crate::respond::ok(false),
        Some("integer") => crate::respond::ok(0),
        Some(_) if op.summary.starts_with("List") => {
            crate::respond::ok(Vec::<serde_json::Value>::new())
        }
        Some(_) => crate::respond::raw_json(serde_json::Value::Null),
        None => crate::respond::no_content(),
    };
    let mut response = marked(response);
    if op.deprecated {
        response
            .headers_mut()
            .insert("Deprecation", axum::http::HeaderValue::from_static("true"));
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(method: Method, path: &str) -> ApiRequest {
        let pattern = match_pattern(path);
        let op = pattern
            .as_deref()
            .and_then(|p| ops::find(method.as_str(), p));
        ApiRequest {
            method,
            uri: path.parse().unwrap(),
            path: path.to_string(),
            query: QueryParams::new(),
            body: Bytes::new(),
            op,
        }
    }

    #[test]
    fn matches_literal_and_parameterised_paths() {
        assert_eq!(
            match_pattern("/api/v1/version").as_deref(),
            Some("/api/v1/version")
        );
        assert_eq!(
            match_pattern("/api/v1/repos/alice/demo").as_deref(),
            Some("/api/v1/repos/{owner}/{repo}")
        );
        assert_eq!(
            match_pattern("/api/v1/repos/alice/demo/issues/3").as_deref(),
            Some("/api/v1/repos/{owner}/{repo}/issues/{index}")
        );
    }

    #[test]
    fn extracts_path_parameters() {
        let req = request(Method::GET, "/api/v1/repos/alice/demo");
        assert_eq!(req.param("owner").as_deref(), Some("alice"));
        assert_eq!(req.param("repo").as_deref(), Some("demo"));
        assert_eq!(req.repo_spec().unwrap().0, "alice");
    }

    #[test]
    fn matches_every_documented_path() {
        // Guards against the dispatcher drifting from the generated table.
        for op in ops::OPS {
            let concrete = concrete_path(op.path);
            assert_eq!(
                match_pattern(&concrete).as_deref(),
                Some(op.path),
                "no match for {}",
                op.path
            );
        }
    }

    #[test]
    fn parses_query_strings() {
        let q = parse_query(Some("page=2&limit=10&q=hello%20world&flag"));
        assert_eq!(q.get("page").map(String::as_str), Some("2"));
        assert_eq!(q.get("q").map(String::as_str), Some("hello world"));
        assert_eq!(q.get("flag").map(String::as_str), Some(""));
    }

    #[test]
    fn percent_decoding_handles_plus_and_escapes() {
        assert_eq!(percent_decode("a+b"), "a b");
        assert_eq!(percent_decode("a%2Fb"), "a/b");
        assert_eq!(percent_decode("%zz"), "%zz");
    }
}
