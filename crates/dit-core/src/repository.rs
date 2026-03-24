//! High-level DIT repository operations.
//!
//! `DitRepository` is the main API that CLI and GUI use. It orchestrates
//! canonical storage, asset management, git operations, and file locking.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use crate::canonical;
use crate::figma::{fig_to_snapshot, download_fig_file, FigmaAuth};
use crate::git_ops::{self, MergeResult};
use crate::lock::LockGuard;
use crate::types::{
    DitBranch, DitCommitMeta, DitConfig, DitPaths, DitSnapshot, DitStatus,
};

/// Result of a clone operation.
#[derive(Debug)]
pub enum CloneResult {
    /// The cloned repo is already a DIT repository — ready to use.
    DitRepo(DitRepository),
    /// The cloned repo is a plain git repo — needs DIT initialization.
    NeedsInit {
        /// Path to the cloned repository.
        path: PathBuf,
    },
}

/// Options for a commit operation.
#[derive(Debug, Clone, Default)]
pub struct CommitOptions {
    /// If true, store a binary export alongside the canonical JSON (future use).
    pub store_binary_export: bool,
    /// Path to a .fig file to store alongside the commit.
    /// When set, the .fig file is copied to `.dit/fig_snapshots/<commit_hash>.fig`
    /// after the git commit completes.
    pub fig_file_path: Option<PathBuf>,
}

/// Result of a restore operation.
#[derive(Debug)]
pub struct RestoreResult {
    /// The restored DIT snapshot (canonical JSON data).
    pub snapshot: DitSnapshot,
    /// Path to the .fig file for this commit, if available locally.
    /// User can open this file directly in Figma for lossless restore.
    pub fig_file_path: Option<PathBuf>,
}

/// The main orchestrator for a DIT repository.
///
/// Wraps a directory that is both a git repo and a DIT repo (has `.dit/`).
#[derive(Debug)]
pub struct DitRepository {
    root: PathBuf,
}

impl DitRepository {
    // ── Construction ─────────────────────────────────────────────────────

    /// Initialize a new DIT repository in `dir`.
    ///
    /// Creates the git repo, `.dit/` structure, writes config, and makes an
    /// initial commit.
    pub fn init(dir: &Path, config: DitConfig) -> Result<Self> {
        // Create directory if it doesn't exist
        std::fs::create_dir_all(dir)
            .with_context(|| format!("failed to create directory: {}", dir.display()))?;

        let root = dir
            .canonicalize()
            .unwrap_or_else(|_| dir.to_path_buf());

        // Initialize git repo if it doesn't already exist.
        // If it's already a git repo (e.g. from `dit clone`), just set up .dit/.
        if !git_ops::is_git_repo(&root) {
            git_ops::init_repository(&root)
                .context("failed to initialize git repository")?;
        } else {
            // Ensure .dit/ exists and .gitignore has the right entries.
            let dit_dir = root.join(DitPaths::DIT_DIR);
            std::fs::create_dir_all(&dit_dir)
                .context("failed to create .dit directory")?;
            let gitignore = root.join(".gitignore");
            let existing = std::fs::read_to_string(&gitignore).unwrap_or_default();
            let mut additions = String::new();
            if !existing.contains(".dit/") {
                additions.push_str(".dit/\n");
            }
            if !existing.contains(".env") {
                additions.push_str(".env\n");
            }
            if !additions.is_empty() {
                std::fs::write(&gitignore, format!("{existing}{additions}"))
                    .context("failed to update .gitignore")?;
            }
        }

        // Write DIT config
        let config_path = root.join(DitPaths::CONFIG_FILE);
        let config_json =
            serde_json::to_string_pretty(&config).context("failed to serialize config")?;
        std::fs::write(&config_path, &config_json)
            .context("failed to write config file")?;

        // Create tracked directories
        std::fs::create_dir_all(root.join(DitPaths::PAGES_DIR))
            .context("failed to create pages directory")?;
        std::fs::create_dir_all(root.join(DitPaths::ASSETS_DIR))
            .context("failed to create assets directory")?;
        std::fs::create_dir_all(root.join(DitPaths::FIG_DIR))
            .context("failed to create fig directory")?;
        std::fs::create_dir_all(root.join(DitPaths::PREVIEWS_DIR))
            .context("failed to create previews directory")?;

        Ok(Self { root })
    }

    /// Clone a git repository and detect whether it is a DIT repo.
    ///
    /// - If the cloned repo has `.dit/`, returns `CloneResult::DitRepo`.
    /// - Otherwise returns `CloneResult::NeedsInit` so the caller can
    ///   run the interactive init flow.
    /// If `ssh_key_path` is provided, it is passed to `git clone` via
    /// `GIT_SSH_COMMAND`.
    pub fn clone(url: &str, dir: &Path, ssh_key_path: Option<&str>) -> Result<CloneResult> {
        let repo_path = git_ops::clone_repo(url, dir, ssh_key_path)
            .context("failed to clone repository")?;

        if git_ops::is_dit_repo(&repo_path) {
            let repo = Self { root: repo_path };
            Ok(CloneResult::DitRepo(repo))
        } else {
            Ok(CloneResult::NeedsInit { path: repo_path })
        }
    }

    /// Open an existing DIT repository at `dir`.
    ///
    /// If `.dit/config.json` is missing (e.g. after cloning — `.dit/` is
    /// git-ignored), it is bootstrapped from `dit.json` (the committed
    /// project file).
    pub fn open(dir: &Path) -> Result<Self> {
        let root = dir
            .canonicalize()
            .unwrap_or_else(|_| dir.to_path_buf());

        if !git_ops::is_dit_repo(&root) {
            bail!(
                "'{}' is not a DIT repository",
                root.display()
            );
        }

        // Bootstrap .dit/config.json from dit.json if missing (post-clone)
        let config_path = root.join(DitPaths::CONFIG_FILE);
        if !config_path.exists() {
            let project_path = root.join(DitPaths::PROJECT_FILE);
            if project_path.exists() {
                // Read file_key and name from the committed project file
                let project_json = std::fs::read_to_string(&project_path)
                    .context("failed to read dit.json")?;
                let project: crate::types::DitProject =
                    crate::canonical::deserialize(&project_json)
                        .context("failed to parse dit.json")?;

                let config = DitConfig {
                    file_key: project.file_key,
                    name: project.name,
                    figma_token: None,
                    schema_version: project.schema_version,
                    ssh_key_path: None,
                };

                // Create .dit/ and write config
                let dit_dir = root.join(DitPaths::DIT_DIR);
                std::fs::create_dir_all(&dit_dir)
                    .context("failed to create .dit directory")?;
                let config_json = serde_json::to_string_pretty(&config)
                    .context("failed to serialize config")?;
                std::fs::write(&config_path, &config_json)
                    .context("failed to write config file")?;
            }
        }

        Ok(Self { root })
    }

    /// Return the repository root path.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Read the repository configuration.
    pub fn config(&self) -> Result<DitConfig> {
        let config_path = self.root.join(DitPaths::CONFIG_FILE);
        let json = std::fs::read_to_string(&config_path)
            .context("failed to read config file")?;
        serde_json::from_str(&json).context("failed to parse config file")
    }

    // ── Commit ───────────────────────────────────────────────────────────

    /// Full commit flow:
    ///
    /// 1. Acquire lock
    /// 2. Write canonical JSON snapshot to disk
    /// 3. If a .fig file is provided, pre-stage it to `dit.fig/latest.fig`
    /// 4. Git commit all DIT-tracked files (includes the .fig)
    /// 5. Store named copies: `dit.fig/<hash>.fig` + `.dit/fig_snapshots/<hash>.fig`
    /// 6. Release lock
    ///
    /// Returns the commit hash.
    pub fn commit(
        &self,
        snapshot: &DitSnapshot,
        message: &str,
        options: &CommitOptions,
    ) -> Result<String> {
        let _lock = LockGuard::acquire(&self.root, "commit")
            .context("failed to acquire commit lock")?;

        // Write canonical snapshot files
        canonical::write_snapshot(&self.root, snapshot)
            .context("failed to write snapshot")?;

        // Pre-stage .fig if provided so it's included in the commit
        if let Some(fig_path) = &options.fig_file_path {
            self.pre_stage_fig(fig_path)
                .context("failed to pre-stage .fig file")?;
        }

        // Git commit
        let hash = git_ops::commit_all(&self.root, message)
            .context("failed to git commit")?;

        // Also store as dit.fig/<hash>.fig + .dit/fig_snapshots/<hash>.fig
        if let Some(fig_path) = &options.fig_file_path {
            self.store_fig_snapshot(&hash, fig_path)
                .context("failed to store .fig snapshot")?;
        }

        Ok(hash)
    }

    /// Commit from a .fig file.
    ///
    /// 1. Download the .fig file from Figma (or use a local path)
    /// 2. Convert .fig → DitSnapshot via fig2json
    /// 3. Write canonical JSON → git commit
    /// 4. Copy .fig to `.dit/fig_snapshots/<commit_hash>.fig`
    ///
    /// If `on_progress` is provided, it will be called with a description
    /// of each sub-step. Otherwise, progress is logged via `tracing::info!`.
    ///
    /// Returns the commit hash.
    pub fn commit_from_fig(
        &self,
        file_key: &str,
        auth: &FigmaAuth,
        message: &str,
        on_progress: Option<&dyn Fn(&str)>,
        on_2fa: Option<&dyn Fn() -> Option<String>>,
    ) -> Result<String> {
        let report = |msg: &str| {
            if let Some(cb) = on_progress {
                cb(msg);
            } else {
                tracing::info!("{}", msg);
            }
        };

        let _lock = LockGuard::acquire(&self.root, "commit")
            .context("failed to acquire commit lock")?;

        // Download .fig to a temporary location inside .dit/
        let fig_tmp = self.root.join(DitPaths::DIT_DIR).join("tmp_download.fig");
        let preview_tmp = self.root.join(DitPaths::DIT_DIR).join("tmp_preview.png");
        report("Downloading .fig file from Figma...");
        download_fig_file(file_key, &fig_tmp, auth, Some(&preview_tmp), on_progress, on_2fa)
            .context("failed to download .fig file")?;

        // Convert .fig → DitSnapshot
        report("Converting .fig to snapshot...");
        let snapshot = fig_to_snapshot(&fig_tmp, file_key)
            .context("failed to convert .fig to snapshot")?;

        // Write canonical snapshot files
        report("Writing canonical JSON...");
        canonical::write_snapshot(&self.root, &snapshot)
            .context("failed to write snapshot")?;

        // Pre-stage .fig in dit.fig/latest.fig so it's included in the commit
        report("Storing .fig snapshot...");
        self.pre_stage_fig(&fig_tmp)
            .context("failed to pre-stage .fig file")?;

        // Pre-stage preview in dit.previews/latest.png so it's included in the commit
        if preview_tmp.exists() {
            self.pre_stage_preview(&preview_tmp)
                .context("failed to pre-stage preview image")?;
        }

        // Git commit (includes dit.fig/latest.fig + dit.previews/latest.png)
        report("Committing to git...");
        let hash = git_ops::commit_all(&self.root, message)
            .context("failed to git commit")?;

        // Also store as dit.fig/<hash>.fig + .dit/fig_snapshots/<hash>.fig
        self.store_fig_snapshot(&hash, &fig_tmp)
            .context("failed to store .fig snapshot")?;

        // Store named preview copy (dit.previews/<hash>.png)
        if preview_tmp.exists() {
            self.store_preview(&hash, &preview_tmp)
                .context("failed to store preview image")?;
            std::fs::remove_file(&preview_tmp).ok();
        }

        // Clean up temp file
        std::fs::remove_file(&fig_tmp).ok();

        Ok(hash)
    }

    /// Commit from a local .fig file (no download needed).
    ///
    /// 1. Convert .fig → DitSnapshot via fig2json
    /// 2. Write canonical JSON → git commit
    /// 3. Copy .fig to `.dit/fig_snapshots/<commit_hash>.fig`
    ///
    /// Returns the commit hash.
    pub fn commit_from_local_fig(
        &self,
        fig_path: &Path,
        file_key: &str,
        message: &str,
    ) -> Result<String> {
        let _lock = LockGuard::acquire(&self.root, "commit")
            .context("failed to acquire commit lock")?;

        // Convert .fig → DitSnapshot
        let snapshot = fig_to_snapshot(fig_path, file_key)
            .context("failed to convert .fig to snapshot")?;

        // Write canonical snapshot files
        canonical::write_snapshot(&self.root, &snapshot)
            .context("failed to write snapshot")?;

        // Pre-stage .fig in dit.fig/latest.fig so it's included in the commit
        self.pre_stage_fig(fig_path)
            .context("failed to pre-stage .fig file")?;

        // Git commit (includes dit.fig/latest.fig)
        let hash = git_ops::commit_all(&self.root, message)
            .context("failed to git commit")?;

        // Also store as dit.fig/<hash>.fig + .dit/fig_snapshots/<hash>.fig
        self.store_fig_snapshot(&hash, fig_path)
            .context("failed to store .fig snapshot")?;

        Ok(hash)
    }

    // ── .fig snapshot storage ────────────────────────────────────────────

    /// Pre-copy a .fig file to `dit.fig/latest.fig` so it is included in the
    /// upcoming git commit. Call this BEFORE `git_ops::commit_all()`.
    fn pre_stage_fig(&self, fig_path: &Path) -> Result<()> {
        let fig_dir = self.root.join(DitPaths::FIG_DIR);
        std::fs::create_dir_all(&fig_dir)
            .context("failed to create fig directory")?;
        let dest = fig_dir.join("latest.fig");
        std::fs::copy(fig_path, &dest)
            .with_context(|| format!("failed to copy .fig to {}", dest.display()))?;
        Ok(())
    }

    /// Post-commit: copy .fig to `.dit/fig_snapshots/<hash>.fig` for fast local lookup.
    /// The git-tracked copy lives at `dit.fig/latest.fig` (staged by `pre_stage_fig`).
    fn store_fig_snapshot(&self, commit_hash: &str, fig_path: &Path) -> Result<()> {
        let snapshots_dir = self.root.join(DitPaths::FIG_SNAPSHOTS_DIR);
        std::fs::create_dir_all(&snapshots_dir)
            .context("failed to create fig_snapshots directory")?;
        let dest = snapshots_dir.join(format!("{commit_hash}.fig"));
        std::fs::copy(fig_path, &dest)
            .with_context(|| format!("failed to copy .fig to {}", dest.display()))?;
        Ok(())
    }

    /// Copy a preview image to `dit.previews/<7char_hash>.png` (git-tracked).
    fn store_preview(&self, commit_hash: &str, preview_path: &Path) -> Result<()> {
        let previews_dir = self.root.join(DitPaths::PREVIEWS_DIR);
        std::fs::create_dir_all(&previews_dir)
            .context("failed to create previews directory")?;

        let short_hash = &commit_hash[..7.min(commit_hash.len())];
        let dest = previews_dir.join(format!("{short_hash}.png"));
        std::fs::copy(preview_path, &dest)
            .with_context(|| format!("failed to copy preview to {}", dest.display()))?;

        Ok(())
    }

    /// Pre-stage a preview image to `dit.previews/latest.png` so it is included
    /// in the upcoming git commit. Call this BEFORE `git_ops::commit_all()`.
    fn pre_stage_preview(&self, preview_path: &Path) -> Result<()> {
        let previews_dir = self.root.join(DitPaths::PREVIEWS_DIR);
        std::fs::create_dir_all(&previews_dir)
            .context("failed to create previews directory")?;
        let dest = previews_dir.join("latest.png");
        std::fs::copy(preview_path, &dest)
            .with_context(|| format!("failed to copy preview to {}", dest.display()))?;
        Ok(())
    }

    /// Get the path to the .fig file for a given commit, if it exists.
    ///
    /// Checks in order:
    /// 1. `.dit/fig_snapshots/<hash>.fig` — local cache (git-ignored)
    /// 2. `dit.fig/<hash>.fig` — git-tracked named copy
    /// 3. `dit.fig/latest.fig` — current working tree's .fig (only valid
    ///    when the working tree is checked out at the target commit)
    pub fn get_fig_file_path(&self, commit_hash: &str) -> Option<PathBuf> {
        // Local cache (.dit/fig_snapshots/<hash>.fig, git-ignored)
        let cached = self.root
            .join(DitPaths::FIG_SNAPSHOTS_DIR)
            .join(format!("{commit_hash}.fig"));
        if cached.exists() {
            return Some(cached);
        }
        // Git-tracked named copy (dit.fig/<hash>.fig)
        let named = self.root
            .join(DitPaths::FIG_DIR)
            .join(format!("{commit_hash}.fig"));
        if named.exists() {
            return Some(named);
        }
        // Git-tracked latest (dit.fig/latest.fig)
        let latest = self.root
            .join(DitPaths::FIG_DIR)
            .join("latest.fig");
        if latest.exists() {
            return Some(latest);
        }
        None
    }

    // ── Restore ──────────────────────────────────────────────────────────

    /// Restore a snapshot from any commit.
    ///
    /// Checks out the target commit, reads the snapshot, then returns to
    /// the original branch. If a .fig file exists for this commit, its
    /// path is returned via `RestoreResult`.
    pub fn restore(&self, commit_hash: &str) -> Result<RestoreResult> {
        let _lock = LockGuard::acquire(&self.root, "restore")
            .context("failed to acquire restore lock")?;

        // Remember current branch
        let status = git_ops::get_status(&self.root)?;
        let original_branch = status.branch.clone();

        // Checkout target commit (detached HEAD)
        git_ops::checkout(&self.root, commit_hash)
            .with_context(|| format!("failed to checkout commit {commit_hash}"))?;

        // Read snapshot and copy .fig to a stable location while at the target commit.
        // We must copy now because `dit.fig/latest.fig` will change when we
        // check out back to the original branch.
        let snapshot = canonical::read_snapshot(&self.root);
        let fig_file_path = if let Some(src) = self.get_fig_file_path(commit_hash) {
            // Copy to .dit/fig_snapshots/<hash>.fig so it persists after checkout
            let snapshots_dir = self.root.join(DitPaths::FIG_SNAPSHOTS_DIR);
            std::fs::create_dir_all(&snapshots_dir).ok();
            let stable = snapshots_dir.join(format!("{commit_hash}.fig"));
            if !stable.exists() {
                std::fs::copy(&src, &stable).ok();
            }
            Some(stable)
        } else {
            None
        };

        // Always return to original branch, even if read failed
        let checkout_result = git_ops::checkout(&self.root, &original_branch);

        // Return the snapshot (propagating any errors)
        let snapshot = snapshot.context("failed to read snapshot from commit")?;
        checkout_result.context("failed to return to original branch")?;

        Ok(RestoreResult {
            snapshot,
            fig_file_path,
        })
    }

    // ── Read current state ───────────────────────────────────────────────

    /// Read the current snapshot from disk (without git operations).
    pub fn read_current_snapshot(&self) -> Result<DitSnapshot> {
        canonical::read_snapshot(&self.root)
            .context("failed to read current snapshot")
    }

    // ── Delegate methods ─────────────────────────────────────────────────

    /// Get the current repository status.
    pub fn status(&self) -> Result<DitStatus> {
        git_ops::get_status(&self.root)
    }

    /// Get commit history (most recent first). `max_count` of 0 means all.
    pub fn log(&self, max_count: usize) -> Result<Vec<DitCommitMeta>> {
        git_ops::get_log(&self.root, max_count)
    }

    /// List all local branches.
    pub fn branches(&self) -> Result<Vec<DitBranch>> {
        git_ops::list_branches(&self.root)
    }

    /// Create a new branch at the current HEAD.
    pub fn create_branch(&self, name: &str) -> Result<()> {
        git_ops::create_branch(&self.root, name)
    }

    /// Checkout a branch or commit.
    pub fn checkout(&self, ref_name: &str) -> Result<()> {
        git_ops::checkout(&self.root, ref_name)
    }

    /// Merge a branch into the current HEAD.
    pub fn merge(&self, branch: &str) -> Result<MergeResult> {
        git_ops::merge(&self.root, branch)
    }

    /// Push a branch to a remote. Uses SSH key from config if set.
    pub fn push(&self, remote: &str, branch: &str) -> Result<()> {
        let ssh_key = self.config().ok().and_then(|c| c.ssh_key_path);
        git_ops::push(&self.root, remote, branch, ssh_key.as_deref())
    }

    /// Pull (fetch + fast-forward) a branch from a remote. Uses SSH key from config if set.
    pub fn pull(&self, remote: &str, branch: &str) -> Result<()> {
        let ssh_key = self.config().ok().and_then(|c| c.ssh_key_path);
        git_ops::pull(&self.root, remote, branch, ssh_key.as_deref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{DesignPlatform, DitPage, DitProject};
    use tempfile::TempDir;

    fn test_config() -> DitConfig {
        DitConfig {
            file_key: "test-file-key".into(),
            name: "Test Project".into(),
            figma_token: None,
            schema_version: 1,
            ssh_key_path: None,
        }
    }

    fn test_snapshot() -> DitSnapshot {
        DitSnapshot {
            project: DitProject {
                file_key: "test-file-key".into(),
                name: "Test Project".into(),
                last_modified: "2025-01-01T00:00:00Z".into(),
                version: "1".into(),
                platform: DesignPlatform::Figma,
                schema_version: 1,
                thumbnail_url: None,
                editor_type: None,
                role: None,
            },
            pages: vec![DitPage {
                id: "0:1".into(),
                name: "Page 1".into(),
                background_color: None,
                children: vec![],
            }],
            components: None,
            component_sets: None,
            styles: None,
        }
    }

    #[test]
    fn init_creates_repo() {
        let tmp = TempDir::new().unwrap();
        let repo = DitRepository::init(tmp.path(), test_config()).unwrap();

        assert!(repo.root().join(DitPaths::DIT_DIR).is_dir());
        assert!(repo.root().join(DitPaths::CONFIG_FILE).exists());
        assert!(repo.root().join(DitPaths::PAGES_DIR).is_dir());
        assert!(repo.root().join(DitPaths::ASSETS_DIR).is_dir());
        assert!(repo.root().join(DitPaths::FIG_DIR).is_dir());
    }

    #[test]
    fn open_existing_repo() {
        let tmp = TempDir::new().unwrap();
        DitRepository::init(tmp.path(), test_config()).unwrap();

        let repo = DitRepository::open(tmp.path());
        assert!(repo.is_ok());
    }

    #[test]
    fn open_non_repo_fails() {
        let tmp = TempDir::new().unwrap();
        let result = DitRepository::open(tmp.path());
        assert!(result.is_err());
    }

    #[test]
    fn config_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let repo = DitRepository::init(tmp.path(), test_config()).unwrap();
        let config = repo.config().unwrap();
        assert_eq!(config.file_key, "test-file-key");
        assert_eq!(config.name, "Test Project");
    }

    #[test]
    fn commit_and_status() {
        let tmp = TempDir::new().unwrap();
        let repo = DitRepository::init(tmp.path(), test_config()).unwrap();

        let hash = repo
            .commit(&test_snapshot(), "Initial save", &CommitOptions::default())
            .unwrap();
        assert!(!hash.is_empty());

        let status = repo.status().unwrap();
        assert!(!status.is_dirty);
    }

    #[test]
    fn commit_and_read_back() {
        let tmp = TempDir::new().unwrap();
        let repo = DitRepository::init(tmp.path(), test_config()).unwrap();

        let snapshot = test_snapshot();
        repo.commit(&snapshot, "Save design", &CommitOptions::default())
            .unwrap();

        let loaded = repo.read_current_snapshot().unwrap();
        assert_eq!(loaded.project.file_key, snapshot.project.file_key);
        assert_eq!(loaded.pages.len(), 1);
        assert_eq!(loaded.pages[0].name, "Page 1");
    }

    #[test]
    fn branch_and_checkout() {
        let tmp = TempDir::new().unwrap();
        let repo = DitRepository::init(tmp.path(), test_config()).unwrap();

        repo.create_branch("experiment").unwrap();
        let branches = repo.branches().unwrap();
        let names: Vec<&str> = branches.iter().map(|b| b.name.as_str()).collect();
        assert!(names.contains(&"experiment"));

        repo.checkout("experiment").unwrap();
        let status = repo.status().unwrap();
        assert_eq!(status.branch, "experiment");
    }

    #[test]
    fn log_entries() {
        let tmp = TempDir::new().unwrap();
        let repo = DitRepository::init(tmp.path(), test_config()).unwrap();

        repo.commit(&test_snapshot(), "First save", &CommitOptions::default())
            .unwrap();

        let log = repo.log(10).unwrap();
        // Should have at least 2 entries: init commit + our commit
        assert!(log.len() >= 2);
    }

    #[test]
    fn restore_from_commit() {
        let tmp = TempDir::new().unwrap();
        let repo = DitRepository::init(tmp.path(), test_config()).unwrap();

        // Commit v1
        let mut snap1 = test_snapshot();
        snap1.project.version = "1".into();
        let hash1 = repo
            .commit(&snap1, "Version 1", &CommitOptions::default())
            .unwrap();

        // Commit v2
        let mut snap2 = test_snapshot();
        snap2.project.version = "2".into();
        repo.commit(&snap2, "Version 2", &CommitOptions::default())
            .unwrap();

        // Current state should be v2
        let current = repo.read_current_snapshot().unwrap();
        assert_eq!(current.project.version, "2");

        // Restore v1
        let restored = repo.restore(&hash1).unwrap();
        assert_eq!(restored.snapshot.project.version, "1");

        // We should still be on the original branch (not detached)
        let status = repo.status().unwrap();
        assert_eq!(status.branch, "main");
    }
}
