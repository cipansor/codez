//! Topic search, the one search endpoint that has no owning resource.
//!
//! Gitea serves search as a sub-resource of what is being searched
//! (`/repos/search`, `/repos/issues/search`, `/users/search`,
//! `/orgs/{org}/teams/search`), so those live with their resource modules.
//! `/topics/search` matches no resource and is handled here.

use axum::response::Response;

use shared::models as dto;

use crate::pagination::Pagination;
use crate::respond;
use crate::state::AppState;

use super::ApiRequest;

/// `GET /api/v1/topics/search` — topics in use across repositories.
pub fn topics(state: &AppState, req: &ApiRequest) -> Response {
    let term = req.query_param("q").unwrap_or_default().to_lowercase();

    let mut names: Vec<String> = Vec::new();
    for repo in state.repos().iter() {
        for topic in &repo.topics {
            if (term.is_empty() || topic.to_lowercase().contains(&term))
                && !names.iter().any(|t| t == topic)
            {
                names.push(topic.clone());
            }
        }
    }
    names.sort();

    let items: Vec<dto::TopicResponse> = names
        .into_iter()
        .map(|name| dto::TopicResponse {
            topic_name: Some(name),
            ..Default::default()
        })
        .collect();

    let pagination = Pagination {
        page: req.query_i64("page"),
        limit: req.query_i64("limit"),
    };
    let total = items.len();
    respond::paged(
        dto::TopicListResponse {
            topics: Some(pagination.apply(&items)),
        },
        &pagination,
        total,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::QueryParams;
    use crate::state::StoredRepo;
    use axum::body::Bytes;
    use axum::http::{Method, StatusCode};

    fn request(query: &[(&str, &str)]) -> ApiRequest {
        let path = "/api/v1/topics/search";
        let pattern = super::super::dispatch::match_pattern(path);
        let op = pattern
            .as_deref()
            .and_then(|p| super::super::ops::find("GET", p));
        ApiRequest {
            method: Method::GET,
            uri: path.parse().unwrap(),
            path: path.to_string(),
            query: query
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect::<QueryParams>(),
            body: Bytes::new(),
            op,
        }
    }

    fn state_with_topics() -> AppState {
        let state = AppState::default();
        state.repos_mut().push(StoredRepo {
            id: 1,
            owner: "alice".into(),
            name: "demo".into(),
            topics: vec!["rust".into(), "axum".into()],
            ..Default::default()
        });
        state
    }

    #[test]
    fn lists_every_topic_without_a_filter() {
        let response = topics(&state_with_topics(), &request(&[]));
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn filters_topics_by_keyword() {
        let response = topics(&state_with_topics(), &request(&[("q", "rus")]));
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn an_empty_store_yields_an_empty_list() {
        let response = topics(&AppState::default(), &request(&[]));
        assert_eq!(response.status(), StatusCode::OK);
    }
}
