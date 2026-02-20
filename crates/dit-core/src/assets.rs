//! Content-addressed asset storage for DIT.
//!
//! Assets (images, videos, etc.) are stored as binary blobs under
//! `dit.assets/sha256_<hex_hash>`. Identical content is automatically
//! deduplicated because the filename is derived from the content hash.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

use crate::types::paths::{self, DitPaths};

/// Compute the SHA-256 hex digest of `data`.
pub fn compute_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Compute the SHA-256 hash of `data` and return a full asset reference
/// string in the form `"sha256:<hex_hash>"`.
pub fn create_asset_ref(data: &[u8]) -> String {
    paths::asset_ref(&compute_hash(data))
}

/// Store `data` as a content-addressed asset under `repo_root`.
///
/// Returns the asset reference string (e.g. `"sha256:abcdef…"`).
/// If an asset with the same hash already exists, the write is skipped
/// (deduplication).
pub fn store_asset(repo_root: &Path, data: &[u8]) -> Result<String> {
    let hash = compute_hash(data);
    let rel = paths::asset_path(&hash);
    let abs = repo_root.join(&rel);

    // Skip if already stored (deduplication).
    if abs.exists() {
        return Ok(paths::asset_ref(&hash));
    }

    // Ensure the assets directory exists.
    let dir = repo_root.join(DitPaths::ASSETS_DIR);
    fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create assets directory: {}", dir.display()))?;

    // Write atomically: write to a temp name, then rename, to avoid
    // partial reads by concurrent processes.
    let tmp = abs.with_extension("tmp");
    fs::write(&tmp, data)
        .with_context(|| format!("failed to write asset: {}", tmp.display()))?;
    fs::rename(&tmp, &abs)
        .with_context(|| format!("failed to rename asset into place: {}", abs.display()))?;

    Ok(paths::asset_ref(&hash))
}

/// Retrieve the binary content of a previously-stored asset.
///
/// `ref_str` must be a valid asset reference (e.g. `"sha256:abcdef…"`).
pub fn retrieve_asset(repo_root: &Path, ref_str: &str) -> Result<Vec<u8>> {
    let hash = paths::parse_asset_ref(ref_str)
        .with_context(|| format!("invalid asset reference: {ref_str}"))?;
    let abs = repo_root.join(paths::asset_path(hash));
    fs::read(&abs).with_context(|| format!("failed to read asset: {}", abs.display()))
}

/// Check whether an asset with the given reference exists on disk.
///
/// Returns `false` for malformed references instead of erroring.
pub fn asset_exists(repo_root: &Path, ref_str: &str) -> bool {
    paths::parse_asset_ref(ref_str)
        .map(|hash| repo_root.join(paths::asset_path(hash)).exists())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn hash_is_deterministic() {
        let a = compute_hash(b"hello world");
        let b = compute_hash(b"hello world");
        assert_eq!(a, b);
        // Known SHA-256 of "hello world"
        assert_eq!(
            a,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn create_ref_includes_prefix() {
        let r = create_asset_ref(b"test data");
        assert!(r.starts_with("sha256:"));
    }

    #[test]
    fn round_trip_store_and_retrieve() {
        let tmp = TempDir::new().unwrap();
        let data = b"PNG image bytes here";

        let ref_str = store_asset(tmp.path(), data).unwrap();
        assert!(ref_str.starts_with("sha256:"));

        let retrieved = retrieve_asset(tmp.path(), &ref_str).unwrap();
        assert_eq!(retrieved, data);
    }

    #[test]
    fn deduplication_does_not_error() {
        let tmp = TempDir::new().unwrap();
        let data = b"duplicate me";

        let ref1 = store_asset(tmp.path(), data).unwrap();
        let ref2 = store_asset(tmp.path(), data).unwrap();
        assert_eq!(ref1, ref2);
    }

    #[test]
    fn asset_exists_works() {
        let tmp = TempDir::new().unwrap();
        let data = b"existence check";

        let ref_str = store_asset(tmp.path(), data).unwrap();
        assert!(asset_exists(tmp.path(), &ref_str));
        assert!(!asset_exists(tmp.path(), "sha256:0000000000000000"));
        assert!(!asset_exists(tmp.path(), "invalid-ref"));
    }

    #[test]
    fn retrieve_missing_asset_errors() {
        let tmp = TempDir::new().unwrap();
        let result = retrieve_asset(tmp.path(), "sha256:does_not_exist");
        assert!(result.is_err());
    }

    #[test]
    fn retrieve_invalid_ref_errors() {
        let tmp = TempDir::new().unwrap();
        let result = retrieve_asset(tmp.path(), "not-a-valid-ref");
        assert!(result.is_err());
    }
}
