# DIT Merge Strategy for .fig-based Architecture

## Context

With the new architecture:
- **Commit**: Playwright downloads `.fig` file → `fig2json` converts to JSON → JSON is committed to git
- **Restore**: User opens the `.fig` file in Figma directly
- `.fig` files are stored locally in `.dit/fig_snapshots/<commit_hash>.fig`
- JSON (from `fig2json`) is the canonical text layer tracked by git

## The Challenge

Git merge operates on the JSON text layer (the canonical representation). After a merge:
1. The merged JSON state exists in the working tree
2. There is **no `.fig` file** that corresponds to the merged JSON state
3. Each branch has `.fig` files only for their own commits

## Merge Strategy Design

### Principle: JSON-first, .fig as reference

The merge strategy treats JSON as the source of truth for merging, and `.fig` files as **reference starting points** for the user to recreate the merged design in Figma.

### Three Merge Scenarios

#### 1. Fast-Forward Merge (no divergence)
- Branch pointer moves forward; no actual merge needed
- The target commit's `.fig` file already exists (from its original commit)
- **User action**: None — the `.fig` file for the new HEAD is already available
- **Output**: Report success + the `.fig` file path if available

#### 2. Clean Merge (no conflicts)
- Git merges the JSON cleanly, creating a merge commit
- No `.fig` file exists for the merge commit
- **User action**: Open the `.fig` from whichever branch is closest to the desired result, make adjustments to match the merged JSON, then `dit commit`
- **Output**: Report success, list available `.fig` files from both branch tips, suggest workflow

#### 3. Conflict Merge
- Git finds conflicts in the JSON files
- Conflict markers are written to the working tree (standard git behavior)
- **User action**: Resolve JSON conflicts (manually or with a tool), then open a `.fig` file as starting point, make changes in Figma, run `dit commit`
- **Output**: List conflicting files, list available `.fig` files from both branches, explain the resolution workflow

### Data Model Changes

#### Updated `MergeResult`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeResult {
    /// True if the merge completed without conflicts.
    pub success: bool,
    /// Paths that have conflicts (empty if `success` is true).
    pub conflicts: Vec<String>,
    /// Resulting commit hash (None if conflicts remain).
    pub commit_hash: Option<String>,
    /// Info about available .fig snapshot files from each branch.
    pub fig_snapshots: MergeFigSnapshots,
}

/// Available .fig snapshot files relevant to a merge operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeFigSnapshots {
    /// .fig snapshot path for the current (ours) branch tip, if it exists on disk.
    pub ours: Option<String>,
    /// Commit hash of the "ours" snapshot.
    pub ours_commit: Option<String>,
    /// .fig snapshot path for the incoming (theirs) branch tip, if it exists on disk.
    pub theirs: Option<String>,
    /// Commit hash of the "theirs" snapshot.
    pub theirs_commit: Option<String>,
}
```

### Implementation Changes

#### `git_ops.rs` — `merge()` function

After performing the merge (all three cases), look up `.fig` snapshot paths:
1. Get the commit hash for the current branch tip (before merge)
2. Get the commit hash for the incoming branch tip
3. Check if `.dit/fig_snapshots/<hash>.fig` exists for each
4. Populate `MergeFigSnapshots` accordingly

```rust
// Helper to check for .fig snapshot
fn fig_snapshot_path(repo_root: &Path, commit_hash: &str) -> Option<String> {
    let path = repo_root
        .join(DitPaths::DIT_DIR)
        .join("fig_snapshots")
        .join(format!("{}.fig", commit_hash));
    if path.exists() {
        Some(path.to_string_lossy().to_string())
    } else {
        None
    }
}
```

#### `repository.rs` — `merge()` wrapper

No changes needed beyond passing through the new `MergeResult`.

#### `main.rs` (CLI) — `cmd_merge()`

Update output messages for each scenario:

**Success (clean merge or fast-forward):**
```
  ✓ Merged feature-branch successfully
  Merge commit: abc1234

  Note: No .fig file exists for the merged state.
  Available .fig snapshots:
    Current branch: .dit/fig_snapshots/abc1234.fig
    Merged branch:  .dit/fig_snapshots/def5678.fig

  To complete the merge visually:
    1. Open a .fig file above in Figma as your starting point
    2. Adjust the design to match the merged state
    3. Run 'dit commit' to capture the merged .fig
```

**Conflicts:**
```
  ✗ Merge conflicts in 2 files:
    C dit.pages/0_1.json
    C dit.components.json

  Available .fig snapshots:
    Current branch: .dit/fig_snapshots/abc1234.fig
    Merged branch:  .dit/fig_snapshots/def5678.fig

  To resolve:
    1. Fix the JSON conflicts in the files above
    2. Open a .fig file as your starting point in Figma
    3. Make adjustments to match the resolved merge
    4. Run 'dit commit -m "Resolve merge"' to finish
```

**Fast-forward (no .fig guidance needed, the .fig already exists):**
```
  ✓ Merged feature-branch successfully (fast-forward)
  Now at: abc1234
  .fig snapshot: .dit/fig_snapshots/abc1234.fig
```

### Edge Cases

1. **Neither branch has a .fig file locally**: This happens if snapshots were cleaned up or the repo was cloned without them (`.dit/` is gitignored). The merge still works on JSON — just omit the .fig guidance and tell the user to run `dit commit` from Figma.

2. **Only one branch has a .fig file**: Show that one as the starting point.

3. **Up-to-date merge**: No action needed, no .fig guidance shown.

### What We Are NOT Doing

- **Automatic .fig merging**: `.fig` is a binary format; we cannot merge two `.fig` files programmatically. The user must recreate the merged state in Figma.
- **fig2json round-trip**: We don't convert JSON back to `.fig`. The conversion is one-way (`.fig` → JSON via `fig2json`).
- **Blocking merge on .fig availability**: The merge operates purely on JSON. `.fig` files are optional convenience references.

### Summary

| Scenario | JSON Layer | .fig Layer | User Action |
|----------|-----------|------------|-------------|
| Fast-forward | Branch pointer moves | .fig exists for target commit | None (or open .fig) |
| Clean merge | Auto-merged, commit created | No .fig for merge commit | Open reference .fig, adjust, commit |
| Conflict merge | Conflict markers in files | No .fig for merge state | Resolve JSON, adjust in Figma, commit |
