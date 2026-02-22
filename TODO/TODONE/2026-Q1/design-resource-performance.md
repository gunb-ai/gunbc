# Resource System Performance Considerations

> **Key Insight**: The current design hashes everything on every check.
> This is fundamentally wrong. Make solved this in 1976: use mtime as fast path.
>
> See `TODO/TODONE/2026-Q1/architecture-debt.md` for the consolidated view of all debt.

## The Problem

Current freshness check:
1. Expand all glob patterns (filesystem walk)
2. Read every matching file into memory
3. Hash all file contents
4. Compare to stored key

This is **O(files × file_size)** on every check, even when nothing changed.

For codegen alone, this means reading ~100+ Rust files on every CI run,
even when the code hasn't changed since the last run.

## The Solution: mtime Fast Path

```
┌─────────────────────────────────────────────────────────────┐
│  Fast Path (99% of checks, O(1))                            │
│                                                             │
│  1. Get manifest entry mtime                                │
│  2. Get max(source file mtimes) — can cache stat results    │
│  3. If manifest_mtime > max_source_mtime → Fresh            │
│     (nothing changed since we last wrote the manifest)      │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  Slow Path (only when sources actually changed)             │
│                                                             │
│  1. Identify which files changed (mtime > manifest_mtime)   │
│  2. Re-hash only changed files                              │
│  3. Combine with cached hashes for unchanged files          │
│  4. Compare computed key to stored key                      │
└─────────────────────────────────────────────────────────────┘
```

## Even Better: Git-Aware Freshness

In a git repo, we can ask git directly:

```bash
git status --porcelain -- 'core/codegen/src/**/*.rs' 'core/ir/src/**/*.rs'
```

If output is empty → nothing in our input patterns changed → Fresh.

This is faster than walking the filesystem ourselves and handles
edge cases (new files, deleted files, renamed files) correctly.

## Current Design

**Status (updated 2026-02-05):** The mtime fast path is now implemented in
`core/infra/src/freshness.rs` via `check_freshness_mtime()`. It returns
`MtimeResult::Fresh` (skip hashing) or `MaybeStale(reason)` (proceed to hash).
`ManifestEntry.input_file_count` tracks expected file count for fast invalidation.
See `architecture-debt.md` Phase B for details.

## Known Performance Issues

### 1. Hash Computation on Every Check

**Problem**: `compute_key()` calls `HashBuilder::update_glob()` which reads and
hashes all matching files every time. For large codebases, this is O(n) file
reads per freshness check.

**Signal**: Freshness checks become slow as codebase grows.

**Solution**: Cache file hashes keyed by (path, mtime, size). Only re-hash if
mtime changed. This is how Cargo tracks dependencies.

### 2. No File Hash Caching

**Problem**: We re-hash the same unchanged files repeatedly across different
resources that share inputs.

**Signal**: Multiple resources with overlapping inputs cause redundant I/O.

**Solution**: Global file hash cache (content-addressable store pattern).

### 3. Redundant Key Computation

**Problem**: In `acquire()`, we call `compute_key()` twice when fresh - once in
`check_state()`, once to return the handle.

**Signal**: Profiling shows duplicate hash computation.

**Solution**: Cache the computed key in `check_state()` result, or restructure
the API to avoid redundant calls.

### 4. Glob Expansion Cost

**Problem**: `glob::glob()` walks the filesystem on every call.

**Signal**: Many resources with glob patterns cause slow startup.

**Solution**:
- Cache glob results keyed by pattern
- Store expanded file list in manifest entry
- Only re-expand when checking for new/deleted files

### 5. Manifest Load/Save Frequency

**Problem**: If manifest is loaded/saved frequently, I/O overhead adds up.

**Signal**: Many small operations cause manifest thrashing.

**Solution**: Keep manifest in memory during execution, save once at end.

## Optimized Freshness Check Design (Future)

```
┌─────────────────────────────────────────────────────────┐
│  Fast Path (most common case)                           │
│                                                         │
│  1. Load manifest entry (has stored key + file list)    │
│  2. For each file in stored list:                       │
│     - Check (mtime, size) from stat cache               │
│     - If unchanged → use cached hash                    │
│     - If changed → re-hash, update cache                │
│  3. Compare computed key to stored key                  │
│  4. If match → Fresh (no glob expansion needed)         │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│  Slow Path (only when potentially stale)                │
│                                                         │
│  1. Re-expand globs (files may have been added/removed) │
│  2. Compare file list to stored list                    │
│  3. If different → Stale (structural change)            │
│  4. If same → compute full key and compare              │
└─────────────────────────────────────────────────────────┘
```

## Manifest Entry Enhancement (Future)

```rust
pub struct ManifestEntry {
    /// Final content hash (current design)
    pub key: ContentHash,

    /// Files that contributed to this key (for fast-path checking)
    pub input_files: Vec<InputFileEntry>,

    /// When this entry was created
    pub created_at: i64,

    /// Output files produced
    pub outputs: Vec<PathBuf>,
}

pub struct InputFileEntry {
    pub path: PathBuf,
    pub hash: ContentHash,
    pub mtime: i64,
    pub size: u64,
}
```

## Bazel Lessons

Bazel is slow because of:
- JVM startup overhead
- Over-aggressive sandboxing
- Complex query language evaluation
- Network overhead for remote execution

We can avoid these by:
- Native code (Rust)
- Lightweight isolation (if needed)
- Simple dependency model (DAG edges, not query language)
- Local-first execution

## When to Optimize

Optimize when we see:
1. Freshness checks taking >100ms on moderate codebases
2. Manifest files growing >1MB
3. Users reporting slow startup times

Until then, keep the simple correct implementation.
