//! Integration test: multi-commit workflows with branching and merging.
//!
//! Uses the `DitRepository` orchestrator to simulate realistic design
//! version control workflows.

mod fixtures;

use dit_core::canonical;
use dit_core::repository::{CommitOptions, DitRepository};
use dit_core::types::*;
use tempfile::TempDir;

fn test_config() -> DitConfig {
    DitConfig {
        file_key: "workflow-test".into(),
        name: "Workflow Test Project".into(),
        figma_token: None,
        schema_version: 1,
        ssh_key_path: None,
    }
}

fn default_opts() -> CommitOptions {
    CommitOptions::default()
}

// ── Multi-commit workflow ───────────────────────────────────────────────────

#[test]
fn multi_commit_preserves_history() {
    let tmp = TempDir::new().unwrap();
    let repo = DitRepository::init(tmp.path(), test_config()).unwrap();

    // Commit v1: basic snapshot
    let mut snap1 = fixtures::realistic_snapshot();
    snap1.project.version = "1".into();
    let hash1 = repo.commit(&snap1, "Version 1", &default_opts()).unwrap();

    // Commit v2: modified snapshot
    let mut snap2 = fixtures::realistic_snapshot();
    snap2.project.version = "2".into();
    snap2.pages[0].name = "Home (redesigned)".into();
    let hash2 = repo.commit(&snap2, "Version 2", &default_opts()).unwrap();

    // Commit v3: more changes
    let mut snap3 = fixtures::realistic_snapshot();
    snap3.project.version = "3".into();
    snap3.pages.push(DitPage {
        id: "0:3".into(),
        name: "Settings".into(),
        background_color: None,
        children: vec![fixtures::frame_with_autolayout()],
    });
    let hash3 = repo.commit(&snap3, "Version 3: add Settings page", &default_opts()).unwrap();

    // Verify all 3 hashes are unique
    assert_ne!(hash1, hash2);
    assert_ne!(hash2, hash3);
    assert_ne!(hash1, hash3);

    // Log should have 4 entries: init + v1 + v2 + v3
    let log = repo.log(10).unwrap();
    assert!(log.len() >= 4, "expected at least 4 commits, got {}", log.len());

    // Current snapshot should be v3
    let current = repo.read_current_snapshot().unwrap();
    assert_eq!(current.project.version, "3");
    assert_eq!(current.pages.len(), 3);
}

#[test]
fn restore_earlier_version() {
    let tmp = TempDir::new().unwrap();
    let repo = DitRepository::init(tmp.path(), test_config()).unwrap();

    // Commit v1
    let mut snap1 = fixtures::realistic_snapshot();
    snap1.project.version = "v1".into();
    let hash1 = repo.commit(&snap1, "v1", &default_opts()).unwrap();

    // Commit v2
    let mut snap2 = fixtures::realistic_snapshot();
    snap2.project.version = "v2".into();
    repo.commit(&snap2, "v2", &default_opts()).unwrap();

    // Restore v1
    let restored = repo.restore(&hash1).unwrap();
    assert_eq!(restored.snapshot.project.version, "v1");

    // We should still be on main after restore
    let status = repo.status().unwrap();
    assert_eq!(status.branch, "main");

    // Current on-disk snapshot should still be v2
    let current = repo.read_current_snapshot().unwrap();
    assert_eq!(current.project.version, "v2");
}

// ── Branch workflow ─────────────────────────────────────────────────────────

#[test]
fn branch_diverge_and_merge() {
    let tmp = TempDir::new().unwrap();
    let repo = DitRepository::init(tmp.path(), test_config()).unwrap();

    // Commit initial state on main
    let snap = fixtures::realistic_snapshot();
    repo.commit(&snap, "initial", &default_opts()).unwrap();

    // Create feature branch and switch to it
    repo.create_branch("feature/redesign").unwrap();
    repo.checkout("feature/redesign").unwrap();

    // Make changes on the feature branch
    let mut feature_snap = fixtures::realistic_snapshot();
    feature_snap.project.version = "feature-v1".into();
    feature_snap.pages[0].name = "Home (redesigned)".into();
    repo.commit(&feature_snap, "redesign home page", &default_opts()).unwrap();

    // Switch back to main
    repo.checkout("main").unwrap();

    // Verify main still has original state
    let main_snap = repo.read_current_snapshot().unwrap();
    assert_eq!(main_snap.pages[0].name, "Home");

    // Merge feature branch
    let merge_result = repo.merge("feature/redesign").unwrap();
    assert!(merge_result.success, "merge should succeed: {:?}", merge_result.conflicts);
    assert!(merge_result.conflicts.is_empty());
    assert!(merge_result.commit_hash.is_some());

    // Main should now have feature branch changes
    let merged_snap = repo.read_current_snapshot().unwrap();
    assert_eq!(merged_snap.pages[0].name, "Home (redesigned)");
}

#[test]
fn multiple_branches_independent() {
    let tmp = TempDir::new().unwrap();
    let repo = DitRepository::init(tmp.path(), test_config()).unwrap();

    // Initial commit
    let snap = fixtures::realistic_snapshot();
    repo.commit(&snap, "initial", &default_opts()).unwrap();

    // Create two branches
    repo.create_branch("feature/a").unwrap();
    repo.create_branch("feature/b").unwrap();

    // Make changes on branch A
    repo.checkout("feature/a").unwrap();
    let mut snap_a = fixtures::realistic_snapshot();
    snap_a.project.version = "branch-a".into();
    repo.commit(&snap_a, "branch a changes", &default_opts()).unwrap();

    // Make different changes on branch B
    repo.checkout("feature/b").unwrap();
    let mut snap_b = fixtures::realistic_snapshot();
    snap_b.project.version = "branch-b".into();
    repo.commit(&snap_b, "branch b changes", &default_opts()).unwrap();

    // Verify branches are independent
    repo.checkout("feature/a").unwrap();
    let loaded_a = repo.read_current_snapshot().unwrap();
    assert_eq!(loaded_a.project.version, "branch-a");

    repo.checkout("feature/b").unwrap();
    let loaded_b = repo.read_current_snapshot().unwrap();
    assert_eq!(loaded_b.project.version, "branch-b");

    // Main should still have original
    repo.checkout("main").unwrap();
    let loaded_main = repo.read_current_snapshot().unwrap();
    assert_ne!(loaded_main.project.version, "branch-a");
    assert_ne!(loaded_main.project.version, "branch-b");
}

// ── Status tracking ─────────────────────────────────────────────────────────

#[test]
fn status_tracks_dit_file_changes() {
    let tmp = TempDir::new().unwrap();
    let repo = DitRepository::init(tmp.path(), test_config()).unwrap();

    // Should start clean (after init commit)
    let status = repo.status().unwrap();
    assert!(!status.is_dirty, "should start clean");
    assert_eq!(status.branch, "main");

    // Write a snapshot (without committing)
    let snap = fixtures::realistic_snapshot();
    canonical::write_snapshot(repo.root(), &snap).unwrap();

    // Now should be dirty
    let status = repo.status().unwrap();
    assert!(status.is_dirty, "should be dirty after writing snapshot");
    assert!(!status.changes.is_empty(), "should have changes");

    // Commit should clean up
    repo.commit(&snap, "commit snapshot", &default_opts()).unwrap();
    let status = repo.status().unwrap();
    assert!(!status.is_dirty, "should be clean after commit");
}

// ── Snapshot evolution ──────────────────────────────────────────────────────

#[test]
fn adding_pages_across_commits() {
    let tmp = TempDir::new().unwrap();
    let repo = DitRepository::init(tmp.path(), test_config()).unwrap();

    // Commit with 2 pages
    let snap1 = fixtures::realistic_snapshot();
    assert_eq!(snap1.pages.len(), 2);
    repo.commit(&snap1, "2 pages", &default_opts()).unwrap();

    let loaded = repo.read_current_snapshot().unwrap();
    assert_eq!(loaded.pages.len(), 2);

    // Commit with 3 pages (add one)
    let mut snap2 = fixtures::realistic_snapshot();
    snap2.pages.push(DitPage {
        id: "0:3".into(),
        name: "New Page".into(),
        background_color: None,
        children: vec![],
    });
    repo.commit(&snap2, "3 pages", &default_opts()).unwrap();

    let loaded = repo.read_current_snapshot().unwrap();
    assert_eq!(loaded.pages.len(), 3);

    // Verify all page names
    let names: Vec<&str> = loaded.pages.iter().map(|p| p.name.as_str()).collect();
    assert!(names.contains(&"Home"));
    assert!(names.contains(&"Components"));
    assert!(names.contains(&"New Page"));
}

#[test]
fn modifying_existing_pages_across_commits() {
    let tmp = TempDir::new().unwrap();
    let repo = DitRepository::init(tmp.path(), test_config()).unwrap();

    // Commit original
    let snap1 = fixtures::realistic_snapshot();
    repo.commit(&snap1, "original", &default_opts()).unwrap();

    // Modify page content and re-commit
    let mut snap2 = fixtures::realistic_snapshot();
    snap2.pages[0].name = "Home (v2)".into();
    snap2.pages[1].children.push(fixtures::vector_node());
    repo.commit(&snap2, "modified pages", &default_opts()).unwrap();

    let loaded = repo.read_current_snapshot().unwrap();
    assert_eq!(loaded.pages[0].name, "Home (v2)");
    // Second page should have original children + added vector node
    assert!(loaded.pages[1].children.len() > snap1.pages[1].children.len());
}

#[test]
fn modifying_node_tree_across_commits() {
    let tmp = TempDir::new().unwrap();
    let repo = DitRepository::init(tmp.path(), test_config()).unwrap();

    // Commit with full snapshot
    let mut snap1 = fixtures::realistic_snapshot();
    repo.commit(&snap1, "initial", &default_opts()).unwrap();

    // Modify a node within a page
    let frame = &mut snap1.pages[0].children[0];
    frame.name = "Updated Frame Name".into();
    frame.opacity = Some(0.8);
    if let Some(ref mut children) = frame.children {
        children.push(fixtures::vector_node());
    }
    repo.commit(&snap1, "update frame", &default_opts()).unwrap();

    let loaded = repo.read_current_snapshot().unwrap();
    let loaded_frame = &loaded.pages[0].children[0];
    assert_eq!(loaded_frame.name, "Updated Frame Name");
    assert_eq!(loaded_frame.opacity, Some(0.8));
    assert_eq!(loaded_frame.children.as_ref().unwrap().len(), 4); // 3 original + 1 new
}

// ── Open/init repo flow (GUI blank screen regression) ───────────────────────

#[test]
fn open_repo_after_init_loads_data() {
    let tmp = TempDir::new().unwrap();

    // Phase 1: Init and verify immediate access
    let repo = DitRepository::init(tmp.path(), test_config()).unwrap();

    // These calls must succeed immediately after init (GUI does this)
    let status = repo.status().unwrap();
    assert_eq!(status.branch, "main");
    assert!(!status.is_dirty);

    let log = repo.log(100).unwrap();
    assert!(!log.is_empty(), "log should have at least the init commit");

    let branches = repo.branches().unwrap();
    assert!(!branches.is_empty(), "should have at least main branch");
    let main_branch = branches.iter().find(|b| b.name == "main");
    assert!(main_branch.is_some(), "main branch should exist");

    // Phase 2: Drop and reopen (simulates closing and reopening GUI)
    drop(repo);
    let repo = DitRepository::open(tmp.path()).unwrap();

    let status = repo.status().unwrap();
    assert_eq!(status.branch, "main");

    let log = repo.log(100).unwrap();
    assert!(!log.is_empty(), "log should persist after reopen");

    let branches = repo.branches().unwrap();
    assert!(!branches.is_empty(), "branches should persist after reopen");
}

#[test]
fn open_repo_after_commit_loads_all_data() {
    let tmp = TempDir::new().unwrap();

    // Init and commit
    let repo = DitRepository::init(tmp.path(), test_config()).unwrap();
    let snap = fixtures::realistic_snapshot();
    let _hash = repo.commit(&snap, "test commit", &default_opts()).unwrap();

    // Drop and reopen
    drop(repo);
    let repo = DitRepository::open(tmp.path()).unwrap();

    // All data should be accessible
    let status = repo.status().unwrap();
    assert_eq!(status.branch, "main");
    assert!(!status.is_dirty);

    let log = repo.log(100).unwrap();
    assert!(log.len() >= 2, "should have init + test commit");
    assert!(log.iter().any(|c| c.message == "test commit"), "commit message should be in log");

    let branches = repo.branches().unwrap();
    assert!(branches.iter().any(|b| b.name == "main"), "main branch should exist");

    let snapshot = repo.read_current_snapshot().unwrap();
    assert_eq!(snapshot.project.file_key, "test_file_key_abc123");
    assert!(!snapshot.pages.is_empty());

    // Config should be readable
    let config = repo.config().unwrap();
    assert_eq!(config.file_key, "workflow-test");
    assert_eq!(config.name, "Workflow Test Project");
}
