# DIT — AI Agent Reference

> Git-style version control for design files. Downloads Figma designs as native `.fig` files, converts them to deterministic JSON for text-based diffs, and stores everything in a normal Git repository.

## Quick Orientation

```
dit-claude-code/
├── crates/
│   ├── dit-core/        Rust core library (types, canonical JSON, assets, git, Figma integration)
│   ├── dit-cli/         CLI tool ("dit") — self-contained binary
│   └── dit-gui/         Desktop GUI (Tauri 2 + React 19 + Tailwind 3)
├── scripts/
│   └── download-fig.mjs Playwright-based .fig file downloader (embedded in binary at compile time)
└── Cargo.toml           Workspace root
```

## Architecture

### Data Flow

```
Figma Editor
    │
    ▼ (Playwright browser automation via download-fig.mjs)
.fig file (native Figma binary, zip format)
    │
    ▼ (fig2json crate)
Raw JSON
    │
    ▼ (normalize_fig_json + canonical serialization)
Deterministic JSON files (dit.json, dit.pages/*.json, dit.styles.json, dit.components.json)
    │
    ▼ (git commit)
Git repository (text-based diffs, branching)
```

### Three Entry Points

1. **CLI** (`dit-cli`) — self-contained binary, all features
2. **GUI** (`dit-gui`) — Tauri desktop app, calls dit-core directly
3. **Library** (`dit-core`) — programmatic Rust API

All three share the same core library. The CLI is the primary interface; the GUI depends on the same core.

## Crate: dit-core

The central library. Every module is re-exported from `lib.rs`.

### Module Map

| Module | File | Purpose |
|--------|------|---------|
| `types` | `src/types/` (12 files) | Complete Figma data model — 150+ types |
| `canonical` | `src/canonical.rs` | Deterministic JSON serialization |
| `assets` | `src/assets.rs` | Content-addressed asset storage (SHA-256) |
| `lock` | `src/lock.rs` | File-based concurrency control |
| `git_ops` | `src/git_ops.rs` | Git operations (libgit2 + CLI for push/pull) |
| `repository` | `src/repository.rs` | High-level orchestrator (`DitRepository`) |
| `figma::downloader` | `src/figma/downloader.rs` | .fig download via Playwright |
| `figma::fig_converter` | `src/figma/fig_converter.rs` | .fig → DitSnapshot conversion |

### types/ — Data Model

**Core types:**

- **`DitSnapshot`** — Complete design state: project metadata + pages + components + styles
- **`DitProject`** — File metadata: file_key, name, version, platform, schema_version
- **`DitPage`** — Single page: id, name, background_color, children (Vec<DitNode>)
- **`DitNode`** — Any design element. 50+ optional fields covering geometry, appearance, layout, text, components, vectors, effects. Recursive via `children: Option<Vec<DitNode>>`
- **`DitConfig`** — Persistent repo config: file_key, name, figma_token, schema_version
- **`DitStatus`** — Working tree state: branch, head, changes, is_dirty
- **`DitCommitMeta`** — Commit info: hash, message, author, timestamp

**Enums:** `NodeType` (33 variants), `BlendMode` (18), `PaintType` (10), `EffectType` (4), `LayoutMode`, `TextAutoResize`, `ChangeType`, `DesignPlatform`, and 20+ more.

**Primitives:** `Color` (RGBA), `Vector` (2D), `Size`, `Rect`, `Transform` (2D affine), `ArcData`, `StrokeWeights`.

**Compound types:** `Paint` (fills/strokes), `Effect` (shadows/blur), `TypeStyle` (text formatting), `LayoutConstraint`, `LayoutGrid`, `VectorNetwork` (paths), `ExportSetting`, `ComponentMetadata`, `StyleDefinition`, `Override`.

**Path constants (`DitPaths`):**

| Constant | Value | Git-tracked? |
|----------|-------|-------------|
| `DIT_DIR` | `.dit` | No (git-ignored) |
| `CONFIG_FILE` | `.dit/config.json` | No |
| `FIG_SNAPSHOTS_DIR` | `.dit/fig_snapshots` | No (legacy local copy) |
| `FIG_DIR` | `dit.fig` | Yes |
| `PROJECT_FILE` | `dit.json` | Yes |
| `PAGES_DIR` | `dit.pages` | Yes |
| `NODES_DIR` | `dit.nodes` | Yes |
| `ASSETS_DIR` | `dit.assets` | Yes |
| `STYLES_FILE` | `dit.styles.json` | Yes |
| `COMPONENTS_FILE` | `dit.components.json` | Yes |

**Path utilities:** `node_id_to_filename("0:1")` → `"0_1"`, `page_path("0:1")` → `"dit.pages/0_1.json"`, `asset_path(hash)` → `"dit.assets/sha256_<hash>"`.

### canonical.rs — Deterministic JSON

Ensures identical data produces byte-identical JSON for clean git diffs:

1. Serialize to `serde_json::Value`
2. Recursively sort all object keys lexicographically
3. Normalize floats to 6 decimal places; integers stay as integers
4. Pretty-print with 2-space indent + trailing newline

**Key functions:** `serialize()`, `deserialize()`, `write_snapshot(root, snapshot)`, `read_snapshot(root)`.

### assets.rs — Content-Addressed Storage

SHA-256 deduplication for binary assets (images, videos):

- `compute_hash(bytes)` → 64-char hex string
- `store_asset(root, bytes)` → asset reference string (`"sha256:<hash>"`)
- `retrieve_asset(root, ref)` → bytes
- Assets stored at `dit.assets/sha256_<hex>` — no file extension
- Identical content always produces the same ref (automatic dedup)

### lock.rs — Concurrency Control

File-based RAII locking in `.dit/locks/`:

- `LockGuard::acquire(root, operation)` — creates lock file with PID + timestamp
- Auto-releases on drop
- Stale lock detection via PID liveness check
- Prevents concurrent commits/merges

### git_ops.rs — Git Operations

Uses `git2` (libgit2) for local operations, shells out to `git` CLI for push/pull (credential helper compatibility).

**Local operations (git2):**
- `init_repository()` — create repo, set HEAD to "main", write .gitignore, initial commit
- `commit_all()` — stages all `DIT_TRACKED` paths, creates commit
- `get_status()`, `get_log()`, `list_branches()`, `create_branch()`, `checkout()`
- ~~`merge()` — merge analysis (up-to-date / fast-forward / recursive), returns `MergeResult` with conflict info and `.fig` snapshot paths from both branches~~ *(not available in MVP)*

**Remote operations (git CLI):**
- `push()`, `pull()` — shell out to `git push`/`git pull` to inherit system credential helpers

**DIT_TRACKED paths:** `["dit.json", "dit.pages/", "dit.nodes/", "dit.assets/", "dit.fig/", "dit.styles.json", "dit.components.json"]`

### repository.rs — DitRepository

High-level API orchestrating all modules:

```rust
DitRepository::init(dir, config) → Result<Self>
DitRepository::open(dir) → Result<Self>

// Commit workflows
repo.commit(snapshot, message, options) → Result<String>         // from DitSnapshot
repo.commit_from_fig(file_key, auth, message) → Result<String>  // download + convert + commit
repo.commit_from_local_fig(path, file_key, message) → Result<String>  // local .fig

// Read
repo.read_current_snapshot() → Result<DitSnapshot>
repo.config() → Result<DitConfig>
repo.status() → Result<DitStatus>
repo.log(max_count) → Result<Vec<DitCommitMeta>>

// Branches
repo.branches() → Result<Vec<DitBranch>>
repo.create_branch(name), repo.checkout(ref_name)
// repo.merge(branch) → Result<MergeResult>  // not available in MVP

// Restore
repo.restore(commit_hash) → Result<RestoreResult>  // returns snapshot + .fig path

// Remote
repo.push(remote, branch), repo.pull(remote, branch)
```

**`commit_from_fig` flow:**
1. Acquire lock
2. Download `.fig` via Playwright → `.dit/tmp_download.fig`
3. Capture preview screenshot → `.dit/tmp_preview.png`
4. Convert `.fig` → `DitSnapshot` via fig2json
5. Write canonical JSON to disk
6. Pre-stage `.fig` to `dit.fig/latest.fig` (git-tracked)
7. Git commit all DIT-tracked files (includes `dit.fig/latest.fig`)
8. Store `.fig` as `dit.fig/<hash>.fig` + `.dit/fig_snapshots/<hash>.fig`
9. Store preview at `.dit/previews/<7char_hash>.png`
10. Clean up temp files, release lock

### figma/downloader.rs — .fig Download

Embeds `scripts/download-fig.mjs` and `scripts/package.json` via `include_str!` at compile time. At runtime:

1. Creates `~/.dit/downloader/` directory
2. Writes embedded scripts there (overwritten each run to stay current)
3. Runs `npm install` if `node_modules/` is missing
4. Shells out to `node download-fig.mjs` with appropriate args

**Node.js binary resolution** (`resolve_command(name)`):
1. Direct `Command::new(name)` (works from terminal)
2. User's shell (`$SHELL` / `/bin/zsh`) with `-lic` and `-lc` flags
3. Common paths: `/opt/homebrew/bin`, `/usr/local/bin`
4. nvm: `~/.nvm/versions/node/*/bin/` (latest first)
5. Volta: `~/.volta/bin/`

**`augment_node_path(bin_path, cmd)`** — adds resolved binary's parent dir to child process PATH so npm/npx scripts can find `node`.

**`FigmaAuth` enum:** `Cookie(String)` or `EmailPassword { email, password }`.

### figma/fig_converter.rs — .fig → Snapshot

Converts `.fig` binary to `DitSnapshot`:

1. `fig2json::convert_raw(bytes)` → raw JSON string
2. `normalize_fig_json(value)` — recursively:
   - Flattens `{"__enum__": "NodeType", "value": "CANVAS"}` → `"CANVAS"`
   - Converts `{"guid": {"sessionID": N, "localID": M}}` → `"N:M"` string IDs
3. Maps fig2json document structure to DIT types (pages, nodes, components, styles)

## Crate: dit-cli

Single file: `src/main.rs`. Uses clap for argument parsing.

**Commands:**

| Command | Function | Description |
|---------|----------|-------------|
| `dit init` | `cmd_init()` | Interactive setup with dialoguer prompts |
| `dit status` | `cmd_status()` | Show branch + changes |
| `dit commit -m "msg"` | `cmd_commit()` | Download .fig, convert, commit |
| `dit commit --fig path -m "msg"` | `cmd_commit()` | Commit from local .fig |
| `dit commit --local -m "msg"` | `cmd_commit()` | Re-commit current on-disk snapshot |
| `dit log [-n N]` | `cmd_log()` | Show commit history |
| `dit branch [name]` | `cmd_branch()` | List or create branches |
| `dit checkout <ref>` | `cmd_checkout()` | Switch branch/commit |
| ~~`dit merge <branch>`~~ | ~~`cmd_merge()`~~ | ~~Merge branch~~ *(not available in MVP)* |
| `dit restore <hash>` | `cmd_restore()` | Get .fig file path for commit |
| `dit push [remote]` | `cmd_push()` | Push to remote |
| `dit pull [remote]` | `cmd_pull()` | Pull from remote |
| `dit diff <c1> <c2>` | `cmd_diff()` | Compare two commits |
| `dit setup` | `cmd_setup()` | Install downloader + Playwright |

**Dependencies:** dit-core, clap, console, indicatif, dialoguer, dotenvy.

## Crate: dit-gui

Tauri 2 desktop app. Backend in `src/lib.rs`, frontend in `frontend/`.

### Backend (src/lib.rs)

Tauri command handlers — thin wrappers around `DitRepository` methods. State managed via `Mutex<Option<DitRepository>>`.

**Key commands:** `init_repo`, `open_repo`, `get_status`, `get_log`, `get_branches`, `commit` (emits `commit-progress` events), `restore`, `checkout`, `create_branch`, `merge`, `push`, `pull`, `get_preview_image` (base64 PNG), `get_commit_tree` (TreeNodeInfo).

### Frontend (React 19 + Tailwind 3)

**Components:**

| Component | File | Purpose |
|-----------|------|---------|
| `App` | `App.tsx` | Root — manages startup vs main view |
| `StartupFlow` | `StartupFlow.tsx` | Repo init wizard (folder, auth, file key) |
| `MainLayout` | `MainLayout.tsx` | Main window with commit list + preview |
| `CommitList` | `CommitList.tsx` | Scrollable list with thumbnails, diff selection |
| `PreviewPanel` | `PreviewPanel.tsx` | Canvas preview + tree viewer |
| `TreeViewer` | `TreeViewer.tsx` | Recursive collapsible node tree |
| `DiffView` | `DiffView.tsx` | Side-by-side before/after previews |
| `ActionToolbar` | `ActionToolbar.tsx` | Commit, push, pull buttons |
| `BranchSelector` | `BranchSelector.tsx` | Branch dropdown + create |
| `CommitOverlay` | `CommitOverlay.tsx` | Progress overlay during commit |
| `CommandBar` | `CommandBar.tsx` | Command history input |

**Theme:** CSS custom properties with `@media (prefers-color-scheme: dark)` in `index.css`. Tailwind configured with CSS variable references.

**Types** (`types.ts`): `CommitInfo`, `RepoStatus`, `BranchInfo`, `DiffResult`, `RestoreInfo`, `TreeNode`, `DirCheck`.

**Build:** Vite + TypeScript. `npm run build` produces `frontend/dist/`.

## scripts/download-fig.mjs

Playwright browser automation script embedded in the Rust binary at compile time.

**Flow:**
1. Launch system Chrome (`channel: "chrome"` — required for WebGL support)
2. Block analytics/tracking scripts (GTM, Sentry, Amplitude, FullStory)
3. Dismiss GDPR/cookie popups
4. Authenticate (cookie injection or email/password login)
5. Wait for `[data-testid="ProfileButton"]` (auth confirmation)
6. Navigate to `https://www.figma.com/design/<file_key>/`
7. Wait for `#toggle-menu-button` (editor loaded)
8. Capture preview screenshot if `--preview-output` specified
9. Click: Main Menu → File → Save local copy
10. Wait for download event, save `.fig` file

**Critical:** Uses `channel: "chrome"` (system Chrome) because Playwright's bundled Chromium lacks WebGL support, which Figma's editor requires.

## Repository Layout (User's Project)

```
my-design/
├── dit.json                  Project metadata (committed)
├── dit.pages/                One JSON file per page (committed)
│   ├── 0_1.json
│   └── 0_2.json
├── dit.styles.json           Shared styles (committed)
├── dit.components.json       Component metadata (committed)
├── dit.assets/               Content-addressed binaries (committed)
│   └── sha256_<hex>
├── dit.fig/                  Git-tracked .fig file snapshots (committed)
│   ├── latest.fig            Current commit's .fig file
│   └── <hash>.fig            Previous commits' .fig files
├── .dit/                     Local metadata (git-ignored)
│   ├── config.json           DIT configuration
│   ├── fig_snapshots/        Legacy local .fig copies
│   │   └── <hash>.fig
│   ├── previews/             Preview PNGs per commit
│   │   └── <7char>.png
│   └── locks/                Active operation locks
├── .env                      Figma credentials (git-ignored)
└── .gitignore
```

## Build & Test

```bash
# Build all crates
cargo build --workspace

# Build release
cargo build --workspace --release

# Run unit + integration tests (fast, no network)
cargo test --workspace

# Run e2e tests (requires Figma credentials in .env + Google Chrome)
cargo test -p dit-core --test e2e_commit -- --ignored

# Build macOS app
cd crates/dit-gui && cargo tauri build

# Install frontend deps (required before GUI build)
cd crates/dit-gui/frontend && npm install
```

### Test Structure

| Test File | Tests | What It Covers |
|-----------|-------|---------------|
| `dit-core/src/` (unit) | 34 | All module internals |
| `tests/deterministic.rs` | 10 | Byte-identical JSON output, key sorting, float normalization |
| `tests/asset_dedup.rs` | 13 | SHA-256 dedup, various sizes, cross-node sharing |
| `tests/lossless_roundtrip.rs` | 13 | Serialize/deserialize preserves all data |
| `tests/workflow.rs` | 10 | Multi-commit, branching, ~~merging~~, restore, open/init flows |
| `tests/e2e_commit.rs` | 4 (ignored) | Full Figma download → commit → restore cycle |

**E2e test modes:**
- **Download mode:** Set `FIGMA_FILE_KEY` + (`FIGMA_AUTH_COOKIE` or `FIGMA_EMAIL`+`FIGMA_PASSWORD`) in `.env`
- **Local mode:** Set `FIGMA_LOCAL_FIG=/path/to/file.fig` + `FIGMA_FILE_KEY` — skips download, no preview

### Dependencies

**Workspace-level:**
- serde 1, serde_json 1, anyhow 1, thiserror 2, sha2 0.10, hex 0.4, git2 0.19, clap 4, tracing 0.1, tokio 1, tempfile 3

**dit-core specific:** fig2json 0.3, dirs 6

**dit-cli specific:** dialoguer, console, indicatif, dotenvy

**dit-gui specific:** tauri 2, tauri-plugin-dialog, tauri-plugin-shell, base64

**Runtime:** Node.js 20+, Google Chrome (for Playwright WebGL), npm

## Key Design Decisions

1. **Canonical JSON over binary diffs** — enables standard git workflows (branching, merging, diffing) on design files
2. **Full lossless storage** — JSON committed to git for human-readable diffs; binary `.fig` files also committed (in `dit.fig/`) so cloning the repo gives full restore capability
3. **Embedded scripts** — `download-fig.mjs` compiled into the binary via `include_str!`, written to `~/.dit/downloader/` at runtime for self-contained distribution
4. **System Chrome over Playwright Chromium** — Figma requires WebGL which Playwright's bundled Chromium lacks in headless mode
5. **git CLI for push/pull** — libgit2 doesn't support system credential helpers (SSH agent, macOS Keychain); other git ops use libgit2 for speed
6. **fig2json normalization** — fig2json returns non-standard enum objects and guid structures that must be flattened before DIT type deserialization
7. **Per-page file splitting** — each page is a separate JSON file for natural diffing and smaller change sets
8. **Content-addressed assets** — SHA-256 dedup prevents bloating git history with duplicate images
