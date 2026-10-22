//! Wire types shared by the Axum backend and the Leptos frontend.
//!
//! `models` is generated from Gitea's OpenAPI 3 specification, so the type
//! surface tracks the upstream API instead of being maintained by hand.

pub mod models;
pub mod wire;

pub use models::*;

/// The API version this crate models.
pub const API_VERSION: &str = "v1";

/// Media type used for every JSON request and response.
pub const JSON_CONTENT_TYPE: &str = "application/json";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repository_round_trips_with_gitea_field_names() {
        let json = r#"{"id":7,"name":"demo","full_name":"alice/demo","private":true,
            "empty":false,"owner":{"login":"alice"},"default_branch":"main"}"#;
        let repo: Repository = serde_json::from_str(json).expect("deserialize");
        assert_eq!(repo.id, Some(7));
        assert_eq!(repo.name.as_deref(), Some("demo"));
        assert_eq!(repo.full_name.as_deref(), Some("alice/demo"));
        assert_eq!(repo.private, Some(true));
        assert_eq!(
            repo.owner.as_ref().and_then(|o| o.login.clone()).as_deref(),
            Some("alice")
        );

        let out = serde_json::to_string(&repo).expect("serialize");
        assert!(out.contains(r#""full_name":"alice/demo""#));
        assert!(!out.contains("fullName"));
    }

    #[test]
    fn optional_fields_accept_absent_keys() {
        let repo: Repository = serde_json::from_str("{}").expect("deserialize");
        assert_eq!(repo.id, None);
        assert_eq!(repo.owner, None);
    }

    #[test]
    fn required_fields_serialize_even_when_empty() {
        // `CreateRepoOption.name` is required, so it is always emitted.
        let opt = CreateRepoOption {
            name: "demo".into(),
            ..Default::default()
        };
        let out = serde_json::to_string(&opt).expect("serialize");
        assert!(out.contains(r#""name":"demo""#));
        assert!(!out.contains("description"));
    }

    #[test]
    fn representative_schemas_are_generated() {
        let _: User = User::default();
        let _: Issue = Issue::default();
        let _: PullRequest = PullRequest::default();
        let _: Organization = Organization::default();
        let _: Team = Team::default();
        let _: Release = Release::default();
        let _: Milestone = Milestone::default();
        let _: Label = Label::default();
        let _: AccessToken = AccessToken::default();
        let _: NotificationThread = NotificationThread::default();
    }
}
