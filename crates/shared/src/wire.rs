//! Wire-value conversions for the generated enums.
//!
//! Kept apart from `models.rs` so the generated type definitions stay
//! purely mechanical. Regenerate with `python3 scripts/gen_gitea_api.py`.

use crate::models::*;

impl AccessLevelName {
    /// The value as it appears on the wire.
    pub const fn as_wire(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Read => "read",
            Self::Write => "write",
            Self::Admin => "admin",
            Self::Owner => "owner",
        }
    }

    /// Parse a wire value, falling back to the first variant.
    pub fn from_wire(value: &str) -> Self {
        match value {
            "none" => Self::None,
            "read" => Self::Read,
            "write" => Self::Write,
            "admin" => Self::Admin,
            "owner" => Self::Owner,
            _ => Self::None,
        }
    }
}

impl ActivityOpType {
    /// The value as it appears on the wire.
    pub const fn as_wire(&self) -> &'static str {
        match self {
            Self::CreateRepo => "create_repo",
            Self::RenameRepo => "rename_repo",
            Self::StarRepo => "star_repo",
            Self::WatchRepo => "watch_repo",
            Self::CommitRepo => "commit_repo",
            Self::CreateIssue => "create_issue",
            Self::CreatePullRequest => "create_pull_request",
            Self::TransferRepo => "transfer_repo",
            Self::PushTag => "push_tag",
            Self::CommentIssue => "comment_issue",
            Self::MergePullRequest => "merge_pull_request",
            Self::CloseIssue => "close_issue",
            Self::ReopenIssue => "reopen_issue",
            Self::ClosePullRequest => "close_pull_request",
            Self::ReopenPullRequest => "reopen_pull_request",
            Self::DeleteTag => "delete_tag",
            Self::DeleteBranch => "delete_branch",
            Self::MirrorSyncPush => "mirror_sync_push",
            Self::MirrorSyncCreate => "mirror_sync_create",
            Self::MirrorSyncDelete => "mirror_sync_delete",
            Self::ApprovePullRequest => "approve_pull_request",
            Self::RejectPullRequest => "reject_pull_request",
            Self::CommentPull => "comment_pull",
            Self::PublishRelease => "publish_release",
            Self::PullReviewDismissed => "pull_review_dismissed",
            Self::PullRequestReadyForReview => "pull_request_ready_for_review",
            Self::AutoMergePullRequest => "auto_merge_pull_request",
        }
    }

    /// Parse a wire value, falling back to the first variant.
    pub fn from_wire(value: &str) -> Self {
        match value {
            "create_repo" => Self::CreateRepo,
            "rename_repo" => Self::RenameRepo,
            "star_repo" => Self::StarRepo,
            "watch_repo" => Self::WatchRepo,
            "commit_repo" => Self::CommitRepo,
            "create_issue" => Self::CreateIssue,
            "create_pull_request" => Self::CreatePullRequest,
            "transfer_repo" => Self::TransferRepo,
            "push_tag" => Self::PushTag,
            "comment_issue" => Self::CommentIssue,
            "merge_pull_request" => Self::MergePullRequest,
            "close_issue" => Self::CloseIssue,
            "reopen_issue" => Self::ReopenIssue,
            "close_pull_request" => Self::ClosePullRequest,
            "reopen_pull_request" => Self::ReopenPullRequest,
            "delete_tag" => Self::DeleteTag,
            "delete_branch" => Self::DeleteBranch,
            "mirror_sync_push" => Self::MirrorSyncPush,
            "mirror_sync_create" => Self::MirrorSyncCreate,
            "mirror_sync_delete" => Self::MirrorSyncDelete,
            "approve_pull_request" => Self::ApprovePullRequest,
            "reject_pull_request" => Self::RejectPullRequest,
            "comment_pull" => Self::CommentPull,
            "publish_release" => Self::PublishRelease,
            "pull_review_dismissed" => Self::PullReviewDismissed,
            "pull_request_ready_for_review" => Self::PullRequestReadyForReview,
            "auto_merge_pull_request" => Self::AutoMergePullRequest,
            _ => Self::CreateRepo,
        }
    }
}

impl ChangeFileOperationOperation {
    /// The value as it appears on the wire.
    pub const fn as_wire(&self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Update => "update",
            Self::Upload => "upload",
            Self::Rename => "rename",
            Self::Delete => "delete",
        }
    }

    /// Parse a wire value, falling back to the first variant.
    pub fn from_wire(value: &str) -> Self {
        match value {
            "create" => Self::Create,
            "update" => Self::Update,
            "upload" => Self::Upload,
            "rename" => Self::Rename,
            "delete" => Self::Delete,
            _ => Self::Create,
        }
    }
}

impl CommitStatusState {
    /// The value as it appears on the wire.
    pub const fn as_wire(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Success => "success",
            Self::Error => "error",
            Self::Failure => "failure",
            Self::Warning => "warning",
            Self::Skipped => "skipped",
        }
    }

    /// Parse a wire value, falling back to the first variant.
    pub fn from_wire(value: &str) -> Self {
        match value {
            "pending" => Self::Pending,
            "success" => Self::Success,
            "error" => Self::Error,
            "failure" => Self::Failure,
            "warning" => Self::Warning,
            "skipped" => Self::Skipped,
            _ => Self::Pending,
        }
    }
}

impl CreateHookOptionType {
    /// The value as it appears on the wire.
    pub const fn as_wire(&self) -> &'static str {
        match self {
            Self::Dingtalk => "dingtalk",
            Self::Discord => "discord",
            Self::Gitea => "gitea",
            Self::Gogs => "gogs",
            Self::Msteams => "msteams",
            Self::Slack => "slack",
            Self::Telegram => "telegram",
            Self::Feishu => "feishu",
            Self::Wechatwork => "wechatwork",
            Self::Packagist => "packagist",
        }
    }

    /// Parse a wire value, falling back to the first variant.
    pub fn from_wire(value: &str) -> Self {
        match value {
            "dingtalk" => Self::Dingtalk,
            "discord" => Self::Discord,
            "gitea" => Self::Gitea,
            "gogs" => Self::Gogs,
            "msteams" => Self::Msteams,
            "slack" => Self::Slack,
            "telegram" => Self::Telegram,
            "feishu" => Self::Feishu,
            "wechatwork" => Self::Wechatwork,
            "packagist" => Self::Packagist,
            _ => Self::Dingtalk,
        }
    }
}

impl CreateRepoOptionTrustModel {
    /// The value as it appears on the wire.
    pub const fn as_wire(&self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Collaborator => "collaborator",
            Self::Committer => "committer",
            Self::Collaboratorcommitter => "collaboratorcommitter",
        }
    }

    /// Parse a wire value, falling back to the first variant.
    pub fn from_wire(value: &str) -> Self {
        match value {
            "default" => Self::Default,
            "collaborator" => Self::Collaborator,
            "committer" => Self::Committer,
            "collaboratorcommitter" => Self::Collaboratorcommitter,
            _ => Self::Default,
        }
    }
}

impl DeployKeyKeyType {
    /// The value as it appears on the wire.
    pub const fn as_wire(&self) -> &'static str {
        match self {
            Self::Ssh => "ssh",
            Self::Token => "token",
        }
    }

    /// Parse a wire value, falling back to the first variant.
    pub fn from_wire(value: &str) -> Self {
        match value {
            "ssh" => Self::Ssh,
            "token" => Self::Token,
            _ => Self::Ssh,
        }
    }
}

impl IssueFormFieldType {
    /// The value as it appears on the wire.
    pub const fn as_wire(&self) -> &'static str {
        match self {
            Self::Markdown => "markdown",
            Self::Textarea => "textarea",
            Self::Input => "input",
            Self::Dropdown => "dropdown",
            Self::Checkboxes => "checkboxes",
        }
    }

    /// Parse a wire value, falling back to the first variant.
    pub fn from_wire(value: &str) -> Self {
        match value {
            "markdown" => Self::Markdown,
            "textarea" => Self::Textarea,
            "input" => Self::Input,
            "dropdown" => Self::Dropdown,
            "checkboxes" => Self::Checkboxes,
            _ => Self::Markdown,
        }
    }
}

impl IssueFormFieldVisible {
    /// The value as it appears on the wire.
    pub const fn as_wire(&self) -> &'static str {
        match self {
            Self::Form => "form",
            Self::Content => "content",
        }
    }

    /// Parse a wire value, falling back to the first variant.
    pub fn from_wire(value: &str) -> Self {
        match value {
            "form" => Self::Form,
            "content" => Self::Content,
            _ => Self::Form,
        }
    }
}

impl MergePullRequestOptionDo {
    /// The value as it appears on the wire.
    pub const fn as_wire(&self) -> &'static str {
        match self {
            Self::Merge => "merge",
            Self::Rebase => "rebase",
            Self::RebaseMerge => "rebase-merge",
            Self::Squash => "squash",
            Self::FastForwardOnly => "fast-forward-only",
            Self::ManuallyMerged => "manually-merged",
        }
    }

    /// Parse a wire value, falling back to the first variant.
    pub fn from_wire(value: &str) -> Self {
        match value {
            "merge" => Self::Merge,
            "rebase" => Self::Rebase,
            "rebase-merge" => Self::RebaseMerge,
            "squash" => Self::Squash,
            "fast-forward-only" => Self::FastForwardOnly,
            "manually-merged" => Self::ManuallyMerged,
            _ => Self::Merge,
        }
    }
}

impl MigrateRepoOptionsService {
    /// The value as it appears on the wire.
    pub const fn as_wire(&self) -> &'static str {
        match self {
            Self::Git => "git",
            Self::Github => "github",
            Self::Gitea => "gitea",
            Self::Gitlab => "gitlab",
            Self::Gogs => "gogs",
            Self::Onedev => "onedev",
            Self::Gitbucket => "gitbucket",
            Self::Codebase => "codebase",
            Self::Codecommit => "codecommit",
        }
    }

    /// Parse a wire value, falling back to the first variant.
    pub fn from_wire(value: &str) -> Self {
        match value {
            "git" => Self::Git,
            "github" => Self::Github,
            "gitea" => Self::Gitea,
            "gitlab" => Self::Gitlab,
            "gogs" => Self::Gogs,
            "onedev" => Self::Onedev,
            "gitbucket" => Self::Gitbucket,
            "codebase" => Self::Codebase,
            "codecommit" => Self::Codecommit,
            _ => Self::Git,
        }
    }
}

impl NotificationSubjectState {
    /// The value as it appears on the wire.
    pub const fn as_wire(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Closed => "closed",
            Self::Merged => "merged",
        }
    }

    /// Parse a wire value, falling back to the first variant.
    pub fn from_wire(value: &str) -> Self {
        match value {
            "open" => Self::Open,
            "closed" => Self::Closed,
            "merged" => Self::Merged,
            _ => Self::Open,
        }
    }
}

impl NotificationSubjectType {
    /// The value as it appears on the wire.
    pub const fn as_wire(&self) -> &'static str {
        match self {
            Self::Issue => "Issue",
            Self::Pull => "Pull",
            Self::Commit => "Commit",
            Self::Repository => "Repository",
        }
    }

    /// Parse a wire value, falling back to the first variant.
    pub fn from_wire(value: &str) -> Self {
        match value {
            "Issue" => Self::Issue,
            "Pull" => Self::Pull,
            "Commit" => Self::Commit,
            "Repository" => Self::Repository,
            _ => Self::Issue,
        }
    }
}

impl ObjectFormatName {
    /// The value as it appears on the wire.
    pub const fn as_wire(&self) -> &'static str {
        match self {
            Self::Sha1 => "sha1",
            Self::Sha256 => "sha256",
        }
    }

    /// Parse a wire value, falling back to the first variant.
    pub fn from_wire(value: &str) -> Self {
        match value {
            "sha1" => Self::Sha1,
            "sha256" => Self::Sha256,
            _ => Self::Sha1,
        }
    }
}

impl RepoWritePermission {
    /// The value as it appears on the wire.
    pub const fn as_wire(&self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
            Self::Admin => "admin",
        }
    }

    /// Parse a wire value, falling back to the first variant.
    pub fn from_wire(value: &str) -> Self {
        match value {
            "read" => Self::Read,
            "write" => Self::Write,
            "admin" => Self::Admin,
            _ => Self::Read,
        }
    }
}

impl ReviewStateType {
    /// The value as it appears on the wire.
    pub const fn as_wire(&self) -> &'static str {
        match self {
            Self::APPROVED => "APPROVED",
            Self::PENDING => "PENDING",
            Self::COMMENT => "COMMENT",
            Self::REQUESTCHANGES => "REQUEST_CHANGES",
            Self::REQUESTREVIEW => "REQUEST_REVIEW",
        }
    }

    /// Parse a wire value, falling back to the first variant.
    pub fn from_wire(value: &str) -> Self {
        match value {
            "APPROVED" => Self::APPROVED,
            "PENDING" => Self::PENDING,
            "COMMENT" => Self::COMMENT,
            "REQUEST_CHANGES" => Self::REQUESTCHANGES,
            "REQUEST_REVIEW" => Self::REQUESTREVIEW,
            _ => Self::APPROVED,
        }
    }
}

impl StateType {
    /// The value as it appears on the wire.
    pub const fn as_wire(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Closed => "closed",
        }
    }

    /// Parse a wire value, falling back to the first variant.
    pub fn from_wire(value: &str) -> Self {
        match value {
            "open" => Self::Open,
            "closed" => Self::Closed,
            _ => Self::Open,
        }
    }
}

impl TeamVisibility {
    /// The value as it appears on the wire.
    pub const fn as_wire(&self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Limited => "limited",
            Self::Private => "private",
        }
    }

    /// Parse a wire value, falling back to the first variant.
    pub fn from_wire(value: &str) -> Self {
        match value {
            "public" => Self::Public,
            "limited" => Self::Limited,
            "private" => Self::Private,
            _ => Self::Public,
        }
    }
}

impl UserTypeString {
    /// The value as it appears on the wire.
    pub const fn as_wire(&self) -> &'static str {
        match self {
            Self::User => "User",
            Self::Organization => "Organization",
            Self::Bot => "Bot",
        }
    }

    /// Parse a wire value, falling back to the first variant.
    pub fn from_wire(value: &str) -> Self {
        match value {
            "User" => Self::User,
            "Organization" => Self::Organization,
            "Bot" => Self::Bot,
            _ => Self::User,
        }
    }
}

impl VisibilityString {
    /// The value as it appears on the wire.
    pub const fn as_wire(&self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Limited => "limited",
            Self::Private => "private",
        }
    }

    /// Parse a wire value, falling back to the first variant.
    pub fn from_wire(value: &str) -> Self {
        match value {
            "public" => Self::Public,
            "limited" => Self::Limited,
            "private" => Self::Private,
            _ => Self::Public,
        }
    }
}
