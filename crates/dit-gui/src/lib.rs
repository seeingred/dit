use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use base64::Engine;
use serde::{Deserialize, Serialize};
use tauri::{Emitter, State};

use dit_core::canonical;
use dit_core::figma::FigmaAuth;
use dit_core::git_ops;
use dit_core::repository::{CloneResult, DitRepository};
use dit_core::types::{DitConfig, DitNode, DitPage, DitPaths, node_id_to_filename};

// ── Tauri managed state ──────────────────────────────────────────────

struct AppState {
    repo: Mutex<Option<DitRepository>>,
    /// Channel for receiving 2FA codes from the frontend.
    twofa_sender: Mutex<Option<std::sync::mpsc::Sender<String>>>,
}

// ── Shared response types ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub date: String,
    pub branch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoStatus {
    pub branch: String,
    pub has_changes: bool,
    pub changed_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    pub name: String,
    pub is_current: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeInfo {
    pub success: bool,
    pub conflicts: Vec<String>,
    pub commit_hash: Option<String>,
    pub fast_forward: bool,
    pub fig_snapshot_ours: Option<String>,
    pub fig_snapshot_theirs: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    pub before_image: Option<String>,
    pub after_image: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreInfo {
    pub message: String,
    pub fig_file_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneInfo {
    /// Whether the cloned repo is already a DIT repository.
    pub is_dit_repo: bool,
    /// Canonical path to the cloned directory.
    pub path: String,
    /// Project name if it's a DIT repo, None otherwise.
    pub name: Option<String>,
    /// Whether Figma credentials are missing (needs auth setup).
    pub needs_auth: bool,
}

// ── Helpers ──────────────────────────────────────────────────────────

fn with_repo<F, T>(state: &State<AppState>, f: F) -> Result<T, String>
where
    F: FnOnce(&DitRepository) -> anyhow::Result<T>,
{
    let guard = state.repo.lock().map_err(|e| format!("lock error: {e}"))?;
    let repo = guard
        .as_ref()
        .ok_or_else(|| "No repository open. Use Open Repository or Initialize New first.".to_string())?;
    f(repo).map_err(|e| format!("{e:#}"))
}

fn figma_auth_for_repo(repo: &DitRepository) -> Result<FigmaAuth, String> {
    // Try .env in repo root.
    let env_path = repo.root().join(".env");
    if env_path.exists() {
        let _ = dotenvy::from_path(&env_path);
    }

    // Try cookie-based auth first.
    if let Ok(cookie) = std::env::var("FIGMA_AUTH_COOKIE") {
        return Ok(FigmaAuth::Cookie(cookie));
    }

    // Try email/password.
    if let (Ok(email), Ok(password)) = (std::env::var("FIGMA_EMAIL"), std::env::var("FIGMA_PASSWORD")) {
        return Ok(FigmaAuth::EmailPassword { email, password });
    }

    Err("No Figma credentials found. Set FIGMA_AUTH_COOKIE or FIGMA_EMAIL+FIGMA_PASSWORD in .env".into())
}

// ── Tauri commands ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirCheck {
    pub exists: bool,
    pub has_dit: bool,
    pub has_git: bool,
}

#[tauri::command]
async fn check_directory(path: String) -> Result<DirCheck, String> {
    let dir = PathBuf::from(&path);
    Ok(DirCheck {
        exists: dir.exists(),
        has_dit: dir.join(".dit").is_dir(),
        has_git: dir.join(".git").is_dir(),
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshKeyInfo {
    pub name: String,
    pub path: String,
}

#[tauri::command]
async fn list_ssh_keys() -> Vec<SshKeyInfo> {
    git_ops::list_ssh_keys()
        .into_iter()
        .map(|k| SshKeyInfo { name: k.name, path: k.path })
        .collect()
}

#[tauri::command]
async fn clone_repo(
    state: State<'_, AppState>,
    url: String,
    path: String,
    ssh_key_path: Option<String>,
) -> Result<CloneInfo, String> {
    let dir = PathBuf::from(&path);

    let result = DitRepository::clone(&url, &dir, ssh_key_path.as_deref())
        .map_err(|e| format!("{e:#}"))?;

    match result {
        CloneResult::DitRepo(repo) => {
            // Store SSH key in config if one was selected.
            if let Some(ref key) = ssh_key_path {
                if let Ok(mut config) = repo.config() {
                    config.ssh_key_path = Some(key.clone());
                    if let Ok(json) = serde_json::to_string_pretty(&config) {
                        let _ = std::fs::write(
                            repo.root().join(DitPaths::CONFIG_FILE),
                            &json,
                        );
                    }
                }
            }
            let name = repo.config().map(|c| c.name).ok();
            let repo_path = repo.root().display().to_string();
            let needs_auth = !repo.root().join(".env").exists();

            let mut guard = state.repo.lock().map_err(|e| format!("lock error: {e}"))?;
            *guard = Some(repo);

            Ok(CloneInfo {
                is_dit_repo: true,
                path: repo_path,
                name,
                needs_auth,
            })
        }
        CloneResult::NeedsInit { path } => {
            Ok(CloneInfo {
                is_dit_repo: false,
                path: path.display().to_string(),
                name: None,
                needs_auth: true,
            })
        }
    }
}

#[tauri::command]
async fn save_credentials(
    state: State<'_, AppState>,
    auth_cookie: Option<String>,
    auth_email: Option<String>,
    auth_password: Option<String>,
) -> Result<(), String> {
    let guard = state.repo.lock().map_err(|e| format!("lock error: {e}"))?;
    let repo = guard
        .as_ref()
        .ok_or_else(|| "No repository open.".to_string())?;

    let mut env_lines = Vec::new();
    if let Some(cookie) = &auth_cookie {
        if !cookie.is_empty() {
            env_lines.push(format!("FIGMA_AUTH_COOKIE={cookie}"));
        }
    }
    if let Some(email) = &auth_email {
        if !email.is_empty() {
            env_lines.push(format!("FIGMA_EMAIL={email}"));
        }
    }
    if let Some(password) = &auth_password {
        if !password.is_empty() {
            env_lines.push(format!("FIGMA_PASSWORD={password}"));
        }
    }

    if env_lines.is_empty() {
        return Err("No credentials provided.".into());
    }

    let env_path = repo.root().join(".env");
    std::fs::write(&env_path, env_lines.join("\n") + "\n")
        .map_err(|e| format!("failed to write .env: {e}"))?;

    // Ensure .env is in .gitignore
    let gitignore_path = repo.root().join(".gitignore");
    let existing = std::fs::read_to_string(&gitignore_path).unwrap_or_default();
    if !existing.contains(".env") {
        std::fs::write(&gitignore_path, format!("{existing}.env\n"))
            .map_err(|e| format!("failed to update .gitignore: {e}"))?;
    }

    Ok(())
}

#[tauri::command]
async fn init_repo(
    state: State<'_, AppState>,
    path: String,
    auth_cookie: Option<String>,
    auth_email: Option<String>,
    auth_password: Option<String>,
    file_key: String,
    file_name: String,
    force: bool,
    ssh_key_path: Option<String>,
) -> Result<String, String> {
    let dir = PathBuf::from(&path);

    // If force-reinitializing, clean existing DIT data only (preserve git).
    if force {
        let dit_dir = dir.join(".dit");
        if dit_dir.exists() {
            std::fs::remove_dir_all(&dit_dir)
                .map_err(|e| format!("failed to remove .dit: {e}"))?;
        }
    }

    let config = DitConfig {
        file_key,
        name: file_name,
        figma_token: None,
        schema_version: 1,
        ssh_key_path,
    };

    let repo = DitRepository::init(&dir, config).map_err(|e| format!("{e:#}"))?;
    let root = repo.root().to_path_buf();

    // Write .env with credentials (git-ignored).
    let mut env_lines = Vec::new();
    if let Some(cookie) = &auth_cookie {
        if !cookie.is_empty() {
            env_lines.push(format!("FIGMA_AUTH_COOKIE={cookie}"));
        }
    }
    if let Some(email) = &auth_email {
        if !email.is_empty() {
            env_lines.push(format!("FIGMA_EMAIL={email}"));
        }
    }
    if let Some(password) = &auth_password {
        if !password.is_empty() {
            env_lines.push(format!("FIGMA_PASSWORD={password}"));
        }
    }
    if !env_lines.is_empty() {
        let env_path = root.join(".env");
        std::fs::write(&env_path, env_lines.join("\n") + "\n")
            .map_err(|e| format!("failed to write .env: {e}"))?;
    }

    // Add .env to .gitignore.
    let gitignore_path = root.join(".gitignore");
    let existing = std::fs::read_to_string(&gitignore_path).unwrap_or_default();
    if !existing.contains(".env") {
        std::fs::write(&gitignore_path, format!("{existing}.env\n"))
            .map_err(|e| format!("failed to update .gitignore: {e}"))?;
    }

    let display = root.display().to_string();

    let mut guard = state.repo.lock().map_err(|e| format!("lock error: {e}"))?;
    *guard = Some(repo);

    Ok(format!("Initialized DIT repository at {display}"))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRepoInfo {
    pub name: String,
    pub needs_auth: bool,
}

#[tauri::command]

async fn open_repo(state: State<'_, AppState>, path: String) -> Result<OpenRepoInfo, String> {
    let dir = PathBuf::from(&path);

    let repo = DitRepository::open(&dir).map_err(|e| format!("{e:#}"))?;
    let name = repo
        .config()
        .map(|c| c.name)
        .unwrap_or_else(|_| "Unknown".into());
    let needs_auth = !repo.root().join(".env").exists();

    let mut guard = state.repo.lock().map_err(|e| format!("lock error: {e}"))?;
    *guard = Some(repo);

    Ok(OpenRepoInfo { name, needs_auth })
}

#[tauri::command]
async fn get_status(state: State<'_, AppState>) -> Result<RepoStatus, String> {
    with_repo(&state, |repo| {
        let status = repo.status()?;
        Ok(RepoStatus {
            branch: status.branch,
            has_changes: status.is_dirty,
            changed_files: status.changes.iter().map(|c| c.path.clone()).collect(),
        })
    })
}

#[tauri::command]
async fn get_log(state: State<'_, AppState>) -> Result<Vec<CommitInfo>, String> {
    with_repo(&state, |repo| {
        let log = repo.log(100)?;
        let status = repo.status()?;

        Ok(log
            .into_iter()
            .map(|entry| CommitInfo {
                hash: entry.hash.clone(),
                message: entry.message.clone(),
                author: entry.author.clone(),
                date: entry.timestamp,
                branch: if entry.hash.starts_with(&status.branch) {
                    Some(status.branch.clone())
                } else {
                    None
                },
            })
            .collect())
    })
}

#[tauri::command]
async fn get_branches(state: State<'_, AppState>) -> Result<Vec<BranchInfo>, String> {
    with_repo(&state, |repo| {
        let branches = repo.branches()?;
        Ok(branches
            .into_iter()
            .map(|b| BranchInfo {
                name: b.name,
                is_current: b.is_current,
            })
            .collect())
    })
}

#[tauri::command]
async fn submit_2fa_code(state: State<'_, AppState>, code: String) -> Result<(), String> {
    let guard = state.twofa_sender.lock().map_err(|e| format!("lock error: {e}"))?;
    if let Some(sender) = guard.as_ref() {
        sender.send(code).map_err(|e| format!("failed to send 2FA code: {e}"))?;
    }
    Ok(())
}

#[tauri::command]
async fn commit(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    message: String,
) -> Result<CommitInfo, String> {
    app.emit("commit-progress", "Validating configuration...").ok();

    // Extract repo root and auth before the async part.
    let (_root, auth) = {
        let guard = state.repo.lock().map_err(|e| format!("lock error: {e}"))?;
        let repo = guard
            .as_ref()
            .ok_or_else(|| "No repository open.".to_string())?;
        let auth = figma_auth_for_repo(repo)?;
        (repo.root().to_path_buf(), auth)
    };

    let config = {
        let guard = state.repo.lock().map_err(|e| format!("lock error: {e}"))?;
        let repo = guard.as_ref().ok_or("No repository open.")?;
        repo.config().map_err(|e| format!("{e:#}"))?
    };

    if config.file_key.is_empty() {
        return Err("No Figma file key configured. Set it in .dit/config.json.".into());
    }

    app.emit("commit-progress", "Downloading design from Figma...").ok();

    // Set up 2FA channel
    let (twofa_tx, twofa_rx) = std::sync::mpsc::channel::<String>();
    {
        let mut guard = state.twofa_sender.lock().map_err(|e| format!("lock error: {e}"))?;
        *guard = Some(twofa_tx);
    }

    // Download .fig and commit using the new flow.
    let app_handle = app.clone();
    let app_handle_2fa = app.clone();
    let hash = {
        let guard = state.repo.lock().map_err(|e| format!("lock error: {e}"))?;
        let repo = guard.as_ref().ok_or("No repository open.")?;
        repo.commit_from_fig(&config.file_key, &auth, &message, Some(&|msg: &str| {
            app_handle.emit("commit-progress", msg).ok();
        }), Some(&|| {
            // Emit event to frontend requesting 2FA code
            app_handle_2fa.emit("2fa-required", ()).ok();
            // Block until the frontend sends the code via submit_2fa_code
            twofa_rx.recv().ok()
        }))
            .map_err(|e| format!("{e:#}"))?
    };

    // Clean up 2FA channel
    {
        let mut guard = state.twofa_sender.lock().map_err(|e| format!("lock error: {e}"))?;
        *guard = None;
    }

    app.emit("commit-progress", "Finalizing...").ok();

    let status = {
        let guard = state.repo.lock().map_err(|e| format!("lock error: {e}"))?;
        let repo = guard.as_ref().ok_or("No repository open.")?;
        repo.status().map_err(|e| format!("{e:#}"))?
    };

    app.emit("commit-progress", "Commit complete!").ok();

    Ok(CommitInfo {
        hash: hash.clone(),
        message,
        author: "DIT".into(),
        date: String::new(),
        branch: Some(status.branch),
    })
}

#[tauri::command]
async fn restore(state: State<'_, AppState>, hash: String) -> Result<RestoreInfo, String> {
    let result = {
        let guard = state.repo.lock().map_err(|e| format!("lock error: {e}"))?;
        let repo = guard.as_ref().ok_or("No repository open.")?;
        repo.restore(&hash).map_err(|e| format!("{e:#}"))?
    };

    let fig_path_str = result.fig_file_path.as_ref().map(|p| p.display().to_string());

    let msg = format!(
        "Restored to {} — {} ({} pages)",
        &hash[..7.min(hash.len())],
        result.snapshot.project.name,
        result.snapshot.pages.len(),
    );

    Ok(RestoreInfo {
        message: msg,
        fig_file_path: fig_path_str,
    })
}

#[tauri::command]
async fn open_fig_file(path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open .fig file: {e}"))?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &path])
            .spawn()
            .map_err(|e| format!("Failed to open .fig file: {e}"))?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open .fig file: {e}"))?;
    }
    Ok(())
}

#[tauri::command]
async fn checkout(state: State<'_, AppState>, reference: String) -> Result<String, String> {
    with_repo(&state, |repo| {
        repo.checkout(&reference)?;
        Ok(format!("Switched to {reference}"))
    })
}

#[tauri::command]
async fn create_branch(state: State<'_, AppState>, name: String) -> Result<String, String> {
    with_repo(&state, |repo| {
        repo.create_branch(&name)?;
        Ok(format!("Created branch {name}"))
    })
}

#[tauri::command]
async fn merge(state: State<'_, AppState>, branch: String) -> Result<MergeInfo, String> {
    with_repo(&state, |repo| {
        let result = repo.merge(&branch)?;
        Ok(MergeInfo {
            success: result.success,
            conflicts: result.conflicts,
            commit_hash: result.commit_hash,
            fast_forward: result.fast_forward,
            fig_snapshot_ours: result.fig_snapshots.ours,
            fig_snapshot_theirs: result.fig_snapshots.theirs,
        })
    })
}

#[tauri::command]
async fn push(state: State<'_, AppState>) -> Result<String, String> {
    with_repo(&state, |repo| {
        let status = repo.status()?;
        repo.push("origin", &status.branch)?;
        Ok(format!("Pushed {} to origin", status.branch))
    })
}

#[tauri::command]
async fn pull(state: State<'_, AppState>) -> Result<String, String> {
    with_repo(&state, |repo| {
        let status = repo.status()?;
        repo.pull("origin", &status.branch)?;
        Ok(format!("Pulled {} from origin", status.branch))
    })
}

#[tauri::command]
async fn get_preview_image(
    state: State<'_, AppState>,
    commit_hash: String,
) -> Result<Option<String>, String> {
    with_repo(&state, |repo| {
        // Try to find a preview image for this commit.
        let short = &commit_hash[..7.min(commit_hash.len())];
        let preview_dir = repo.root().join(DitPaths::PREVIEWS_DIR);

        // 1. Named copy: dit.previews/<hash>.png
        let named = preview_dir.join(format!("{short}.png"));
        // 2. Latest: dit.previews/latest.png (most recent commit)
        let latest = preview_dir.join("latest.png");
        // 3. Legacy: .dit/previews/<hash>.png
        let legacy = repo.root()
            .join(DitPaths::DIT_DIR)
            .join("previews")
            .join(format!("{short}.png"));

        // Check if this commit is HEAD (latest.png is valid for HEAD)
        let head_hash = repo.status().ok().and_then(|s| s.head);
        let is_head = head_hash
            .as_ref()
            .map(|h| h.starts_with(short))
            .unwrap_or(false);

        let path = if named.exists() {
            Some(named)
        } else if is_head && latest.exists() {
            Some(latest)
        } else if legacy.exists() {
            Some(legacy)
        } else {
            None
        };

        if let Some(p) = path {
            let bytes = std::fs::read(&p)?;
            let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
            Ok(Some(encoded))
        } else {
            Ok(None)
        }
    })
}

#[tauri::command]
async fn get_diff_previews(
    state: State<'_, AppState>,
    hash1: String,
    hash2: String,
) -> Result<DiffResult, String> {
    with_repo(&state, |repo| {
        let preview_dir = repo.root().join(DitPaths::PREVIEWS_DIR);
        let legacy_dir = repo.root().join(DitPaths::DIT_DIR).join("previews");

        let load = |hash: &str| -> Option<String> {
            let short = &hash[..7.min(hash.len())];
            let name = format!("{short}.png");
            let path = preview_dir.join(&name);
            let legacy = legacy_dir.join(&name);
            let p = if path.exists() { path } else { legacy };
            std::fs::read(&p)
                .ok()
                .map(|bytes| base64::engine::general_purpose::STANDARD.encode(&bytes))
        };

        Ok(DiffResult {
            before_image: load(&hash1),
            after_image: load(&hash2),
        })
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct TreeNodeInfo {
    pub id: String,
    pub name: String,
    pub node_type: String,
    pub children: Vec<TreeNodeInfo>,
}

fn dit_node_to_tree(node: &DitNode) -> TreeNodeInfo {
    TreeNodeInfo {
        id: node.id.clone(),
        name: node.name.clone(),
        node_type: format!("{:?}", node.node_type),
        children: node
            .children
            .as_ref()
            .map(|kids| kids.iter().map(|c| dit_node_to_tree(c)).collect())
            .unwrap_or_default(),
    }
}

#[tauri::command]
async fn get_commit_tree(state: State<'_, AppState>) -> Result<Vec<TreeNodeInfo>, String> {
    with_repo(&state, |repo| {
        let snapshot = canonical::read_snapshot(repo.root())?;
        let tree: Vec<TreeNodeInfo> = snapshot
            .pages
            .iter()
            .map(|page| TreeNodeInfo {
                id: page.id.clone(),
                name: page.name.clone(),
                node_type: "Page".into(),
                children: page.children.iter().map(|c| dit_node_to_tree(c)).collect(),
            })
            .collect();
        Ok(tree)
    })
}

// ── Diff tree types & command ────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct DiffTreeNode {
    pub id: String,
    pub name: String,
    pub node_type: String,
    /// "added", "removed", "modified", or null (unchanged)
    pub change_type: Option<String>,
    pub children: Vec<DiffTreeNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiffTreeResult {
    pub before: Vec<DiffTreeNode>,
    pub after: Vec<DiffTreeNode>,
}

/// Read page JSON files from a git commit tree without checking out.
fn read_pages_at_commit(
    git_repo: &git2::Repository,
    commit_hash: &str,
) -> Result<Vec<DitPage>, String> {
    let oid = git2::Oid::from_str(commit_hash)
        .map_err(|e| format!("invalid commit hash: {e}"))?;
    let commit = git_repo
        .find_commit(oid)
        .map_err(|e| format!("commit not found: {e}"))?;
    let tree = commit
        .tree()
        .map_err(|e| format!("failed to get commit tree: {e}"))?;

    // Find the dit.pages directory in the commit tree
    let pages_entry = tree
        .get_path(std::path::Path::new(DitPaths::PAGES_DIR))
        .map_err(|e| format!("dit.pages not found in commit: {e}"))?;
    let pages_tree = git_repo
        .find_tree(pages_entry.id())
        .map_err(|e| format!("dit.pages is not a tree: {e}"))?;

    let mut pages = Vec::new();
    for entry in pages_tree.iter() {
        let name = entry.name().unwrap_or("");
        if !name.ends_with(".json") {
            continue;
        }
        let blob = git_repo
            .find_blob(entry.id())
            .map_err(|e| format!("failed to read page blob: {e}"))?;
        let content = std::str::from_utf8(blob.content())
            .map_err(|e| format!("page blob is not UTF-8: {e}"))?;
        let page: DitPage = canonical::deserialize(content)
            .map_err(|e| format!("failed to parse page JSON: {e}"))?;
        pages.push(page);
    }

    // Sort by filename for deterministic ordering
    pages.sort_by(|a, b| {
        node_id_to_filename(&a.id).cmp(&node_id_to_filename(&b.id))
    });

    Ok(pages)
}

/// Collect all node IDs and their (name, type) into a flat map for comparison.
fn collect_node_signatures(
    nodes: &[DiffTreeNode],
    map: &mut HashMap<String, (String, String)>,
) {
    for node in nodes {
        map.insert(
            node.id.clone(),
            (node.name.clone(), node.node_type.clone()),
        );
        collect_node_signatures(&node.children, map);
    }
}

/// Build DiffTreeNode from DitNode, initially with no change_type.
fn dit_node_to_diff_tree(node: &DitNode) -> DiffTreeNode {
    DiffTreeNode {
        id: node.id.clone(),
        name: node.name.clone(),
        node_type: format!("{:?}", node.node_type),
        change_type: None,
        children: node
            .children
            .as_ref()
            .map(|kids| kids.iter().map(dit_node_to_diff_tree).collect())
            .unwrap_or_default(),
    }
}

/// Build DiffTreeNode from DitPage.
fn page_to_diff_tree(page: &DitPage) -> DiffTreeNode {
    DiffTreeNode {
        id: page.id.clone(),
        name: page.name.clone(),
        node_type: "Page".into(),
        change_type: None,
        children: page.children.iter().map(dit_node_to_diff_tree).collect(),
    }
}

/// Apply change_type annotations to tree nodes based on the comparison maps.
fn annotate_changes(
    nodes: &mut [DiffTreeNode],
    own_sigs: &HashMap<String, (String, String)>,
    other_sigs: &HashMap<String, (String, String)>,
    side: &str, // "before" or "after"
) {
    for node in nodes.iter_mut() {
        match side {
            "before" => {
                if !other_sigs.contains_key(&node.id) {
                    node.change_type = Some("removed".into());
                } else if other_sigs.get(&node.id) != own_sigs.get(&node.id) {
                    node.change_type = Some("modified".into());
                }
            }
            "after" => {
                if !other_sigs.contains_key(&node.id) {
                    node.change_type = Some("added".into());
                } else if other_sigs.get(&node.id) != own_sigs.get(&node.id) {
                    node.change_type = Some("modified".into());
                }
            }
            _ => {}
        }
        annotate_changes(&mut node.children, own_sigs, other_sigs, side);
    }
}

#[tauri::command]
async fn get_diff_trees(
    state: State<'_, AppState>,
    hash1: String,
    hash2: String,
) -> Result<DiffTreeResult, String> {
    with_repo(&state, |repo| {
        let git_repo = git2::Repository::open(repo.root())
            .map_err(|e| anyhow::anyhow!("failed to open git repo: {e}"))?;

        let pages_before = read_pages_at_commit(&git_repo, &hash1)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        let pages_after = read_pages_at_commit(&git_repo, &hash2)
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        let mut before: Vec<DiffTreeNode> =
            pages_before.iter().map(page_to_diff_tree).collect();
        let mut after: Vec<DiffTreeNode> =
            pages_after.iter().map(page_to_diff_tree).collect();

        // Collect signatures for comparison
        let mut before_sigs = HashMap::new();
        let mut after_sigs = HashMap::new();
        collect_node_signatures(&before, &mut before_sigs);
        collect_node_signatures(&after, &mut after_sigs);

        // Annotate changes
        annotate_changes(&mut before, &before_sigs, &after_sigs, "before");
        annotate_changes(&mut after, &after_sigs, &before_sigs, "after");

        Ok(DiffTreeResult { before, after })
    })
}

// ── App entry ────────────────────────────────────────────────────────

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            repo: Mutex::new(None),
            twofa_sender: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            check_directory,
            list_ssh_keys,
            clone_repo,
            save_credentials,
            init_repo,
            open_repo,
            get_status,
            get_log,
            get_branches,
            commit,
            submit_2fa_code,
            restore,
            open_fig_file,
            checkout,
            create_branch,
            merge,
            push,
            pull,
            get_preview_image,
            get_diff_previews,
            get_commit_tree,
            get_diff_trees,
        ])
        .run(tauri::generate_context!())
        .expect("error while running DIT application");
}
