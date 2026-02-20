//! End-to-end tests for commit, preview capture, and restore.
//!
//! Supports two modes:
//!
//! **Local mode** (no network needed):
//!   Set `FIGMA_LOCAL_FIG=/path/to/file.fig` and `FIGMA_FILE_KEY=<key>`.
//!   Uses `commit_from_local_fig()`. Preview assertions are skipped (preview
//!   capture only happens during Playwright download).
//!
//! **Download mode** (needs network + credentials):
//!   Set `FIGMA_FILE_KEY=<key>` and either `FIGMA_AUTH_COOKIE` or
//!   `FIGMA_EMAIL` + `FIGMA_PASSWORD`. Uses `commit_from_fig()`.
//!
//! All env vars can be placed in `<project_root>/.env`.
//!
//! Run with: `cargo test -p dit-core --test e2e_commit -- --ignored`

use std::path::{Path, PathBuf};

use dit_core::figma::FigmaAuth;
use dit_core::repository::DitRepository;
use dit_core::types::{DitConfig, DitPaths};
use tempfile::TempDir;

/// Project root: two levels up from `crates/dit-core/`.
fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("failed to resolve project root")
}

/// Load `.env` from project root (idempotent).
fn load_dotenv() {
    let env_path = project_root().join(".env");
    if env_path.exists() {
        let _ = dotenvy::from_path(&env_path);
    }
}

/// Load Figma credentials from env vars.
fn load_auth() -> Option<FigmaAuth> {
    load_dotenv();

    if let Ok(cookie) = std::env::var("FIGMA_AUTH_COOKIE") {
        return Some(FigmaAuth::Cookie(cookie));
    }

    if let (Ok(email), Ok(password)) = (
        std::env::var("FIGMA_EMAIL"),
        std::env::var("FIGMA_PASSWORD"),
    ) {
        return Some(FigmaAuth::EmailPassword { email, password });
    }

    None
}

/// Load the Figma file key from env var `FIGMA_FILE_KEY`.
fn load_file_key() -> Option<String> {
    load_dotenv();
    std::env::var("FIGMA_FILE_KEY").ok()
}

/// Load optional local .fig file path from env var `FIGMA_LOCAL_FIG`.
fn load_local_fig() -> Option<PathBuf> {
    load_dotenv();
    std::env::var("FIGMA_LOCAL_FIG").ok().map(PathBuf::from)
}

/// Whether we're using local .fig mode (no download, no preview).
fn is_local_mode() -> bool {
    load_local_fig().is_some()
}

/// Commit mode: either local .fig or download from Figma.
enum CommitMode {
    /// Use a local .fig file — no preview capture.
    Local { fig_path: PathBuf },
    /// Download from Figma — preview is captured by Playwright.
    Download { auth: FigmaAuth },
}

/// Determine commit mode from env vars. Returns None if neither mode is configured.
fn commit_mode() -> Option<CommitMode> {
    if let Some(fig_path) = load_local_fig() {
        assert!(
            fig_path.exists(),
            "FIGMA_LOCAL_FIG points to non-existent file: {}",
            fig_path.display()
        );
        return Some(CommitMode::Local { fig_path });
    }

    load_auth().map(|auth| CommitMode::Download { auth })
}

/// Perform a commit using the configured mode. Returns the commit hash.
fn do_commit(repo: &DitRepository, file_key: &str, message: &str, mode: &CommitMode) -> String {
    match mode {
        CommitMode::Local { fig_path } => repo
            .commit_from_local_fig(fig_path, file_key, message)
            .expect("commit_from_local_fig failed"),
        CommitMode::Download { auth } => repo
            .commit_from_fig(file_key, auth, message, None)
            .expect("commit_from_fig failed"),
    }
}

/// Set up a temp DIT repo and return (TempDir, DitRepository, file_key, CommitMode).
/// Panics with a descriptive message if required env vars are missing.
fn setup() -> (TempDir, DitRepository, String, CommitMode) {
    let file_key = load_file_key().expect(
        "FIGMA_FILE_KEY must be set (in env or .env)"
    );

    let mode = commit_mode().expect(
        "Set FIGMA_LOCAL_FIG for local mode, or FIGMA_AUTH_COOKIE / \
         FIGMA_EMAIL+FIGMA_PASSWORD for download mode (in env or .env)"
    );

    let tmp = TempDir::new().expect("failed to create temp dir");
    let config = DitConfig {
        file_key: file_key.clone(),
        name: "E2E Test Project".into(),
        figma_token: None,
        schema_version: 1,
    };
    let repo = DitRepository::init(tmp.path(), config).expect("failed to init repo");

    (tmp, repo, file_key, mode)
}

/// Assert that a preview PNG exists and is valid for the given commit hash.
fn assert_preview_exists(repo_root: &Path, commit_hash: &str) {
    let short_hash = &commit_hash[..7.min(commit_hash.len())];
    let preview_path = repo_root
        .join(DitPaths::DIT_DIR)
        .join("previews")
        .join(format!("{short_hash}.png"));
    assert!(
        preview_path.exists(),
        "expected preview image at {}",
        preview_path.display()
    );

    let data = std::fs::read(&preview_path).expect("failed to read preview");
    assert!(data.len() > 8, "preview file too small ({} bytes)", data.len());

    // PNG magic bytes: 89 50 4E 47 0D 0A 1A 0A
    let png_magic: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    assert_eq!(
        &data[..8], &png_magic,
        "preview file does not start with PNG magic bytes"
    );

    assert!(
        data.len() > 1024,
        "preview too small ({} bytes), likely invalid",
        data.len()
    );
    assert!(
        data.len() < 10 * 1024 * 1024,
        "preview too large ({} bytes)",
        data.len()
    );
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[test]
#[ignore]
fn test_commit_downloads_fig_and_creates_snapshot() {
    let (_tmp, repo, file_key, mode) = setup();

    let hash = do_commit(&repo, &file_key, "e2e: initial commit", &mode);

    // Hash should be a non-empty hex string.
    assert!(!hash.is_empty(), "commit hash is empty");
    assert!(hash.len() >= 7, "commit hash too short: {hash}");

    // .fig snapshot should exist.
    let fig_path = repo
        .root()
        .join(DitPaths::FIG_SNAPSHOTS_DIR)
        .join(format!("{hash}.fig"));
    assert!(
        fig_path.exists(),
        "expected .fig snapshot at {}",
        fig_path.display()
    );
    let fig_size = std::fs::metadata(&fig_path).unwrap().len();
    assert!(
        fig_size > 100,
        ".fig file is suspiciously small ({fig_size} bytes)"
    );

    // Canonical JSON should be written.
    let project_file = repo.root().join(DitPaths::PROJECT_FILE);
    assert!(project_file.exists(), "dit.json not found after commit");

    let pages_dir = repo.root().join(DitPaths::PAGES_DIR);
    assert!(pages_dir.is_dir(), "dit.pages/ not found after commit");

    let page_count = std::fs::read_dir(&pages_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "json")
                .unwrap_or(false)
        })
        .count();
    assert!(
        page_count >= 1,
        "expected at least 1 page file in dit.pages/, found {page_count}"
    );

    // Snapshot should be readable.
    let snapshot = repo
        .read_current_snapshot()
        .expect("failed to read snapshot");
    assert!(!snapshot.pages.is_empty(), "snapshot has no pages");
    assert_eq!(
        snapshot.project.file_key, file_key,
        "file_key mismatch in snapshot"
    );
}

#[test]
#[ignore]
fn test_commit_creates_preview_image() {
    if is_local_mode() {
        eprintln!(
            "Skipping preview test: FIGMA_LOCAL_FIG is set — \
             preview capture only works with Playwright download mode"
        );
        return;
    }

    let (_tmp, repo, file_key, mode) = setup();

    let hash = do_commit(&repo, &file_key, "e2e: preview test", &mode);

    assert_preview_exists(repo.root(), &hash);
}

#[test]
#[ignore]
fn test_restore_returns_fig_and_snapshot() {
    let (_tmp, repo, file_key, mode) = setup();

    // Commit once.
    let hash = do_commit(&repo, &file_key, "e2e: for restore", &mode);

    // Restore from that commit.
    let result = repo.restore(&hash).expect("restore failed");

    // Snapshot should have pages.
    assert!(
        !result.snapshot.pages.is_empty(),
        "restored snapshot has no pages"
    );
    assert_eq!(
        result.snapshot.project.file_key, file_key,
        "file_key mismatch in restored snapshot"
    );

    // fig_file_path should point to the stored .fig file.
    let fig_file_path = result
        .fig_file_path
        .expect("RestoreResult.fig_file_path is None");
    assert!(
        fig_file_path.exists(),
        "restored .fig file does not exist at {}",
        fig_file_path.display()
    );

    // We should still be on main after restore.
    let status = repo.status().expect("status failed");
    assert_eq!(
        status.branch, "main",
        "expected to be on 'main' after restore, got '{}'",
        status.branch
    );
}

#[test]
#[ignore]
fn test_multiple_commits_have_separate_snapshots() {
    let (_tmp, repo, file_key, mode) = setup();

    // First commit.
    let hash1 = do_commit(&repo, &file_key, "e2e: commit 1", &mode);

    // Second commit.
    let hash2 = do_commit(&repo, &file_key, "e2e: commit 2", &mode);

    // Hashes should differ (different git commits even if same .fig content).
    assert_ne!(hash1, hash2, "two commits produced the same hash");

    // Each should have its own .fig snapshot.
    let fig1 = repo
        .root()
        .join(DitPaths::FIG_SNAPSHOTS_DIR)
        .join(format!("{hash1}.fig"));
    let fig2 = repo
        .root()
        .join(DitPaths::FIG_SNAPSHOTS_DIR)
        .join(format!("{hash2}.fig"));
    assert!(fig1.exists(), ".fig snapshot for commit 1 not found");
    assert!(fig2.exists(), ".fig snapshot for commit 2 not found");

    // Preview assertions only apply in download mode.
    if !is_local_mode() {
        let short1 = &hash1[..7.min(hash1.len())];
        let short2 = &hash2[..7.min(hash2.len())];
        let preview1 = repo
            .root()
            .join(DitPaths::DIT_DIR)
            .join("previews")
            .join(format!("{short1}.png"));
        let preview2 = repo
            .root()
            .join(DitPaths::DIT_DIR)
            .join("previews")
            .join(format!("{short2}.png"));

        assert!(
            preview1.exists(),
            "preview for commit 1 not found at {}",
            preview1.display()
        );
        assert!(
            preview2.exists(),
            "preview for commit 2 not found at {}",
            preview2.display()
        );

        assert_ne!(
            short1, short2,
            "short hashes are the same — previews would collide"
        );
    }
}
