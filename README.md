# DIT — Design Version Control

Git-style version control for design files. DIT downloads your Figma designs as native `.fig` files, converts them to deterministic JSON for text-based diffs, and stores everything in a normal Git repository — enabling branching, ~~merging~~, diffing, and full history for design work.

**MVP scope:** Figma integration only.

## Architecture

```
crates/
  dit-core/          Rust core library (types, canonical JSON, assets, git, fig2json)
  dit-cli/           CLI tool ("dit")
  dit-gui/           Desktop GUI (Tauri 2 + React)
scripts/
  download-fig.mjs   Playwright-based .fig file downloader
```

## Prerequisites

- **Rust** (1.75+): https://rustup.rs
- **Node.js** (20+): for the GUI frontend and Playwright downloader
- **Playwright Chromium**: installed automatically via `npx playwright install chromium`
- **Figma account**: with either:
  - Browser auth cookie (`__Host-figma.authn`), or
  - Email and password (no 2FA)

## Build

```bash
# Build all Rust crates (core, CLI, GUI backend)
cargo build --workspace

# Run tests
cargo test --workspace

# Build the CLI in release mode
cargo build -p dit-cli --release
# Binary at: target/release/dit

# Install Playwright dependencies
cd scripts && npm install && npx playwright install chromium && cd ..
```

### GUI

```bash
cd crates/dit-gui/frontend
npm install
cd ../../..
cargo build -p dit-gui
```

To run the GUI in development mode (requires Tauri CLI):

```bash
cargo install tauri-cli
cd crates/dit-gui
cargo tauri dev
```

## Quick Start

### 1. Initialize a repository

```bash
mkdir my-design && cd my-design
dit init
```

DIT will prompt you to:
- Select your design platform (Figma)
- Choose auth method: browser cookie or email/password
- Enter the file key (from the Figma file URL: `figma.com/design/<FILE_KEY>/...`)

Your credentials are saved to `.env` (git-ignored).

### 2. Commit a snapshot

```bash
dit commit -m "Initial design snapshot"
```

This uses Playwright to download the `.fig` file from Figma, converts it to deterministic JSON via `fig2json`, and commits both the JSON and the `.fig` file to Git. The `.fig` file is stored in `dit.fig/` so that cloning the repo gives you full restore capability.

### 3. Work with branches

```bash
dit branch feature/new-header    # Create a branch
dit checkout feature/new-header  # Switch to it

# Make changes in Figma, then:
dit commit -m "Redesigned header"

dit checkout main                # Switch back
~~dit merge feature/new-header     # Merge the branch~~
```

### 4. View history

```bash
dit log          # Show commit history
dit status       # Show current branch and changes
dit branch       # List all branches
```

### 5. Restore a previous version

```bash
dit log                    # Find the commit hash
dit restore <commit-hash>  # Restore that version
```

DIT will show you the path to the `.fig` file for that commit. Open it in Figma to restore the design — that's it.

### 6. Remote collaboration

```bash
# Add a remote (standard git)
git remote add origin <url>

dit push    # Push to remote
dit pull    # Pull from remote
```

Both the canonical JSON and `.fig` snapshot files are pushed and pulled. After cloning or pulling, you can restore any commit's `.fig` file directly — no re-download needed.

## CLI Reference

| Command | Description |
|---------|-------------|
| `dit init` | Initialize a new DIT repository |
| `dit status` | Show branch and change status |
| `dit commit -m "msg"` | Download .fig, convert to JSON, commit |
| `dit log [-n N]` | Show commit history |
| `dit branch [name]` | List branches or create a new one |
| `dit checkout <ref>` | Switch to a branch or commit |
| ~~`dit merge <branch>`~~ | ~~Merge a branch into current~~ *(not available in MVP)* |
| `dit restore <commit>` | Get .fig file path for a commit |
| `dit push [remote]` | Push to remote repository |
| `dit pull [remote]` | Pull from remote repository |
| `dit diff <c1> <c2>` | Compare two commits |

## Repository Layout

```
my-design/
  dit.json                  Project metadata
  dit.pages/                One JSON file per page (from fig2json)
    0_1.json
    0_2.json
  dit.styles.json           Shared style definitions
  dit.components.json       Component & component set metadata
  dit.assets/               Content-addressed binary assets
    sha256_<hash>
  dit.fig/                  .fig file snapshots (committed to git)
    latest.fig              Current commit's .fig file
    <hash>.fig              Previous commits' .fig files
  .dit/                     Internal metadata (git-ignored)
    config.json             DIT configuration
    fig_snapshots/          Legacy local .fig copies
      <hash>.fig
  .env                      Figma credentials (git-ignored)
  .gitignore
```

All `dit.*` files are deterministic canonical JSON — sorted keys, stable float precision, 2-space indent. This means `git diff` produces clean, meaningful diffs of design changes.

## How It Works

1. **Commit**: DIT uses Playwright to download the native `.fig` file from Figma, then uses `fig2json` to convert it to canonical JSON. Both the JSON and the `.fig` file are committed to Git, so every commit carries a full lossless snapshot that can be restored from any clone.

2. **Restore**: DIT locates the `.fig` file for the target commit and presents it to the user. Open the `.fig` file in Figma to restore the design in two clicks.

3. ~~**Merge**: Merging happens on the JSON text layer using standard Git merge. After merge, DIT shows available `.fig` files from both branches as starting points. The user opens one in Figma, adjusts to match the merged state, and commits. See [docs/merge-strategy.md](docs/merge-strategy.md) for details.~~ *(Merge is not available in MVP due to complexity.)*

4. **Lossless guarantee**: The `.fig` file is Figma's native format — it contains the complete design state with zero loss. The JSON layer provides human-readable diffs and Git-compatible merging.

## Authentication

DIT uses Playwright to automate `.fig` file downloads from Figma. Two auth methods are supported:

**Cookie-based (recommended):**
1. Log in to Figma in your browser
2. Open DevTools → Application → Cookies → `www.figma.com`
3. Copy the value of `__Host-figma.authn`
4. Provide it during `dit init` or set `FIGMA_AUTH_COOKIE` in `.env`

**Email/password:**
- Provide during `dit init`
- Stored in `.env` as `FIGMA_EMAIL` and `FIGMA_PASSWORD`
- Note: 2FA is not supported with this method

## Acknowledgements

- [fig2json](https://github.com/kreako/fig2json) — Rust crate for converting `.fig` files to JSON, which makes DIT's deterministic design diffing possible.

## License

MIT
