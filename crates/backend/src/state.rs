//! In-memory stores and the conversions that render them as Gitea DTOs.
//!
//! The domain structs here are the mutable source of truth; the `to_dto`
//! methods produce the wire types from `shared::models`. Keeping the two
//! separate means the API shape is defined once, by the specification.

use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use shared::models as dto;

use crate::ids::IdGen;
use crate::time;

/// Read a lock without propagating poisoning: a panicking handler must not
/// take the whole server down.
fn read<'a, T>(lock: &'a RwLock<T>) -> RwLockReadGuard<'a, T> {
    lock.read().unwrap_or_else(|e| e.into_inner())
}

/// Write counterpart of [`read`].
fn write<'a, T>(lock: &'a RwLock<T>) -> RwLockWriteGuard<'a, T> {
    lock.write().unwrap_or_else(|e| e.into_inner())
}

// ---------------------------------------------------------------------------
// Domain structs
// ---------------------------------------------------------------------------

/// A registered account.
#[derive(Debug, Clone, Default)]
pub struct StoredUser {
    pub id: i64,
    pub login: String,
    pub full_name: String,
    pub email: String,
    pub description: String,
    pub location: String,
    pub website: String,
    pub is_admin: bool,
    pub active: bool,
    pub created: String,
    /// Demo-only credential store; not a real password hash.
    pub password: String,
}

impl StoredUser {
    /// Render as the `User` wire type.
    pub fn to_dto(&self) -> dto::User {
        dto::User {
            id: Some(self.id),
            login: Some(self.login.clone()),
            login_name: Some(self.login.clone()),
            full_name: Some(self.full_name.clone()),
            email: Some(self.email.clone()),
            description: Some(self.description.clone()),
            location: Some(self.location.clone()),
            website: Some(self.website.clone()),
            is_admin: Some(self.is_admin),
            active: Some(self.active),
            created: Some(self.created.clone()),
            avatar_url: Some(format!("/avatars/{}", self.login)),
            ..Default::default()
        }
    }
}

/// A repository.
#[derive(Debug, Clone, Default)]
pub struct StoredRepo {
    pub id: i64,
    /// Login of the owning user or organization.
    pub owner: String,
    pub name: String,
    pub description: String,
    pub private: bool,
    /// True until the first commit lands.
    pub empty: bool,
    pub default_branch: String,
    pub website: String,
    pub created: String,
    pub updated: String,
    pub topics: Vec<String>,
    pub stars: Vec<String>,
    pub watchers: Vec<String>,
    pub forks: Vec<i64>,
    pub size_kb: i64,
    pub archived: bool,
}

impl StoredRepo {
    /// `owner/name`, the canonical repository identifier in the API.
    pub fn full_name(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }

    /// Render as the `Repository` wire type.
    pub fn to_dto(&self) -> dto::Repository {
        dto::Repository {
            id: Some(self.id),
            name: Some(self.name.clone()),
            full_name: Some(self.full_name()),
            owner: Some(dto::User {
                id: Some(self.id),
                login: Some(self.owner.clone()),
                login_name: Some(self.owner.clone()),
                ..Default::default()
            }),
            description: Some(self.description.clone()),
            private: Some(self.private),
            empty: Some(self.empty),
            fork: Some(!self.forks.is_empty()),
            default_branch: Some(self.default_branch.clone()),
            website: Some(self.website.clone()),
            html_url: Some(format!("/{}", self.full_name())),
            clone_url: Some(format!("{}.git", self.full_name())),
            ssh_url: Some(format!("git@localhost:{}.git", self.full_name())),
            stars_count: Some(self.stars.len() as i64),
            forks_count: Some(self.forks.len() as i64),
            watchers_count: Some(self.watchers.len() as i64),
            open_issues_count: Some(0),
            open_pr_counter: Some(0),
            size: Some(self.size_kb),
            created_at: Some(self.created.clone()),
            updated_at: Some(self.updated.clone()),
            archived: Some(self.archived),
            topics: Some(self.topics.clone()),
            ..Default::default()
        }
    }
}

/// A comment on an issue or pull request.
#[derive(Debug, Clone, Default)]
pub struct StoredComment {
    pub id: i64,
    pub issue_id: i64,
    pub body: String,
    pub poster: String,
    pub created: String,
    pub updated: String,
}

impl StoredComment {
    /// Render as the `Comment` wire type, given the owning repository.
    pub fn to_dto(&self, repo: &StoredRepo, index: i64) -> dto::Comment {
        dto::Comment {
            id: Some(self.id),
            body: Some(self.body.clone()),
            user: Some(dto::User {
                login: Some(self.poster.clone()),
                login_name: Some(self.poster.clone()),
                ..Default::default()
            }),
            created_at: Some(self.created.clone()),
            updated_at: Some(self.updated.clone()),
            html_url: Some(format!(
                "/{}/issues/{}/#issuecomment-{}",
                repo.full_name(),
                index,
                self.id
            )),
            issue_url: Some(format!(
                "/api/v1/repos/{}/issues/{}",
                repo.full_name(),
                index
            )),
            ..Default::default()
        }
    }
}

/// A branch protection rule.
#[derive(Debug, Clone, Default)]
pub struct StoredBranchProtection {
    pub repo_id: i64,
    pub branch_name: String,
    pub enable_push: bool,
    pub enable_push_whitelist: bool,
    pub push_whitelist_usernames: Vec<String>,
    pub required_approvals: i64,
    pub enable_approvals_whitelist: bool,
    pub approvals_whitelist_usernames: Vec<String>,
    pub block_on_rejected_reviews: bool,
    pub block_on_official_review_requests: bool,
    pub block_on_outdated_branch: bool,
    pub dismiss_stale_approvals: bool,
    pub require_signed_commits: bool,
    pub created: String,
    pub updated: String,
    /// Sort position among the repository's branch protections.
    pub priority: i64,
}

/// A tag protection rule.
#[derive(Debug, Clone, Default)]
pub struct StoredTagProtection {
    pub id: i64,
    pub repo_id: i64,
    pub name: String,
    pub whitelist_usernames: Vec<String>,
    pub whitelist_teams: Vec<String>,
    pub created: String,
}

/// A push mirror of a repository.
#[derive(Debug, Clone, Default)]
pub struct StoredPushMirror {
    pub repo_id: i64,
    pub remote_name: String,
    pub remote_address: String,
    pub interval: String,
    pub sync_on_commit: bool,
    pub use_ssh: bool,
    pub created: String,
    pub last_update: String,
    pub last_error: String,
}

/// A commit status report.
#[derive(Debug, Clone, Default)]
pub struct StoredCommitStatus {
    pub id: i64,
    pub repo_id: i64,
    pub sha: String,
    pub context: String,
    pub description: String,
    pub target_url: String,
    /// `pending`, `success`, `error` or `failure`.
    pub state: String,
    pub creator: String,
    pub created: String,
    pub updated: String,
}

impl StoredCommitStatus {
    /// Render as the `CommitStatus` wire type, given the owning repository.
    pub fn to_dto(&self, repo: &StoredRepo) -> dto::CommitStatus {
        dto::CommitStatus {
            id: Some(self.id),
            status: Some(dto::CommitStatusState::from_wire(&self.state)),
            description: Some(self.description.clone()),
            context: Some(self.context.clone()),
            target_url: Some(self.target_url.clone()),
            created_at: Some(self.created.clone()),
            updated_at: Some(self.updated.clone()),
            creator: Some(dto::User {
                login: Some(self.creator.clone()),
                ..Default::default()
            }),
            url: Some(format!(
                "/api/v1/repos/{}/statuses/{}",
                repo.full_name(),
                self.sha
            )),
        }
    }
}

/// A release (or issue) attachment.
#[derive(Debug, Clone, Default)]
pub struct StoredAttachment {
    pub id: i64,
    pub repo_id: i64,
    /// Zero for issue and comment attachments.
    pub release_id: i64,
    /// The owning issue id, or zero for release and comment attachments.
    pub issue_id: i64,
    /// The owning comment id, or zero for release and issue attachments.
    pub comment_id: i64,
    pub name: String,
    pub size: i64,
    pub uuid: String,
    pub download_count: i64,
    pub created: String,
}

impl StoredAttachment {
    /// Render as the `Attachment` wire type.
    pub fn to_dto(&self) -> dto::Attachment {
        dto::Attachment {
            id: Some(self.id),
            name: Some(self.name.clone()),
            size: Some(self.size),
            uuid: Some(self.uuid.clone()),
            download_count: Some(self.download_count),
            created_at: Some(self.created.clone()),
            browser_download_url: Some(format!("/attachments/{}", self.uuid)),
        }
    }
}

/// A pull request review.
#[derive(Debug, Clone, Default)]
pub struct StoredReview {
    pub id: i64,
    pub repo_id: i64,
    pub index: i64,
    pub body: String,
    /// `APPROVE`, `REQUEST_CHANGES` or `COMMENT`.
    pub state: String,
    pub reviewer: String,
    pub commit_id: String,
    pub dismissed: bool,
    pub official: bool,
    pub created: String,
    pub updated: String,
}

/// A line comment attached to a pull request review.
#[derive(Debug, Clone, Default)]
pub struct StoredReviewComment {
    pub id: i64,
    pub repo_id: i64,
    pub index: i64,
    /// The review the comment was submitted with.
    pub review_id: i64,
    pub body: String,
    pub path: String,
    pub position: i64,
    pub commit_id: String,
    pub resolver: Option<String>,
    pub created: String,
    pub updated: String,
}

impl StoredReviewComment {
    /// Render as the `PullReviewComment` wire type.
    pub fn to_dto(&self) -> dto::PullReviewComment {
        dto::PullReviewComment {
            id: Some(self.id),
            body: Some(self.body.clone()),
            path: Some(self.path.clone()),
            position: Some(self.position.max(0) as u64),
            original_position: Some(self.position.max(0) as u64),
            commit_id: Some(self.commit_id.clone()),
            original_commit_id: Some(self.commit_id.clone()),
            pull_request_review_id: Some(self.review_id),
            resolver: self.resolver.as_ref().map(|login| dto::User {
                login: Some(login.clone()),
                login_name: Some(login.clone()),
                ..Default::default()
            }),
            created_at: Some(self.created.clone()),
            updated_at: Some(self.updated.clone()),
            ..Default::default()
        }
    }
}

/// A user granted access to a repository beyond the owner.
#[derive(Debug, Clone, Default)]
pub struct StoredCollaborator {
    pub repo_id: i64,
    pub login: String,
    /// `read`, `write`, `admin` or `owner`.
    pub permission: String,
}

/// An issue, or a pull request when `is_pull` is set.
#[derive(Debug, Clone, Default)]
pub struct StoredIssue {
    pub id: i64,
    pub repo_id: i64,
    /// Per-repository display number.
    pub index: i64,
    pub title: String,
    pub body: String,
    /// `open` or `closed`.
    pub state: String,
    pub is_pull: bool,
    pub poster: String,
    pub assignees: Vec<String>,
    pub label_ids: Vec<i64>,
    pub milestone_id: Option<i64>,
    pub created: String,
    pub updated: String,
    pub closed: Option<String>,
    pub comments: Vec<StoredComment>,
    /// Head and base branch, for pull requests only.
    pub head: Option<String>,
    pub base: Option<String>,
    pub merged: bool,
    /// Issue indices this issue blocks.
    pub blocks: Vec<i64>,
    /// Issue indices this issue depends on.
    pub dependencies: Vec<i64>,
    /// Logins watching the issue.
    pub subscribers: Vec<String>,
    pub locked: bool,
    pub pinned: bool,
    pub pin_order: i64,
    /// RFC 3339 deadline, when one is set.
    pub deadline: Option<String>,
    /// Seconds tracked by the issue stopwatch.
    pub stopwatch_seconds: Option<i64>,
    /// Logins asked to review the pull request.
    pub requested_reviewers: Vec<String>,
}

impl StoredIssue {
    /// Render as the `Issue` wire type.
    pub fn to_dto(
        &self,
        repo: &StoredRepo,
        labels: &[dto::Label],
        milestone: Option<dto::Milestone>,
    ) -> dto::Issue {
        dto::Issue {
            id: Some(self.id),
            number: Some(self.index),
            title: Some(self.title.clone()),
            body: Some(self.body.clone()),
            state: Some(dto::StateType::from_wire(&self.state)),
            user: Some(dto::User {
                login: Some(self.poster.clone()),
                login_name: Some(self.poster.clone()),
                ..Default::default()
            }),
            assignee: self.assignees.first().map(|login| dto::User {
                login: Some(login.clone()),
                login_name: Some(login.clone()),
                ..Default::default()
            }),
            assignees: Some(
                self.assignees
                    .iter()
                    .map(|login| dto::User {
                        login: Some(login.clone()),
                        login_name: Some(login.clone()),
                        ..Default::default()
                    })
                    .collect(),
            ),
            labels: Some(labels.to_vec()),
            milestone,
            comments: Some(self.comments.len() as i64),
            is_locked: Some(false),
            created_at: Some(self.created.clone()),
            updated_at: Some(self.updated.clone()),
            closed_at: self.closed.clone(),
            due_date: None,
            url: Some(format!(
                "/api/v1/repos/{}/issues/{}",
                repo.full_name(),
                self.index
            )),
            html_url: Some(format!("/{}/issues/{}", repo.full_name(), self.index)),
            repository: Some(dto::RepositoryMeta {
                id: Some(repo.id),
                name: Some(repo.name.clone()),
                owner: Some(repo.owner.clone()),
                full_name: Some(repo.full_name()),
            }),
            ..Default::default()
        }
    }

    /// Render as the `PullRequest` wire type.
    pub fn to_pull_dto(&self, repo: &StoredRepo, labels: &[dto::Label]) -> dto::PullRequest {
        let branch = |name: &Option<String>| -> Option<dto::PRBranchInfo> {
            name.as_ref().map(|n| dto::PRBranchInfo {
                label: Some(n.clone()),
                ref_: Some(n.clone()),
                sha: Some(short_sha(&format!("{}:{}", self.id, n))),
                repo_id: Some(repo.id),
                repo: Some(repo.to_dto()),
            })
        };
        dto::PullRequest {
            id: Some(self.id),
            number: Some(self.index),
            title: Some(self.title.clone()),
            body: Some(self.body.clone()),
            state: Some(dto::StateType::from_wire(&self.state)),
            user: Some(dto::User {
                login: Some(self.poster.clone()),
                login_name: Some(self.poster.clone()),
                ..Default::default()
            }),
            labels: Some(labels.to_vec()),
            milestone: None,
            comments: Some(self.comments.len() as i64),
            created_at: Some(self.created.clone()),
            updated_at: Some(self.updated.clone()),
            closed_at: self.closed.clone(),
            merged: Some(self.merged),
            merged_at: if self.merged {
                self.closed.clone()
            } else {
                None
            },
            mergeable: Some(!self.merged),
            base: branch(&self.base),
            head: branch(&self.head),
            diff_url: Some(format!("/{}/pulls/{}.diff", repo.full_name(), self.index)),
            patch_url: Some(format!("/{}/pulls/{}.patch", repo.full_name(), self.index)),
            html_url: Some(format!("/{}/pulls/{}", repo.full_name(), self.index)),
            url: Some(format!(
                "/api/v1/repos/{}/pulls/{}",
                repo.full_name(),
                self.index
            )),
            ..Default::default()
        }
    }
}

/// A label, scoped either to a repository (`repo_id > 0`) or to an
/// organization (`repo_id` is the negated organization id).
#[derive(Debug, Clone, Default)]
pub struct StoredLabel {
    pub id: i64,
    pub repo_id: i64,
    pub name: String,
    pub color: String,
    pub description: String,
}

impl StoredLabel {
    /// Render as the `Label` wire type.
    pub fn to_dto(&self, repo: &StoredRepo) -> dto::Label {
        dto::Label {
            id: Some(self.id),
            name: Some(self.name.clone()),
            color: Some(self.color.clone()),
            description: Some(self.description.clone()),
            exclusive: Some(false),
            is_archived: Some(false),
            url: Some(format!(
                "/api/v1/repos/{}/labels/{}",
                repo.full_name(),
                self.id
            )),
        }
    }
}

/// A repository milestone.
#[derive(Debug, Clone, Default)]
pub struct StoredMilestone {
    pub id: i64,
    pub repo_id: i64,
    pub title: String,
    pub description: String,
    /// `open` or `closed`.
    pub state: String,
    pub deadline: Option<String>,
    pub created: String,
    pub updated: String,
    pub closed: Option<String>,
}

impl StoredMilestone {
    /// Render as the `Milestone` wire type.
    pub fn to_dto(&self, _repo: &StoredRepo, open: i64, closed: i64) -> dto::Milestone {
        dto::Milestone {
            id: Some(self.id),
            title: Some(self.title.clone()),
            description: Some(self.description.clone()),
            state: Some(dto::StateType::from_wire(&self.state)),
            open_issues: Some(open),
            closed_issues: Some(closed),
            due_on: self.deadline.clone(),
            created_at: Some(self.created.clone()),
            updated_at: Some(self.updated.clone()),
            closed_at: self.closed.clone(),
        }
    }
}

/// A repository release.
#[derive(Debug, Clone, Default)]
pub struct StoredRelease {
    pub id: i64,
    pub repo_id: i64,
    pub tag_name: String,
    pub name: String,
    pub body: String,
    pub draft: bool,
    pub prerelease: bool,
    pub target: String,
    pub publisher: String,
    pub created: String,
}

impl StoredRelease {
    /// Render as the `Release` wire type.
    pub fn to_dto(&self, repo: &StoredRepo) -> dto::Release {
        dto::Release {
            id: Some(self.id),
            tag_name: Some(self.tag_name.clone()),
            target_commitish: Some(self.target.clone()),
            name: Some(self.name.clone()),
            body: Some(self.body.clone()),
            draft: Some(self.draft),
            prerelease: Some(self.prerelease),
            author: Some(dto::User {
                login: Some(self.publisher.clone()),
                login_name: Some(self.publisher.clone()),
                ..Default::default()
            }),
            created_at: Some(self.created.clone()),
            published_at: Some(self.created.clone()),
            html_url: Some(format!(
                "/{}/releases/tag/{}",
                repo.full_name(),
                self.tag_name
            )),
            tarball_url: Some(format!(
                "/{}/archive/{}.tar.gz",
                repo.full_name(),
                self.tag_name
            )),
            zipball_url: Some(format!(
                "/{}/archive/{}.zip",
                repo.full_name(),
                self.tag_name
            )),
            ..Default::default()
        }
    }
}

/// An organization team.
#[derive(Debug, Clone, Default)]
pub struct StoredTeam {
    pub id: i64,
    pub org: String,
    pub name: String,
    pub description: String,
    /// `none`, `read`, `write`, `admin` or `owner`.
    pub permission: String,
    pub members: Vec<String>,
    pub units: Vec<String>,
}

impl StoredTeam {
    /// Render as the `Team` wire type.
    pub fn to_dto(&self, org: &StoredOrg) -> dto::Team {
        dto::Team {
            id: Some(self.id),
            name: Some(self.name.clone()),
            description: Some(self.description.clone()),
            permission: Some(dto::AccessLevelName::from_wire(&self.permission)),
            organization: Some(org.to_dto()),
            units: Some(self.units.clone()),
            can_create_org_repo: Some(true),
            includes_all_repositories: Some(
                self.permission == "owner" || self.permission == "admin",
            ),
            ..Default::default()
        }
    }
}

/// An organization.
#[derive(Debug, Clone, Default)]
pub struct StoredOrg {
    pub id: i64,
    pub name: String,
    pub full_name: String,
    pub description: String,
    pub email: String,
    pub location: String,
    pub website: String,
    pub visibility: String,
    pub created: String,
    pub members: Vec<String>,
}

impl StoredOrg {
    /// Render as the `Organization` wire type.
    pub fn to_dto(&self) -> dto::Organization {
        dto::Organization {
            id: Some(self.id),
            name: Some(self.name.clone()),
            full_name: Some(self.full_name.clone()),
            description: Some(self.description.clone()),
            email: Some(self.email.clone()),
            location: Some(self.location.clone()),
            website: Some(self.website.clone()),
            visibility: Some(dto::VisibilityString::from_wire(&self.visibility)),
            avatar_url: Some(format!("/avatars/{}", self.name)),
            ..Default::default()
        }
    }
}

/// An API access token.
#[derive(Debug, Clone, Default)]
pub struct StoredToken {
    pub id: i64,
    pub user: String,
    pub name: String,
    pub sha1: String,
    pub scopes: Vec<String>,
    pub created: String,
    pub updated: String,
}

impl StoredToken {
    /// Render as the `AccessToken` wire type.
    pub fn to_dto(&self) -> dto::AccessToken {
        dto::AccessToken {
            id: Some(self.id),
            name: Some(self.name.clone()),
            sha1: Some(self.sha1.clone()),
            token_last_eight: Some(last_eight(&self.sha1)),
            scopes: Some(self.scopes.clone()),
            created_at: Some(self.created.clone()),
            last_used_at: Some(self.updated.clone()),
        }
    }
}

/// An SSH or GPG public key.
#[derive(Debug, Clone, Default)]
pub struct StoredKey {
    pub id: i64,
    pub user: String,
    pub title: String,
    pub key: String,
    pub fingerprint: String,
    pub created: String,
    pub updated: String,
    /// `ssh` or `gpg`.
    pub kind: String,
}

impl StoredKey {
    /// Render as the `PublicKey` wire type.
    pub fn to_dto(&self) -> dto::PublicKey {
        dto::PublicKey {
            id: Some(self.id),
            title: Some(self.title.clone()),
            key: Some(self.key.clone()),
            fingerprint: Some(self.fingerprint.clone()),
            key_type: Some("ssh-rsa".into()),
            read_only: Some(false),
            url: Some(format!("/api/v1/user/keys/{}", self.id)),
            user: None,
            created_at: Some(self.created.clone()),
            last_used_at: Some(self.updated.clone()),
        }
    }

    /// Render as the `DeployKey` wire type used by the repository key routes.
    pub fn to_deploy_dto(&self) -> dto::DeployKey {
        dto::DeployKey {
            id: Some(self.id),
            title: Some(self.title.clone()),
            key: Some(self.key.clone()),
            fingerprint: Some(self.fingerprint.clone()),
            key_type: Some(dto::DeployKeyKeyType::Ssh),
            read_only: Some(true),
            created_at: Some(self.created.clone()),
            ..Default::default()
        }
    }
}

/// A notification thread.
#[derive(Debug, Clone, Default)]
pub struct StoredNotification {
    pub id: i64,
    pub user: String,
    pub repo_id: i64,
    pub subject_title: String,
    /// `Issue`, `Pull` or `Repository`.
    pub subject_type: String,
    pub subject_url: String,
    pub unread: bool,
    pub pinned: bool,
    pub updated: String,
}

impl StoredNotification {
    /// Render as the `NotificationThread` wire type.
    pub fn to_dto(&self, repo: Option<&StoredRepo>) -> dto::NotificationThread {
        dto::NotificationThread {
            id: Some(self.id),
            unread: Some(self.unread),
            pinned: Some(self.pinned),
            updated_at: Some(self.updated.clone()),
            url: Some(format!("/api/v1/notifications/threads/{}", self.id)),
            repository: repo.map(|r| r.to_dto()),
            subject: Some(dto::NotificationSubject {
                title: Some(self.subject_title.clone()),
                type_: Some(dto::NotificationSubjectType::from_wire(&self.subject_type)),
                state: Some(dto::NotificationSubjectState::Open),
                latest_comment_html_url: None,
                url: Some(self.subject_url.clone()),
                html_url: Some(self.subject_url.clone()),
                latest_comment_url: None,
            }),
        }
    }
}

/// A branch head in a repository.
#[derive(Debug, Clone, Default)]
pub struct StoredBranch {
    pub repo_id: i64,
    pub name: String,
    pub protected: bool,
    pub sha: String,
    /// Branch this one was created from, when known.
    pub from: Option<String>,
}

impl StoredBranch {
    /// Render as the `Branch` wire type.
    pub fn to_dto(&self, commit: Option<dto::PayloadCommit>) -> dto::Branch {
        dto::Branch {
            name: Some(self.name.clone()),
            protected: Some(self.protected),
            commit,
            effective_branch_protection_name: None,
            enable_status_check: Some(false),
            required_approvals: Some(0),
            status_check_contexts: Some(Vec::new()),
            user_can_merge: Some(true),
            user_can_push: Some(true),
        }
    }
}

/// A commit recorded in a repository.
#[derive(Debug, Clone, Default)]
pub struct StoredCommit {
    pub repo_id: i64,
    pub sha: String,
    pub message: String,
    pub author_name: String,
    pub author_email: String,
    pub created: String,
    /// Changed paths with their line deltas.
    pub files: Vec<(String, i64, i64)>,
}

impl StoredCommit {
    /// Render as the `Commit` wire type.
    pub fn to_dto(&self, repo: &StoredRepo) -> dto::Commit {
        dto::Commit {
            sha: Some(self.sha.clone()),
            created: Some(self.created.clone()),
            html_url: Some(format!("/{}/commit/{}", repo.full_name(), self.sha)),
            url: Some(format!(
                "/api/v1/repos/{}/git/commits/{}",
                repo.full_name(),
                self.sha
            )),
            commit: Some(dto::RepoCommit {
                message: Some(self.message.clone()),
                url: Some(format!(
                    "/api/v1/repos/{}/git/commits/{}",
                    repo.full_name(),
                    self.sha
                )),
                author: Some(dto::CommitUser {
                    name: Some(self.author_name.clone()),
                    email: Some(self.author_email.clone()),
                    date: Some(self.created.clone()),
                }),
                committer: Some(dto::CommitUser {
                    name: Some(self.author_name.clone()),
                    email: Some(self.author_email.clone()),
                    date: Some(self.created.clone()),
                }),
                ..Default::default()
            }),
            author: Some(dto::User {
                login: Some(self.author_name.clone()),
                login_name: Some(self.author_name.clone()),
                ..Default::default()
            }),
            committer: Some(dto::User {
                login: Some(self.author_name.clone()),
                login_name: Some(self.author_name.clone()),
                ..Default::default()
            }),
            files: Some(
                self.files
                    .iter()
                    .map(|(name, _additions, _deletions)| dto::CommitAffectedFiles {
                        filename: Some(name.clone()),
                        status: Some("modified".into()),
                    })
                    .collect(),
            ),
            parents: Some(Vec::new()),
            ..Default::default()
        }
    }
}

/// A file or directory in a repository tree.
#[derive(Debug, Clone, Default)]
pub struct StoredFile {
    pub repo_id: i64,
    pub branch: String,
    pub path: String,
    pub content: String,
    pub sha: String,
    pub is_dir: bool,
    pub updated: String,
}

impl StoredFile {
    /// Render as the `ContentsResponse` wire type.
    pub fn to_contents_dto(&self, repo: &StoredRepo) -> dto::ContentsResponse {
        dto::ContentsResponse {
            name: Some(
                self.path
                    .rsplit('/')
                    .next()
                    .unwrap_or(&self.path)
                    .to_string(),
            ),
            path: Some(self.path.clone()),
            sha: Some(self.sha.clone()),
            type_: Some(if self.is_dir {
                "dir".into()
            } else {
                "file".into()
            }),
            size: Some(self.content.len() as i64),
            content: if self.is_dir {
                None
            } else {
                Some(base64(&self.content))
            },
            encoding: if self.is_dir {
                None
            } else {
                Some("base64".into())
            },
            url: Some(format!(
                "/api/v1/repos/{}/contents/{}",
                repo.full_name(),
                self.path
            )),
            html_url: Some(format!(
                "/{}/src/branch/{}/{}",
                repo.full_name(),
                self.branch,
                self.path
            )),
            ..Default::default()
        }
    }
}

/// A follow edge between two accounts.
#[derive(Debug, Clone, Default)]
pub struct StoredFollow {
    pub user: String,
    pub target: String,
    pub created: String,
}

/// A user blocking another user.
#[derive(Debug, Clone, Default)]
pub struct StoredBlock {
    pub user: String,
    pub target: String,
    pub created: String,
}

/// A user blocked by an organization.
#[derive(Debug, Clone, Default)]
pub struct StoredOrgBlock {
    pub org: String,
    pub user: String,
    pub created: String,
}

/// A git hook configured on a repository.
#[derive(Debug, Clone, Default)]
pub struct StoredGitHook {
    pub repo_id: i64,
    /// `pre-receive`, `update` or `post-receive`.
    pub name: String,
    pub content: String,
    pub active: bool,
}

/// An additional email address belonging to a user.
#[derive(Debug, Clone, Default)]
pub struct StoredEmail {
    pub id: i64,
    pub user: String,
    pub email: String,
    pub primary: bool,
    pub verified: bool,
}

/// A reaction left on an issue or an issue comment.
#[derive(Debug, Clone, Default)]
pub struct StoredReaction {
    pub id: i64,
    pub repo_id: i64,
    /// The issue index, or the comment id when `comment` is set.
    pub target: i64,
    /// True when `target` names a comment rather than an issue index.
    pub comment: bool,
    pub user: String,
    /// Emoji name, for example `+1` or `heart`.
    pub content: String,
    pub created: String,
}

impl StoredReaction {
    /// Render as the `Reaction` wire type.
    pub fn to_dto(&self) -> dto::Reaction {
        dto::Reaction {
            content: Some(self.content.clone()),
            created_at: Some(self.created.clone()),
            user: Some(dto::User {
                login: Some(self.user.clone()),
                login_name: Some(self.user.clone()),
                ..Default::default()
            }),
        }
    }
}

/// A webhook belonging to a user, organization or the instance.
#[derive(Debug, Clone, Default)]
pub struct StoredHook {
    pub id: i64,
    /// The owning scope: a login, an org name, or empty for the instance.
    pub owner: String,
    /// `user`, `org` or `admin`.
    pub scope: String,
    pub kind: String,
    pub name: String,
    pub active: bool,
    pub events: Vec<String>,
    pub config: HashMap<String, String>,
    pub branch_filter: String,
    pub authorization_header: String,
    pub created: String,
    pub updated: String,
}

impl StoredHook {
    /// Render as the `Hook` wire type.
    pub fn to_dto(&self) -> dto::Hook {
        dto::Hook {
            id: Some(self.id),
            type_: Some(self.kind.clone()),
            name: Some(self.name.clone()),
            active: Some(self.active),
            events: Some(self.events.clone()),
            config: Some(self.config.clone()),
            branch_filter: Some(self.branch_filter.clone()),
            authorization_header: Some(self.authorization_header.clone()),
            created_at: Some(self.created.clone()),
            updated_at: Some(self.updated.clone()),
        }
    }
}

/// A project board (issue tracker view).
#[derive(Debug, Clone, Default)]
pub struct StoredProject {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub owner_id: i64,
    /// `individual`, `repository` or `organization`.
    pub kind: String,
    pub repo_id: Option<i64>,
    pub card_type: String,
    pub template_type: String,
    pub closed: bool,
    pub creator: String,
    pub created: String,
    pub updated: String,
}

impl StoredProject {
    /// Render as the `Project` wire type.
    pub fn to_dto(&self) -> dto::Project {
        dto::Project {
            id: Some(self.id),
            title: Some(self.title.clone()),
            description: Some(self.description.clone()),
            owner_id: Some(self.owner_id),
            repo_id: self.repo_id,
            type_: Some(self.kind.clone()),
            card_type: Some(self.card_type.clone()),
            template_type: Some(self.template_type.clone()),
            state: Some(if self.closed {
                dto::StateType::Closed
            } else {
                dto::StateType::Open
            }),
            is_closed: Some(self.closed),
            created_at: Some(self.created.clone()),
            updated_at: Some(self.updated.clone()),
            ..Default::default()
        }
    }
}

/// A column on a project board.
#[derive(Debug, Clone, Default)]
pub struct StoredProjectColumn {
    pub id: i64,
    pub project_id: i64,
    pub title: String,
    pub sorting: i64,
    pub is_default: bool,
    /// Display indices of the issues placed in this column.
    pub issues: Vec<i64>,
    pub created: String,
    pub updated: String,
}

impl StoredProjectColumn {
    /// Render as the `ProjectColumn` wire type.
    pub fn to_dto(&self) -> dto::ProjectColumn {
        dto::ProjectColumn {
            id: Some(self.id),
            project_id: Some(self.project_id),
            title: Some(self.title.clone()),
            sorting: Some(self.sorting),
            default: Some(self.is_default),
            created_at: Some(self.created.clone()),
            updated_at: Some(self.updated.clone()),
            ..Default::default()
        }
    }
}

/// An OAuth2 application registered by a user.
#[derive(Debug, Clone, Default)]
pub struct StoredOauthApp {
    pub id: i64,
    pub user: String,
    pub name: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uris: Vec<String>,
    pub confidential: bool,
    pub skip_secondary_authorization: bool,
    pub created: String,
}

impl StoredOauthApp {
    /// Render as the `OAuth2Application` wire type.
    pub fn to_dto(&self) -> dto::OAuth2Application {
        dto::OAuth2Application {
            id: Some(self.id),
            name: Some(self.name.clone()),
            client_id: Some(self.client_id.clone()),
            client_secret: Some(self.client_secret.clone()),
            redirect_uris: Some(self.redirect_uris.clone()),
            confidential_client: Some(self.confidential),
            skip_secondary_authorization: Some(self.skip_secondary_authorization),
            created: Some(self.created.clone()),
        }
    }
}

/// A package registered against an owner.
#[derive(Debug, Clone, Default)]
pub struct StoredPackage {
    pub id: i64,
    /// Login or organization that owns the package.
    pub owner: String,
    pub kind: String,
    pub name: String,
    pub version: String,
    pub created: String,
    /// `owner/name` of the repository the package is linked to, when any.
    pub repository: Option<String>,
    /// Files belonging to this version.
    pub files: Vec<StoredPackageFile>,
}

impl StoredPackage {
    /// Render as the `Package` wire type.
    pub fn to_dto(&self, owner: Option<dto::User>) -> dto::Package {
        dto::Package {
            id: Some(self.id),
            name: Some(self.name.clone()),
            version: Some(self.version.clone()),
            type_: Some(self.kind.clone()),
            owner,
            created_at: Some(self.created.clone()),
            html_url: Some(format!(
                "/{}/packages/{}/{}",
                self.owner, self.kind, self.name
            )),
            repository: None,
            ..Default::default()
        }
    }

    /// The version the registry treats as "latest".
    ///
    /// Gitea orders versions per registry protocol; the in-memory port uses the
    /// most recently created row, which matches the demo seed's intent.
    pub fn is_latest(&self, all: &[StoredPackage]) -> bool {
        all.iter()
            .filter(|p| p.owner == self.owner && p.kind == self.kind && p.name == self.name)
            .map(|p| p.created.as_str())
            .max()
            == Some(self.created.as_str())
    }
}

/// A file attached to a package version.
#[derive(Debug, Clone, Default)]
pub struct StoredPackageFile {
    pub id: i64,
    pub name: String,
    pub size: i64,
}

impl StoredPackageFile {
    /// Render as the `PackageFile` wire type.
    pub fn to_dto(&self) -> dto::PackageFile {
        dto::PackageFile {
            id: Some(self.id),
            name: Some(self.name.clone()),
            size: Some(self.size),
            ..Default::default()
        }
    }
}

/// A revision of a wiki page.
#[derive(Debug, Clone, Default)]
pub struct StoredWikiRevision {
    pub sha: String,
    pub message: String,
    pub author: String,
    pub created: String,
    /// Page content at this revision.
    pub content: String,
}

impl StoredWikiRevision {
    /// Render as the `WikiCommit` wire type.
    pub fn to_dto(&self) -> dto::WikiCommit {
        dto::WikiCommit {
            sha: Some(self.sha.clone()),
            message: Some(self.message.clone()),
            author: Some(dto::CommitUser {
                name: Some(self.author.clone()),
                date: Some(self.created.clone()),
                ..Default::default()
            }),
            commiter: Some(dto::CommitUser {
                name: Some(self.author.clone()),
                date: Some(self.created.clone()),
                ..Default::default()
            }),
        }
    }
}

/// A repository wiki page, with its revision history.
#[derive(Debug, Clone, Default)]
pub struct StoredWikiPage {
    pub repo_id: i64,
    pub title: String,
    /// Newest revision last.
    pub revisions: Vec<StoredWikiRevision>,
    pub updated: String,
}

impl StoredWikiPage {
    /// Content of the newest revision.
    pub fn content(&self) -> &str {
        self.revisions
            .last()
            .map(|r| r.content.as_str())
            .unwrap_or("")
    }

    /// Render as the `WikiPage` wire type.
    ///
    /// `repo` supplies the `html_url` prefix, matching how Gitea builds it.
    pub fn to_dto(&self, repo: &StoredRepo) -> dto::WikiPage {
        dto::WikiPage {
            title: Some(self.title.clone()),
            content_base64: Some(base64_encode(self.content())),
            commit_count: Some(self.revisions.len() as i64),
            last_commit: self.revisions.last().map(|r| r.to_dto()),
            html_url: Some(format!(
                "/{}/wiki/{}",
                repo.full_name(),
                url_encode(&self.title)
            )),
            sub_url: Some(format!(
                "/{}/wiki/{}",
                repo.full_name(),
                url_encode(&self.title)
            )),
            sidebar: None,
            footer: None,
        }
    }
}

/// Percent-encode a wiki page title for use in a URL path segment.
fn url_encode(input: &str) -> String {
    let mut out = String::new();
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

/// Standard base64, no line breaks.
pub fn base64_encode(input: &str) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(ALPHABET[(triple >> 18) as usize & 0x3f] as char);
        out.push(ALPHABET[(triple >> 12) as usize & 0x3f] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[(triple >> 6) as usize & 0x3f] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[triple as usize & 0x3f] as char
        } else {
            '='
        });
    }
    out
}

/// Decode standard base64, ignoring whitespace.
///
/// Returns the input unchanged when it is not valid base64, so a client that
/// posts plain text is still readable.
pub fn base64_decode(input: &str) -> String {
    let filtered: Vec<u8> = input.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    let mut out: Vec<u8> = Vec::with_capacity(filtered.len() / 4 * 3);
    let value = |c: u8| -> Option<u32> {
        match c {
            b'A'..=b'Z' => Some((c - b'A') as u32),
            b'a'..=b'z' => Some((c - b'a' + 26) as u32),
            b'0'..=b'9' => Some((c - b'0' + 52) as u32),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    };
    for chunk in filtered.chunks(4) {
        let mut triple = 0u32;
        let mut count = 0;
        for &c in chunk {
            if c == b'=' {
                break;
            }
            let Some(v) = value(c) else {
                return input.to_string();
            };
            triple = (triple << 6) | v;
            count += 1;
        }
        if count < 2 {
            return input.to_string();
        }
        triple <<= 6 * (4 - count);
        out.push((triple >> 16) as u8);
        if count > 2 {
            out.push((triple >> 8) as u8);
        }
        if count > 3 {
            out.push(triple as u8);
        }
    }
    String::from_utf8(out).unwrap_or_else(|_| input.to_string())
}

/// A user badge (achievement).
#[derive(Debug, Clone, Default)]
pub struct StoredBadge {
    pub id: i64,
    pub user: String,
    pub slug: String,
    pub description: String,
    pub image_url: String,
}

impl StoredBadge {
    /// Render as the `Badge` wire type.
    pub fn to_dto(&self) -> dto::Badge {
        dto::Badge {
            id: Some(self.id),
            slug: Some(self.slug.clone()),
            description: Some(self.description.clone()),
            image_url: Some(self.image_url.clone()),
        }
    }
}

/// A self-hosted Actions runner.
#[derive(Debug, Clone, Default)]
pub struct StoredRunner {
    pub id: i64,
    /// Owning login, org name, or empty for an instance runner.
    pub owner: String,
    pub name: String,
    pub labels: Vec<String>,
    pub busy: bool,
    pub disabled: bool,
    pub ephemeral: bool,
    pub status: String,
}

impl StoredRunner {
    /// Render as the `ActionRunner` wire type.
    pub fn to_dto(&self) -> dto::ActionRunner {
        dto::ActionRunner {
            id: Some(self.id),
            name: Some(self.name.clone()),
            busy: Some(self.busy),
            disabled: Some(self.disabled),
            ephemeral: Some(self.ephemeral),
            status: Some(self.status.clone()),
            labels: Some(
                self.labels
                    .iter()
                    .map(|name| dto::ActionRunnerLabel {
                        name: Some(name.clone()),
                        ..Default::default()
                    })
                    .collect(),
            ),
        }
    }
}

/// A repository scheduled for adoption from the unadopted list.
#[derive(Debug, Clone, Default)]
pub struct StoredUnadopted {
    pub owner: String,
    pub repo: String,
}

/// A workflow definition found in a repository's workflow directory.
#[derive(Debug, Clone, Default)]
pub struct StoredWorkflow {
    pub id: i64,
    pub repo_id: i64,
    pub name: String,
    pub path: String,
    /// `active` or `disabled_manually`.
    pub state: String,
}

impl StoredWorkflow {
    /// The workflow file name, which Gitea uses as the workflow's `id`.
    pub fn file_name(&self) -> String {
        self.path
            .rsplit('/')
            .next()
            .unwrap_or(&self.path)
            .to_string()
    }

    /// Render as the `ActionWorkflow` wire type.
    pub fn to_dto(&self, repo: &StoredRepo) -> dto::ActionWorkflow {
        dto::ActionWorkflow {
            id: Some(self.file_name()),
            name: Some(self.name.clone()),
            path: Some(self.path.clone()),
            state: Some(self.state.clone()),
            html_url: Some(format!(
                "/{}/actions?workflow={}",
                repo.full_name(),
                self.path
            )),
            url: Some(format!(
                "/api/v1/repos/{}/actions/workflows/{}",
                repo.full_name(),
                self.path
            )),
            badge_url: Some(format!(
                "/{}/actions/workflows/{}/badge.svg",
                repo.full_name(),
                self.path
            )),
            ..Default::default()
        }
    }
}

/// A recorded workflow run.
#[derive(Debug, Clone, Default)]
pub struct StoredWorkflowRun {
    pub id: i64,
    pub repo_id: i64,
    /// Path of the workflow that produced the run.
    pub workflow_path: String,
    pub run_number: i64,
    pub run_attempt: i64,
    pub display_title: String,
    pub event: String,
    pub head_branch: String,
    pub head_sha: String,
    pub status: String,
    pub conclusion: String,
    pub actor: String,
    pub created: String,
    pub updated: String,
}

impl StoredWorkflowRun {
    /// Render as the `ActionWorkflowRun` wire type.
    pub fn to_dto(&self, repo: &StoredRepo, actor: Option<dto::User>) -> dto::ActionWorkflowRun {
        let base = format!(
            "/api/v1/repos/{}/actions/runs/{}",
            repo.full_name(),
            self.id
        );
        dto::ActionWorkflowRun {
            id: Some(self.id),
            run_number: Some(self.run_number),
            run_attempt: Some(self.run_attempt),
            display_title: Some(self.display_title.clone()),
            event: Some(self.event.clone()),
            head_branch: Some(self.head_branch.clone()),
            head_sha: Some(self.head_sha.clone()),
            status: Some(self.status.clone()),
            conclusion: Some(self.conclusion.clone()),
            actor,
            path: Some(self.workflow_path.clone()),
            created_at: Some(self.created.clone()),
            updated_at: Some(self.updated.clone()),
            url: Some(base.clone()),
            html_url: Some(format!(
                "/{}/actions/runs/{}",
                repo.full_name(),
                self.run_number
            )),
            jobs_url: Some(format!("{base}/jobs")),
            logs_url: Some(format!("{base}/logs")),
            artifacts_url: Some(format!("{base}/artifacts")),
            cancel_url: Some(format!("{base}/cancel")),
            rerun_url: Some(format!("{base}/rerun")),
            ..Default::default()
        }
    }
}

/// A job belonging to a workflow run.
#[derive(Debug, Clone, Default)]
pub struct StoredWorkflowJob {
    pub id: i64,
    pub run_id: i64,
    pub repo_id: i64,
    pub name: String,
    pub status: String,
    pub conclusion: String,
    pub runner_id: i64,
    pub runner_name: String,
    pub started: String,
    pub completed: String,
    /// `(name, status, conclusion)` for each step.
    pub steps: Vec<(String, String, String)>,
}

impl StoredWorkflowJob {
    /// Render as the `ActionWorkflowJob` wire type.
    pub fn to_dto(&self, repo: &StoredRepo) -> dto::ActionWorkflowJob {
        dto::ActionWorkflowJob {
            id: Some(self.id),
            run_id: Some(self.run_id),
            name: Some(self.name.clone()),
            status: Some(self.status.clone()),
            conclusion: Some(self.conclusion.clone()),
            runner_id: Some(self.runner_id),
            runner_name: Some(self.runner_name.clone()),
            started_at: Some(self.started.clone()),
            completed_at: Some(self.completed.clone()),
            url: Some(format!(
                "/api/v1/repos/{}/actions/jobs/{}",
                repo.full_name(),
                self.id
            )),
            html_url: Some(format!(
                "/{}/actions/runs/{}",
                repo.full_name(),
                self.run_id
            )),
            steps: Some(
                self.steps
                    .iter()
                    .enumerate()
                    .map(
                        |(index, (name, status, conclusion))| dto::ActionWorkflowStep {
                            number: Some(index as i64 + 1),
                            name: Some(name.clone()),
                            status: Some(status.clone()),
                            conclusion: Some(conclusion.clone()),
                            ..Default::default()
                        },
                    )
                    .collect(),
            ),
            ..Default::default()
        }
    }
}

/// An artifact uploaded by a workflow run.
#[derive(Debug, Clone, Default)]
pub struct StoredArtifact {
    pub id: i64,
    pub run_id: i64,
    pub repo_id: i64,
    pub name: String,
    pub size: i64,
    pub expired: bool,
    pub created: String,
    pub updated: String,
}

impl StoredArtifact {
    /// Render as the `ActionArtifact` wire type.
    pub fn to_dto(&self, repo: &StoredRepo) -> dto::ActionArtifact {
        let base = format!(
            "/api/v1/repos/{}/actions/artifacts/{}",
            repo.full_name(),
            self.id
        );
        dto::ActionArtifact {
            id: Some(self.id),
            name: Some(self.name.clone()),
            size_in_bytes: Some(self.size),
            expired: Some(self.expired),
            created_at: Some(self.created.clone()),
            updated_at: Some(self.updated.clone()),
            expires_at: Some(self.updated.clone()),
            url: Some(base.clone()),
            archive_download_url: Some(format!("{base}/zip")),
            workflow_run: Some(dto::ActionWorkflowRun {
                id: Some(self.run_id),
                ..Default::default()
            }),
        }
    }
}

/// A repository, organization or user secret.
#[derive(Debug, Clone, Default)]
pub struct StoredSecret {
    /// Scope key: `repo:{id}`, `org:{name}` or `user:{login}`.
    pub scope: String,
    pub name: String,
    pub description: String,
    pub created: String,
}

impl StoredSecret {
    /// Render as the `Secret` wire type.
    pub fn to_dto(&self) -> dto::Secret {
        dto::Secret {
            name: Some(self.name.clone()),
            description: Some(self.description.clone()),
            created_at: Some(self.created.clone()),
        }
    }
}

/// A repository, organization or user Actions variable.
#[derive(Debug, Clone, Default)]
pub struct StoredVariable {
    /// Scope key: `repo:{id}`, `org:{name}` or `user:{login}`.
    pub scope: String,
    pub name: String,
    pub value: String,
    pub description: String,
}

impl StoredVariable {
    /// Render as the `ActionVariable` wire type.
    pub fn to_dto(&self) -> dto::ActionVariable {
        dto::ActionVariable {
            name: Some(self.name.clone()),
            data: Some(self.value.clone()),
            description: Some(self.description.clone()),
            ..Default::default()
        }
    }
}

/// Every in-memory store, guarded independently.
#[derive(Debug, Default)]
pub struct Stores {
    pub users: RwLock<Vec<StoredUser>>,
    pub repos: RwLock<Vec<StoredRepo>>,
    pub issues: RwLock<Vec<StoredIssue>>,
    pub reviews: RwLock<Vec<StoredReview>>,
    pub branch_protections: RwLock<Vec<StoredBranchProtection>>,
    pub tag_protections: RwLock<Vec<StoredTagProtection>>,
    pub push_mirrors: RwLock<Vec<StoredPushMirror>>,
    pub commit_statuses: RwLock<Vec<StoredCommitStatus>>,
    pub attachments: RwLock<Vec<StoredAttachment>>,
    pub labels: RwLock<Vec<StoredLabel>>,
    pub milestones: RwLock<Vec<StoredMilestone>>,
    pub releases: RwLock<Vec<StoredRelease>>,
    pub orgs: RwLock<Vec<StoredOrg>>,
    pub teams: RwLock<Vec<StoredTeam>>,
    pub tokens: RwLock<Vec<StoredToken>>,
    pub keys: RwLock<Vec<StoredKey>>,
    pub notifications: RwLock<Vec<StoredNotification>>,
    pub branches: RwLock<Vec<StoredBranch>>,
    pub commits: RwLock<Vec<StoredCommit>>,
    pub files: RwLock<Vec<StoredFile>>,
    pub follows: RwLock<Vec<StoredFollow>>,
    pub blocks: RwLock<Vec<StoredBlock>>,
    /// Users blocked by an organization, keyed by the organization name.
    pub org_blocks: RwLock<Vec<StoredOrgBlock>>,
    pub reactions: RwLock<Vec<StoredReaction>>,
    /// Secondary email addresses, keyed by user login.
    pub emails: RwLock<Vec<StoredEmail>>,
    /// Git hooks configured per repository.
    pub git_hooks: RwLock<Vec<StoredGitHook>>,
    /// Line comments attached to pull request reviews.
    pub review_comments: RwLock<Vec<StoredReviewComment>>,
    /// Users granted access to a repository beyond its owner.
    pub collaborators: RwLock<Vec<StoredCollaborator>>,
    pub hooks: RwLock<Vec<StoredHook>>,
    pub projects: RwLock<Vec<StoredProject>>,
    pub project_columns: RwLock<Vec<StoredProjectColumn>>,
    pub oauth_apps: RwLock<Vec<StoredOauthApp>>,
    pub packages: RwLock<Vec<StoredPackage>>,
    pub badges: RwLock<Vec<StoredBadge>>,
    pub runners: RwLock<Vec<StoredRunner>>,
    pub wiki_pages: RwLock<Vec<StoredWikiPage>>,
    pub workflows: RwLock<Vec<StoredWorkflow>>,
    pub workflow_runs: RwLock<Vec<StoredWorkflowRun>>,
    pub workflow_jobs: RwLock<Vec<StoredWorkflowJob>>,
    pub artifacts: RwLock<Vec<StoredArtifact>>,
    pub secrets: RwLock<Vec<StoredSecret>>,
    pub variables: RwLock<Vec<StoredVariable>>,
    pub unadopted: RwLock<Vec<StoredUnadopted>>,
    pub ids: IdGen,
    /// Counters for endpoints whose payload is echoed rather than stored.
    pub counters: RwLock<HashMap<String, i64>>,
}

/// Shared application state handed to every handler.
pub type AppState = Arc<Stores>;

impl Stores {
    /// Read guard over the users.
    pub fn users(&self) -> RwLockReadGuard<'_, Vec<StoredUser>> {
        read(&self.users)
    }

    /// Write guard over the users.
    pub fn users_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredUser>> {
        write(&self.users)
    }

    /// Read guard over the repositories.
    pub fn repos(&self) -> RwLockReadGuard<'_, Vec<StoredRepo>> {
        read(&self.repos)
    }

    /// Write guard over the repositories.
    pub fn repos_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredRepo>> {
        write(&self.repos)
    }

    /// Read guard over the issues.
    pub fn issues(&self) -> RwLockReadGuard<'_, Vec<StoredIssue>> {
        read(&self.issues)
    }

    /// Write guard over the issues.
    pub fn issues_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredIssue>> {
        write(&self.issues)
    }

    /// Read guard over the labels.
    pub fn labels(&self) -> RwLockReadGuard<'_, Vec<StoredLabel>> {
        read(&self.labels)
    }

    /// Read guard over the milestones.
    pub fn milestones(&self) -> RwLockReadGuard<'_, Vec<StoredMilestone>> {
        read(&self.milestones)
    }

    /// Read guard over the releases.
    pub fn releases(&self) -> RwLockReadGuard<'_, Vec<StoredRelease>> {
        read(&self.releases)
    }

    /// Read guard over the organizations.
    pub fn orgs(&self) -> RwLockReadGuard<'_, Vec<StoredOrg>> {
        read(&self.orgs)
    }

    /// Write guard over the organizations.
    pub fn orgs_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredOrg>> {
        write(&self.orgs)
    }

    /// Read guard over the teams.
    pub fn teams(&self) -> RwLockReadGuard<'_, Vec<StoredTeam>> {
        read(&self.teams)
    }

    /// Read guard over the access tokens.
    pub fn tokens(&self) -> RwLockReadGuard<'_, Vec<StoredToken>> {
        read(&self.tokens)
    }

    /// Write guard over the access tokens.
    pub fn tokens_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredToken>> {
        write(&self.tokens)
    }

    /// Read guard over the public keys.
    pub fn keys(&self) -> RwLockReadGuard<'_, Vec<StoredKey>> {
        read(&self.keys)
    }

    /// Write guard over the public keys.
    pub fn keys_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredKey>> {
        write(&self.keys)
    }

    /// Read guard over the notifications.
    pub fn notifications(&self) -> RwLockReadGuard<'_, Vec<StoredNotification>> {
        read(&self.notifications)
    }

    /// Write guard over the notifications.
    pub fn notifications_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredNotification>> {
        write(&self.notifications)
    }

    /// Read guard over the branch heads.
    pub fn branches(&self) -> RwLockReadGuard<'_, Vec<StoredBranch>> {
        read(&self.branches)
    }

    /// Write guard over the branch heads.
    pub fn branches_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredBranch>> {
        write(&self.branches)
    }

    /// Read guard over the commits.
    pub fn commits(&self) -> RwLockReadGuard<'_, Vec<StoredCommit>> {
        read(&self.commits)
    }

    /// Write guard over the commits.
    pub fn commits_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredCommit>> {
        write(&self.commits)
    }

    /// Read guard over the tree entries.
    pub fn files(&self) -> RwLockReadGuard<'_, Vec<StoredFile>> {
        read(&self.files)
    }

    /// Write guard over the tree entries.
    pub fn files_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredFile>> {
        write(&self.files)
    }

    /// Read guard over the follow edges.
    pub fn follows(&self) -> RwLockReadGuard<'_, Vec<StoredFollow>> {
        read(&self.follows)
    }

    /// Write guard over the follow edges.
    pub fn follows_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredFollow>> {
        write(&self.follows)
    }

    /// Read guard over the block edges.
    pub fn blocks(&self) -> RwLockReadGuard<'_, Vec<StoredBlock>> {
        read(&self.blocks)
    }

    /// Write guard over the block edges.
    pub fn blocks_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredBlock>> {
        write(&self.blocks)
    }

    /// Read guard over the webhooks.
    pub fn hooks(&self) -> RwLockReadGuard<'_, Vec<StoredHook>> {
        read(&self.hooks)
    }

    /// Write guard over the webhooks.
    pub fn hooks_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredHook>> {
        write(&self.hooks)
    }

    /// Read guard over the project boards.
    pub fn projects(&self) -> RwLockReadGuard<'_, Vec<StoredProject>> {
        read(&self.projects)
    }

    /// Write guard over the project boards.
    pub fn projects_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredProject>> {
        write(&self.projects)
    }

    /// Read guard over the project columns.
    pub fn project_columns(&self) -> RwLockReadGuard<'_, Vec<StoredProjectColumn>> {
        read(&self.project_columns)
    }

    /// Write guard over the project columns.
    pub fn project_columns_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredProjectColumn>> {
        write(&self.project_columns)
    }

    /// Read guard over the OAuth2 applications.
    pub fn oauth_apps(&self) -> RwLockReadGuard<'_, Vec<StoredOauthApp>> {
        read(&self.oauth_apps)
    }

    /// Write guard over the OAuth2 applications.
    pub fn oauth_apps_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredOauthApp>> {
        write(&self.oauth_apps)
    }

    /// Read guard over the packages.
    pub fn packages(&self) -> RwLockReadGuard<'_, Vec<StoredPackage>> {
        read(&self.packages)
    }

    /// Write guard over the packages.
    pub fn packages_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredPackage>> {
        write(&self.packages)
    }

    /// Read guard over the user badges.
    pub fn badges(&self) -> RwLockReadGuard<'_, Vec<StoredBadge>> {
        read(&self.badges)
    }

    /// Write guard over the user badges.
    pub fn badges_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredBadge>> {
        write(&self.badges)
    }

    /// Read guard over the Actions runners.
    pub fn runners(&self) -> RwLockReadGuard<'_, Vec<StoredRunner>> {
        read(&self.runners)
    }

    /// Write guard over the Actions runners.
    pub fn runners_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredRunner>> {
        write(&self.runners)
    }

    /// Read guard over the unadopted repositories.
    pub fn unadopted(&self) -> RwLockReadGuard<'_, Vec<StoredUnadopted>> {
        read(&self.unadopted)
    }

    /// Write guard over the unadopted repositories.
    pub fn unadopted_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredUnadopted>> {
        write(&self.unadopted)
    }

    /// Whether `user` follows `target`.
    pub fn follows_user(&self, user: &str, target: &str) -> bool {
        self.follows()
            .iter()
            .any(|f| f.user.eq_ignore_ascii_case(user) && f.target.eq_ignore_ascii_case(target))
    }

    /// Whether `user` blocks `target`.
    pub fn blocks_user(&self, user: &str, target: &str) -> bool {
        self.blocks()
            .iter()
            .any(|b| b.user.eq_ignore_ascii_case(user) && b.target.eq_ignore_ascii_case(target))
    }

    /// Read guard over the organization blocks.
    pub fn org_blocks(&self) -> RwLockReadGuard<'_, Vec<StoredOrgBlock>> {
        read(&self.org_blocks)
    }

    /// Write guard over the organization blocks.
    pub fn org_blocks_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredOrgBlock>> {
        write(&self.org_blocks)
    }

    /// Read guard over the issue and comment reactions.
    pub fn reactions(&self) -> RwLockReadGuard<'_, Vec<StoredReaction>> {
        read(&self.reactions)
    }

    /// Write guard over the issue and comment reactions.
    pub fn reactions_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredReaction>> {
        write(&self.reactions)
    }

    /// Read guard over the secondary email addresses.
    pub fn emails(&self) -> RwLockReadGuard<'_, Vec<StoredEmail>> {
        read(&self.emails)
    }

    /// Write guard over the secondary email addresses.
    pub fn emails_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredEmail>> {
        write(&self.emails)
    }

    /// Read guard over the repository git hooks.
    pub fn git_hooks(&self) -> RwLockReadGuard<'_, Vec<StoredGitHook>> {
        read(&self.git_hooks)
    }

    /// Write guard over the repository git hooks.
    pub fn git_hooks_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredGitHook>> {
        write(&self.git_hooks)
    }

    /// Read guard over the pull request review comments.
    pub fn review_comments(&self) -> RwLockReadGuard<'_, Vec<StoredReviewComment>> {
        read(&self.review_comments)
    }

    /// Write guard over the pull request review comments.
    pub fn review_comments_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredReviewComment>> {
        write(&self.review_comments)
    }

    /// Read guard over the repository collaborators.
    pub fn collaborators(&self) -> RwLockReadGuard<'_, Vec<StoredCollaborator>> {
        read(&self.collaborators)
    }

    /// Write guard over the repository collaborators.
    pub fn collaborators_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredCollaborator>> {
        write(&self.collaborators)
    }

    /// Read guard over the pull request reviews.
    pub fn reviews(&self) -> RwLockReadGuard<'_, Vec<StoredReview>> {
        read(&self.reviews)
    }

    /// Write guard over the pull request reviews.
    pub fn reviews_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredReview>> {
        write(&self.reviews)
    }

    /// Read guard over the branch protections.
    pub fn branch_protections(&self) -> RwLockReadGuard<'_, Vec<StoredBranchProtection>> {
        read(&self.branch_protections)
    }

    /// Write guard over the branch protections.
    pub fn branch_protections_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredBranchProtection>> {
        write(&self.branch_protections)
    }

    /// Read guard over the tag protections.
    pub fn tag_protections(&self) -> RwLockReadGuard<'_, Vec<StoredTagProtection>> {
        read(&self.tag_protections)
    }

    /// Write guard over the tag protections.
    pub fn tag_protections_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredTagProtection>> {
        write(&self.tag_protections)
    }

    /// Read guard over the push mirrors.
    pub fn push_mirrors(&self) -> RwLockReadGuard<'_, Vec<StoredPushMirror>> {
        read(&self.push_mirrors)
    }

    /// Write guard over the push mirrors.
    pub fn push_mirrors_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredPushMirror>> {
        write(&self.push_mirrors)
    }

    /// Read guard over the commit statuses.
    pub fn commit_statuses(&self) -> RwLockReadGuard<'_, Vec<StoredCommitStatus>> {
        read(&self.commit_statuses)
    }

    /// Write guard over the commit statuses.
    pub fn commit_statuses_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredCommitStatus>> {
        write(&self.commit_statuses)
    }

    /// Read guard over the attachments.
    pub fn attachments(&self) -> RwLockReadGuard<'_, Vec<StoredAttachment>> {
        read(&self.attachments)
    }

    /// Write guard over the attachments.
    pub fn attachments_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredAttachment>> {
        write(&self.attachments)
    }

    /// Write guard over the labels.
    pub fn labels_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredLabel>> {
        write(&self.labels)
    }

    /// Write guard over the milestones.
    pub fn milestones_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredMilestone>> {
        write(&self.milestones)
    }

    /// Write guard over the releases.
    pub fn releases_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredRelease>> {
        write(&self.releases)
    }

    /// Write guard over the teams.
    pub fn teams_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredTeam>> {
        write(&self.teams)
    }

    /// Read guard over the wiki pages.
    pub fn wiki_pages(&self) -> RwLockReadGuard<'_, Vec<StoredWikiPage>> {
        read(&self.wiki_pages)
    }

    /// Write guard over the wiki pages.
    pub fn wiki_pages_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredWikiPage>> {
        write(&self.wiki_pages)
    }

    /// The wiki pages of a repository, ordered by title.
    pub fn repo_wiki_pages(&self, repo_id: i64) -> Vec<StoredWikiPage> {
        let mut pages: Vec<StoredWikiPage> = self
            .wiki_pages()
            .iter()
            .filter(|p| p.repo_id == repo_id)
            .cloned()
            .collect();
        pages.sort_by(|a, b| a.title.cmp(&b.title));
        pages
    }

    /// A single wiki page of a repository.
    pub fn repo_wiki_page(&self, repo_id: i64, title: &str) -> Option<StoredWikiPage> {
        self.wiki_pages()
            .iter()
            .find(|p| p.repo_id == repo_id && p.title.eq_ignore_ascii_case(title))
            .cloned()
    }

    /// All package versions owned by `owner`, newest first.
    pub fn owner_packages(&self, owner: &str) -> Vec<StoredPackage> {
        let mut packages: Vec<StoredPackage> = self
            .packages()
            .iter()
            .filter(|p| p.owner == owner)
            .cloned()
            .collect();
        packages.sort_by(|a, b| b.created.cmp(&a.created));
        packages
    }

    /// The runners visible to a scope: instance (`""`), a login, or an org.
    pub fn scoped_runners(&self, owner: &str) -> Vec<StoredRunner> {
        self.runners()
            .iter()
            .filter(|r| r.owner == owner)
            .cloned()
            .collect()
    }

    /// Read guard over the workflow definitions.
    pub fn workflows(&self) -> RwLockReadGuard<'_, Vec<StoredWorkflow>> {
        read(&self.workflows)
    }

    /// Write guard over the workflow definitions.
    pub fn workflows_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredWorkflow>> {
        write(&self.workflows)
    }

    /// Read guard over the workflow runs.
    pub fn workflow_runs(&self) -> RwLockReadGuard<'_, Vec<StoredWorkflowRun>> {
        read(&self.workflow_runs)
    }

    /// Write guard over the workflow runs.
    pub fn workflow_runs_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredWorkflowRun>> {
        write(&self.workflow_runs)
    }

    /// Read guard over the workflow jobs.
    pub fn workflow_jobs(&self) -> RwLockReadGuard<'_, Vec<StoredWorkflowJob>> {
        read(&self.workflow_jobs)
    }

    /// Write guard over the workflow jobs.
    pub fn workflow_jobs_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredWorkflowJob>> {
        write(&self.workflow_jobs)
    }

    /// Read guard over the artifacts.
    pub fn artifacts(&self) -> RwLockReadGuard<'_, Vec<StoredArtifact>> {
        read(&self.artifacts)
    }

    /// Write guard over the artifacts.
    pub fn artifacts_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredArtifact>> {
        write(&self.artifacts)
    }

    /// Read guard over the Actions secrets.
    pub fn secrets(&self) -> RwLockReadGuard<'_, Vec<StoredSecret>> {
        read(&self.secrets)
    }

    /// Write guard over the Actions secrets.
    pub fn secrets_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredSecret>> {
        write(&self.secrets)
    }

    /// Read guard over the Actions variables.
    pub fn variables(&self) -> RwLockReadGuard<'_, Vec<StoredVariable>> {
        read(&self.variables)
    }

    /// Write guard over the Actions variables.
    pub fn variables_mut(&self) -> RwLockWriteGuard<'_, Vec<StoredVariable>> {
        write(&self.variables)
    }

    /// Workflows defined in a repository.
    pub fn repo_workflows(&self, repo_id: i64) -> Vec<StoredWorkflow> {
        self.workflows()
            .iter()
            .filter(|w| w.repo_id == repo_id)
            .cloned()
            .collect()
    }

    /// Runs of a repository, newest first, optionally limited to one workflow.
    pub fn repo_workflow_runs(
        &self,
        repo_id: i64,
        workflow_path: Option<&str>,
    ) -> Vec<StoredWorkflowRun> {
        let mut runs: Vec<StoredWorkflowRun> = self
            .workflow_runs()
            .iter()
            .filter(|r| r.repo_id == repo_id)
            .filter(|r| workflow_path.is_none_or(|path| r.workflow_path == path))
            .cloned()
            .collect();
        runs.sort_by_key(|r| std::cmp::Reverse(r.id));
        runs
    }

    /// The secrets visible in a scope key (`repo:{id}`, `org:{name}`, `user:{login}`).
    pub fn scoped_secrets(&self, scope: &str) -> Vec<StoredSecret> {
        self.secrets()
            .iter()
            .filter(|s| s.scope == scope)
            .cloned()
            .collect()
    }

    /// The variables visible in a scope key.
    pub fn scoped_variables(&self, scope: &str) -> Vec<StoredVariable> {
        self.variables()
            .iter()
            .filter(|v| v.scope == scope)
            .cloned()
            .collect()
    }

    /// Allocate the next id.
    pub fn next_id(&self) -> i64 {
        self.ids.next()
    }

    /// Read guard over the free-form counters.
    pub fn counters(&self) -> RwLockReadGuard<'_, HashMap<String, i64>> {
        read(&self.counters)
    }

    /// Write guard over the free-form counters.
    pub fn counters_mut(&self) -> RwLockWriteGuard<'_, HashMap<String, i64>> {
        write(&self.counters)
    }

    /// Find a user by login, case-insensitively.
    pub fn user(&self, login: &str) -> Option<StoredUser> {
        self.users()
            .iter()
            .find(|u| u.login.eq_ignore_ascii_case(login))
            .cloned()
    }

    /// Find a repository by `owner/name`, case-insensitively.
    pub fn repo(&self, owner: &str, name: &str) -> Option<StoredRepo> {
        self.repos()
            .iter()
            .find(|r| r.owner.eq_ignore_ascii_case(owner) && r.name.eq_ignore_ascii_case(name))
            .cloned()
    }

    /// Find a repository by id.
    pub fn repo_by_id(&self, id: i64) -> Option<StoredRepo> {
        self.repos().iter().find(|r| r.id == id).cloned()
    }

    /// Find an issue or pull request by repository and display index.
    pub fn issue(&self, repo_id: i64, index: i64) -> Option<StoredIssue> {
        self.issues()
            .iter()
            .find(|i| i.repo_id == repo_id && i.index == index)
            .cloned()
    }

    /// Find an organization by name, case-insensitively.
    pub fn org(&self, name: &str) -> Option<StoredOrg> {
        self.orgs()
            .iter()
            .find(|o| o.name.eq_ignore_ascii_case(name))
            .cloned()
    }

    /// Next display index for a repository's issues and pull requests.
    pub fn next_index(&self, repo_id: i64) -> i64 {
        self.issues()
            .iter()
            .filter(|i| i.repo_id == repo_id)
            .map(|i| i.index)
            .max()
            .unwrap_or(0)
            + 1
    }
}

/// Last eight characters of a token value, as Gitea reports it.
pub fn last_eight(value: &str) -> String {
    let len = value.len();
    value.chars().skip(len.saturating_sub(8)).collect()
}

/// Derive a deterministic short SHA, standing in for a real git hash.
pub fn short_sha(seed: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in seed.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    let mut out = String::with_capacity(40);
    let mut state = hash;
    for _ in 0..40 {
        out.push(char::from_digit((state & 0xf) as u32, 16).unwrap_or('0'));
        state = state.rotate_left(7) ^ 0x9e37_79b9_7f4a_7c15;
    }
    out
}

/// Standard base64, used for file contents the way Gitea encodes them.
pub fn base64(input: &str) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            chunk.get(1).copied().unwrap_or(0),
            chunk.get(2).copied().unwrap_or(0),
        ];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        out.push(TABLE[(n >> 18) as usize & 63] as char);
        out.push(TABLE[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            TABLE[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

/// Current time as an RFC 3339 string, re-exported for handlers.
pub fn now() -> String {
    time::now()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn demo_repo() -> StoredRepo {
        StoredRepo {
            id: 1,
            owner: "alice".into(),
            name: "demo".into(),
            default_branch: "main".into(),
            empty: false,
            created: "2026-01-01T00:00:00Z".into(),
            updated: "2026-01-02T00:00:00Z".into(),
            ..Default::default()
        }
    }

    #[test]
    fn repo_dto_uses_gitea_field_names() {
        let dto = demo_repo().to_dto();
        assert_eq!(dto.full_name.as_deref(), Some("alice/demo"));
        assert_eq!(dto.default_branch.as_deref(), Some("main"));
        assert_eq!(dto.empty, Some(false));
        let json = serde_json::to_value(&dto).unwrap();
        assert_eq!(json["full_name"], "alice/demo");
        assert!(json.get("fullName").is_none());
    }

    #[test]
    fn issue_state_maps_to_wire_enum() {
        let mut issue = StoredIssue {
            id: 1,
            repo_id: 1,
            index: 1,
            title: "broken".into(),
            state: "closed".into(),
            ..Default::default()
        };
        let repo = demo_repo();
        assert_eq!(
            issue.to_dto(&repo, &[], None).state,
            Some(dto::StateType::Closed)
        );
        issue.state = "open".into();
        assert_eq!(
            issue.to_dto(&repo, &[], None).state,
            Some(dto::StateType::Open)
        );
    }

    #[test]
    fn access_level_maps_to_wire_enum() {
        assert_eq!(
            dto::AccessLevelName::from_wire("write"),
            dto::AccessLevelName::Write
        );
        assert_eq!(
            dto::AccessLevelName::from_wire("owner"),
            dto::AccessLevelName::Owner
        );
        assert_eq!(
            dto::AccessLevelName::from_wire("bogus"),
            dto::AccessLevelName::None
        );
    }

    #[test]
    fn base64_matches_known_vectors() {
        assert_eq!(base64(""), "");
        assert_eq!(base64("f"), "Zg==");
        assert_eq!(base64("fo"), "Zm8=");
        assert_eq!(base64("foo"), "Zm9v");
        assert_eq!(base64("hello world"), "aGVsbG8gd29ybGQ=");
    }

    #[test]
    fn short_sha_is_stable_and_hex() {
        let a = short_sha("alice/demo:1");
        assert_eq!(a.len(), 40);
        assert_eq!(a, short_sha("alice/demo:1"));
        assert_ne!(a, short_sha("alice/demo:2"));
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn lookups_are_case_insensitive() {
        let stores = Stores::default();
        stores.users_mut().push(StoredUser {
            id: 1,
            login: "Alice".into(),
            ..Default::default()
        });
        assert!(stores.user("alice").is_some());
        assert!(stores.user("ALICE").is_some());
        assert!(stores.user("bob").is_none());
    }
}
