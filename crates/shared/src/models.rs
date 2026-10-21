//! Generated from Gitea's OpenAPI 3 specification.
//!
//! Source: `gitea/templates/swagger/v1-openapi3.generated.json`
//! Regenerate with `python3 scripts/gen_gitea_api.py`.
//!
//! These are the wire types Gitea serves on `/api/v1`; the backend builds
//! them from its stores and the frontend consumes them unchanged.

#![allow(clippy::all)]

use serde::{Deserialize, Serialize};

/// APIError is an api error with a message
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct APIError {
    /// Message contains the error description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// URL contains the documentation URL for this error
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum AccessLevelName {
    #[default]
    #[serde(rename = "none")]
    None,
    #[serde(rename = "read")]
    Read,
    #[serde(rename = "write")]
    Write,
    #[serde(rename = "admin")]
    Admin,
    #[serde(rename = "owner")]
    Owner,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AccessToken {
    /// The timestamp when the token was created
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// The unique identifier of the access token
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// The timestamp when the token was last used
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<String>,
    /// The name of the access token
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The scopes granted to this access token
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
    /// The SHA1 hash of the access token
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,
    /// The last eight characters of the token
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_last_eight: Option<String>,
}

/// ActionArtifact represents a ActionArtifact
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ActionArtifact {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archive_download_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expired: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_in_bytes: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_run: Option<ActionWorkflowRun>,
}

/// ActionArtifactsResponse returns ActionArtifacts
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ActionArtifactsResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifacts: Option<Vec<ActionArtifact>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_count: Option<i64>,
}

/// ActionRunner represents a Runner
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ActionRunner {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub busy: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ephemeral: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<ActionRunnerLabel>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// ActionRunnerLabel represents a Runner Label
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ActionRunnerLabel {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
}

/// ActionRunnersResponse returns Runners
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ActionRunnersResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runners: Option<Vec<ActionRunner>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_count: Option<i64>,
}

/// ActionTask represents a ActionTask
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ActionTask {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// DisplayTitle is the display title for the workflow run
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_title: Option<String>,
    /// Event is the type of event that triggered the workflow
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
    /// HeadBranch is the branch that triggered the workflow
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head_branch: Option<String>,
    /// HeadSHA is the commit SHA that triggered the workflow
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head_sha: Option<String>,
    /// ID is the unique identifier for the action task
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// Name is the name of the workflow
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// RunNumber is the sequential number of the workflow run
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_number: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_started_at: Option<String>,
    /// Status indicates the current status of the workflow run
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    /// URL is the API URL for this workflow run
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// WorkflowID is the identifier of the workflow
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_id: Option<String>,
}

/// ActionTaskResponse returns a ActionTask
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ActionTaskResponse {
    /// TotalCount is the total number of workflow runs
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_count: Option<i64>,
    /// Entries contains the list of workflow runs
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_runs: Option<Vec<ActionTask>>,
}

/// ActionVariable return value of the query API
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ActionVariable {
    /// the value of the variable
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    /// the description of the variable
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// the name of the variable
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// the owner to which the variable belongs
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<i64>,
    /// the repository to which the variable belongs
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo_id: Option<i64>,
}

/// ActionWorkflow represents a ActionWorkflow
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ActionWorkflow {
    /// BadgeURL is the URL for the workflow badge
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub badge_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<String>,
    /// HTMLURL is the web URL for viewing the workflow
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
    /// ID is the unique identifier for the workflow
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Name is the name of the workflow
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Path is the file path of the workflow
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// State indicates if the workflow is active or disabled
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    /// URL is the API URL for this workflow
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// ActionWorkflowJob represents a WorkflowJob
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ActionWorkflowJob {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conclusion: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head_branch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head_sha: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_attempt: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runner_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runner_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub steps: Option<Vec<ActionWorkflowStep>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// ActionWorkflowJobsResponse returns ActionWorkflowJobs
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ActionWorkflowJobsResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jobs: Option<Vec<ActionWorkflowJob>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_count: Option<i64>,
}

/// ActionWorkflowResponse returns a ActionWorkflow
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ActionWorkflowResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_count: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflows: Option<Vec<ActionWorkflow>>,
}

/// ActionWorkflowRun represents a WorkflowRun
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ActionWorkflowRun {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor: Option<User>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifacts_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cancel_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conclusion: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head_branch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head_repository: Option<Repository>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head_sha: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jobs_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logs_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// PreviousAttemptURL is the API URL of the previous attempt of this run, e.g. ".../actions/runs/{run_id}/attempts/{attempt-1}". It is set only when the current attempt is > 1 (i.e. a rerun). For the first attempt, or for legacy runs that pre-date ActionRunAttempt, it is null.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_attempt_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pull_requests: Option<Vec<PullRequestMinimal>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository: Option<Repository>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rerun_url: Option<String>,
    /// RunAttempt is 1-based for runs created after ActionRunAttempt was introduced. A value of 0 is a legacy-only sentinel for runs created before attempts existed and indicates no corresponding /attempts/{n} resource is available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_attempt: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_number: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_actor: Option<User>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// ActionWorkflowRunsResponse returns ActionWorkflowRuns
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ActionWorkflowRunsResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_count: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_runs: Option<Vec<ActionWorkflowRun>>,
}

/// ActionWorkflowStep represents a step of a WorkflowJob
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ActionWorkflowStep {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conclusion: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Activity {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub act_user: Option<User>,
    /// The ID of the user who performed the action
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub act_user_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<Comment>,
    /// The ID of the comment associated with the activity (if applicable)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment_id: Option<i64>,
    /// Additional content or details about the activity
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// The date and time when the activity occurred
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    /// The unique identifier of the activity
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// Whether this activity is from a private repository
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_private: Option<bool>,
    /// the type of action
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub op_type: Option<ActivityOpType>,
    /// The name of the git reference (branch/tag) associated with the activity
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ref_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo: Option<Repository>,
    /// The ID of the repository associated with the activity
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo_id: Option<i64>,
    /// The ID of the user who receives/sees this activity
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<i64>,
}

/// AddCollaboratorOption options when adding a user as a collaborator of a repository
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AddCollaboratorOption {
    /// Permission level to grant the collaborator
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permission: Option<RepoWritePermission>,
}

/// AddTimeOption options for adding time to an issue
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AddTimeOption {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    /// time in seconds
    pub time: i64,
    /// username of the user who spent the time working on the issue (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_name: Option<String>,
}

/// AnnotatedTag represents an annotated tag
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AnnotatedTag {
    /// The message associated with the annotated tag
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: Option<AnnotatedTagObject>,
    /// The SHA hash of the annotated tag
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha: Option<String>,
    /// The name of the annotated tag
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tagger: Option<CommitUser>,
    /// The URL to access the annotated tag
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification: Option<PayloadCommitVerification>,
}

/// AnnotatedTagObject contains meta information of the tag object
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AnnotatedTagObject {
    /// The SHA hash of the tagged object
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha: Option<String>,
    /// The type of the tagged object (e.g., commit, tree)
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
    /// The URL to access the tagged object
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// ApplyDiffPatchFileOptions options for applying a diff patch Note: `author` and `committer` are optional (if only one is given, it will be used for the other, otherwise the authenticated user will be used)
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ApplyDiffPatchFileOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<Identity>,
    /// branch (optional) is the base branch for the changes. If not supplied, the default branch is used
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub committer: Option<Identity>,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dates: Option<CommitDateOptions>,
    /// force_push (optional) will do a force-push if the new branch already exists
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub force_push: Option<bool>,
    /// message (optional) is the commit message of the changes. If not supplied, a default message will be used
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// new_branch (optional) will make a new branch from base branch for the changes. If not supplied, the changes will be committed to the base branch
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_branch: Option<String>,
    /// Add a Signed-off-by trailer by the committer at the end of the commit log message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signoff: Option<bool>,
}

/// Attachment a generic attachment
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Attachment {
    /// DownloadURL is the URL to download the attachment
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub browser_download_url: Option<String>,
    /// Created is the time when the attachment was uploaded
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// DownloadCount is the number of times the attachment has been downloaded
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub download_count: Option<i64>,
    /// ID is the unique identifier for the attachment
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// Name is the filename of the attachment
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Size is the file size in bytes
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
    /// UUID is the unique identifier for the attachment file
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
}

/// Badge represents a user badge
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Badge {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
}

/// Branch represents a repository branch
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Branch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit: Option<PayloadCommit>,
    /// EffectiveBranchProtectionName is the name of the effective branch protection rule
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_branch_protection_name: Option<String>,
    /// EnableStatusCheck indicates if status checks are enabled
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_status_check: Option<bool>,
    /// Name is the branch name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Protected indicates if the branch is protected
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protected: Option<bool>,
    /// RequiredApprovals is the number of required approvals for pull requests
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_approvals: Option<i64>,
    /// StatusCheckContexts contains the list of required status check contexts
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_check_contexts: Option<Vec<String>>,
    /// UserCanMerge indicates if the current user can merge to this branch
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_can_merge: Option<bool>,
    /// UserCanPush indicates if the current user can push to this branch
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_can_push: Option<bool>,
}

/// BranchProtection represents a branch protection for a repository
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BranchProtection {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approvals_whitelist_teams: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approvals_whitelist_username: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_admin_merge_override: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_on_codeowner_reviews: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_on_official_review_requests: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_on_outdated_branch: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_on_rejected_reviews: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bypass_allowlist_teams: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bypass_allowlist_usernames: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deletion_allowlist_teams: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deletion_allowlist_usernames: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dismiss_stale_approvals: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_approvals_whitelist: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_bypass_allowlist: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_deletion: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_deletion_allowlist: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_force_push: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_force_push_allowlist: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_merge_whitelist: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_push: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_push_whitelist: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_status_check: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub force_push_allowlist_deploy_keys: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub force_push_allowlist_teams: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub force_push_allowlist_usernames: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ignore_stale_approvals: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merge_whitelist_teams: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merge_whitelist_usernames: Option<Vec<String>>,
    /// Priority is the priority of this branch protection rule
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protected_file_patterns: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub push_whitelist_deploy_keys: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub push_whitelist_teams: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub push_whitelist_usernames: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub require_signed_commits: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_approvals: Option<i64>,
    /// RuleName is the name of the branch protection rule
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rule_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_check_contexts: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unprotected_file_patterns: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

/// ChangeFileOperation for creating, updating or deleting a file
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ChangeFileOperation {
    /// new or updated file content, it must be base64 encoded
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// old path of the file to move
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_path: Option<String>,
    /// indicates what to do with the file: "create" for creating a new file, "update" for updating an existing file, "upload" for creating or updating a file, "rename" for renaming a file, and "delete" for deleting an existing file.
    pub operation: ChangeFileOperationOperation,
    /// path to the existing or new file
    pub path: String,
    /// the blob ID (SHA) for the file that already exists, required for changing existing files
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha: Option<String>,
}

/// ChangeFilesOptions options for creating, updating or deleting multiple files Note: `author` and `committer` are optional (if only one is given, it will be used for the other, otherwise the authenticated user will be used)
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ChangeFilesOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<Identity>,
    /// branch (optional) is the base branch for the changes. If not supplied, the default branch is used
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub committer: Option<Identity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dates: Option<CommitDateOptions>,
    /// list of file operations
    pub files: Vec<ChangeFileOperation>,
    /// force_push (optional) will do a force-push if the new branch already exists
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub force_push: Option<bool>,
    /// message (optional) is the commit message of the changes. If not supplied, a default message will be used
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// new_branch (optional) will make a new branch from base branch for the changes. If not supplied, the changes will be committed to the base branch
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_branch: Option<String>,
    /// Add a Signed-off-by trailer by the committer at the end of the commit log message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signoff: Option<bool>,
}

/// ChangedFile store information about files affected by the pull request
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ChangedFile {
    /// The number of lines added to the file
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub additions: Option<i64>,
    /// The total number of changes to the file
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changes: Option<i64>,
    /// The API URL to get the file contents
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contents_url: Option<String>,
    /// The number of lines deleted from the file
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deletions: Option<i64>,
    /// The name of the changed file
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    /// The HTML URL to view the file changes
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
    /// The previous filename if the file was renamed
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_filename: Option<String>,
    /// The raw URL to download the file
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_url: Option<String>,
    /// The status of the file change (added, modified, deleted, etc.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// CombinedStatus holds the combined state of several statuses for a single commit
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CombinedStatus {
    /// CommitURL is the API URL for the commit
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository: Option<Repository>,
    /// SHA is the commit SHA this status applies to
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha: Option<String>,
    /// State is the overall combined status state
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<CommitStatusState>,
    /// Statuses contains all individual commit statuses
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub statuses: Option<Vec<CommitStatus>>,
    /// TotalCount is the total number of statuses
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_count: Option<i64>,
    /// URL is the API URL for this combined status
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Comment represents a comment on a commit or issue
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Comment {
    /// Attachments contains files attached to the comment
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assets: Option<Vec<Attachment>>,
    /// Body contains the comment text content
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// HTMLURL is the web URL for viewing the comment
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
    /// ID is the unique identifier for the comment
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// IssueURL is the API URL for the issue
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue_url: Option<String>,
    /// OriginalAuthor is the original author name (for imported comments)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_author: Option<String>,
    /// OriginalAuthorID is the original author ID (for imported comments)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_author_id: Option<i64>,
    /// PRURL is the API URL for the pull request (if applicable)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pull_request_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Commit {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<User>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit: Option<RepoCommit>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub committer: Option<User>,
    /// Created is the time when the commit was created
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    /// Files contains information about files affected by the commit
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<CommitAffectedFiles>>,
    /// HTMLURL is the web URL for viewing the commit
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
    /// Parents contains the parent commit information
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parents: Option<Vec<CommitMeta>>,
    /// SHA is the commit SHA hash
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stats: Option<CommitStats>,
    /// URL is the API URL for the commit
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// CommitAffectedFiles store information about files affected by the commit
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CommitAffectedFiles {
    /// Filename is the path of the affected file
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    /// Status indicates how the file was affected (added, modified, deleted)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// CommitDateOptions store dates for GIT_AUTHOR_DATE and GIT_COMMITTER_DATE
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CommitDateOptions {
    /// Author is the author date for the commit
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// Committer is the committer date for the commit
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub committer: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CommitMeta {
    /// Created is the time when the commit was created
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    /// SHA is the commit SHA hash
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha: Option<String>,
    /// URL is the API URL for the commit
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// CommitStats is statistics for a RepoCommit
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CommitStats {
    /// Additions is the number of lines added
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub additions: Option<i64>,
    /// Deletions is the number of lines deleted
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deletions: Option<i64>,
    /// Total is the total number of lines changed
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,
}

/// CommitStatus holds a single status of a single Commit
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CommitStatus {
    /// Context is the unique context identifier for the status
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub creator: Option<User>,
    /// Description provides a brief description of the status
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// ID is the unique identifier for the commit status
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// State represents the status state (pending, success, error, failure)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<CommitStatusState>,
    /// TargetURL is the URL to link to for more details
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    /// URL is the API URL for this status
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum CommitStatusState {
    #[default]
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "success")]
    Success,
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "failure")]
    Failure,
    #[serde(rename = "warning")]
    Warning,
    #[serde(rename = "skipped")]
    Skipped,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CommitUser {
    /// Date is the commit date in string format
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    /// Email is the person's email address
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Name is the person's name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Compare {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commits: Option<Vec<Commit>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_commits: Option<i64>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ContentsExtResponse {
    /// DirContents contains directory listing when the path represents a directory
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dir_contents: Option<Vec<ContentsResponse>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_contents: Option<ContentsResponse>,
}

/// ContentsResponse contains information about a repo's entry's (dir, file, symlink, submodule) metadata and content
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ContentsResponse {
    #[serde(rename = "_links", default, skip_serializing_if = "Option::is_none")]
    pub links: Option<FileLinksResponse>,
    /// `content` is populated when `type` is `file`, otherwise null
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// DownloadURL is the direct download URL for this file
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub download_url: Option<String>,
    /// `encoding` is populated when `type` is `file`, otherwise null
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encoding: Option<String>,
    /// GitURL is the Git API URL for this blob or tree
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_url: Option<String>,
    /// HTMLURL is the web URL for this file or directory
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_author_date: Option<String>,
    /// LastCommitMessage is the message of the last commit that affected this file
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_commit_message: Option<String>,
    /// LastCommitSHA is the SHA of the last commit that affected this file
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_commit_sha: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_committer_date: Option<String>,
    /// LfsOid is the Git LFS object ID if this file is stored in LFS
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lfs_oid: Option<String>,
    /// LfsSize is the file size if this file is stored in LFS
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lfs_size: Option<i64>,
    /// `mode` is the Git file mode as an octal string, e.g. `100644` (regular), `100755` (executable), `120000` (symlink), `160000` (submodule)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    /// Name is the file or directory name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Path is the full path to the file or directory
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// SHA is the Git blob or tree SHA
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha: Option<String>,
    /// Size is the file size in bytes
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
    /// `submodule_git_url` is populated when `type` is `submodule`, otherwise null
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub submodule_git_url: Option<String>,
    /// `target` is populated when `type` is `symlink`, otherwise null
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    /// `type` will be `file`, `dir`, `symlink`, or `submodule`
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
    /// URL is the API URL for this file or directory
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// CreateAccessTokenOption options when create access token
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateAccessTokenOption {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
}

/// CreateActionWorkflowDispatch represents the payload for triggering a workflow dispatch event
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateActionWorkflowDispatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inputs: Option<std::collections::HashMap<String, String>>,
    #[serde(rename = "ref")]
    pub ref_: String,
}

/// CreateBranchProtectionOption options for creating a branch protection
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateBranchProtectionOption {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approvals_whitelist_teams: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approvals_whitelist_username: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_admin_merge_override: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_on_codeowner_reviews: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_on_official_review_requests: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_on_outdated_branch: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_on_rejected_reviews: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bypass_allowlist_teams: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bypass_allowlist_usernames: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deletion_allowlist_teams: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deletion_allowlist_usernames: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dismiss_stale_approvals: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_approvals_whitelist: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_bypass_allowlist: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_deletion: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_deletion_allowlist: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_force_push: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_force_push_allowlist: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_merge_whitelist: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_push: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_push_whitelist: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_status_check: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub force_push_allowlist_deploy_keys: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub force_push_allowlist_teams: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub force_push_allowlist_usernames: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ignore_stale_approvals: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merge_whitelist_teams: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merge_whitelist_usernames: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protected_file_patterns: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub push_whitelist_deploy_keys: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub push_whitelist_teams: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub push_whitelist_usernames: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub require_signed_commits: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_approvals: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rule_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_check_contexts: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unprotected_file_patterns: Option<String>,
}

/// CreateBranchRepoOption options when creating a branch in a repository
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateBranchRepoOption {
    /// Name of the branch to create
    pub new_branch_name: String,
    /// Name of the old branch to create from
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub old_branch_name: Option<String>,
    /// Name of the old branch/tag/commit to create from
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub old_ref_name: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateDeployKeyTokenOption {
    /// Describe if the token has only read access or read/write
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read_only: Option<bool>,
    /// Title of the token to add
    pub title: String,
}

/// CreateEmailOption options when creating email addresses
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateEmailOption {
    /// email addresses to add
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emails: Option<Vec<String>>,
}

/// CreateFileOptions options for creating a file Note: `author` and `committer` are optional (if only one is given, it will be used for the other, otherwise the authenticated user will be used)
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateFileOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<Identity>,
    /// branch (optional) is the base branch for the changes. If not supplied, the default branch is used
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub committer: Option<Identity>,
    /// content must be base64 encoded
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dates: Option<CommitDateOptions>,
    /// force_push (optional) will do a force-push if the new branch already exists
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub force_push: Option<bool>,
    /// message (optional) is the commit message of the changes. If not supplied, a default message will be used
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// new_branch (optional) will make a new branch from base branch for the changes. If not supplied, the changes will be committed to the base branch
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_branch: Option<String>,
    /// Add a Signed-off-by trailer by the committer at the end of the commit log message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signoff: Option<bool>,
}

/// CreateForkOption options for creating a fork
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateForkOption {
    /// name of the forked repository
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// organization name, if forking into an organization
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,
}

/// CreateGPGKeyOption options create user GPG key
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateGPGKeyOption {
    /// An armored GPG key to add
    pub armored_public_key: String,
    /// An optional armored signature for the GPG key
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub armored_signature: Option<String>,
}

/// CreateHookOption options when create a hook
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateHookOption {
    /// Whether the webhook should be active upon creation
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    /// Authorization header to include in webhook requests
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authorization_header: Option<String>,
    /// Branch filter pattern to determine which branches trigger the webhook
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch_filter: Option<String>,
    pub config: CreateHookOptionConfig,
    /// List of events that will trigger this webhook
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub events: Option<Vec<String>>,
    /// Optional human-readable name for the webhook
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The type of the webhook to create
    #[serde(rename = "type")]
    pub type_: CreateHookOptionType,
}

/// CreateHookOptionConfig has all config options in it required are "content_type" and "url" Required
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateHookOptionConfig {
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

/// CreateIssueCommentOption options for creating a comment on an issue
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateIssueCommentOption {
    /// Body is the comment text content
    pub body: String,
}

/// CreateIssueOption options to create one issue
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateIssueOption {
    /// deprecated
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assignees: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closed: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    /// list of label ids
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<i64>>,
    /// milestone id
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub milestone: Option<i64>,
    /// list of project ids
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projects: Option<Vec<i64>>,
    #[serde(rename = "ref", default, skip_serializing_if = "Option::is_none")]
    pub ref_: Option<String>,
    pub title: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateKeyOption {
    /// An armored SSH key to add
    pub key: String,
    /// Describe if the key has only read access or read/write
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read_only: Option<bool>,
    /// Title of the key to add
    pub title: String,
}

/// CreateLabelOption options for creating a label
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateLabelOption {
    pub color: String,
    /// Description provides additional context about the label's purpose
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclusive: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_archived: Option<bool>,
    /// Name is the display name for the new label
    pub name: String,
}

/// CreateMilestoneOption options for creating a milestone
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateMilestoneOption {
    /// Description provides details about the milestone
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Deadline is the due date for the milestone
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due_on: Option<String>,
    /// State indicates the initial state of the milestone
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<StateType>,
    /// Title is the title of the new milestone
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// CreateOAuth2ApplicationOptions holds options to create an oauth2 application
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateOAuth2ApplicationOptions {
    /// Whether the client is confidential
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidential_client: Option<bool>,
    /// The name of the OAuth2 application
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The list of allowed redirect URIs
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redirect_uris: Option<Vec<String>>,
    /// Whether to skip secondary authorization
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skip_secondary_authorization: Option<bool>,
}

/// CreateOrUpdateSecretOption options when creating or updating secret
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateOrUpdateSecretOption {
    /// Data of the secret to update
    pub data: String,
    /// Description of the secret to update
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// CreateOrgOption options for creating an organization
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateOrgOption {
    /// The description of the organization
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The email address of the organization
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// The full display name of the organization
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_name: Option<String>,
    /// The location of the organization
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    /// Whether repository administrators can change team access
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo_admin_change_team_access: Option<bool>,
    /// username of the organization
    pub username: String,
    /// possible values are `public` (default), `limited` or `private`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<VisibilityString>,
    /// The website URL of the organization
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
}

/// CreateProjectColumnOption represents options for creating a project column
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateProjectColumnOption {
    /// Column color in 6-digit hex format, e.g. #FF0000
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    pub title: String,
}

/// CreateProjectOption represents options for creating a project
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateProjectOption {
    /// Card type: "text_only" or "images_and_text"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub card_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Template type: "none", "basic_kanban" or "bug_triage"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template_type: Option<String>,
    pub title: String,
}

/// CreatePullRequestOption options when creating a pull request
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreatePullRequestOption {
    /// Whether maintainers can edit the pull request
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_maintainer_edit: Option<bool>,
    /// The primary assignee username
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,
    /// The list of assignee usernames
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assignees: Option<Vec<String>>,
    /// The base branch for the pull request
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base: Option<String>,
    /// The description body of the pull request
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    /// The head branch for the pull request, it could be a branch name on the base repository or a form like `<username>:<branch>` which refers to the user's fork repository's branch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head: Option<String>,
    /// The list of label IDs to assign to the pull request
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<i64>>,
    /// The milestone ID to assign to the pull request
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub milestone: Option<i64>,
    /// The list of reviewer usernames
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewers: Option<Vec<String>>,
    /// The list of team reviewer names
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team_reviewers: Option<Vec<String>>,
    /// The title of the pull request
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// CreatePullReviewComment represent a review comment for creation api
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreatePullReviewComment {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// if comment to new file line or 0
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_position: Option<i64>,
    /// if comment to old file line or 0
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub old_position: Option<i64>,
    /// the tree path
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// CreatePullReviewCommentReplyOptions are options to reply to a pull request review comment
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreatePullReviewCommentReplyOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

/// CreatePullReviewOptions are options to create a pull request review
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreatePullReviewOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comments: Option<Vec<CreatePullReviewComment>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event: Option<ReviewStateType>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreatePushMirrorOption {
    /// The sync interval for automatic updates
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interval: Option<String>,
    /// The remote repository URL to push to
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_address: Option<String>,
    /// The password for authentication with the remote repository
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_password: Option<String>,
    /// The username for authentication with the remote repository
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_username: Option<String>,
    /// Whether to sync on every commit
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sync_on_commit: Option<bool>,
}

/// CreateReleaseOption options when creating a release
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateReleaseOption {
    /// The release notes or description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// Whether to create the release as a draft
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub draft: Option<bool>,
    /// The display title of the release
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Whether to mark the release as a prerelease
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prerelease: Option<bool>,
    /// The message for the git tag
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag_message: Option<String>,
    pub tag_name: String,
    /// The target commitish for the release
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_commitish: Option<String>,
}

/// CreateRepoOption options when creating repository
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateRepoOption {
    /// Whether the repository should be auto-initialized?
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_init: Option<bool>,
    /// DefaultBranch of the repository (used when initializes and in template)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_branch: Option<String>,
    /// Description of the repository to create
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Gitignores to use
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gitignores: Option<String>,
    /// Label-Set to use
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue_labels: Option<String>,
    /// License to use
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    /// Name of the repository to create
    pub name: String,
    /// ObjectFormatName of the underlying git repository, empty string for default (sha1)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object_format_name: Option<ObjectFormatName>,
    /// Whether the repository is private
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub private: Option<bool>,
    /// Readme of the repository to create
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub readme: Option<String>,
    /// Whether the repository is template
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<bool>,
    /// TrustModel of the repository
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trust_model: Option<CreateRepoOptionTrustModel>,
}

/// CreateStatusOption holds the information needed to create a new CommitStatus for a Commit
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateStatusOption {
    /// Context is the unique context identifier for the status
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    /// Description provides a brief description of the status
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// State represents the status state to set (pending, success, error, failure)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<CommitStatusState>,
    /// TargetURL is the URL to link to for more details
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_url: Option<String>,
}

/// CreateTagOption options when creating a tag
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateTagOption {
    /// The message to associate with the tag
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// The name of the tag to create
    pub tag_name: String,
    /// The target commit SHA or branch name for the tag
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

/// CreateTagProtectionOption options for creating a tag protection
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateTagProtectionOption {
    /// The pattern to match tag names for protection
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name_pattern: Option<String>,
    /// List of team names allowed to create/delete protected tags
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub whitelist_teams: Option<Vec<String>>,
    /// List of usernames allowed to create/delete protected tags
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub whitelist_usernames: Option<Vec<String>>,
}

/// CreateTeamOption options for creating a team
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateTeamOption {
    /// Whether the team can create repositories in the organization
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_create_org_repo: Option<bool>,
    /// The description of the team
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether the team has access to all repositories in the organization
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub includes_all_repositories: Option<bool>,
    pub name: String,
    /// All units have this permission (read/write/admin)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permission: Option<RepoWritePermission>,
    /// Deprecated: This variable should be replaced by UnitsMap and will be dropped in later versions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub units: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub units_map: Option<std::collections::HashMap<String, String>>,
    /// Team visibility within the organization. Defaults to "private".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<TeamVisibility>,
}

/// CreateUserOption create user options
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateUserOption {
    /// For explicitly setting the user creation timestamp. Useful when users are migrated from other systems. When omitted, the user's creation timestamp will be set to "now".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    pub email: String,
    /// The full display name of the user
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_name: Option<String>,
    /// identifier of the user, provided by the external authenticator (if configured)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub login_name: Option<String>,
    /// Whether the user must change password on first login
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub must_change_password: Option<bool>,
    /// The plain text password for the user
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    /// Whether the user has restricted access privileges
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restricted: Option<bool>,
    /// Whether to send welcome notification email to the user
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub send_notify: Option<bool>,
    /// The authentication source ID to associate with the user
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_id: Option<i64>,
    /// username of the user
    pub username: String,
    /// User visibility level: public, limited, or private
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<VisibilityString>,
}

/// CreateVariableOption the option when creating variable
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateVariableOption {
    /// Description of the variable to create
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Value of the variable to create
    pub value: String,
}

/// CreateWikiPageOptions form for creating wiki
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateWikiPageOptions {
    /// content must be base64 encoded
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_base64: Option<String>,
    /// optional commit message summarizing the change
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// page title. leave empty to keep unchanged
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Cron represents a Cron task
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Cron {
    /// The total number of times this cron task has been executed
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exec_times: Option<i64>,
    /// The name of the cron task
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The next scheduled execution time
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
    /// The previous execution time
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev: Option<String>,
    /// The cron schedule expression (e.g., "0 0 * * *")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schedule: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CurrentAccessToken {
    /// The timestamp when the token was created
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// The unique identifier of the access token
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// The timestamp when the token was last used
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<String>,
    /// The name of the access token
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The scopes granted to this access token
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<UserMeta>,
}

/// DeleteEmailOption options when deleting email addresses
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DeleteEmailOption {
    /// email addresses to delete
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emails: Option<Vec<String>>,
}

/// DeleteFileOptions options for deleting a file Note: `author` and `committer` are optional (if only one is given, it will be used for the other, otherwise the authenticated user will be used)
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DeleteFileOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<Identity>,
    /// branch (optional) is the base branch for the changes. If not supplied, the default branch is used
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub committer: Option<Identity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dates: Option<CommitDateOptions>,
    /// force_push (optional) will do a force-push if the new branch already exists
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub force_push: Option<bool>,
    /// message (optional) is the commit message of the changes. If not supplied, a default message will be used
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// new_branch (optional) will make a new branch from base branch for the changes. If not supplied, the changes will be committed to the base branch
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_branch: Option<String>,
    /// the blob ID (SHA) for the file to delete
    pub sha: String,
    /// Add a Signed-off-by trailer by the committer at the end of the commit log message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signoff: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DeployKey {
    /// Created is the time when the deploy-key was added
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// Fingerprint is the key's fingerprint
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,
    /// ID is the unique identifier for the deploy-key
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// Key contains the actual SSH key content
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// KeyID is the associated public key ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key_id: Option<i64>,
    /// Type tells whether the key authenticates over SSH or with a token over HTTPS
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key_type: Option<DeployKeyKeyType>,
    /// ReadOnly indicates if the key has read-only access
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read_only: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository: Option<Repository>,
    /// Title is the human-readable name for the key
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Token is the plaintext token of an HTTPS key, only returned when it is created
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    /// URL is the API URL for this deploy-key
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// DismissPullReviewOptions are options to dismiss a pull request review
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DismissPullReviewOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priors: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EditActionRunnerOption {
    pub disabled: bool,
}

/// EditAttachmentOptions options for editing attachments
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EditAttachmentOptions {
    /// Name is the new filename for the attachment
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// EditBranchProtectionOption options for editing a branch protection
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EditBranchProtectionOption {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approvals_whitelist_teams: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approvals_whitelist_username: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_admin_merge_override: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_on_codeowner_reviews: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_on_official_review_requests: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_on_outdated_branch: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_on_rejected_reviews: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bypass_allowlist_teams: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bypass_allowlist_usernames: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deletion_allowlist_teams: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deletion_allowlist_usernames: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dismiss_stale_approvals: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_approvals_whitelist: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_bypass_allowlist: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_deletion: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_deletion_allowlist: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_force_push: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_force_push_allowlist: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_merge_whitelist: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_push: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_push_whitelist: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_status_check: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub force_push_allowlist_deploy_keys: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub force_push_allowlist_teams: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub force_push_allowlist_usernames: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ignore_stale_approvals: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merge_whitelist_teams: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merge_whitelist_usernames: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protected_file_patterns: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub push_whitelist_deploy_keys: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub push_whitelist_teams: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub push_whitelist_usernames: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub require_signed_commits: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_approvals: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_check_contexts: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unprotected_file_patterns: Option<String>,
}

/// EditDeadlineOption options for creating a deadline
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EditDeadlineOption {
    pub due_date: String,
}

/// EditGitHookOption options when modifying one Git hook
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EditGitHookOption {
    /// Content is the new script content for the hook
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

/// EditHookOption options when modify one hook
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EditHookOption {
    /// Whether the webhook is active and will be triggered
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    /// Authorization header to include in webhook requests
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authorization_header: Option<String>,
    /// Branch filter pattern to determine which branches trigger the webhook
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch_filter: Option<String>,
    /// Configuration settings for the webhook
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config: Option<std::collections::HashMap<String, String>>,
    /// List of events that trigger this webhook
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub events: Option<Vec<String>>,
    /// Optional human-readable name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// EditIssueCommentOption options for editing a comment
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EditIssueCommentOption {
    /// Body is the updated comment text content
    pub body: String,
}

/// EditIssueOption options for editing an issue
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EditIssueOption {
    /// deprecated
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assignees: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// The current version of the issue content to detect conflicts during editing
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_version: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub milestone: Option<i64>,
    /// list of project ids to set (replaces existing projects)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projects: Option<Vec<i64>>,
    #[serde(rename = "ref", default, skip_serializing_if = "Option::is_none")]
    pub ref_: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unset_due_date: Option<bool>,
}

/// EditLabelOption options for editing a label
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EditLabelOption {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Description provides additional context about the label's purpose
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclusive: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_archived: Option<bool>,
    /// Name is the new display name for the label
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// EditMilestoneOption options for editing a milestone
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EditMilestoneOption {
    /// Description provides updated details about the milestone
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Deadline is the updated due date for the milestone
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due_on: Option<String>,
    /// State indicates the updated state of the milestone
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<StateType>,
    /// Title is the updated title of the milestone
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// EditOrgOption options for editing an organization
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EditOrgOption {
    /// The description of the organization
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The email address of the organization; use empty string to clear
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// The full display name of the organization
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_name: Option<String>,
    /// The location of the organization
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    /// Whether repository administrators can change team access
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo_admin_change_team_access: Option<bool>,
    /// possible values are `public`, `limited` or `private`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<VisibilityString>,
    /// The website URL of the organization
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
}

/// EditProjectColumnOption represents options for editing a project column
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EditProjectColumnOption {
    /// Column color in 6-digit hex format, e.g. #FF0000
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Position of the column within the project, between -128 and 127
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sorting: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// EditProjectOption represents options for editing a project
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EditProjectOption {
    /// Card type: "text_only" or "images_and_text"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub card_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<StateType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// EditPullRequestOption options when modify pull request
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EditPullRequestOption {
    /// Whether to allow maintainer edits
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_maintainer_edit: Option<bool>,
    /// The new primary assignee username
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,
    /// The new list of assignee usernames
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assignees: Option<Vec<String>>,
    /// The new base branch for the pull request
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base: Option<String>,
    /// The new description body for the pull request
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// The current version of the pull request content to detect conflicts during editing
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_version: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    /// The new list of label IDs for the pull request
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<i64>>,
    /// The new milestone ID for the pull request
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub milestone: Option<i64>,
    /// The new state for the pull request
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    /// The new title for the pull request
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Whether to remove the current deadline
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unset_due_date: Option<bool>,
}

/// EditReactionOption contain the reaction type
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EditReactionOption {
    /// The reaction content (e.g., emoji or reaction type)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

/// EditReleaseOption options when editing a release
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EditReleaseOption {
    /// The new release notes or description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// Whether to change the draft status
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub draft: Option<bool>,
    /// The new display title of the release
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Whether to change the prerelease status
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prerelease: Option<bool>,
    /// The new name of the git tag
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag_name: Option<String>,
    /// The new target commitish for the release
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_commitish: Option<String>,
}

/// EditRepoOption options when editing a repository's properties
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EditRepoOption {
    /// either `true` to allow fast-forward-only merging pull requests, or `false` to prevent fast-forward-only merging.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_fast_forward_only_merge: Option<bool>,
    /// either `true` to allow mark pr as merged manually, or `false` to prevent it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_manual_merge: Option<bool>,
    /// either `true` to allow merging pull requests with a merge commit, or `false` to prevent merging pull requests with merge commits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_merge_commits: Option<bool>,
    /// either `true` to allow updating pull request branch by merge, or `false` to prevent it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_merge_update: Option<bool>,
    /// either `true` to allow rebase-merging pull requests, or `false` to prevent rebase-merging.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_rebase: Option<bool>,
    /// either `true` to allow rebase with explicit merge commits (--no-ff), or `false` to prevent rebase with explicit merge commits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_rebase_explicit: Option<bool>,
    /// either `true` to allow updating pull request branch by rebase, or `false` to prevent it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_rebase_update: Option<bool>,
    /// either `true` to allow squash-merging pull requests, or `false` to prevent squash-merging.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_squash_merge: Option<bool>,
    /// set to `true` to archive this repository.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archived: Option<bool>,
    /// either `true` to enable AutodetectManualMerge, or `false` to prevent it. Note: In some special cases, misjudgments can occur.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub autodetect_manual_merge: Option<bool>,
    /// set to `true` to allow edits from maintainers by default
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_allow_maintainer_edit: Option<bool>,
    /// sets the default branch for this repository.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_branch: Option<String>,
    /// set to `true` to delete pr branch after merge by default
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_delete_branch_after_merge: Option<bool>,
    /// set to a merge style to be used by this repository: "merge", "rebase", "rebase-merge", "squash", or "fast-forward-only".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_merge_style: Option<String>,
    /// set to an update style to be used by this repository: "merge" or "rebase".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_update_style: Option<String>,
    /// a short description of the repository.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// enable prune - remove obsolete remote-tracking references when mirroring
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_prune: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_tracker: Option<ExternalTracker>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_wiki: Option<ExternalWiki>,
    /// either `true` to enable actions unit, or `false` to disable them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_actions: Option<bool>,
    /// either `true` to enable code for this repository or `false` to disable it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_code: Option<bool>,
    /// either `true` to enable issues for this repository or `false` to disable them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_issues: Option<bool>,
    /// either `true` to enable packages unit, or `false` to disable them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_packages: Option<bool>,
    /// either `true` to enable project unit, or `false` to disable them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_projects: Option<bool>,
    /// either `true` to allow pull requests, or `false` to prevent pull request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_pull_requests: Option<bool>,
    /// either `true` to enable releases unit, or `false` to disable them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_releases: Option<bool>,
    /// either `true` to enable the wiki for this repository or `false` to disable it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_wiki: Option<bool>,
    /// either `true` to ignore whitespace for conflicts, or `false` to not ignore whitespace.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ignore_whitespace_conflicts: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub internal_tracker: Option<InternalTracker>,
    /// set to a string like `8h30m0s` to set the mirror interval time
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mirror_interval: Option<String>,
    /// authentication password for the remote repository (mirrors)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mirror_password: Option<String>,
    /// authentication token for the remote repository (mirrors)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mirror_token: Option<String>,
    /// authentication username for the remote repository (mirrors)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mirror_username: Option<String>,
    /// name of the repository
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// either `true` to make the repository private or `false` to make it public. Note: you will get a 422 error if the organization restricts changing repository visibility to organization owners and a non-owner tries to change the value of private.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub private: Option<bool>,
    /// `repo` to only allow repo-level projects, `owner` to only allow owner projects, `all` to allow both.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projects_mode: Option<String>,
    /// set the default squash merge commit message body: "PR_BODY", "COMMIT_MESSAGES", or "BLANK".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub squash_merge_commit_message: Option<String>,
    /// either `true` to make this repository a template or `false` to make it a normal repository
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<bool>,
    /// a URL with more information about the repository.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
}

/// EditTagProtectionOption options for editing a tag protection
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EditTagProtectionOption {
    /// The pattern to match tag names for protection
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name_pattern: Option<String>,
    /// List of team names allowed to create/delete protected tags
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub whitelist_teams: Option<Vec<String>>,
    /// List of usernames allowed to create/delete protected tags
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub whitelist_usernames: Option<Vec<String>>,
}

/// EditTeamOption options for editing a team
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EditTeamOption {
    /// Whether the team can create repositories in the organization
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_create_org_repo: Option<bool>,
    /// The description of the team
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether the team has access to all repositories in the organization
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub includes_all_repositories: Option<bool>,
    pub name: String,
    /// All units have this permission (read/write/admin)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permission: Option<RepoWritePermission>,
    /// Deprecated: This variable should be replaced by UnitsMap and will be dropped in later versions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub units: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub units_map: Option<std::collections::HashMap<String, String>>,
    /// Team visibility within the organization. When omitted, visibility is left unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<TeamVisibility>,
}

/// EditUserOption edit user options
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EditUserOption {
    /// Whether the user account is active
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    /// Whether the user has administrator privileges
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admin: Option<bool>,
    /// Whether the user can create organizations
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_create_organization: Option<bool>,
    /// Whether the user can use Git hooks
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_git_hook: Option<bool>,
    /// Whether the user can import local repositories
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_import_local: Option<bool>,
    /// The user's personal description or bio
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The email address of the user
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// The full display name of the user
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_name: Option<String>,
    /// The user's location or address
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    /// identifier of the user, provided by the external authenticator (if configured)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub login_name: Option<String>,
    /// Maximum number of repositories the user can create
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_repo_creation: Option<i64>,
    /// Whether the user must change password on next login
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub must_change_password: Option<bool>,
    /// The plain text password for the user
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    /// Whether the user is prohibited from logging in
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prohibit_login: Option<bool>,
    /// Whether the user has restricted access privileges
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restricted: Option<bool>,
    /// The authentication source ID to associate with the user
    pub source_id: i64,
    /// The user type
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_: Option<UserTypeString>,
    /// User visibility level: public, limited, or private
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<VisibilityString>,
    /// The user's personal website URL
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
}

/// Email an email address belonging to a user
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Email {
    /// The email address
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Whether this is the primary email address
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary: Option<bool>,
    /// The unique identifier of the user who owns this email
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<i64>,
    /// username of the user
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// Whether the email address has been verified
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verified: Option<bool>,
}

/// ExternalTracker represents settings for external tracker
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ExternalTracker {
    /// External Issue Tracker URL Format. Use the placeholders {user}, {repo} and {index} for the username, repository name and issue index.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_tracker_format: Option<String>,
    /// External Issue Tracker issue regular expression
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_tracker_regexp_pattern: Option<String>,
    /// External Issue Tracker Number Format, either `numeric`, `alphanumeric`, or `regexp`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_tracker_style: Option<String>,
    /// URL of external issue tracker.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_tracker_url: Option<String>,
}

/// ExternalWiki represents setting for external wiki
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ExternalWiki {
    /// URL of external wiki.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_wiki_url: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FileCommitResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<CommitUser>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub committer: Option<CommitUser>,
    /// Created is the time when the commit was created
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    /// HTMLURL is the web URL for viewing this commit
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
    /// Message is the commit message
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Parents contains parent commit metadata
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parents: Option<Vec<CommitMeta>>,
    /// SHA is the commit SHA hash
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tree: Option<CommitMeta>,
    /// URL is the API URL for the commit
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// FileDeleteResponse contains information about a repo's file that was deleted
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FileDeleteResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit: Option<FileCommitResponse>,
    /// Content is always null for delete operations
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification: Option<PayloadCommitVerification>,
}

/// FileLinksResponse contains the links for a repo's file
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FileLinksResponse {
    /// GitURL is the Git API URL for this file
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git: Option<String>,
    /// HTMLURL is the web URL for this file
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
    /// Self is the API URL for this file
    #[serde(rename = "self", default, skip_serializing_if = "Option::is_none")]
    pub self_: Option<String>,
}

/// FileResponse contains information about a repo's file
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FileResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit: Option<FileCommitResponse>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<ContentsResponse>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification: Option<PayloadCommitVerification>,
}

/// FilesResponse contains information about multiple files from a repo
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FilesResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit: Option<FileCommitResponse>,
    /// Files contains the list of file contents and metadata
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<ContentsResponse>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification: Option<PayloadCommitVerification>,
}

/// GPGKey a user GPG key to sign commit and tag in repository
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GPGKey {
    /// Whether the key can be used for certification
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_certify: Option<bool>,
    /// Whether the key can be used for encrypting communications
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_encrypt_comms: Option<bool>,
    /// Whether the key can be used for encrypting storage
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_encrypt_storage: Option<bool>,
    /// Whether the key can be used for signing
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_sign: Option<bool>,
    /// The date and time when the GPG key was created
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// List of email addresses associated with this GPG key
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emails: Option<Vec<GPGKeyEmail>>,
    /// The date and time when the GPG key expires
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    /// The unique identifier of the GPG key
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// The key ID of the GPG key
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key_id: Option<String>,
    /// The primary key ID of the GPG key
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_key_id: Option<String>,
    /// The public key content in armored format
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,
    /// List of subkeys of this GPG key
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subkeys: Option<Vec<GPGKey>>,
    /// Whether the GPG key has been verified
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verified: Option<bool>,
}

/// GPGKeyEmail an email attached to a GPGKey
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GPGKeyEmail {
    /// The email address associated with the GPG key
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Whether the email address has been verified
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verified: Option<bool>,
}

/// GeneralAPISettings contains global api settings exposed by it
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GeneralAPISettings {
    /// DefaultGitTreesPerPage is the default number of Git tree items per page
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_git_trees_per_page: Option<i64>,
    /// DefaultMaxBlobSize is the default maximum blob size for API responses
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_max_blob_size: Option<i64>,
    /// DefaultMaxResponseSize is the default maximum response size
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_max_response_size: Option<i64>,
    /// DefaultPagingNum is the default number of items per page
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_paging_num: Option<i64>,
    /// MaxResponseItems is the maximum number of items returned in API responses
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_response_items: Option<i64>,
}

/// GeneralAttachmentSettings contains global Attachment settings exposed by API
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GeneralAttachmentSettings {
    /// AllowedTypes contains the allowed file types for attachments
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_types: Option<String>,
    /// Enabled indicates if file attachments are enabled
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// MaxFiles is the maximum number of files per attachment
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_files: Option<i64>,
    /// MaxSize is the maximum size for individual attachments
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_size: Option<i64>,
}

/// GeneralRepoSettings contains global repository settings exposed by API
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GeneralRepoSettings {
    /// HTTPGitDisabled indicates if HTTP Git operations are disabled
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_git_disabled: Option<bool>,
    /// LFSDisabled indicates if Git LFS support is disabled
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lfs_disabled: Option<bool>,
    /// MigrationsDisabled indicates if repository migrations are disabled
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub migrations_disabled: Option<bool>,
    /// MirrorsDisabled indicates if repository mirroring is disabled
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mirrors_disabled: Option<bool>,
    /// StarsDisabled indicates if repository starring is disabled
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stars_disabled: Option<bool>,
    /// TimeTrackingDisabled indicates if time tracking is disabled
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_tracking_disabled: Option<bool>,
}

/// GeneralUISettings contains global ui settings exposed by API
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GeneralUISettings {
    /// AllowedReactions contains the list of allowed emoji reactions
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_reactions: Option<Vec<String>>,
    /// CustomEmojis contains the list of custom emojis
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_emojis: Option<Vec<String>>,
    /// DefaultTheme is the default UI theme
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_theme: Option<String>,
}

/// GenerateRepoOption options when creating a repository using a template
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GenerateRepoOption {
    /// include avatar of the template repo
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar: Option<bool>,
    /// Default branch of the new repository
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_branch: Option<String>,
    /// Description of the repository to create
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// include git content of default branch in template repo
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_content: Option<bool>,
    /// include git hooks in template repo
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_hooks: Option<bool>,
    /// include labels in template repo
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub labels: Option<bool>,
    pub name: String,
    /// the organization's name or individual user's name who will own the new repository
    pub owner: String,
    /// Whether the repository is private
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub private: Option<bool>,
    /// include protected branches in template repo
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protected_branch: Option<bool>,
    /// include topics in template repo
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topics: Option<bool>,
    /// include webhooks in template repo
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub webhooks: Option<bool>,
}

/// GetFilesOptions options for retrieving metadate and content of multiple files
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GetFilesOptions {
    /// Files is the list of file paths to retrieve
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<String>>,
}

/// GitBlobResponse represents a git blob
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GitBlobResponse {
    /// The content of the git blob (may be base64 encoded)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// The encoding used for the content (e.g., "base64")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encoding: Option<String>,
    /// The LFS object ID if this blob is stored in LFS
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lfs_oid: Option<String>,
    /// The size of the LFS object if this blob is stored in LFS
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lfs_size: Option<i64>,
    /// The SHA hash of the git blob
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha: Option<String>,
    /// The size of the git blob in bytes
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
    /// The URL to access this git blob
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// GitEntry represents a git tree
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GitEntry {
    /// Mode is the file mode (permissions)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    /// Path is the file or directory path
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// SHA is the Git object SHA
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha: Option<String>,
    /// Size is the file size in bytes
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
    /// Type indicates if this is a file, directory, or symlink
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
    /// URL is the API URL for this tree entry
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// GitHook represents a Git repository hook
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GitHook {
    /// Content contains the script content of the hook
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// IsActive indicates if the hook is active
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
    /// Name is the name of the Git hook
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GitObject {
    /// The SHA hash of the Git object
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha: Option<String>,
    /// The type of the Git object (e.g., commit, tag, tree, blob)
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
    /// The URL to access this Git object
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// GitTreeResponse returns a git tree
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GitTreeResponse {
    /// Page is the current page number for pagination
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page: Option<i64>,
    /// SHA is the tree object SHA
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha: Option<String>,
    /// TotalCount is the total number of entries in the tree
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_count: Option<i64>,
    /// Entries contains the tree entries (files and directories)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tree: Option<Vec<GitEntry>>,
    /// Truncated indicates if the response was truncated due to size
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub truncated: Option<bool>,
    /// URL is the API URL for this tree
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// GitignoreTemplateInfo name and text of a gitignore template
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GitignoreTemplateInfo {
    /// Name is the name of the gitignore template
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Source contains the content of the gitignore template
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// Hook a hook is a web hook when one repository changed
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Hook {
    /// Whether the webhook is active and will be triggered
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    /// Authorization header to include in webhook requests
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authorization_header: Option<String>,
    /// Branch filter pattern to determine which branches trigger the webhook
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch_filter: Option<String>,
    /// Configuration settings for the webhook
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config: Option<std::collections::HashMap<String, String>>,
    /// The date and time when the webhook was created
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// List of events that trigger this webhook
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub events: Option<Vec<String>>,
    /// The unique identifier of the webhook
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// Optional human-readable name for the webhook
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The type of the webhook (e.g., gitea, slack, discord)
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
    /// The date and time when the webhook was last updated
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

/// Identity for a person's identity like an author or committer
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Identity {
    /// Email is the person's email address
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Name is the person's name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// InternalTracker represents settings for internal tracker
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct InternalTracker {
    /// Let only contributors track time (Built-in issue tracker)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_only_contributors_to_track_time: Option<bool>,
    /// Enable dependencies for issues and pull requests (Built-in issue tracker)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_issue_dependencies: Option<bool>,
    /// Enable time tracking (Built-in issue tracker)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_time_tracker: Option<bool>,
}

/// Issue represents an issue in a repository
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Issue {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assets: Option<Vec<Attachment>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assignee: Option<User>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assignees: Option<Vec<User>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comments: Option<i64>,
    /// The version of the issue content for optimistic locking
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_version: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_locked: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<Label>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub milestone: Option<Milestone>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_author: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_author_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pin_order: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projects: Option<Vec<Project>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pull_request: Option<PullRequestMeta>,
    #[serde(rename = "ref", default, skip_serializing_if = "Option::is_none")]
    pub ref_: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository: Option<RepositoryMeta>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<StateType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_estimate: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,
}

/// IssueAssigneesOption options for adding/removing issue assignees
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct IssueAssigneesOption {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assignees: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct IssueConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blank_issues_enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contact_links: Option<Vec<IssueConfigContactLink>>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct IssueConfigContactLink {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub about: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct IssueConfigValidation {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid: Option<bool>,
}

/// IssueDeadline represents an issue deadline
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct IssueDeadline {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
}

/// IssueFormField represents a form field
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct IssueFormField {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attributes: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_: Option<IssueFormFieldType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validations: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible: Option<Vec<IssueFormFieldVisible>>,
}

/// IssueLabelsOption a collection of labels
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct IssueLabelsOption {
    /// Labels can be a list of integers representing label IDs or a list of strings representing label names
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<serde_json::Value>>,
}

/// IssueMeta basic issue information
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct IssueMeta {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<i64>,
    /// owner of the issue's repo
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
}

/// IssueTemplate represents an issue template for a repository
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct IssueTemplate {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub about: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assignees: Option<IssueTemplateStringSlice>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<Vec<IssueFormField>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub labels: Option<IssueTemplateStringSlice>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "ref", default, skip_serializing_if = "Option::is_none")]
    pub ref_: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct IssueTemplateStringSlice {
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

/// Label a label to an issue or a pr
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Label {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Description provides additional context about the label's purpose
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclusive: Option<bool>,
    /// ID is the unique identifier for the label
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_archived: Option<bool>,
    /// Name is the display name of the label
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// URL is the API endpoint for accessing this label
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// LabelTemplate info of a Label template
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LabelTemplate {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Description provides additional context about the label template's purpose
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclusive: Option<bool>,
    /// Name is the display name of the label template
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// LicensesInfo contains information about a License
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LicenseTemplateInfo {
    /// Body contains the full text of the license
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// Implementation contains license implementation details
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub implementation: Option<String>,
    /// Key is the unique identifier for the license template
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// Name is the display name of the license
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// URL is the reference URL for the license
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// LicensesListEntry is used for the API
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LicensesTemplateListEntry {
    /// Key is the unique identifier for the license template
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// Name is the display name of the license
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// URL is the reference URL for the license
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// LockIssueOption options to lock an issue
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LockIssueOption {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lock_reason: Option<String>,
}

/// MarkdownOption markdown options
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MarkdownOption {
    /// URL path for rendering issue, media and file links Expected format: /subpath/{user}/{repo}/src/{branch, commit, tag}/{identifier/path}/{file/dir}
    #[serde(rename = "Context", default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    /// Mode to render (markdown, comment, wiki, file)
    #[serde(rename = "Mode", default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    /// Text markdown to render
    #[serde(rename = "Text", default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Is it a wiki page? (use mode=wiki instead)
    #[serde(rename = "Wiki", default, skip_serializing_if = "Option::is_none")]
    pub wiki: Option<bool>,
}

/// MarkupOption markup options
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MarkupOption {
    /// URL path for rendering issue, media and file links Expected format: /subpath/{user}/{repo}/src/{branch, commit, tag}/{identifier/path}/{file/dir}
    #[serde(rename = "Context", default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    /// File path for detecting extension in file mode
    #[serde(rename = "FilePath", default, skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    /// Mode to render (markdown, comment, wiki, file)
    #[serde(rename = "Mode", default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    /// Text markup to render
    #[serde(rename = "Text", default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Is it a wiki page? (use mode=wiki instead)
    #[serde(rename = "Wiki", default, skip_serializing_if = "Option::is_none")]
    pub wiki: Option<bool>,
}

/// MergePullRequestForm form for merging Pull Request
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MergePullRequestOption {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delete_branch_after_merge: Option<bool>,
    #[serde(rename = "do")]
    pub do_: MergePullRequestOptionDo,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub force_merge: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head_commit_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merge_commit_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merge_message_field: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merge_title_field: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merge_when_checks_succeed: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MergeUpstreamRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ff_only: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MergeUpstreamResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merge_type: Option<String>,
}

/// MigrateRepoOptions options for migrating repository's this is used to interact with api v1
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MigrateRepoOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_password: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aws_access_key_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aws_secret_access_key: Option<String>,
    pub clone_addr: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issues: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub labels: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lfs: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lfs_endpoint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub milestones: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mirror: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mirror_interval: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub private: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pull_requests: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub releases: Option<bool>,
    pub repo_name: String,
    /// the organization's name or individual user's name who will own the migrated repository
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo_owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service: Option<MigrateRepoOptionsService>,
    /// deprecated (only for backwards compatibility, use repo_owner instead)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uid: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wiki: Option<bool>,
}

/// Milestone milestone is a collection of issues on one repository
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Milestone {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<String>,
    /// ClosedIssues is the number of closed issues in this milestone
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closed_issues: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// Description provides details about the milestone
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due_on: Option<String>,
    /// ID is the unique identifier for the milestone
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// OpenIssues is the number of open issues in this milestone
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_issues: Option<i64>,
    /// State indicates if the milestone is open or closed
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<StateType>,
    /// Title is the title of the milestone
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

/// MoveProjectColumnsOption represents options for reordering a project's columns
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MoveProjectColumnsOption {
    /// Every column ID of the project, in the desired left-to-right order
    pub column_ids: Vec<i64>,
}

/// MoveProjectIssueOption represents options for moving an issue between columns
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MoveProjectIssueOption {
    /// Target column to move the issue into
    pub column_id: i64,
    /// Position within the column, ascending. Omit to append. Negative values sort above the rest, equal values are ordered newest first.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sorting: Option<i64>,
}

/// NewIssuePinsAllowed represents an API response that says if new Issue Pins are allowed
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NewIssuePinsAllowed {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issues: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pull_requests: Option<bool>,
}

/// Note contains information related to a git note
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Note {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit: Option<Commit>,
    /// The content message of the git note
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// NotificationCount number of unread notifications
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NotificationCount {
    /// New is the number of unread notifications
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new: Option<i64>,
}

/// NotificationSubject contains the notification subject (Issue/Pull/Commit)
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NotificationSubject {
    /// HTMLURL is the web URL for the notification subject
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
    /// LatestCommentHTMLURL is the web URL for the latest comment
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_comment_html_url: Option<String>,
    /// LatestCommentURL is the API URL for the latest comment
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_comment_url: Option<String>,
    /// State indicates the current state of the notification subject
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<NotificationSubjectState>,
    /// Title is the title of the notification subject
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Type indicates the type of the notification subject
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_: Option<NotificationSubjectType>,
    /// URL is the API URL for the notification subject
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// NotificationThread expose Notification on API
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NotificationThread {
    /// ID is the unique identifier for the notification thread
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// Pinned indicates if the notification is pinned
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pinned: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository: Option<Repository>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<NotificationSubject>,
    /// Unread indicates if the notification has been read
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unread: Option<bool>,
    /// UpdatedAt is the time when the notification was last updated
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    /// URL is the API URL for this notification thread
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct OAuth2Application {
    /// The client ID of the OAuth2 application
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    /// The client secret of the OAuth2 application
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
    /// Whether the client is confidential
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidential_client: Option<bool>,
    /// The timestamp when the application was created
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    /// The unique identifier of the OAuth2 application
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// The name of the OAuth2 application
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The list of allowed redirect URIs
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redirect_uris: Option<Vec<String>>,
    /// Whether to skip secondary authorization
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skip_secondary_authorization: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum ObjectFormatName {
    #[default]
    #[serde(rename = "sha1")]
    Sha1,
    #[serde(rename = "sha256")]
    Sha256,
}

/// Organization represents an organization
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Organization {
    /// The URL of the organization's avatar
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    /// The description of the organization
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The email address of the organization
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// The full display name of the organization
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_name: Option<String>,
    /// The unique identifier of the organization
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// The location of the organization
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    /// The name of the organization
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Whether repository administrators can change team access
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo_admin_change_team_access: Option<bool>,
    /// username of the organization deprecated
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// The visibility level of the organization (public, limited, private)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<VisibilityString>,
    /// The website URL of the organization
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
}

/// OrganizationPermissions list different users permissions on an organization
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct OrganizationPermissions {
    /// Whether the user can create repositories in the organization
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_create_repository: Option<bool>,
    /// Whether the user can read the organization
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_read: Option<bool>,
    /// Whether the user can write to the organization
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_write: Option<bool>,
    /// Whether the user is an admin of the organization
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_admin: Option<bool>,
    /// Whether the user is an owner of the organization
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_owner: Option<bool>,
}

/// PRBranchInfo information about a branch
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PRBranchInfo {
    /// The display name of the branch
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The git reference of the branch
    #[serde(rename = "ref", default, skip_serializing_if = "Option::is_none")]
    pub ref_: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo: Option<Repository>,
    /// The unique identifier of the repository
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo_id: Option<i64>,
    /// The commit SHA of the branch head
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha: Option<String>,
}

/// Package represents a package
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Package {
    /// The date and time when the package was created
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub creator: Option<User>,
    /// The HTML URL to view the package
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
    /// The unique identifier of the package
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// The name of the package
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<User>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository: Option<Repository>,
    /// The type of the package (e.g., npm, maven, docker)
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
    /// The version of the package
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// PackageFile represents a package file
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PackageFile {
    /// The unique identifier of the package file
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// The MD5 hash of the package file
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub md5: Option<String>,
    /// The name of the package file
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The SHA1 hash of the package file
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,
    /// The SHA256 hash of the package file
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    /// The SHA512 hash of the package file
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha512: Option<String>,
    /// The size of the package file in bytes
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
}

/// PayloadCommit represents a commit
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PayloadCommit {
    /// List of files added in this commit
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub added: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<PayloadUser>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub committer: Option<PayloadUser>,
    /// sha1 hash of the commit
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The commit message
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// List of files modified in this commit
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modified: Option<Vec<String>>,
    /// List of files removed in this commit
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub removed: Option<Vec<String>>,
    /// The timestamp when the commit was made
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    /// The URL to view this commit
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification: Option<PayloadCommitVerification>,
}

/// PayloadCommitVerification represents the GPG verification of a commit
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PayloadCommitVerification {
    /// The signed payload content
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<String>,
    /// The reason for the verification status
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// The GPG signature of the commit
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signer: Option<PayloadUser>,
    /// Whether the commit signature is verified
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verified: Option<bool>,
}

/// PayloadUser represents the author or committer of a commit
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PayloadUser {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Full name of the commit author
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// username of the user
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
}

/// Permission represents a set of permissions
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Permission {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admin: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pull: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub push: Option<bool>,
}

/// Projects track issues and pull requests, standalone note cards are not supported.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Project {
    /// Card type: "text_only" or "images_and_text"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub card_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub creator: Option<User>,
    /// Deprecated: use Creator instead
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub creator_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// Deprecated: use State instead
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_closed: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub num_closed_issues: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub num_issues: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub num_open_issues: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<StateType>,
    /// Template type: "none", "basic_kanban" or "bug_triage"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Project type: "individual", "repository" or "organization"
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
    /// null only for legacy rows that carry no update timestamp
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

/// ProjectColumn represents a project column (board)
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ProjectColumn {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub creator: Option<User>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sorting: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// null only for legacy rows that carry no update timestamp
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

/// PublicKey publickey is a user key to push code to repository
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PublicKey {
    /// Created is the time when the key was added
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// Fingerprint is the key's fingerprint
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,
    /// ID is the unique identifier for the public key
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// Key contains the actual SSH public key content
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// KeyType indicates the type of the SSH key
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key_type: Option<String>,
    /// Updated is the time when the key was last used
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<String>,
    /// ReadOnly indicates if the key has read-only access
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read_only: Option<bool>,
    /// Title is the human-readable name for the key
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// URL is the API URL for this key
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,
}

/// PullRequest represents a pull request
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PullRequest {
    /// The number of lines added in the pull request
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub additions: Option<i64>,
    /// Whether maintainers can edit the pull request
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_maintainer_edit: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assignee: Option<User>,
    /// The list of users assigned to the pull request
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assignees: Option<Vec<User>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base: Option<PRBranchInfo>,
    /// The description body of the pull request
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// The number of files changed in the pull request
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changed_files: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<String>,
    /// The number of comments on the pull request
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comments: Option<i64>,
    /// The version of the pull request content for optimistic locking
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_version: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// The number of lines deleted in the pull request
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deletions: Option<i64>,
    /// The URL to download the diff patch
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diff_url: Option<String>,
    /// Whether the pull request is a draft
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub draft: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head: Option<PRBranchInfo>,
    /// The HTML URL to view the pull request
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
    /// The unique identifier of the pull request
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// Whether the pull request conversation is locked
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_locked: Option<bool>,
    /// The labels attached to the pull request
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<Label>>,
    /// The merge base commit SHA
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merge_base: Option<String>,
    /// The SHA of the merge commit
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merge_commit_sha: Option<String>,
    /// Whether the pull request can be merged
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mergeable: Option<bool>,
    /// Whether the pull request has been merged
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merged: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merged_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merged_by: Option<User>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub milestone: Option<Milestone>,
    /// The pull request number
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number: Option<i64>,
    /// The URL to download the patch file
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patch_url: Option<String>,
    /// The pin order for the pull request
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pin_order: Option<i64>,
    /// The users requested to review the pull request
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_reviewers: Option<Vec<User>>,
    /// The teams requested to review the pull request
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_reviewers_teams: Option<Vec<Team>>,
    /// number of review comments made on the diff of a PR review (not including comments on commits or issues in a PR)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review_comments: Option<i64>,
    /// The current state of the pull request
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<StateType>,
    /// The title of the pull request
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    /// The API URL of the pull request
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,
}

/// PullRequestMeta PR info if an issue is a PR
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PullRequestMeta {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub draft: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merged: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merged_at: Option<String>,
}

/// PullRequestMinimal is the minimal information about a pull request, as returned in the `pull_requests` field of a workflow run.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PullRequestMinimal {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base: Option<PullRequestMinimalHead>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head: Option<PullRequestMinimalHead>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PullRequestMinimalHead {
    #[serde(rename = "ref", default, skip_serializing_if = "Option::is_none")]
    pub ref_: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo: Option<PullRequestMinimalHeadRepo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PullRequestMinimalHeadRepo {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// PullReview represents a pull request review
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PullReview {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comments_count: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dismissed: Option<bool>,
    /// HTMLURL is the web URL for viewing the review
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub official: Option<bool>,
    /// HTMLPullURL is the web URL for the pull request
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pull_request_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stale: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<ReviewStateType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub submitted_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team: Option<Team>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,
}

/// PullReviewComment represents a comment on a pull request review
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PullReviewComment {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diff_hunk: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_commit_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_position: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pull_request_review_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pull_request_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolver: Option<User>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,
}

/// PullReviewRequestOptions are options to add or remove pull request review requests
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PullReviewRequestOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewers: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team_reviewers: Option<Vec<String>>,
}

/// PushMirror represents information of a push mirror
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PushMirror {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    /// The sync interval for automatic updates
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interval: Option<String>,
    /// The last error message encountered during sync
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_update: Option<String>,
    /// The remote repository URL being mirrored to
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_address: Option<String>,
    /// The name of the remote in the git configuration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_name: Option<String>,
    /// The name of the source repository
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo_name: Option<String>,
    /// Whether to sync on every commit
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sync_on_commit: Option<bool>,
}

/// Reaction contain one reaction
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Reaction {
    /// The reaction content (e.g., emoji or reaction type)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// The date and time when the reaction was created
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Reference {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: Option<GitObject>,
    /// The name of the Git reference (e.g., refs/heads/main)
    #[serde(rename = "ref", default, skip_serializing_if = "Option::is_none")]
    pub ref_: Option<String>,
    /// The URL to access this Git reference
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Release represents a repository release
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Release {
    /// The files attached to the release
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assets: Option<Vec<Attachment>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<User>,
    /// The release notes or description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// Whether the release is a draft
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub draft: Option<bool>,
    /// The HTML URL to view the release
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
    /// The unique identifier of the release
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// The display title of the release
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Whether the release is a prerelease
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prerelease: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub published_at: Option<String>,
    /// The name of the git tag associated with the release
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag_name: Option<String>,
    /// The URL to download the tarball archive
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tarball_url: Option<String>,
    /// The target commitish for the release
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_commitish: Option<String>,
    /// The URL template for uploading release assets
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upload_url: Option<String>,
    /// The API URL of the release
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// The URL to download the zip archive
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zipball_url: Option<String>,
}

/// RenameBranchRepoOption options when renaming a branch in a repository
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RenameBranchRepoOption {
    /// New branch name
    pub name: String,
}

/// RenameOrgOption options when renaming an organization
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RenameOrgOption {
    /// New username for this org. This name cannot be in use yet by any other user.
    pub new_name: String,
}

/// RenameUserOption options when renaming a user
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RenameUserOption {
    /// New username for this user. This name cannot be in use yet by any other user.
    pub new_username: String,
}

/// RepoCollaboratorPermission to get repository permission for a collaborator
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RepoCollaboratorPermission {
    /// Permission level of the collaborator
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permission: Option<AccessLevelName>,
    /// RoleName is the name of the permission role
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RepoCommit {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<CommitUser>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub committer: Option<CommitUser>,
    /// Message is the commit message
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tree: Option<CommitMeta>,
    /// URL is the API URL for the commit
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification: Option<PayloadCommitVerification>,
}

/// RepoTopicOptions a collection of repo topic names
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RepoTopicOptions {
    /// list of topic names
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topics: Option<Vec<String>>,
}

/// RepoTransfer represents a pending repo transfer
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RepoTransfer {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doer: Option<User>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recipient: Option<User>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub teams: Option<Vec<Team>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum RepoWritePermission {
    #[default]
    #[serde(rename = "read")]
    Read,
    #[serde(rename = "write")]
    Write,
    #[serde(rename = "admin")]
    Admin,
}

/// Repository represents a repository
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Repository {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_fast_forward_only_merge: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_manual_merge: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_merge_commits: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_merge_update: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_rebase: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_rebase_explicit: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_rebase_update: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_squash_merge: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archived: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archived_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub autodetect_manual_merge: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch_count: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clone_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_allow_maintainer_edit: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_branch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_delete_branch_after_merge: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_merge_style: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_target_branch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_update_style: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub empty: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_tracker: Option<ExternalTracker>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_wiki: Option<ExternalWiki>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fork: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forks_count: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_actions: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_code: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_issues: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_packages: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_projects: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_pull_requests: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_releases: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_wiki: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ignore_whitespace_conflicts: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub internal: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub internal_tracker: Option<InternalTracker>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub languages_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub licenses: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mirror: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mirror_interval: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mirror_last_sync_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mirror_updated: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// ObjectFormatName of the underlying git repository
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object_format_name: Option<ObjectFormatName>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_issues_count: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_pr_counter: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<User>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<Box<Repository>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Permission>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub private: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projects_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release_counter: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo_transfer: Option<RepoTransfer>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub squash_merge_commit_message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ssh_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stars_count: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topics: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub watchers_count: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
}

/// RepositoryMeta basic repository information
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RepositoryMeta {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum ReviewStateType {
    #[default]
    APPROVED,
    PENDING,
    COMMENT,
    #[serde(rename = "REQUEST_CHANGES")]
    REQUESTCHANGES,
    #[serde(rename = "REQUEST_REVIEW")]
    REQUESTREVIEW,
}

/// RunDetails returns workflow_dispatch runid and url
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RunDetails {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_run_id: Option<i64>,
}

/// SearchResults results of a successful search
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SearchResults {
    /// Data contains the repository search results
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<Repository>>,
    /// OK indicates if the search was successful
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ok: Option<bool>,
}

/// Secret represents a secret
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Secret {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// the secret's description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// the secret's name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// ServerVersion wraps the version of the server
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ServerVersion {
    /// Version is the server version string
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum StateType {
    #[default]
    #[serde(rename = "open")]
    Open,
    #[serde(rename = "closed")]
    Closed,
}

/// StopWatch represent a running stopwatch
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StopWatch {
    /// Created is the time when the stopwatch was started
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    /// Duration is a human-readable duration string
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<String>,
    /// IssueIndex is the index number of the associated issue
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue_index: Option<i64>,
    /// IssueTitle is the title of the associated issue
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue_title: Option<String>,
    /// RepoName is the name of the repository
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo_name: Option<String>,
    /// RepoOwnerName is the name of the repository owner
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo_owner_name: Option<String>,
    /// Seconds is the total elapsed time in seconds
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seconds: Option<i64>,
}

/// SubmitPullReviewOptions are options to submit a pending pull request review
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SubmitPullReviewOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event: Option<ReviewStateType>,
}

/// Tag represents a repository tag
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Tag {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit: Option<CommitMeta>,
    /// The ID (SHA) of the tag
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The message associated with the tag
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// The name of the tag
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The URL to download the tarball archive
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tarball_url: Option<String>,
    /// The URL to download the zipball archive
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zipball_url: Option<String>,
}

/// TagProtection represents a tag protection
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TagProtection {
    /// The date and time when the tag protection was created
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// The unique identifier of the tag protection
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// The pattern to match tag names for protection
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name_pattern: Option<String>,
    /// The date and time when the tag protection was last updated
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    /// List of team names allowed to create/delete protected tags
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub whitelist_teams: Option<Vec<String>>,
    /// List of usernames allowed to create/delete protected tags
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub whitelist_usernames: Option<Vec<String>>,
}

/// Team represents a team in an organization
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Team {
    /// Whether the team can create repositories in the organization
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_create_org_repo: Option<bool>,
    /// The description of the team
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The unique identifier of the team
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// Whether the team has access to all repositories in the organization
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub includes_all_repositories: Option<bool>,
    /// The name of the team
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization: Option<Organization>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permission: Option<AccessLevelName>,
    /// Deprecated: This variable should be replaced by UnitsMap and will be dropped in later versions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub units: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub units_map: Option<std::collections::HashMap<String, String>>,
    /// Team visibility within the organization. "private" teams are only listable by members and org owners; "limited" teams are listable by any organization member; "public" teams are listable by any signed-in user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<TeamVisibility>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum TeamVisibility {
    #[default]
    #[serde(rename = "public")]
    Public,
    #[serde(rename = "limited")]
    Limited,
    #[serde(rename = "private")]
    Private,
}

/// TimeStamp defines a timestamp
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TimeStamp {
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

/// TimelineComment represents a timeline comment (comment of any type) on a commit or issue
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TimelineComment {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assignee: Option<User>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assignee_team: Option<Team>,
    /// Body contains the timeline event content
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dependent_issue: Option<Issue>,
    /// HTMLURL is the web URL for viewing the comment
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
    /// ID is the unique identifier for the timeline comment
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// IssueURL is the API URL for the issue
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<Label>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub milestone: Option<Milestone>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub old_milestone: Option<Milestone>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub old_project_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub old_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub old_title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<i64>,
    /// PRURL is the API URL for the pull request (if applicable)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pull_request_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ref_action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ref_comment: Option<Comment>,
    /// commit SHA where issue/PR was referenced
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ref_commit_sha: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ref_issue: Option<Issue>,
    /// whether the assignees were removed or added
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub removed_assignee: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolve_doer: Option<User>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tracked_time: Option<TrackedTime>,
    /// Type indicates the type of timeline event
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,
}

/// TopicListResponse returns a list of TopicResponse
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TopicListResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topics: Option<Vec<TopicResponse>>,
}

/// TopicName a list of repo topic names
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TopicName {
    /// List of topic names
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topics: Option<Vec<String>>,
}

/// TopicResponse for returning topics
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TopicResponse {
    /// The date and time when the topic was created
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    /// The unique identifier of the topic
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// The number of repositories using this topic
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo_count: Option<i64>,
    /// The name of the topic
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topic_name: Option<String>,
    /// The date and time when the topic was last updated
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated: Option<String>,
}

/// TrackedTime worked time for an issue / pr
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TrackedTime {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    /// ID is the unique identifier for the tracked time entry
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue: Option<Issue>,
    /// deprecated (only for backwards compatibility)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue_id: Option<i64>,
    /// Time in seconds
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time: Option<i64>,
    /// deprecated (only for backwards compatibility)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<i64>,
    /// username of the user
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_name: Option<String>,
}

/// TransferRepoOption options when transfer a repository's ownership
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TransferRepoOption {
    pub new_owner: String,
    /// ID of the team or teams to add to the repository. Teams can only be added to organization-owned repositories.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team_ids: Option<Vec<i64>>,
}

/// UpdateBranchProtectionPriories a list to update the branch protection rule priorities
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UpdateBranchProtectionPriories {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ids: Option<Vec<i64>>,
}

/// UpdateBranchRepoOption options when updating a branch reference in a repository
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UpdateBranchRepoOption {
    /// Force update even if the change is not a fast-forward
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub force: Option<bool>,
    /// New commit SHA (or any ref) the branch should point to
    pub new_commit_id: String,
    /// Expected old commit SHA of the branch; if provided it must match the current tip
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub old_commit_id: Option<String>,
}

/// UpdateFileOptions options for updating or creating a file Note: `author` and `committer` are optional (if only one is given, it will be used for the other, otherwise the authenticated user will be used)
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UpdateFileOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<Identity>,
    /// branch (optional) is the base branch for the changes. If not supplied, the default branch is used
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub committer: Option<Identity>,
    /// content must be base64 encoded
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dates: Option<CommitDateOptions>,
    /// force_push (optional) will do a force-push if the new branch already exists
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub force_push: Option<bool>,
    /// from_path (optional) is the path of the original file which will be moved/renamed to the path in the URL
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_path: Option<String>,
    /// message (optional) is the commit message of the changes. If not supplied, a default message will be used
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// new_branch (optional) will make a new branch from base branch for the changes. If not supplied, the changes will be committed to the base branch
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_branch: Option<String>,
    /// the blob ID (SHA) for the file that already exists to update, or leave it empty to create a new file
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha: Option<String>,
    /// Add a Signed-off-by trailer by the committer at the end of the commit log message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signoff: Option<bool>,
}

/// UpdateRepoAvatarUserOption options when updating the repo avatar
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UpdateRepoAvatarOption {
    /// image must be base64 encoded
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
}

/// UpdateUserAvatarUserOption options when updating the user avatar
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UpdateUserAvatarOption {
    /// image must be base64 encoded
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
}

/// UpdateVariableOption the option when updating variable
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UpdateVariableOption {
    /// Description of the variable to update
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// New name for the variable. If the field is empty, the variable name won't be updated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Value of the variable to update
    pub value: String,
}

/// User represents a user
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct User {
    /// Is user active
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    /// URL to the user's avatar
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    /// the user's description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// user counts
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub followers_count: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub following_count: Option<i64>,
    /// the user's full name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_name: Option<String>,
    /// URL to the user's gitea page
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
    /// the user's id
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// Is the user an administrator
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_admin: Option<bool>,
    /// User locale
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_login: Option<String>,
    /// the user's location
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    /// login of the user, same as `username`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub login: Option<String>,
    /// identifier of the user, provided by the external authenticator (if configured)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub login_name: Option<String>,
    /// Is user login prohibited
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prohibit_login: Option<bool>,
    /// Is user restricted
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restricted: Option<bool>,
    /// The ID of the user's Authentication Source
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub starred_repos_count: Option<i64>,
    /// the user type
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_: Option<UserTypeString>,
    /// User visibility level option: public, limited, private
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<VisibilityString>,
    /// the user's website
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
}

/// UserBadgeOption options for link between users and badges
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UserBadgeOption {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub badge_slugs: Option<Vec<String>>,
}

/// UserHeatmapData represents the data needed to create a heatmap
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UserHeatmapData {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contributions: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<TimeStamp>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UserMeta {
    /// The unique identifier of the user
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// The username of the user
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub login: Option<String>,
}

/// UserSettings represents user settings
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UserSettings {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diff_view_style: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hide_activity: Option<bool>,
    /// Privacy
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hide_email: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
}

/// UserSettingsOptions represents options to change user settings
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UserSettingsOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diff_view_style: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hide_activity: Option<bool>,
    /// Privacy
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hide_email: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum UserTypeString {
    #[default]
    User,
    Organization,
    Bot,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum VisibilityString {
    #[default]
    #[serde(rename = "public")]
    Public,
    #[serde(rename = "limited")]
    Limited,
    #[serde(rename = "private")]
    Private,
}

/// WatchInfo represents an API watch status of one repository
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WatchInfo {
    /// The timestamp when the watch status was created
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// Whether notifications for the repository are ignored
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ignored: Option<bool>,
    /// The reason for the current watch status
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<serde_json::Value>,
    /// The URL of the repository being watched
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository_url: Option<String>,
    /// Whether the repository is being watched for notifications
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subscribed: Option<bool>,
    /// The URL for managing the watch status
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// WikiCommit page commit/revision
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WikiCommit {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<CommitUser>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commiter: Option<CommitUser>,
    /// The commit message
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// The commit SHA hash
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha: Option<String>,
}

/// WikiCommitList commit/revision list
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WikiCommitList {
    /// The list of wiki commits
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commits: Option<Vec<WikiCommit>>,
    /// The total count of commits
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count: Option<i64>,
}

/// WikiPage a wiki page
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WikiPage {
    /// The number of commits that modified this page
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit_count: Option<i64>,
    /// Page content, base64 encoded
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_base64: Option<String>,
    /// The footer content for the wiki page
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub footer: Option<String>,
    /// The HTML URL to view the wiki page
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_commit: Option<WikiCommit>,
    /// The sidebar content for the wiki page
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sidebar: Option<String>,
    /// The sub URL path for the wiki page
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sub_url: Option<String>,
    /// The title of the wiki page
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// WikiPageMetaData wiki page meta information
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WikiPageMetaData {
    /// The HTML URL to view the wiki page
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_commit: Option<WikiCommit>,
    /// The sub URL path for the wiki page
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sub_url: Option<String>,
    /// The title of the wiki page
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum ActivityOpType {
    #[default]
    #[serde(rename = "create_repo")]
    CreateRepo,
    #[serde(rename = "rename_repo")]
    RenameRepo,
    #[serde(rename = "star_repo")]
    StarRepo,
    #[serde(rename = "watch_repo")]
    WatchRepo,
    #[serde(rename = "commit_repo")]
    CommitRepo,
    #[serde(rename = "create_issue")]
    CreateIssue,
    #[serde(rename = "create_pull_request")]
    CreatePullRequest,
    #[serde(rename = "transfer_repo")]
    TransferRepo,
    #[serde(rename = "push_tag")]
    PushTag,
    #[serde(rename = "comment_issue")]
    CommentIssue,
    #[serde(rename = "merge_pull_request")]
    MergePullRequest,
    #[serde(rename = "close_issue")]
    CloseIssue,
    #[serde(rename = "reopen_issue")]
    ReopenIssue,
    #[serde(rename = "close_pull_request")]
    ClosePullRequest,
    #[serde(rename = "reopen_pull_request")]
    ReopenPullRequest,
    #[serde(rename = "delete_tag")]
    DeleteTag,
    #[serde(rename = "delete_branch")]
    DeleteBranch,
    #[serde(rename = "mirror_sync_push")]
    MirrorSyncPush,
    #[serde(rename = "mirror_sync_create")]
    MirrorSyncCreate,
    #[serde(rename = "mirror_sync_delete")]
    MirrorSyncDelete,
    #[serde(rename = "approve_pull_request")]
    ApprovePullRequest,
    #[serde(rename = "reject_pull_request")]
    RejectPullRequest,
    #[serde(rename = "comment_pull")]
    CommentPull,
    #[serde(rename = "publish_release")]
    PublishRelease,
    #[serde(rename = "pull_review_dismissed")]
    PullReviewDismissed,
    #[serde(rename = "pull_request_ready_for_review")]
    PullRequestReadyForReview,
    #[serde(rename = "auto_merge_pull_request")]
    AutoMergePullRequest,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum ChangeFileOperationOperation {
    #[default]
    #[serde(rename = "create")]
    Create,
    #[serde(rename = "update")]
    Update,
    #[serde(rename = "upload")]
    Upload,
    #[serde(rename = "rename")]
    Rename,
    #[serde(rename = "delete")]
    Delete,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum CreateHookOptionType {
    #[default]
    #[serde(rename = "dingtalk")]
    Dingtalk,
    #[serde(rename = "discord")]
    Discord,
    #[serde(rename = "gitea")]
    Gitea,
    #[serde(rename = "gogs")]
    Gogs,
    #[serde(rename = "msteams")]
    Msteams,
    #[serde(rename = "slack")]
    Slack,
    #[serde(rename = "telegram")]
    Telegram,
    #[serde(rename = "feishu")]
    Feishu,
    #[serde(rename = "wechatwork")]
    Wechatwork,
    #[serde(rename = "packagist")]
    Packagist,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum CreateRepoOptionTrustModel {
    #[default]
    #[serde(rename = "default")]
    Default,
    #[serde(rename = "collaborator")]
    Collaborator,
    #[serde(rename = "committer")]
    Committer,
    #[serde(rename = "collaboratorcommitter")]
    Collaboratorcommitter,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum DeployKeyKeyType {
    #[default]
    #[serde(rename = "ssh")]
    Ssh,
    #[serde(rename = "token")]
    Token,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum IssueFormFieldType {
    #[default]
    #[serde(rename = "markdown")]
    Markdown,
    #[serde(rename = "textarea")]
    Textarea,
    #[serde(rename = "input")]
    Input,
    #[serde(rename = "dropdown")]
    Dropdown,
    #[serde(rename = "checkboxes")]
    Checkboxes,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum IssueFormFieldVisible {
    #[default]
    #[serde(rename = "form")]
    Form,
    #[serde(rename = "content")]
    Content,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum MergePullRequestOptionDo {
    #[default]
    #[serde(rename = "merge")]
    Merge,
    #[serde(rename = "rebase")]
    Rebase,
    #[serde(rename = "rebase-merge")]
    RebaseMerge,
    #[serde(rename = "squash")]
    Squash,
    #[serde(rename = "fast-forward-only")]
    FastForwardOnly,
    #[serde(rename = "manually-merged")]
    ManuallyMerged,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum MigrateRepoOptionsService {
    #[default]
    #[serde(rename = "git")]
    Git,
    #[serde(rename = "github")]
    Github,
    #[serde(rename = "gitea")]
    Gitea,
    #[serde(rename = "gitlab")]
    Gitlab,
    #[serde(rename = "gogs")]
    Gogs,
    #[serde(rename = "onedev")]
    Onedev,
    #[serde(rename = "gitbucket")]
    Gitbucket,
    #[serde(rename = "codebase")]
    Codebase,
    #[serde(rename = "codecommit")]
    Codecommit,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum NotificationSubjectState {
    #[default]
    #[serde(rename = "open")]
    Open,
    #[serde(rename = "closed")]
    Closed,
    #[serde(rename = "merged")]
    Merged,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum NotificationSubjectType {
    #[default]
    Issue,
    Pull,
    Commit,
    Repository,
}
