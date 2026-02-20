/// Constants describing the DIT repository directory layout.
///
/// A DIT repository lives inside a normal git repo and uses the following structure:
///
/// ```text
/// project/
/// ├── .dit/              # DIT metadata (git-ignored internals)
/// │   ├── config.json    # Project config (file key, name, token)
/// │   └── lock           # Lock file for concurrent-access protection
/// ├── dit.json           # Project-level metadata (committed)
/// ├── dit.pages/         # One JSON file per page
/// │   ├── 0_1.json       # Page "0:1" (colons → underscores)
/// │   └── 12_34.json
/// ├── dit.nodes/         # Optional: exploded per-node JSON (for large files)
/// │   └── ...
/// ├── dit.assets/        # Content-addressed asset blobs
/// │   ├── sha256_<hash>  # Image/video asset files
/// │   └── ...
/// ├── dit.fig/           # Git-tracked .fig file snapshots
/// │   ├── <commit_hash>.fig
/// │   └── ...
/// ├── dit.styles.json    # Shared style definitions
/// ├── dit.components.json# Component & component-set metadata
/// └── .gitignore
/// ```
pub struct DitPaths;

impl DitPaths {
    // ── Top-level directory names ────────────────────────────────────────
    /// Hidden metadata directory (git-ignored).
    pub const DIT_DIR: &str = ".dit";
    /// Directory containing per-page JSON files.
    pub const PAGES_DIR: &str = "dit.pages";
    /// Directory containing per-node JSON files (optional, for large files).
    pub const NODES_DIR: &str = "dit.nodes";
    /// Directory containing content-addressed asset blobs.
    pub const ASSETS_DIR: &str = "dit.assets";
    /// Directory containing git-tracked .fig file snapshots.
    pub const FIG_DIR: &str = "dit.fig";

    // ── Files inside .dit/ ──────────────────────────────────────────────
    /// Project configuration (token, file key, etc.).
    pub const CONFIG_FILE: &str = ".dit/config.json";
    /// Lock file for concurrent-access protection.
    pub const LOCK_FILE: &str = ".dit/lock";
    /// Directory for .fig file snapshots (git-ignored, local only).
    pub const FIG_SNAPSHOTS_DIR: &str = ".dit/fig_snapshots";

    // ── Top-level committed files ───────────────────────────────────────
    /// Project-level metadata (committed to git).
    pub const PROJECT_FILE: &str = "dit.json";
    /// Shared style definitions.
    pub const STYLES_FILE: &str = "dit.styles.json";
    /// Component and component-set metadata.
    pub const COMPONENTS_FILE: &str = "dit.components.json";

    // ── Asset reference format ───────────────────────────────────────────
    /// Prefix for content-addressed asset references.
    pub const ASSET_REF_PREFIX: &str = "sha256:";
}

/// Convert a Figma node ID (e.g. "0:1") into a filesystem-safe name ("0_1").
pub fn node_id_to_filename(id: &str) -> String {
    id.replace(':', "_")
}

/// Convert a filesystem-safe name back to a Figma node ID ("0_1" → "0:1").
pub fn filename_to_node_id(filename: &str) -> String {
    filename.replace('_', ":")
}

/// Build the path for a page JSON file given its node ID.
/// Returns e.g. "dit.pages/0_1.json".
pub fn page_path(page_id: &str) -> String {
    format!("{}/{}.json", DitPaths::PAGES_DIR, node_id_to_filename(page_id))
}

/// Build the path for an asset file given its SHA-256 hex hash.
/// Returns e.g. "dit.assets/sha256_abc123".
pub fn asset_path(sha256_hex: &str) -> String {
    format!("{}/sha256_{}", DitPaths::ASSETS_DIR, sha256_hex)
}

/// Build the full asset reference string from a SHA-256 hex hash.
/// Returns e.g. "sha256:abc123".
pub fn asset_ref(sha256_hex: &str) -> String {
    format!("{}{}", DitPaths::ASSET_REF_PREFIX, sha256_hex)
}

/// Parse an asset reference string into its hex hash.
/// "sha256:abc123" → Some("abc123")
pub fn parse_asset_ref(reference: &str) -> Option<&str> {
    reference.strip_prefix(DitPaths::ASSET_REF_PREFIX)
}
