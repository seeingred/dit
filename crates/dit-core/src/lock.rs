//! File-based locking for DIT repository operations.
//!
//! Prevents concurrent mutations by creating a lock file at
//! `.dit/locks/<operation>.lock` containing the PID, timestamp, and
//! operation name.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use crate::types::DitLock;

/// Directory inside `.dit/` where lock files are stored.
const LOCKS_DIR: &str = ".dit/locks";

/// Acquire a lock for the given operation.
///
/// Creates `.dit/locks/<operation>.lock`. Fails if a lock for this
/// operation already exists (another process is running).
pub fn acquire_lock(repo_root: &Path, operation: &str) -> Result<PathBuf> {
    let locks_dir = repo_root.join(LOCKS_DIR);
    std::fs::create_dir_all(&locks_dir)
        .with_context(|| format!("failed to create locks directory: {}", locks_dir.display()))?;

    let lock_path = lock_file_path(repo_root, operation);

    // Check for stale lock
    if lock_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&lock_path) {
            if let Ok(existing) = serde_json::from_str::<DitLock>(&content) {
                // Check if the process is still alive
                if is_process_alive(existing.pid) {
                    bail!(
                        "operation '{}' is locked by PID {} (since {}). \
                         If this is stale, delete {}",
                        existing.operation,
                        existing.pid,
                        existing.acquired_at,
                        lock_path.display()
                    );
                }
                // Stale lock — process is dead, we can take over
            }
        }
        // Lock file is corrupt or stale — remove it
        std::fs::remove_file(&lock_path).ok();
    }

    let lock = DitLock {
        pid: std::process::id(),
        acquired_at: now_iso8601(),
        operation: operation.to_string(),
    };

    let json = serde_json::to_string_pretty(&lock).context("failed to serialize lock")?;
    std::fs::write(&lock_path, json)
        .with_context(|| format!("failed to write lock file: {}", lock_path.display()))?;

    Ok(lock_path)
}

/// Release a previously acquired lock.
///
/// Silently succeeds if the lock file doesn't exist (idempotent).
pub fn release_lock(repo_root: &Path, operation: &str) {
    let lock_path = lock_file_path(repo_root, operation);
    std::fs::remove_file(&lock_path).ok();
}

/// Check whether an operation lock is currently held.
pub fn is_locked(repo_root: &Path, operation: &str) -> bool {
    let lock_path = lock_file_path(repo_root, operation);
    if !lock_path.exists() {
        return false;
    }
    // Check if the lock holder is still alive
    if let Ok(content) = std::fs::read_to_string(&lock_path) {
        if let Ok(lock) = serde_json::from_str::<DitLock>(&content) {
            return is_process_alive(lock.pid);
        }
    }
    false
}

/// RAII guard that releases the lock on drop.
pub struct LockGuard {
    repo_root: PathBuf,
    operation: String,
}

impl LockGuard {
    /// Acquire a lock and return a guard that releases it on drop.
    pub fn acquire(repo_root: &Path, operation: &str) -> Result<Self> {
        acquire_lock(repo_root, operation)?;
        Ok(Self {
            repo_root: repo_root.to_path_buf(),
            operation: operation.to_string(),
        })
    }
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        release_lock(&self.repo_root, &self.operation);
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn lock_file_path(repo_root: &Path, operation: &str) -> PathBuf {
    repo_root
        .join(LOCKS_DIR)
        .join(format!("{operation}.lock"))
}

fn now_iso8601() -> String {
    // Simple UTC timestamp without pulling in chrono.
    let dur = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    let days = secs / 86400;
    let rem = secs % 86400;
    let h = rem / 3600;
    let m = (rem % 3600) / 60;
    let s = rem % 60;
    let (y, mo, d) = days_to_ymd(days);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{s:02}Z")
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

fn is_process_alive(pid: u32) -> bool {
    // Check /proc/<pid> on Linux, or use kill(0) via std::process::Command on Unix.
    #[cfg(unix)]
    {
        use std::process::Command;
        // `kill -0 <pid>` succeeds (exit 0) if the process exists.
        Command::new("kill")
            .args(["-0", &pid.to_string()])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(true) // If we can't check, assume alive (safe default)
    }
    #[cfg(not(unix))]
    {
        let _ = pid;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DitPaths;
    use tempfile::TempDir;

    fn setup() -> (TempDir, std::path::PathBuf) {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().to_path_buf();
        // Create .dit dir
        std::fs::create_dir_all(path.join(DitPaths::DIT_DIR)).unwrap();
        (tmp, path)
    }

    #[test]
    fn acquire_and_release() {
        let (_tmp, path) = setup();
        let lock_path = acquire_lock(&path, "test-op").unwrap();
        assert!(lock_path.exists());
        assert!(is_locked(&path, "test-op"));

        release_lock(&path, "test-op");
        assert!(!lock_path.exists());
        assert!(!is_locked(&path, "test-op"));
    }

    #[test]
    fn double_acquire_fails() {
        let (_tmp, path) = setup();
        acquire_lock(&path, "commit").unwrap();
        let result = acquire_lock(&path, "commit");
        assert!(result.is_err());
    }

    #[test]
    fn different_operations_independent() {
        let (_tmp, path) = setup();
        acquire_lock(&path, "commit").unwrap();
        // A different operation should not conflict
        acquire_lock(&path, "restore").unwrap();
    }

    #[test]
    fn lock_guard_releases_on_drop() {
        let (_tmp, path) = setup();
        {
            let _guard = LockGuard::acquire(&path, "auto").unwrap();
            assert!(is_locked(&path, "auto"));
        }
        assert!(!is_locked(&path, "auto"));
    }

    #[test]
    fn release_nonexistent_is_ok() {
        let (_tmp, path) = setup();
        release_lock(&path, "nonexistent"); // should not panic
    }
}
