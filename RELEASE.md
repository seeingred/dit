# DIT Release Guide

Step-by-step instructions for creating a GitHub release of DIT.

---

## 1. Pre-release Checklist

- [ ] All tests pass:
  ```bash
  cargo test --workspace
  ```
- [ ] Version numbers updated in all locations (see [Versioning](#7-versioning) below)
- [ ] README.md is up to date with any new commands or features
- [ ] All changes committed and pushed to `main`
- [ ] `cargo build --workspace` succeeds with no warnings

## 2. Environment Setup

### Apple Signing & Notarization

The following environment variables are needed for code signing and notarization. They can be stored in `.env` (git-ignored):

```bash
# Apple Developer ID certificate (required for Gatekeeper)
APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAMID)"

# Notarization credentials
APPLE_ID="your@email.com"
APPLE_PASSWORD="xxxx-xxxx-xxxx-xxxx"   # App-specific password from appleid.apple.com
APPLE_TEAM_ID="TEAMID"
```

> **App-specific password:** Generate at https://appleid.apple.com/account/manage → Sign-In and Security → App-Specific Passwords

> **Developer ID certificate:** Create at https://developer.apple.com/account/resources/certificates/list → "+" → Developer ID Application (G2 Sub-CA)

### Load from .env

```bash
export $(grep -E '^APPLE_' .env | xargs)
```

## 3. Building for macOS

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
# Should show: Mach-O universal binary with 2 architectures
```

> **Note:** You may need to add the targets first if they aren't installed:
> ```bash
> rustup target add aarch64-apple-darwin x86_64-apple-darwin
> ```

### GUI App (Tauri) — Signed & Notarized

**Install frontend dependencies first:**
```bash
cd crates/dit-gui/frontend && npm install && cd ../../..
```

**Apple Silicon (arm64):**
```bash
cd crates/dit-gui && cargo tauri build --target aarch64-apple-darwin && cd ../..
# .app at: target/aarch64-apple-darwin/release/bundle/macos/DIT.app
# .dmg at: target/aarch64-apple-darwin/release/bundle/dmg/DIT_<VERSION>_aarch64.dmg
```

**Intel (x86_64):**
```bash
cd crates/dit-gui && cargo tauri build --target x86_64-apple-darwin && cd ../..
```

**Universal GUI App (arm64 + x86_64):**
```bash
cd crates/dit-gui && cargo tauri build --target universal-apple-darwin && cd ../..
```

> Tauri automatically signs and notarizes when `APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_PASSWORD`, and `APPLE_TEAM_ID` are set. The notarization ticket is stapled to the app so users can open it without Gatekeeper warnings.

> **Important:** OpenSSL is vendored (`openssl-sys` with `features = ["vendored"]`) to avoid dynamic linking against Homebrew's libssl, which causes code signature team ID mismatches on macOS.

### Prepare Release Assets

```bash
VERSION="0.2.0"

# CLI binaries
cp target/aarch64-apple-darwin/release/dit dit-macos-arm64
cp target/x86_64-apple-darwin/release/dit dit-macos-x64

# DMG files
cp "target/aarch64-apple-darwin/release/bundle/dmg/DIT_${VERSION}_aarch64.dmg" DIT-macos-arm64.dmg
cp "target/x86_64-apple-darwin/release/bundle/dmg/DIT_${VERSION}_x86_64.dmg" DIT-macos-x64.dmg
```

## 4. Creating the GitHub Release

### Using `gh` CLI (Recommended)

```bash
VERSION="0.2.0"

# Make sure you're on the seeingred account
gh auth switch --user seeingred

gh release create "v${VERSION}" \
  dit-macos-arm64 \
  DIT-macos-arm64.dmg \
  --title "DIT v${VERSION}" \
  --notes "Release notes here..."
```

To replace assets on an existing release:
```bash
gh release upload "v${VERSION}" dit-macos-arm64 DIT-macos-arm64.dmg --clobber
```

## 5. Release Assets

| Asset | Description |
|-------|-------------|
| `dit-macos-arm64` | CLI binary for macOS Apple Silicon (M1/M2/M3/M4) |
| `dit-macos-x64` | CLI binary for macOS Intel |
| `DIT-macos-arm64.dmg` | GUI desktop app for macOS Apple Silicon (signed & notarized) |
| `DIT-macos-x64.dmg` | GUI desktop app for macOS Intel (signed & notarized) |

## 6. Quick Release Script

One-command release for Apple Silicon:

```bash
VERSION="0.2.0"

# Load signing credentials
export $(grep -E '^APPLE_' .env | xargs)

# Build CLI
cargo build -p dit-cli --release --target aarch64-apple-darwin

# Build GUI (signed + notarized)
cd crates/dit-gui && cargo tauri build --target aarch64-apple-darwin && cd ../..

# Prepare assets
cp target/aarch64-apple-darwin/release/dit dit-macos-arm64
cp "target/aarch64-apple-darwin/release/bundle/dmg/DIT_${VERSION}_aarch64.dmg" DIT-macos-arm64.dmg

# Create release
gh auth switch --user seeingred
gh release create "v${VERSION}" \
  dit-macos-arm64 \
  DIT-macos-arm64.dmg \
  --title "DIT v${VERSION}" \
  --notes "See CHANGELOG for details."

# Clean up
rm dit-macos-arm64 DIT-macos-arm64.dmg
```

## 7. Versioning

DIT uses [Semantic Versioning](https://semver.org/):

- **MAJOR** (`X.0.0`): Breaking changes to CLI commands, repo format, or config
- **MINOR** (`0.X.0`): New features, new commands, backwards-compatible additions
- **PATCH** (`0.0.X`): Bug fixes, performance improvements, documentation updates

### Where Version Numbers Live

| File | Field | Notes |
|------|-------|-------|
| `Cargo.toml` (workspace root) | `workspace.package.version` | Single source of truth for all Rust crates |
| `crates/dit-gui/tauri.conf.json` | `version` | Must match the workspace version |

### Bumping the Version

```bash
VERSION="0.3.0"

# 1. Update workspace Cargo.toml
sed -i '' "s/^version = \".*\"/version = \"${VERSION}\"/" Cargo.toml

# 2. Update tauri.conf.json
cd crates/dit-gui
jq ".version = \"${VERSION}\"" tauri.conf.json > tmp.json && mv tmp.json tauri.conf.json
cd ../..

# 3. Verify
grep 'version' Cargo.toml | head -1
grep '"version"' crates/dit-gui/tauri.conf.json

# 4. Commit the version bump
git add Cargo.toml Cargo.lock crates/dit-gui/tauri.conf.json
git commit -m "Bump version to ${VERSION}"

# 5. Tag the release
git tag "v${VERSION}"
git push origin main --tags
```

Tags should follow the format `v<MAJOR>.<MINOR>.<PATCH>` (e.g., `v0.2.0`, `v1.0.0`).
