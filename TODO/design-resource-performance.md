# Resource System Performance Considerations

Tracking performance issues to address after core modeling is solid.

## Current Design (Phase 1)

The current design prioritizes correctness and clear modeling over performance.
This is intentional - we want robust abstractions before optimizing.

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
