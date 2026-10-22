//! Demo state, so the server has something to serve on first start.
//!
//! Mirrors the seed Gitea ships for its own demo instance: a couple of users,
//! an organization, and repositories carrying issues, labels and releases.

use crate::state::{
    now, short_sha, AppState, StoredArtifact, StoredBranch, StoredCommit, StoredFile, StoredIssue,
    StoredLabel, StoredMilestone, StoredNotification, StoredOrg, StoredPackage, StoredPackageFile,
    StoredRelease, StoredRepo, StoredRunner, StoredTeam, StoredUser, StoredWikiPage,
    StoredWikiRevision, StoredWorkflow, StoredWorkflowJob, StoredWorkflowRun,
};

/// Populate `state` with the demo dataset.
pub fn load(state: &AppState) {
    let created = "2026-01-01T00:00:00Z".to_string();

    let users = vec![
        StoredUser {
            id: 1,
            login: "alice".into(),
            full_name: "Alice Liddell".into(),
            email: "alice@example.com".into(),
            description: "Maintainer of the demo organization".into(),
            location: "Bandung".into(),
            website: "https://example.com".into(),
            is_admin: true,
            active: true,
            created: created.clone(),
            password: "password".into(),
        },
        StoredUser {
            id: 2,
            login: "bob".into(),
            full_name: "Bob Builder".into(),
            email: "bob@example.com".into(),
            active: true,
            created: created.clone(),
            password: "password".into(),
            ..Default::default()
        },
        StoredUser {
            id: 3,
            login: "codez-org".into(),
            full_name: "Codez Org".into(),
            email: "team@codez.example".into(),
            active: true,
            created: created.clone(),
            ..Default::default()
        },
    ];
    *state.users_mut() = users;

    *state.orgs_mut() = vec![StoredOrg {
        id: 10,
        name: "codez-org".into(),
        full_name: "Codez".into(),
        description: "The organization behind Codez".into(),
        email: "team@codez.example".into(),
        location: "Bandung".into(),
        website: "https://codez.example".into(),
        visibility: "public".into(),
        created: created.clone(),
        members: vec!["alice".into(), "bob".into()],
    }];

    *state.teams_mut() = vec![StoredTeam {
        id: 11,
        org: "codez-org".into(),
        name: "owners".into(),
        description: "Organization owners".into(),
        permission: "owner".into(),
        members: vec!["alice".into()],
        units: vec!["repo.code".into(), "repo.issues".into()],
    }];

    let repos = vec![
        StoredRepo {
            id: 100,
            owner: "codez-org".into(),
            name: "codez".into(),
            description: "A Gitea-compatible code hosting platform in Rust".into(),
            private: false,
            empty: false,
            default_branch: "main".into(),
            website: "https://codez.example".into(),
            created: created.clone(),
            updated: created.clone(),
            topics: vec!["rust".into(), "axum".into(), "leptos".into()],
            stars: vec!["bob".into()],
            watchers: vec!["alice".into(), "bob".into()],
            size_kb: 512,
            ..Default::default()
        },
        StoredRepo {
            id: 101,
            owner: "alice".into(),
            name: "notes".into(),
            description: "Personal notes and drafts".into(),
            private: true,
            empty: false,
            default_branch: "main".into(),
            created: created.clone(),
            updated: created.clone(),
            size_kb: 32,
            ..Default::default()
        },
    ];
    *state.repos_mut() = repos;

    *state.branches_mut() = vec![
        StoredBranch {
            repo_id: 100,
            name: "main".into(),
            protected: true,
            sha: short_sha("codez:main"),
            from: None,
        },
        StoredBranch {
            repo_id: 100,
            name: "feat/gitea-port".into(),
            protected: false,
            sha: short_sha("codez:feat"),
            from: Some("main".into()),
        },
        StoredBranch {
            repo_id: 101,
            name: "main".into(),
            protected: false,
            sha: short_sha("notes:main"),
            from: None,
        },
    ];

    *state.commits_mut() = vec![
        StoredCommit {
            repo_id: 100,
            sha: short_sha("codez:main"),
            message: "Port the Gitea API surface to Axum".into(),
            author_name: "alice".into(),
            author_email: "alice@example.com".into(),
            created: created.clone(),
            files: vec![("crates/backend/src/api/mod.rs".into(), 42, 0)],
        },
        StoredCommit {
            repo_id: 100,
            sha: short_sha("codez:feat"),
            message: "Add generated DTOs from the OpenAPI document".into(),
            author_name: "bob".into(),
            author_email: "bob@example.com".into(),
            created: "2026-01-02T00:00:00Z".into(),
            files: vec![("crates/shared/src/models.rs".into(), 5100, 0)],
        },
    ];

    *state.files_mut() = vec![
        StoredFile {
            repo_id: 100,
            branch: "main".into(),
            path: "README.md".into(),
            content: "# codez\n\nA Gitea-compatible platform in Rust.\n".into(),
            sha: short_sha("readme:100"),
            is_dir: false,
            updated: created.clone(),
        },
        StoredFile {
            repo_id: 100,
            branch: "main".into(),
            path: "Cargo.toml".into(),
            content: "[workspace]\nmembers = [\"crates/backend\", \"crates/frontend\", \"crates/shared\"]\n".into(),
            sha: short_sha("cargo:100"),
            is_dir: false,
            updated: created.clone(),
        },
        StoredFile {
            repo_id: 101,
            branch: "main".into(),
            path: "todo.md".into(),
            content: "- [x] port the API\n- [ ] port the web UI\n".into(),
            sha: short_sha("todo:101"),
            is_dir: false,
            updated: created.clone(),
        },
    ];

    *state.labels_mut() = vec![
        StoredLabel {
            id: 200,
            repo_id: 100,
            name: "bug".into(),
            color: "d73a4a".into(),
            description: "Something is not working".into(),
        },
        StoredLabel {
            id: 201,
            repo_id: 100,
            name: "enhancement".into(),
            color: "a2eeef".into(),
            description: "New feature or request".into(),
        },
    ];

    *state.milestones_mut() = vec![StoredMilestone {
        id: 300,
        repo_id: 100,
        title: "v1.0".into(),
        description: "First Gitea-compatible release".into(),
        state: "open".into(),
        deadline: Some("2026-12-31T00:00:00Z".into()),
        created: created.clone(),
        updated: created.clone(),
        closed: None,
    }];

    *state.issues_mut() = vec![
        StoredIssue {
            id: 400,
            repo_id: 100,
            index: 1,
            title: "Serve the full /api/v1 surface".into(),
            body: "Every documented operation should answer, not just the ones we use.".into(),
            state: "open".into(),
            is_pull: false,
            poster: "bob".into(),
            assignees: vec!["alice".into()],
            label_ids: vec![201],
            milestone_id: Some(300),
            created: created.clone(),
            updated: created.clone(),
            comments: Vec::new(),
            ..Default::default()
        },
        StoredIssue {
            id: 401,
            repo_id: 100,
            index: 2,
            title: "Port the web UI to Leptos".into(),
            body: "The server-rendered templates become a Leptos SPA.".into(),
            state: "open".into(),
            is_pull: true,
            poster: "alice".into(),
            assignees: Vec::new(),
            label_ids: Vec::new(),
            milestone_id: None,
            created: "2026-01-03T00:00:00Z".into(),
            updated: "2026-01-03T00:00:00Z".into(),
            comments: Vec::new(),
            head: Some("feat/leptos-ui".into()),
            base: Some("main".into()),
            merged: false,
            ..Default::default()
        },
    ];

    *state.releases_mut() = vec![StoredRelease {
        id: 500,
        repo_id: 100,
        tag_name: "v0.1.0".into(),
        name: "Initial port".into(),
        body: "The Gitea API surface now runs on Axum.".into(),
        draft: false,
        prerelease: false,
        target: "main".into(),
        publisher: "alice".into(),
        created: created.clone(),
    }];

    *state.notifications_mut() = vec![StoredNotification {
        id: 600,
        user: "alice".into(),
        repo_id: 100,
        subject_title: "Serve the full /api/v1 surface".into(),
        subject_type: "Issue".into(),
        subject_url: "/codez-org/codez/issues/1".into(),
        unread: true,
        pinned: false,
        updated: now(),
    }];

    // Seeded rows use hand-written ids; keep allocation above all of them.
    state.ids.ensure_at_least(1000);

    load_extras(state);
}

/// Seed the resources that back the CI, registry and wiki surfaces.
///
/// These are kept in a separate function so the id floor in `load` is applied
/// first; every row here also uses a hand-written id below it.
fn load_extras(state: &AppState) {
    let created = "2026-01-02T00:00:00Z".to_string();

    *state.runners_mut() = vec![
        StoredRunner {
            id: 700,
            owner: String::new(),
            name: "instance-runner".into(),
            labels: vec!["ubuntu-latest".into(), "self-hosted".into()],
            busy: false,
            disabled: false,
            ephemeral: false,
            status: "online".into(),
        },
        StoredRunner {
            id: 701,
            owner: "codez-org".into(),
            name: "org-runner".into(),
            labels: vec!["ubuntu-latest".into()],
            busy: true,
            disabled: false,
            ephemeral: false,
            status: "online".into(),
        },
    ];

    *state.packages_mut() = vec![
        StoredPackage {
            id: 800,
            owner: "codez-org".into(),
            kind: "container".into(),
            name: "codez-backend".into(),
            version: "1.0.0".into(),
            created: created.clone(),
            repository: None,
            files: vec![StoredPackageFile {
                id: 801,
                name: "codez-backend-1.0.0.tar.gz".into(),
                size: 4096,
            }],
        },
        StoredPackage {
            id: 802,
            owner: "codez-org".into(),
            kind: "container".into(),
            name: "codez-backend".into(),
            version: "1.1.0".into(),
            created: "2026-02-01T00:00:00Z".into(),
            repository: None,
            files: vec![StoredPackageFile {
                id: 803,
                name: "codez-backend-1.1.0.tar.gz".into(),
                size: 5120,
            }],
        },
        StoredPackage {
            id: 804,
            owner: "alice".into(),
            kind: "npm".into(),
            name: "leptos-widgets".into(),
            version: "0.3.0".into(),
            created: created.clone(),
            repository: None,
            files: vec![StoredPackageFile {
                id: 805,
                name: "leptos-widgets-0.3.0.tgz".into(),
                size: 2048,
            }],
        },
    ];

    *state.wiki_pages_mut() = vec![StoredWikiPage {
        repo_id: 100,
        title: "Home".into(),
        updated: created.clone(),
        revisions: vec![
            StoredWikiRevision {
                sha: short_sha("Home"),
                message: "Start the wiki".into(),
                author: "alice".into(),
                created: created.clone(),
                content: "# Codez\n\nA Gitea-inspired code host in Rust.".into(),
            },
            StoredWikiRevision {
                sha: short_sha("Home:2"),
                message: "Note the stack".into(),
                author: "alice".into(),
                created: "2026-01-04T00:00:00Z".into(),
                content: "# Codez\n\nA Gitea-inspired code host: Axum + Leptos.".into(),
            },
        ],
    }];

    *state.workflows_mut() = vec![
        StoredWorkflow {
            id: 900,
            repo_id: 100,
            name: "CI".into(),
            path: ".gitea/workflows/ci.yaml".into(),
            state: "active".into(),
        },
        StoredWorkflow {
            id: 901,
            repo_id: 100,
            name: "Release".into(),
            path: ".gitea/workflows/release.yaml".into(),
            state: "disabled_manually".into(),
        },
    ];

    let finished = "2026-01-05T09:15:00Z".to_string();
    *state.workflow_runs_mut() = vec![StoredWorkflowRun {
        id: 950,
        repo_id: 100,
        workflow_path: ".gitea/workflows/ci.yaml".into(),
        run_number: 1,
        run_attempt: 1,
        display_title: "CI".into(),
        event: "push".into(),
        head_branch: "main".into(),
        head_sha: short_sha("ci:main"),
        status: "completed".into(),
        conclusion: "success".into(),
        actor: "alice".into(),
        created: finished.clone(),
        updated: finished.clone(),
    }];

    *state.workflow_jobs_mut() = vec![StoredWorkflowJob {
        id: 960,
        run_id: 950,
        repo_id: 100,
        name: "build".into(),
        status: "completed".into(),
        conclusion: "success".into(),
        runner_id: 700,
        runner_name: "instance-runner".into(),
        started: finished.clone(),
        completed: finished.clone(),
        steps: vec![
            ("Set up job".into(), "completed".into(), "success".into()),
            ("Build".into(), "completed".into(), "success".into()),
        ],
    }];

    *state.artifacts_mut() = vec![StoredArtifact {
        id: 970,
        run_id: 950,
        repo_id: 100,
        name: "codez-linux-amd64".into(),
        size: 18_432_000,
        expired: false,
        created: finished.clone(),
        updated: finished.clone(),
    }];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_is_non_empty_and_internally_consistent() {
        let state = AppState::default();
        load(&state);

        assert_eq!(state.users().len(), 3);
        assert_eq!(state.repos().len(), 2);
        assert!(state.org("codez-org").is_some());
        assert!(state.repo("codez-org", "codez").is_some());

        // The CI, registry and wiki surfaces are seeded too, so their pages are
        // not permanently empty.
        assert!(!state.runners().is_empty());
        assert!(!state.packages().is_empty());
        assert!(!state.repo_wiki_pages(100).is_empty());
        assert!(!state.repo_workflows(100).is_empty());
        assert!(!state.workflow_runs().is_empty());

        // Every branch, issue and commit points at a repository that exists.
        for branch in state.branches().iter() {
            assert!(state.repo_by_id(branch.repo_id).is_some());
        }
        for issue in state.issues().iter() {
            assert!(state.repo_by_id(issue.repo_id).is_some());
        }
        for commit in state.commits().iter() {
            assert!(state.repo_by_id(commit.repo_id).is_some());
        }
    }

    #[test]
    fn seeded_ids_do_not_collide_with_new_ones() {
        let state = AppState::default();
        load(&state);
        let fresh = state.next_id();
        assert!(!state.repos().iter().any(|r| r.id == fresh));
        assert!(!state.users().iter().any(|u| u.id == fresh));
    }
}
