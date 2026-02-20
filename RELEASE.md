# DIT Release Guide

Step-by-step instructions for creating a GitHub release of DIT.

---

## 1. Pre-release Checklist

- [ ] All tests pass:
  ```bash
  cargo test --workspace
  ```
- [ ] Version numbers updated in all locations (see [Versioning](#6-versioning) below)
- [ ] README.md is up to date with any new commands or features
- [ ] All changes committed and pushed to `main`
- [ ] `cargo build --workspace` succeeds with no warnings

## 2. Building for macOS

### CLI Binary

**Apple Silicon (arm64):**
```bash
cargo build -p dit-cli --release --target aarch64-apple-darwin
# Binary at: target/aarch64-apple-darwin/release/dit
```

**Intel (x86_64):**
```bash
cargo build -p dit-cli --release --target x86_64-apple-darwin
# Binary at: target/x86_64-apple-darwin/release/dit
```

**Universal Binary (arm64 + x86_64):**
```bash
# Build both architectures
cargo build -p dit-cli --release --target aarch64-apple-darwin
cargo build -p dit-cli --release --target x86_64-apple-darwin

# Combine with lipo
lipo -create \
  target/aarch64-apple-darwin/release/dit \
  target/x86_64-apple-darwin/release/dit \
  -output target/release/dit-macos-universal

# Verify
file target/release/dit-macos-universal
# Should show: Mach-O universal binary with 2 architectures:
#   [x86_64:Mach-O 64-bit executable x86_64]
#   [arm64:Mach-O 64-bit executable arm64]
```

> **Note:** You may need to add the targets first if they aren't installed:
> ```bash
> rustup target add aarch64-apple-darwin x86_64-apple-darwin
> ```

### GUI App (Tauri)

**Install frontend dependencies first:**
```bash
cd crates/dit-gui/frontend && npm install && cd ../../..
```

**Apple Silicon (arm64):**
```bash
cd crates/dit-gui && cargo tauri build --target aarch64-apple-darwin && cd ../..
# .app at: target/aarch64-apple-darwin/release/bundle/macos/DIT.app
# .dmg at: target/aarch64-apple-darwin/release/bundle/dmg/DIT_0.1.0_aarch64.dmg
```

**Intel (x86_64):**
```bash
cd crates/dit-gui && cargo tauri build --target x86_64-apple-darwin && cd ../..
# .app at: target/x86_64-apple-darwin/release/bundle/macos/DIT.app
# .dmg at: target/x86_64-apple-darwin/release/bundle/dmg/DIT_0.1.0_x86_64.dmg
```

**Universal GUI App (arm64 + x86_64):**
```bash
cd crates/dit-gui && cargo tauri build --target universal-apple-darwin && cd ../..
# .app at: target/universal-apple-darwin/release/bundle/macos/DIT.app
# .dmg at: target/universal-apple-darwin/release/bundle/dmg/DIT_0.1.0_universal.dmg
```

> **Note:** Tauri's `universal-apple-darwin` target handles the lipo merge automatically.

### Prepare Release Assets

Rename binaries to include architecture for clarity:

```bash
VERSION="0.1.0"

# CLI binaries
cp target/aarch64-apple-darwin/release/dit dit-macos-arm64
cp target/x86_64-apple-darwin/release/dit dit-macos-x64

# DMG files (already named by Tauri, but rename for consistency)
cp "target/aarch64-apple-darwin/release/bundle/dmg/DIT_${VERSION}_aarch64.dmg" DIT-macos-arm64.dmg
cp "target/x86_64-apple-darwin/release/bundle/dmg/DIT_${VERSION}_x86_64.dmg" DIT-macos-x64.dmg
```

## 3. Creating the GitHub Release

### Using `gh` CLI (Recommended)

```bash
VERSION="0.1.0"

gh release create "v${VERSION}" \
  dit-macos-arm64 \
  dit-macos-x64 \
  DIT-macos-arm64.dmg \
  DIT-macos-x64.dmg \
  --title "DIT v${VERSION}" \
  --notes-file RELEASE_NOTES.md
```

Or with inline notes:

```bash
VERSION="0.1.0"

gh release create "v${VERSION}" \
  dit-macos-arm64 \
  dit-macos-x64 \
  DIT-macos-arm64.dmg \
  DIT-macos-x64.dmg \
  --title "DIT v${VERSION}" \
  --notes "## What's New

- First public release of DIT
- CLI and GUI for Figma version control
- Git-style branching, merging, and diffing for design files

See the [README](README.md) for full documentation."
```

To create a draft release first (recommended for review):

```bash
gh release create "v${VERSION}" \
  dit-macos-arm64 \
  dit-macos-x64 \
  DIT-macos-arm64.dmg \
  DIT-macos-x64.dmg \
  --title "DIT v${VERSION}" \
  --notes-file RELEASE_NOTES.md \
  --draft
```

### Using the GitHub Web Interface

1. Go to the repository page on GitHub
2. Click **Releases** in the right sidebar (or navigate to `/<owner>/<repo>/releases`)
3. Click **Draft a new release**
4. Click **Choose a tag** and type `v0.1.0`, then select **Create new tag: v0.1.0 on publish**
5. Set the **Target** branch to `main`
6. Set the **Release title** to `DIT v0.1.0`
7. Write release notes in the description field (see template in section 5)
8. Drag and drop or click **Attach binaries** to upload:
   - `dit-macos-arm64`
   - `dit-macos-x64`
   - `DIT-macos-arm64.dmg`
   - `DIT-macos-x64.dmg`
9. Check **Set as the latest release**
10. Click **Publish release** (or **Save draft** to review first)

## 4. Release Assets to Include

| Asset | Description |
|-------|-------------|
| `dit-macos-arm64` | CLI binary for macOS Apple Silicon (M1/M2/M3/M4) |
| `dit-macos-x64` | CLI binary for macOS Intel |
| `DIT-macos-arm64.dmg` | GUI desktop app for macOS Apple Silicon |
| `DIT-macos-x64.dmg` | GUI desktop app for macOS Intel |

## 5. Release Description Template

Use this as the body of the GitHub release:

```markdown
![DIT](dit.svg)

# DIT — Design Version Control

Git-style version control for design files. DIT downloads your Figma designs as
native `.fig` files, converts them to deterministic JSON for text-based diffs, and
stores everything in a normal Git repository — enabling branching, ~~merging~~, diffing,
and full history for design work.

> This project is **vibe coded** — built collaboratively with AI assistance.

## Features

- **Download & snapshot** Figma files as native `.fig` with zero loss
- **Deterministic JSON** conversion for clean, meaningful `git diff` output
- **Branch and diff** design changes just like code
- ~~**Merge** design branches~~ *(not available in MVP)*
- **Restore** any previous version by opening the `.fig` file in Figma
- **Full clone restore** — `.fig` files are committed to git, so cloning gives you every version
- **Remote collaboration** via standard Git remotes (push/pull)
- **Desktop GUI** with visual preview and one-click operations
- **CLI** for scripting and automation

## Installation (macOS)

### CLI

Download the binary for your architecture and make it executable:

**Apple Silicon (M1/M2/M3/M4):**
```bash
curl -L -o dit https://github.com/<owner>/dit/releases/download/v0.1.0/dit-macos-arm64
chmod +x dit
sudo mv dit /usr/local/bin/
```

**Intel:**
```bash
curl -L -o dit https://github.com/<owner>/dit/releases/download/v0.1.0/dit-macos-x64
chmod +x dit
sudo mv dit /usr/local/bin/
```

### GUI

Download the `.dmg` for your architecture, open it, and drag **DIT.app** to
your Applications folder.

## Quick Start

```bash
# Initialize a new DIT repo
mkdir my-design && cd my-design
dit init

# Commit a snapshot from Figma
dit commit -m "Initial design snapshot"

# Branch and iterate
dit branch feature/new-header
dit checkout feature/new-header
dit commit -m "Redesigned header"

# View history
dit log
```

## Prerequisites

- **Node.js 20+** (for the Playwright-based Figma downloader)
- **Figma account** with a browser auth cookie or email/password

## Acknowledgements

- [fig2json](https://github.com/kreako/fig2json) — Rust crate for converting `.fig` files to JSON

## Links

- [README](README.md) — full documentation
- [Repository](https://github.com/<owner>/dit)
```

> **Remember:** Replace `<owner>` with the actual GitHub username or organization.

## 6. Versioning

DIT uses [Semantic Versioning](https://semver.org/):

- **MAJOR** (`X.0.0`): Breaking changes to CLI commands, repo format, or config
- **MINOR** (`0.X.0`): New features, new commands, backwards-compatible additions
- **PATCH** (`0.0.X`): Bug fixes, performance improvements, documentation updates

### Where Version Numbers Live

Version numbers must be updated in **two places** before a release:

| File | Field | Notes |
|------|-------|-------|
| `Cargo.toml` (workspace root) | `workspace.package.version` | Single source of truth for all Rust crates |
| `crates/dit-gui/tauri.conf.json` | `version` | Must match the workspace version |

### Bumping the Version

```bash
VERSION="0.2.0"

# 1. Update workspace Cargo.toml
sed -i '' "s/^version = \".*\"/version = \"${VERSION}\"/" Cargo.toml

# 2. Update tauri.conf.json
cd crates/dit-gui
# Use jq or manually edit tauri.conf.json
jq ".version = \"${VERSION}\"" tauri.conf.json > tmp.json && mv tmp.json tauri.conf.json
cd ../..

# 3. Verify
grep 'version' Cargo.toml | head -1
grep '"version"' crates/dit-gui/tauri.conf.json

# 4. Commit the version bump
git add Cargo.toml crates/dit-gui/tauri.conf.json
git commit -m "Bump version to ${VERSION}"

# 5. Tag the release
git tag "v${VERSION}"
git push origin main --tags
```

### Git Tags

Always create an annotated or lightweight tag matching the version:

```bash
git tag v0.1.0
git push origin v0.1.0
```

Tags should follow the format `v<MAJOR>.<MINOR>.<PATCH>` (e.g., `v0.1.0`, `v1.0.0`).
