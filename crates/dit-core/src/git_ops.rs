//! Git operations wrapper for DIT repositories.
//!
//! All git interactions go through `git2` (libgit2) — no shelling out.

use std::path::Path;

use anyhow::{bail, Context, Result};
use git2::{
    BranchType, IndexAddOption, Oid, Repository, Signature, StatusOptions,
};
use serde::{Deserialize, Serialize};

use crate::types::{
    ChangeType, DitBranch, DitCommitMeta, DitPaths, DitStatus, DitStatusChange,
};

/// DIT-tracked path prefixes that are staged on commit.
const DIT_TRACKED: &[&str] = &[
    DitPaths::PROJECT_FILE,      // dit.json
    DitPaths::PAGES_DIR,         // dit.pages/
    DitPaths::NODES_DIR,         // dit.nodes/
    DitPaths::ASSETS_DIR,        // dit.assets/
    DitPaths::FIG_DIR,           // dit.fig/
    DitPaths::STYLES_FILE,       // dit.styles.json
    DitPaths::COMPONENTS_FILE,   // dit.components.json
];

// ── Result types ─────────────────────────────────────────────────────

/// Available `.fig` snapshot files relevant to a merge operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeFigSnapshots {
    /// `.fig` snapshot path for the current (ours) branch tip, if it exists on disk.
    pub ours: Option<String>,
    /// Commit hash of the "ours" snapshot.
    pub ours_commit: Option<String>,
    /// `.fig` snapshot path for the incoming (theirs) branch tip, if it exists on disk.
    pub theirs: Option<String>,
    /// Commit hash of the "theirs" snapshot.
    pub theirs_commit: Option<String>,
}

/// Outcome of a merge operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeResult {
    /// True if the merge completed without conflicts.
    pub success: bool,
    /// Paths that have conflicts (empty if `success` is true).
    pub conflicts: Vec<String>,
    /// Resulting commit hash (None if conflicts remain).
    pub commit_hash: Option<String>,
    /// Info about available `.fig` snapshot files from each branch.
    pub fig_snapshots: MergeFigSnapshots,
    /// True if this was a fast-forward merge.
    pub fast_forward: bool,
}

// ── Repository init / detection ──────────────────────────────────────

/// Initialise a new git repository at `path` with an initial commit on "main".
pub fn init_repository(path: &Path) -> Result<()> {
    let repo = Repository::init(path)
        .with_context(|| format!("failed to init git repo at {}", path.display()))?;

    // Set HEAD to point to "main" (git2 defaults to "master").
    repo.set_head("refs/heads/main")
        .context("failed to set HEAD to refs/heads/main")?;

    // Create .dit directory.
    let dit_dir = path.join(DitPaths::DIT_DIR);
    std::fs::create_dir_all(&dit_dir)
        .with_context(|| format!("failed to create {}", dit_dir.display()))?;

    // Write a .gitignore that hides DIT internals.
    let gitignore = path.join(".gitignore");
    if !gitignore.exists() {
        std::fs::write(&gitignore, ".dit/\n")
            .context("failed to write .gitignore")?;
    }

    // Create an initial empty commit so HEAD is valid.
    let sig = default_signature(&repo)?;
    let mut index = repo.index().context("failed to open index")?;
    index
        .add_path(Path::new(".gitignore"))
        .context("failed to stage .gitignore")?;
    index.write().context("failed to write index")?;
    let tree_oid = index.write_tree().context("failed to write tree")?;
    let tree = repo.find_tree(tree_oid)?;
    repo.commit(Some("HEAD"), &sig, &sig, "Initialize DIT repository", &tree, &[])
        .context("failed to create initial commit")?;

    Ok(())
}

/// Check whether `path` is inside a git repository.
pub fn is_git_repo(path: &Path) -> bool {
    Repository::discover(path).is_ok()
}

/// Check whether `path` is a DIT repository (git repo + `.dit/` directory).
pub fn is_dit_repo(path: &Path) -> bool {
    is_git_repo(path) && path.join(DitPaths::DIT_DIR).is_dir()
}

// ── Status ───────────────────────────────────────────────────────────

/// Return the current status of the DIT repository.
pub fn get_status(repo_root: &Path) -> Result<DitStatus> {
    let repo = open(repo_root)?;

    let branch = current_branch_name(&repo);
    let head = repo.head().ok().and_then(|h| h.target()).map(|o| o.to_string());

    let mut opts = StatusOptions::new();
    opts.include_untracked(true);
    opts.recurse_untracked_dirs(true);
    let statuses = repo.statuses(Some(&mut opts)).context("failed to get status")?;

    let mut changes = Vec::new();
    for entry in statuses.iter() {
        let path_str = entry.path().unwrap_or("").to_string();
        if !is_dit_path(&path_str) {
            continue;
        }
        let st = entry.status();
        let change_type = if st.is_wt_new() || st.is_index_new() {
            ChangeType::Added
        } else if st.is_wt_deleted() || st.is_index_deleted() {
            ChangeType::Deleted
        } else {
            ChangeType::Modified
        };
        changes.push(DitStatusChange {
            path: path_str,
            change_type,
        });
    }

    let is_dirty = !changes.is_empty();
    Ok(DitStatus {
        branch,
        head,
        changes,
        is_dirty,
    })
}

// ── Commit ───────────────────────────────────────────────────────────

/// Stage all DIT-tracked files and create a commit. Returns the commit hash.
pub fn commit_all(repo_root: &Path, message: &str) -> Result<String> {
    let repo = open(repo_root)?;
    let mut index = repo.index().context("failed to open index")?;

    // Add all DIT-tracked paths (handles both new and modified files).
    // We use add_all with the tracked prefixes as pathspecs.
    let specs: Vec<&str> = DIT_TRACKED.to_vec();
    index
        .add_all(specs.iter(), IndexAddOption::DEFAULT, None)
        .context("failed to stage DIT files")?;

    // Also remove any deleted files from the index.
    index
        .update_all(specs.iter(), None)
        .context("failed to update index for deletions")?;

    index.write().context("failed to write index")?;

    let tree_oid = index.write_tree().context("failed to write tree")?;
    let tree = repo.find_tree(tree_oid)?;
    let sig = default_signature(&repo)?;

    let commit_oid = if let Ok(head) = repo.head() {
        let parent = head.peel_to_commit().context("HEAD is not a commit")?;
        repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[&parent])?
    } else {
        // First commit (no parent).
        repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[])?
    };

    Ok(commit_oid.to_string())
}

// ── Log ──────────────────────────────────────────────────────────────

/// Return the commit history, most recent first. `max_count` of 0 means all.
pub fn get_log(repo_root: &Path, max_count: usize) -> Result<Vec<DitCommitMeta>> {
    let repo = open(repo_root)?;
    let mut revwalk = repo.revwalk().context("failed to create revwalk")?;
    revwalk.push_head().context("failed to push HEAD to revwalk")?;
    revwalk.set_sorting(git2::Sort::TIME)?;

    let mut entries = Vec::new();
    for (i, oid_result) in revwalk.enumerate() {
        if max_count > 0 && i >= max_count {
            break;
        }
        let oid = oid_result.context("revwalk error")?;
        let commit = repo.find_commit(oid)?;
        entries.push(commit_to_meta(&commit));
    }

    Ok(entries)
}

// ── Branches ─────────────────────────────────────────────────────────

/// List all local branches.
pub fn list_branches(repo_root: &Path) -> Result<Vec<DitBranch>> {
    let repo = open(repo_root)?;
    let current = current_branch_name(&repo);
    let mut result = Vec::new();

    for branch_result in repo.branches(Some(BranchType::Local))? {
        let (branch, _) = branch_result?;
        let name = branch.name()?.unwrap_or("").to_string();
        let head = branch
            .get()
            .target()
            .map(|o| o.to_string())
            .unwrap_or_default();
        result.push(DitBranch {
            is_current: name == current,
            name,
            head,
        });
    }

    Ok(result)
}

/// Create a new branch at the current HEAD.
pub fn create_branch(repo_root: &Path, name: &str) -> Result<()> {
    let repo = open(repo_root)?;
    let head = repo
        .head()
        .context("HEAD not found — is the repo empty?")?
        .peel_to_commit()
        .context("HEAD does not point to a commit")?;
    repo.branch(name, &head, false)
        .with_context(|| format!("failed to create branch '{name}'"))?;
    Ok(())
}

/// Check out a branch or commit reference.
pub fn checkout(repo_root: &Path, ref_name: &str) -> Result<()> {
    let repo = open(repo_root)?;

    // Try as branch first.
    if let Ok(branch) = repo.find_branch(ref_name, BranchType::Local) {
        let refname = branch
            .get()
            .name()
            .context("branch ref has no name")?
            .to_string();
        let obj = branch.get().peel(git2::ObjectType::Commit)?;
        repo.checkout_tree(&obj, None)?;
        repo.set_head(&refname)?;
        return Ok(());
    }

    // Try as a commit SHA.
    if let Ok(oid) = Oid::from_str(ref_name) {
        let commit = repo.find_commit(oid)?;
        let obj = commit.as_object();
        repo.checkout_tree(obj, None)?;
        repo.set_head_detached(oid)?;
        return Ok(());
    }

    bail!("reference '{ref_name}' not found as branch or commit");
}

// ── Merge ────────────────────────────────────────────────────────────

/// Check if a `.fig` snapshot exists for a given commit hash.
/// Returns the path string if the file exists on disk.
/// Checks git-tracked `dit.fig/` first, then falls back to `.dit/fig_snapshots/`.
fn fig_snapshot_path(repo_root: &Path, commit_hash: &str) -> Option<String> {
    // Local cache (.dit/fig_snapshots/<hash>.fig)
    let cached = repo_root
        .join(DitPaths::FIG_SNAPSHOTS_DIR)
        .join(format!("{}.fig", commit_hash));
    if cached.exists() {
        return Some(cached.to_string_lossy().to_string());
    }
    // Git-tracked latest (dit.fig/latest.fig)
    let latest = repo_root
        .join(DitPaths::FIG_DIR)
        .join("latest.fig");
    if latest.exists() {
        return Some(latest.to_string_lossy().to_string());
    }
    None
}

/// Build `MergeFigSnapshots` for the two branch tips involved in a merge.
fn collect_fig_snapshots(
    repo_root: &Path,
    ours_hash: Option<&str>,
    theirs_hash: Option<&str>,
) -> MergeFigSnapshots {
    MergeFigSnapshots {
        ours: ours_hash.and_then(|h| fig_snapshot_path(repo_root, h)),
        ours_commit: ours_hash.map(|h| h.to_string()),
        theirs: theirs_hash.and_then(|h| fig_snapshot_path(repo_root, h)),
        theirs_commit: theirs_hash.map(|h| h.to_string()),
    }
}

/// Merge `branch` into the current HEAD.
pub fn merge(repo_root: &Path, branch: &str) -> Result<MergeResult> {
    let repo = open(repo_root)?;

    let their_branch = repo
        .find_branch(branch, BranchType::Local)
        .with_context(|| format!("branch '{branch}' not found"))?;
    let their_commit = their_branch
        .get()
        .peel_to_commit()
        .context("branch does not point to a commit")?;
    let their_annotated = repo.find_annotated_commit(their_commit.id())?;

    // Capture branch tip hashes before the merge for .fig snapshot lookup.
    let ours_hash = repo
        .head()
        .ok()
        .and_then(|h| h.target())
        .map(|o| o.to_string());
    let theirs_hash = their_commit.id().to_string();

    let (analysis, _) = repo.merge_analysis(&[&their_annotated])?;

    // ── Case 1: Already up-to-date ───────────────────────────────────
    if analysis.is_up_to_date() {
        return Ok(MergeResult {
            success: true,
            conflicts: vec![],
            commit_hash: repo.head().ok().and_then(|h| h.target()).map(|o| o.to_string()),
            fig_snapshots: MergeFigSnapshots::default(),
            fast_forward: false,
        });
    }

    // ── Case 2: Fast-forward merge ───────────────────────────────────
    if analysis.is_fast_forward() {
        let refname = format!("refs/heads/{}", current_branch_name(&repo));
        repo.reference(&refname, their_commit.id(), true, "fast-forward merge")?;
        repo.set_head(&refname)?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;

        let snapshots = collect_fig_snapshots(
            repo_root,
            ours_hash.as_deref(),
            Some(&theirs_hash),
        );
        return Ok(MergeResult {
            success: true,
            conflicts: vec![],
            commit_hash: Some(their_commit.id().to_string()),
            fig_snapshots: snapshots,
            fast_forward: true,
        });
    }

    // ── Case 3: Regular merge (potential conflicts) ──────────────────
    repo.merge(&[&their_annotated], None, None)?;

    let snapshots = collect_fig_snapshots(
        repo_root,
        ours_hash.as_deref(),
        Some(&theirs_hash),
    );

    let index = repo.index().context("failed to get index after merge")?;
    if index.has_conflicts() {
        let conflicts: Vec<String> = index
            .conflicts()?
            .filter_map(|c| c.ok())
            .filter_map(|c| {
                c.our
                    .as_ref()
                    .or(c.their.as_ref())
                    .and_then(|e| String::from_utf8(e.path.clone()).ok())
            })
            .collect();
        return Ok(MergeResult {
            success: false,
            conflicts,
            commit_hash: None,
            fig_snapshots: snapshots,
            fast_forward: false,
        });
    }

    // No conflicts — create merge commit.
    let mut index = repo.index()?;
    let tree_oid = index.write_tree()?;
    let tree = repo.find_tree(tree_oid)?;
    let sig = default_signature(&repo)?;
    let our_commit = repo.head()?.peel_to_commit()?;
    let msg = format!("Merge branch '{branch}'");
    let oid = repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        &msg,
        &tree,
        &[&our_commit, &their_commit],
    )?;
    repo.cleanup_state()?;

    Ok(MergeResult {
        success: true,
        conflicts: vec![],
        commit_hash: Some(oid.to_string()),
        fig_snapshots: snapshots,
        fast_forward: false,
    })
}

// ── Push / Pull ──────────────────────────────────────────────────────

/// Push `branch` to `remote`.
///
/// Shells out to `git push` so that the user's configured credential helpers
/// (SSH agent, macOS Keychain, credential.helper, etc.) work automatically.
pub fn push(repo_root: &Path, remote: &str, branch: &str) -> Result<()> {
    let output = std::process::Command::new("git")
        .args(["push", remote, branch])
        .current_dir(repo_root)
        .output()
        .context("failed to run `git push`")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("failed to push {branch} to {remote}: {stderr}");
    }
    Ok(())
}

/// Pull `branch` from `remote` (fetch + fast-forward).
///
/// Shells out to `git pull` so that the user's configured credential helpers
/// (SSH agent, macOS Keychain, credential.helper, etc.) work automatically.
pub fn pull(repo_root: &Path, remote: &str, branch: &str) -> Result<()> {
    let output = std::process::Command::new("git")
        .args(["pull", remote, branch])
        .current_dir(repo_root)
        .output()
        .context("failed to run `git pull`")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("failed to pull {branch} from {remote}: {stderr}");
    }
    Ok(())
}

// ── Helpers ──────────────────────────────────────────────────────────

fn open(repo_root: &Path) -> Result<Repository> {
    Repository::open(repo_root)
        .with_context(|| format!("failed to open git repo at {}", repo_root.display()))
}

fn default_signature(repo: &Repository) -> Result<Signature<'_>> {
    repo.signature().or_else(|_| {
        Signature::now("DIT", "dit@localhost").context("failed to create fallback signature")
    })
}

fn current_branch_name(repo: &Repository) -> String {
    repo.head()
        .ok()
        .and_then(|h| h.shorthand().map(String::from))
        .unwrap_or_else(|| "HEAD".into())
}

/// Check whether a path belongs to DIT-tracked content.
fn is_dit_path(path: &str) -> bool {
    DIT_TRACKED.iter().any(|prefix| {
        path == *prefix || path.starts_with(&format!("{prefix}/"))
    })
}

fn commit_to_meta(commit: &git2::Commit) -> DitCommitMeta {
    let ts = commit.time();
    let secs = ts.seconds();
    let timestamp = chrono_free_iso(secs);

    let message = commit.message().unwrap_or("").to_string();
    let author = commit
        .author()
        .name()
        .unwrap_or("Unknown")
        .to_string();

    DitCommitMeta {
        hash: commit.id().to_string(),
        message,
        author,
        timestamp,
        figma_version: String::new(),
        page_count: 0,
        node_count: 0,
        asset_count: 0,
    }
}

/// Produce a rough ISO-8601 timestamp from unix seconds without pulling in chrono.
fn chrono_free_iso(secs: i64) -> String {
    // We only need a human-readable string; the git commit is the source of truth.
    let s = secs.unsigned_abs();
    let days = s / 86400;
    let rem = s % 86400;
    let h = rem / 3600;
    let m = (rem % 3600) / 60;
    let sec = rem % 60;

    // Approximate year/month/day from days since epoch (1970-01-01).
    let (y, mo, d) = days_to_ymd(days);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{sec:02}Z")
}

fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    let mut year = 1970u64;
    loop {
        let ydays = if is_leap(year) { 366 } else { 365 };
        if days < ydays {
            break;
        }
        days -= ydays;
        year += 1;
    }
    let leap = is_leap(year);
    let mdays = [
        31,
        if leap { 29 } else { 28 },
        31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
    ];
    let mut month = 1u64;
    for &md in &mdays {
        if days < md {
            break;
        }
        days -= md;
        month += 1;
    }
    (year, month, days + 1)
}

fn is_leap(y: u64) -> bool {
    y.is_multiple_of(4) && (!y.is_multiple_of(100) || y.is_multiple_of(400))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> (TempDir, std::path::PathBuf) {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().to_path_buf();
        (tmp, path)
    }

    #[test]
    fn init_creates_dit_repo() {
        let (_tmp, path) = setup();
        init_repository(&path).unwrap();
        assert!(is_git_repo(&path));
        assert!(is_dit_repo(&path));
        assert!(path.join(".gitignore").exists());
        assert!(path.join(DitPaths::DIT_DIR).is_dir());
    }

    #[test]
    fn non_repo_is_not_dit() {
        let (_tmp, path) = setup();
        assert!(!is_git_repo(&path));
        assert!(!is_dit_repo(&path));
    }

    #[test]
    fn status_on_clean_repo() {
        let (_tmp, path) = setup();
        init_repository(&path).unwrap();
        let status = get_status(&path).unwrap();
        assert_eq!(status.branch, "main");
        assert!(!status.is_dirty);
    }

    #[test]
    fn commit_and_log() {
        let (_tmp, path) = setup();
        init_repository(&path).unwrap();

        // Create a DIT file.
        std::fs::write(path.join(DitPaths::PROJECT_FILE), "{}").unwrap();

        let hash = commit_all(&path, "Add project file").unwrap();
        assert!(!hash.is_empty());

        let log = get_log(&path, 10).unwrap();
        // 2 commits: init + our commit
        assert!(log.len() >= 2);
    }

    #[test]
    fn branch_create_and_list() {
        let (_tmp, path) = setup();
        init_repository(&path).unwrap();

        create_branch(&path, "feature/test").unwrap();
        let branches = list_branches(&path).unwrap();
        let names: Vec<&str> = branches.iter().map(|b| b.name.as_str()).collect();
        assert!(names.contains(&"main"));
        assert!(names.contains(&"feature/test"));
    }

    #[test]
    fn checkout_branch() {
        let (_tmp, path) = setup();
        init_repository(&path).unwrap();
        create_branch(&path, "dev").unwrap();
        checkout(&path, "dev").unwrap();
        let status = get_status(&path).unwrap();
        assert_eq!(status.branch, "dev");
    }

    #[test]
    fn fast_forward_merge() {
        let (_tmp, path) = setup();
        init_repository(&path).unwrap();

        // Create a branch and add a commit on it.
        create_branch(&path, "feature").unwrap();
        checkout(&path, "feature").unwrap();
        std::fs::write(path.join(DitPaths::PROJECT_FILE), r#"{"v":1}"#).unwrap();
        commit_all(&path, "feature commit").unwrap();

        // Switch back to main and merge.
        checkout(&path, "main").unwrap();
        let result = merge(&path, "feature").unwrap();
        assert!(result.success);
        assert!(result.conflicts.is_empty());
    }

    #[test]
    fn create_branch_then_commit_lands_on_new_branch() {
        let (_tmp, path) = setup();
        init_repository(&path).unwrap();

        // Create and checkout a new branch.
        create_branch(&path, "feature").unwrap();
        checkout(&path, "feature").unwrap();

        // Commit on the new branch.
        std::fs::write(path.join(DitPaths::PROJECT_FILE), r#"{"v":1}"#).unwrap();
        commit_all(&path, "feature commit").unwrap();

        // Verify we're on "feature" and the commit is there.
        let status = get_status(&path).unwrap();
        assert_eq!(status.branch, "feature");

        let log = get_log(&path, 5).unwrap();
        assert_eq!(log[0].message, "feature commit");

        // Verify main didn't get the commit.
        checkout(&path, "main").unwrap();
        let main_log = get_log(&path, 5).unwrap();
        assert!(main_log.iter().all(|c| c.message != "feature commit"));
    }

    #[test]
    fn is_dit_path_works() {
        assert!(is_dit_path("dit.json"));
        assert!(is_dit_path("dit.pages/0_1.json"));
        assert!(is_dit_path("dit.assets/sha256_abc"));
        assert!(is_dit_path("dit.fig/abc123.fig"));
        assert!(is_dit_path("dit.styles.json"));
        assert!(!is_dit_path("readme.md"));
        assert!(!is_dit_path(".dit/config.json"));
    }
}
