use serde::{Deserialize, Serialize};

use super::enums::ChangeType;

// ─── DitConfig ───────────────────────────────────────────────────────────────

/// Persistent configuration stored in `.dit/config.json`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DitConfig {
    /// Figma file key.
    pub file_key: String,
    /// Human-readable project name.
    pub name: String,
    /// Figma personal access token (stored locally, never committed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub figma_token: Option<String>,
    /// Schema version for forward-compatibility.
    pub schema_version: u32,
    /// SSH private key path for remote operations (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub ssh_key_path: Option<String>,
}

// ─── DitCommitMeta ───────────────────────────────────────────────────────────

/// Extra metadata DIT attaches to each git commit (stored in commit message or notes).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DitCommitMeta {
    /// Git commit hash.
    pub hash: String,
    /// Commit message.
    pub message: String,
    /// Author name.
    pub author: String,
    /// ISO 8601 timestamp.
    pub timestamp: String,
    /// Figma file version at the time of this snapshot.
    pub figma_version: String,
    /// Number of pages in the snapshot.
    pub page_count: usize,
    /// Number of top-level nodes across all pages.
    pub node_count: usize,
    /// Number of assets (images) referenced.
    pub asset_count: usize,
}

// ─── DitBranch ───────────────────────────────────────────────────────────────

/// Information about a DIT branch (maps 1-1 with a git branch).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DitBranch {
    /// Branch name.
    pub name: String,
    /// Latest commit SHA on this branch.
    pub head: String,
    /// Whether this is the currently checked-out branch.
    pub is_current: bool,
}

// ─── DitStatus ───────────────────────────────────────────────────────────────

/// The result of `dit status`: working-tree state relative to last commit.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DitStatus {
    /// Current branch name.
    pub branch: String,
    /// HEAD commit SHA (None if no commits yet).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head: Option<String>,
    /// Changed files since last commit.
    pub changes: Vec<DitStatusChange>,
    /// Whether there are uncommitted changes.
    pub is_dirty: bool,
}

/// A single changed file in the working tree.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DitStatusChange {
    /// Relative path of the changed file.
    pub path: String,
    /// Kind of change.
    pub change_type: ChangeType,
}

// ─── DitLock ─────────────────────────────────────────────────────────────────

/// A lightweight lock file (`.dit/lock`) to prevent concurrent mutations.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DitLock {
    /// PID of the process that holds the lock.
    pub pid: u32,
    /// ISO 8601 timestamp when the lock was acquired.
    pub acquired_at: String,
    /// Human-readable description of the operation.
    pub operation: String,
}
